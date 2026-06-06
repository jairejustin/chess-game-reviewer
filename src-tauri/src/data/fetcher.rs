use crate::models::fetch::{
    ChessComCursor, FetchResult, GamePlayer,
    GameSummary, Platform, PlayerProfile,
    RawArchives, RawGame, RawMonthlyGames,
    RawProfile,
};
use reqwest::Client;

const GAMES_PER_PAGE: usize = 20;

/// Normalize game result to user perpective
fn normalize_result(
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

    // Prefer white's read as the game result
    // since both should agree on draws
    let game_result =
        white_normalized.to_string();

    (
        white_normalized.to_string(),
        black_normalized.to_string(),
        game_result,
    )
}

/// Fetch user profile
pub async fn fetch_profile(
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

    // Extract country code from URL
    // e.g. "https://api.chess.com/pub/country/US" -> "US"
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

/// Fetch user game archive
async fn fetch_archives(
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

/// Fetch user archive by month
async fn fetch_month(
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

/// Extract and map game pgn and metadata
fn map_game(
    raw: &RawGame,
) -> Option<GameSummary> {
    // Skip games without PGN
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
        normalize_result(
            white_result,
            black_result,
        );

    // Extract game ID from URL last segment
    // e.g. "https://www.chess.com/game/live/12345" -> "12345"
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

/// Fetch chess.com API
pub async fn fetch_chesscom_games(
    username: &str,
    cursor: Option<ChessComCursor>,
) -> Result<FetchResult, String> {
    let client = Client::new();

    // Fetch profile and archives concurrently
    let (profile, archives) = tokio::try_join!(
        fetch_profile(&client, username),
        fetch_archives(&client, username),
    )?;

    if archives.is_empty() {
        return Ok(FetchResult {
            profile,
            games: vec![],
            cursor: None,
        });
    }

    // Determine starting position from cursor
    // or default to the most recent month
    let mut archive_index = cursor
        .as_ref()
        .map_or(0, |c| c.archive_index);
    let mut offset =
        cursor.as_ref().map_or(0, |c| c.offset);

    let mut collected: Vec<GameSummary> =
        Vec::new();

    // Archives are oldest-first so we reverse-iterate
    let reversed: Vec<&String> =
        archives.iter().rev().collect();

    while collected.len() < GAMES_PER_PAGE
        && archive_index < reversed.len()
    {
        let month_url = reversed[archive_index];
        let raw_games =
            fetch_month(&client, month_url)
                .await?;

        // Most recent games are at the end
        let available: Vec<GameSummary> =
            raw_games
                .iter()
                .rev()
                .skip(offset)
                .filter_map(map_game)
                .take(
                    GAMES_PER_PAGE
                        - collected.len(),
                )
                .collect();

        let took = available.len();
        collected.extend(available);

        // Advance cursor state
        offset += took;

        // If we consumed everything in this month,
        // move to next archive
        let month_total = raw_games
            .iter()
            .filter_map(|g| g.pgn.as_ref())
            .count();

        if offset >= month_total {
            archive_index += 1;
            offset = 0;
        }
    }

    // Build next cursor — None if no more archives
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_result_mappings() {
        // Test White Win conditions
        assert_eq!(
            normalize_result("win", "timeout"),
            (
                "1-0".to_string(),
                "1-0".to_string(),
                "1-0".to_string()
            )
        );
        assert_eq!(
            normalize_result("win", "resigned"),
            (
                "1-0".to_string(),
                "1-0".to_string(),
                "1-0".to_string()
            )
        );

        // Symmetrical Draw condition
        assert_eq!(
            normalize_result(
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
            normalize_result(
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
            normalize_result("checkmated", "win"),
            (
                "0-1".to_string(),
                "0-1".to_string(),
                "0-1".to_string()
            )
        );
        assert_eq!(
            normalize_result("timeout", "win"),
            (
                "0-1".to_string(),
                "0-1".to_string(),
                "0-1".to_string()
            )
        );
    }
}
