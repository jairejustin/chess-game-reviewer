use crate::models::fetch::{
    ChessComCursor, FetchResult, GamePlayer,
    GameSummary, LichessRawGame,
    LichessRawProfile, Platform, PlayerProfile,
    RawArchives, RawGame, RawMonthlyGames,
    RawProfile,
};
use reqwest::Client;

const GAMES_PER_PAGE: usize = 20;

/// Normalize Lichess game result to standard notation based on the global winner field
fn normalize_lichess_result(
    winner: Option<&str>,
) -> (String, String, String) {
    let (white_normalized, black_normalized) =
        match winner {
            Some("white") => ("1-0", "0-1"),
            Some("black") => ("0-1", "1-0"),
            _ => ("1/2-1/2", "1/2-1/2"),
        };

    let game_result =
        white_normalized.to_string();

    (
        white_normalized.to_string(),
        black_normalized.to_string(),
        game_result,
    )
}

/// Normalize Chess.com game result to user perspective
fn normalize_chesscom_result(
    white_result: &str,
    black_result: &str,
) -> (String, String, String) {
    let white_normalized = match white_result {
        "win" => "1-0",
        "drawn" | "agreed" | "repetition"
        | "stalemate" | "insufficient"
        | "50move" | "timevsinsufficient" => {
            "1/2-1/2"
        }
        _ => "0-1",
    };

    let black_normalized = match black_result {
        "win" => "0-1",
        "drawn" | "agreed" | "repetition"
        | "stalemate" | "insufficient"
        | "50move" | "timevsinsufficient" => {
            "1/2-1/2"
        }
        _ => "1-0",
    };

    let game_result =
        white_normalized.to_string();

    (
        white_normalized.to_string(),
        black_normalized.to_string(),
        game_result,
    )
}

/// Fetch Chess.com user profile
pub async fn fetch_chesscom_profile(
    client: &Client,
    username: &str,
) -> Result<PlayerProfile, String> {
    let url = format!(
        "https://api.chess.com/pub/player/{}",
        username.to_lowercase()
    );

    let raw: RawProfile = client
        .get(&url)
        .header(
            "User-Agent",
            "chess-analyzer-app",
        )
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let country_code = raw
        .country
        .as_deref()
        .and_then(|c| c.split('/').last())
        .map(|s| s.to_string());

    Ok(PlayerProfile {
        username: raw.username,
        title: raw.title,
        avatar_url: raw.avatar,
        country_code,
        joined: raw.joined,
        last_online: raw.last_online,
        followers: raw.followers,
        is_streamer: raw.is_streamer,
        twitch_url: raw.twitch_url,
        fide: raw.fide,
    })
}

/// Fetch Chess.com user game archive list
async fn fetch_chesscom_archives(
    client: &Client,
    username: &str,
) -> Result<Vec<String>, String> {
    let url = format!(
        "https://api.chess.com/pub/player/{}/games/archives",
        username.to_lowercase()
    );

    let raw: RawArchives = client
        .get(&url)
        .header(
            "User-Agent",
            "chess-analyzer-app",
        )
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(raw.archives)
}

