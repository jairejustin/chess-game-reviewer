use super::uci_engine::*;
use crate::models::engine_config::EngineConfig;
use crate::uci::evaluation::engine_to_white_pov;

// Run with: cargo test -- --ignored --test-threads=1

fn engine_path() -> String {
    std::env::current_dir()
        .unwrap()
        .join("core/engine/stockfish-ubuntu-x86-64-bmi2")
        .to_string_lossy()
        .to_string()
}

fn default_engine() -> UciEngine {
    UciEngine::new(
        &engine_path(),
        &EngineConfig {
            threads: Some(1),
            ..EngineConfig::default()
        },
    )
}

// A UCI move is either 4 chars (e.g. "e2e4") or 5 for promotions (e.g. "e7e8q").
fn is_valid_uci_move(s: &str) -> bool {
    matches!(s.len(), 4 | 5)
        && s.chars().all(|c| c.is_ascii_alphanumeric())
}

// Startup and teardown

#[test]
#[ignore = "Requires Engine binary"]
fn engine_spawns_and_completes_uci_handshake() {
    let engine = default_engine();
    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn quit_shuts_down_cleanly() {
    let mut engine = default_engine();
    engine.send_command("isready");
    engine.quit();
}

// apply_config

#[test]
#[ignore = "Requires Engine binary"]
fn apply_config_with_non_default_values_does_not_break_engine() {
    let mut engine = default_engine();

    engine.apply_config(&EngineConfig {
        hash_mb: Some(128),
        threads: Some(1),
        multi_pv: Some(1),
        analysis_time_ms: None,
    });

    engine.send_command("isready");
    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn apply_config_with_all_none_is_a_noop() {
    let mut engine = default_engine();

    engine.apply_config(&EngineConfig {
        hash_mb: None,
        threads: None,
        multi_pv: None,
        analysis_time_ms: None,
    });

    engine.send_command("isready");
    engine.quit();
}

// analyze_position

#[test]
#[ignore = "Requires Engine binary"]
fn starting_position_returns_valid_best_move() {
    let mut engine = default_engine();

    let (_, best_move, _, _) = engine.analyze_position(
        "position startpos",
        "go movetime 100",
    );

    assert!(
        is_valid_uci_move(&best_move),
        "expected a valid UCI move, got: {:?}",
        best_move
    );

    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn starting_position_eval_is_near_zero() {
    let mut engine = default_engine();

    let (eval, _, _, _) = engine.analyze_position(
        "position startpos",
        "go movetime 100",
    );

    let cp = match eval {
        Evaluation::Cp(cp) => cp,
        Evaluation::Mate(_) => panic!(
            "engine returned a mate score from the starting position"
        ),
    };

    assert!(
        cp.abs() <= 50,
        "starting position eval should be near zero, got {} cp",
        cp
    );

    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn starting_position_returns_non_empty_pv() {
    let mut engine = default_engine();

    let (_, _, pv, _) = engine.analyze_position(
        "position startpos",
        "go movetime 100",
    );

    assert!(!pv.is_empty(), "PV should not be empty");
    assert!(
        pv.iter().all(|m| is_valid_uci_move(m)),
        "all PV moves should be valid UCI moves, got: {:?}",
        pv
    );

    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn mate_in_one_position_returns_mate_evaluation() {
    let mut engine = default_engine();

    // FEN: White to move, Qxf7# is the mating move
    let fen =
        "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4";

    let (eval, best_move, _, _) = engine.analyze_position(
        &format!("position fen {}", fen),
        "go movetime 500",
    );

    assert_eq!(
        eval,
        Evaluation::Mate(1),
        "expected Mate(1), got {:?}",
        eval
    );
    assert_eq!(
        best_move, "h5f7",
        "expected mating move h5f7, got {:?}",
        best_move
    );

    engine.quit();
}

#[test]
#[ignore = "Requires Engine binary"]
fn sequential_calls_do_not_bleed_state() {
    let mut engine = default_engine();

    let fen = "8/8/8/8/8/4k3/8/3qK3 b - - 0 1";
    let is_white = fen.split_whitespace().nth(1).unwrap_or("w") == "w";

    let (eval2, _, _, _) = engine.analyze_position(
        &format!("position fen {}", fen),
        "go movetime 100",
    );

    let normalized = match eval2 {
        Evaluation::Mate(m) => {
            Evaluation::Mate(engine_to_white_pov(m, is_white))
        }
        Evaluation::Cp(cp) => {
            Evaluation::Cp(engine_to_white_pov(cp, is_white))
        }
    };

    assert!(
        matches!(normalized, Evaluation::Mate(m) if m < 0),
        "Black delivering mate should be negative from White POV, got {:?}",
        normalized
    );
}