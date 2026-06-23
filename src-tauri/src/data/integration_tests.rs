use super::fetcher::*;
// Run manually using: cargo test -- --ignored --test-threads=1

#[tokio::test]
#[ignore = "Live Lichess API calls"]
async fn test_live_lichess_profile_and_games() {
    let client = reqwest::Client::new();
    let username = "penguingim1";

    // Test Profile
    let profile_res =
        fetch_lichess_profile(&client, username)
            .await;
    assert!(
        profile_res.is_ok(),
        "Lichess profile fetch failed"
    );
    let profile = profile_res.unwrap();
    assert_eq!(
        profile.username.to_lowercase(),
        username
    );

    // Test Games Stream
    let games_res =
        fetch_lichess_games(username, None).await;
    assert!(
        games_res.is_ok(),
        "Lichess games fetch failed"
    );
    let fetch_result = games_res.unwrap();
    assert!(
        !fetch_result.games.is_empty(),
        "Expected to fetch at least one game"
    );
}

#[tokio::test]
#[ignore = "Live Chess.com API calls"]
async fn test_live_chesscom_profile_and_games() {
    let client = reqwest::Client::new();
    let username = "hikaru";

    // Test Profile
    let profile_res =
        fetch_chesscom_profile(&client, username)
            .await;
    assert!(
        profile_res.is_ok(),
        "Chess.com profile fetch failed"
    );
    let profile = profile_res.unwrap();
    assert_eq!(
        profile.username.to_lowercase(),
        username
    );

    // Test Games (Initial Fetch)
    let games_res =
        fetch_chesscom_games(username, None)
            .await;
    assert!(
        games_res.is_ok(),
        "Chess.com games fetch failed"
    );
    let fetch_result = games_res.unwrap();

    // Assert we got games and a cursor for pagination
    assert!(
        !fetch_result.games.is_empty(),
        "Expected to fetch at least one game"
    );
    assert!(fetch_result.cursor.is_some(), "Expected a pagination cursor from Chess.com");
}