/// Fetch Chess.com user archive games by month URL
async fn fetch_chesscom_month(
    client: &Client,
    archive_url: &str,
) -> Result<Vec<RawGame>, String> {
    let response = client
        .get(archive_url)
        .header(
            "User-Agent",
            "chess-analyzer-app",
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let raw: RawMonthlyGames = match response
        .json::<RawMonthlyGames>()
        .await
    {
        Ok(data) => data,
        Err(_) => return Ok(vec![]),
    };

    Ok(raw.games)
}

/// Extract and map Chess.com raw game payload to generic GameSummary
fn map_chesscom_game(
    raw: &RawGame,
) -> Option<GameSummary> {
    let pgn = raw.pgn.clone()?;
    let white_raw = raw.white.as_ref()?;
    let black_raw = raw.black.as_ref()?;

    let white_result = white_raw
        .result
        .as_deref()
        .unwrap_or("unknown");
    let black_result = black_raw
        .result
        .as_deref()
        .unwrap_or("unknown");

    let (white_normalized, black_normalized, _) =
        normalize_chesscom_result(
            white_result,
            black_result,
        );

    let id = raw
        .url
        .as_deref()
        .and_then(|u| u.split('/').last())
        .unwrap_or("unknown")
        .to_string();

    Some(GameSummary {
        id,
        pgn,
        platform: Platform::ChessCom,
        time_class: raw
            .time_class
            .clone()
            .unwrap_or_default(),
        time_control: raw.time_control.clone(),
        played_at: raw.end_time.unwrap_or(0),
        rated: raw.rated.unwrap_or(false),
        white: GamePlayer {
            username: white_raw
                .username
                .clone()
                .unwrap_or_default(),
            rating: white_raw.rating,
            result: white_normalized,
        },
        black: GamePlayer {
            username: black_raw
                .username
                .clone()
                .unwrap_or_default(),
            rating: black_raw.rating,
            result: black_normalized,
        },
    })
}

/// Core handler to bulk fetch and chunk Chess.com games with cursor state
pub async fn fetch_chesscom_games(
    username: &str,
    cursor: Option<ChessComCursor>,
) -> Result<FetchResult, String> {
    let client = Client::new();

    let (profile, archives) = tokio::try_join!(
        fetch_chesscom_profile(&client, username),
        fetch_chesscom_archives(
            &client, username
        ),
    )?;

    if archives.is_empty() {
        return Ok(FetchResult {
            profile,
            games: vec![],
            cursor: None,
        });
    }

    let mut archive_index = cursor
        .as_ref()
        .map_or(0, |c| c.archive_index);
    let mut offset =
        cursor.as_ref().map_or(0, |c| c.offset);
    let mut collected: Vec<GameSummary> =
        Vec::new();

    let reversed: Vec<&String> =
        archives.iter().rev().collect();

    while collected.len() < GAMES_PER_PAGE
        && archive_index < reversed.len()
    {
        let month_url = reversed[archive_index];
        let raw_games = fetch_chesscom_month(
            &client, month_url,
        )
        .await?;

        let available: Vec<GameSummary> =
            raw_games
                .iter()
                .rev()
                .skip(offset)
                .filter_map(map_chesscom_game)
                .take(
                    GAMES_PER_PAGE
                        - collected.len(),
                )
                .collect();

        let took = available.len();
        collected.extend(available);
        offset += took;

        let month_total = raw_games
            .iter()
            .filter_map(|g| g.pgn.as_ref())
            .count();

        if offset >= month_total {
            archive_index += 1;
            offset = 0;
        }
    }

    let next_cursor =
        if archive_index < reversed.len() {
            Some(ChessComCursor {
                archive_index,
                offset,
            })
        } else {
            None
        };

    Ok(FetchResult {
        profile,
        games: collected,
        cursor: next_cursor,
    })
}

/// Fetch Lichess user profile data
pub async fn fetch_lichess_profile(
    client: &Client,
    username: &str,
) -> Result<PlayerProfile, String> {
    let url = format!(
        "https://lichess.org/api/user/{}",
        username.to_lowercase()
    );

    let raw: LichessRawProfile = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Lichess profile: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse Lichess profile: {}", e))?;

    let country_code = raw
        .profile
        .as_ref()
        .and_then(|p| p.country.clone());
    let fide = raw
        .profile
        .as_ref()
        .and_then(|p| p.fide_rating);

    Ok(PlayerProfile {
        username: raw.username,
        title: raw.title,
        avatar_url: None,
        country_code,
        joined: raw.created_at.map(|t| t / 1000), // convert ms to seconds
        last_online: raw
            .seen_at
            .map(|t| t / 1000),
        followers: raw
            .count
            .and_then(|c| c.followers),
        is_streamer: None,
        twitch_url: None,
        fide,
    })
}

/// Core handler to pull raw Lichess data in dynamic bulk (NDJSON stream)
pub async fn fetch_lichess_games(
    username: &str,
    _cursor: Option<ChessComCursor>,
) -> Result<FetchResult, String> {
    let client = Client::new();
    let profile =
        fetch_lichess_profile(&client, username)
            .await?;

    let url = format!(
        "https://lichess.org/api/games/user/{}?max=100&pgnInJson=true",
        username.to_lowercase()
    );

    let response = client
        .get(&url)
        .header("Accept", "application/x-ndjson")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Lichess game stream: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let mut games = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(raw) = serde_json::from_str::<
            LichessRawGame,
        >(line)
        {
            let pgn = raw.pgn.unwrap_or_default();
            if pgn.is_empty() {
                continue;
            }

            let white_user = raw
                .players
                .as_ref()
                .and_then(|p| p.white.as_ref())
                .and_then(|w| w.user.as_ref())
                .and_then(|u| u.name.clone())
                .unwrap_or_else(|| {
                    "Unknown".to_string()
                });

            let white_rating = raw
                .players
                .as_ref()
                .and_then(|p| p.white.as_ref())
                .and_then(|w| w.rating);

            let black_user = raw
                .players
                .as_ref()
                .and_then(|p| p.black.as_ref())
                .and_then(|w| w.user.as_ref())
                .and_then(|u| u.name.clone())
                .unwrap_or_else(|| {
                    "Unknown".to_string()
                });

            let black_rating = raw
                .players
                .as_ref()
                .and_then(|p| p.black.as_ref())
                .and_then(|w| w.rating);

            let (white_result, black_result, _) =
                normalize_lichess_result(
                    raw.winner.as_deref(),
                );

            let time_control =
                raw.clock.map(|c| {
                    format!(
                        "{}+{}",
                        c.initial.unwrap_or(0),
                        c.increment.unwrap_or(0)
                    )
                });

            games.push(GameSummary {
                id: raw.id,
                pgn,
                platform: Platform::Lichess,
                time_class: raw
                    .speed
                    .unwrap_or_default(),
                time_control,
                played_at: raw
                    .created_at
                    .unwrap_or(0)
                    / 1000,
                rated: raw.rated.unwrap_or(false),
                white: GamePlayer {
                    username: white_user,
                    rating: white_rating,
                    result: white_result,
                },
                black: GamePlayer {
                    username: black_user,
                    rating: black_rating,
                    result: black_result,
                },
            });
        }
    }

    Ok(FetchResult {
        profile,
        games,
        cursor: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_chesscom_result_mappings() {
        // Test White Win conditions
        assert_eq!(
            normalize_chesscom_result(
                "win", "timeout"
            ),
            (
                "1-0".to_string(),
                "1-0".to_string(),
                "1-0".to_string()
            )
        );
        assert_eq!(
            normalize_chesscom_result(
                "win", "resigned"
            ),
            (
                "1-0".to_string(),
                "1-0".to_string(),
                "1-0".to_string()
            )
        );

        // Symmetrical Draw condition
        assert_eq!(
            normalize_chesscom_result(
                "repetition",
                "repetition"
            ),
            (
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string()
            )
        );

        // Asymmetrical Draw condition (Insufficient vs Timeout)
        assert_eq!(
            normalize_chesscom_result(
                "insufficient",
                "timeout"
            ),
            (
                "1/2-1/2".to_string(),
                "1-0".to_string(),
                "1/2-1/2".to_string()
            )
        );

        // Test Black Win conditions
        assert_eq!(
            normalize_chesscom_result(
                "checkmated",
                "win"
            ),
            (
                "0-1".to_string(),
                "0-1".to_string(),
                "0-1".to_string()
            )
        );
        assert_eq!(
            normalize_chesscom_result(
                "timeout", "win"
            ),
            (
                "0-1".to_string(),
                "0-1".to_string(),
                "0-1".to_string()
            )
        );
    }

    #[test]
    fn test_normalize_lichess_result_mappings() {
        // Test White Win
        assert_eq!(
            normalize_lichess_result(Some(
                "white"
            )),
            (
                "1-0".to_string(),
                "0-1".to_string(),
                "1-0".to_string()
            )
        );

        // Test Black Win
        assert_eq!(
            normalize_lichess_result(Some(
                "black"
            )),
            (
                "0-1".to_string(),
                "1-0".to_string(),
                "0-1".to_string()
            )
        );

        // Test Draw
        assert_eq!(
            normalize_lichess_result(None),
            (
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string()
            )
        );

        // Test unexpected values
        assert_eq!(
            normalize_lichess_result(Some(
                "aborted"
            )),
            (
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string(),
                "1/2-1/2".to_string()
            )
        );
    }
}
