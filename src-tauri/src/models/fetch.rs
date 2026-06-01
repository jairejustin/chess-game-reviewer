use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone,
)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    ChessCom,
    Lichess,
}

#[derive(
    Serialize, Deserialize, Debug, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct ChessComCursor {
    pub archive_index: usize,
    pub offset: usize,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub username: String,
    pub title: Option<String>,
    pub avatar_url: Option<String>,
    pub country_code: Option<String>,
    pub joined: Option<u64>,
    pub last_online: Option<u64>,
    pub followers: Option<u64>,
    pub is_streamer: Option<bool>,
    pub twitch_url: Option<String>,
    pub fide: Option<u64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayer {
    pub username: String,
    pub rating: Option<u32>,
    pub result: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub id: String,
    pub pgn: String,
    pub platform: Platform,
    pub time_class: String,
    pub time_control: Option<String>,
    pub played_at: u64,
    pub rated: bool,
    pub white: GamePlayer,
    pub black: GamePlayer,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub profile: PlayerProfile,
    pub games: Vec<GameSummary>,
    pub cursor: Option<ChessComCursor>,
}

#[derive(Deserialize)]
pub struct RawProfile {
    pub username: String,
    pub avatar: Option<String>,
    pub title: Option<String>,
    pub country: Option<String>,
    pub joined: Option<u64>,
    pub last_online: Option<u64>,
    pub followers: Option<u64>,
    pub is_streamer: Option<bool>,
    pub twitch_url: Option<String>,
    pub fide: Option<u64>,
}

#[derive(Deserialize)]
pub struct RawArchives {
    pub archives: Vec<String>,
}

#[derive(Deserialize)]
pub struct RawMonthlyGames {
    pub games: Vec<RawGame>,
}

#[derive(Deserialize)]
pub struct RawGame {
    pub url: Option<String>,
    pub pgn: Option<String>,
    pub time_class: Option<String>,
    pub time_control: Option<String>,
    pub end_time: Option<u64>,
    pub rated: Option<bool>,
    pub white: Option<RawGamePlayer>,
    pub black: Option<RawGamePlayer>,
}

#[derive(Deserialize)]
pub struct RawGamePlayer {
    pub username: Option<String>,
    pub rating: Option<u32>,
    pub result: Option<String>,
}
