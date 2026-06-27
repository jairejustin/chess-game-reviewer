use crate::models::engine_config::write_config_options;
use crate::models::live::LivePayload;
use crate::uci::evaluation::{
    engine_to_white_pov, format_eval,
};
use crate::uci::live_manager::LiveCommand;
use crate::uci::uci_engine::{
    Evaluation, UciEngine,
};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::task;

/// Spawns the stdin writer task for a live engine session.
///
/// Receives `LiveCommand`s from `internal_rx` and translates them into UCI
/// protocol writes on `stdin`. Coordinates position-change synchronisation
/// with the stdout loop via `ready_barrier` and `is_white_turn`.
pub fn spawn_stdin_loop(
    mut stdin: std::process::ChildStdin,
    mut rx: mpsc::UnboundedReceiver<LiveCommand>,
    is_white_turn: Arc<AtomicBool>,
    ready_barrier: Arc<AtomicBool>,
    current_fen: Arc<Mutex<String>>,
) {
    task::spawn_blocking(move || {
        while let Some(cmd) = rx.blocking_recv() {
            let result = match cmd {
                LiveCommand::Analyze {
                    fen,
                    multipv,
                } => {
                    // Record whose turn it is before flushing "go infinite" so the
                    // stdout loop normalises evals to the correct POV.
                    let is_white = fen
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("w")
                        == "w";
                    is_white_turn.store(
                        is_white,
                        Ordering::SeqCst,
                    );

                    // Lock the barrier: stdout loop must not emit until "readyok".
                    ready_barrier.store(
                        false,
                        Ordering::SeqCst,
                    );

                    let write_result =
                        writeln!(stdin, "stop")
                            .and_then(|_| {
                                writeln!(
                                    stdin,
                                    "isready"
                                )
                            })
                            .and_then(|_| {
                                stdin.flush()
                            });

                    if write_result.is_err() {
                        break;
                    }

                    // Spin-wait until stdout loop confirms engine is idle.
                    while !ready_barrier
                        .load(Ordering::SeqCst)
                    {
                        std::thread::sleep(
                            Duration::from_millis(
                                1,
                            ),
                        );
                    }

                    *current_fen
                        .lock()
                        .unwrap() = fen.clone();

                    writeln!(stdin, "setoption name MultiPV value {}", multipv)
                        .and_then(|_| writeln!(stdin, "position fen {}", fen))
                        .and_then(|_| writeln!(stdin, "go infinite"))
                        .and_then(|_| stdin.flush())
                }

                LiveCommand::Configure {
                    config,
                } => {
                    // Stop any running search first; setoption is invalid mid-search.
                    ready_barrier.store(
                        false,
                        Ordering::SeqCst,
                    );

                    let write_result =
                        writeln!(stdin, "stop")
                            .and_then(|_| {
                                writeln!(
                                    stdin,
                                    "isready"
                                )
                            })
                            .and_then(|_| {
                                stdin.flush()
                            });

                    if write_result.is_err() {
                        break;
                    }

                    while !ready_barrier
                        .load(Ordering::SeqCst)
                    {
                        std::thread::sleep(
                            Duration::from_millis(
                                1,
                            ),
                        );
                    }

                    write_config_options(
                        &mut stdin, &config,
                    )
                    .and_then(|_| stdin.flush())
                }

                LiveCommand::Stop => {
                    writeln!(stdin, "stop")
                        .and_then(|_| {
                            stdin.flush()
                        })
                }

                LiveCommand::Terminate => {
                    let _ =
                        writeln!(stdin, "quit")
                            .and_then(|_| {
                                stdin.flush()
                            });
                    break;
                }

                // Start is handled by the manager; the stdin loop never sees it.
                LiveCommand::Start { .. } => {
                    Ok(())
                }
            };

            if let Err(e) = result {
                eprintln!("[live_io] stdin write error: {}", e);
                break;
            }
        }
    });
}

/// Spawns the stdout reader task for a live engine session.
///
/// Reads UCI info lines produced by the engine, normalises evaluations to
/// Absolute White POV, throttles output to 50 ms per PV line, and emits
/// `"live-engine-info"` events carrying `LivePayload` to the frontend.
/// Also unblocks the stdin loop on `"readyok"`.
pub fn spawn_stdout_loop(
    stdout: BufReader<std::process::ChildStdout>,
    app_handle: AppHandle,
    is_white_turn: Arc<AtomicBool>,
    current_fen: Arc<Mutex<String>>,
    ready_barrier: Arc<AtomicBool>,
) {
    task::spawn_blocking(move || {
        let mut reader = stdout;
        let mut line = String::new();
        // Per-multipv throttle: track the last time we emitted for each PV index.
        let mut last_emit_times: HashMap<
            usize,
            Instant,
        > = HashMap::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    eprintln!("[live_io] engine stdout closed (EOF)");
                    break;
                }
                Err(e) => {
                    eprintln!("[live_io] stdout read error: {}", e);
                    break;
                }
                Ok(_) => {}
            }

            let trimmed = line.trim();

            if trimmed == "readyok" {
                ready_barrier.store(
                    true,
                    Ordering::SeqCst,
                );
                last_emit_times.clear();
                continue;
            }

            if trimmed.starts_with("bestmove") {
                continue;
            }

            if let Some((
                depth,
                multipv,
                eval,
                pv_moves,
            )) = UciEngine::parse_info_line(
                trimmed,
            ) {
                let is_mate = matches!(
                    eval,
                    Evaluation::Mate(_)
                );

                // Skip bound updates with no PV unless it is a mate.
                // Stockfish sometimes omits the PV for transposition-table hits.
                if pv_moves.is_empty() && !is_mate
                {
                    continue;
                }

                let is_white = is_white_turn
                    .load(Ordering::SeqCst);

                let normalized_eval = match eval {
                    Evaluation::Cp(cp) => {
                        Evaluation::Cp(
                            engine_to_white_pov(
                                cp, is_white,
                            ),
                        )
                    }
                    Evaluation::Mate(m) => {
                        Evaluation::Mate(
                            engine_to_white_pov(
                                m, is_white,
                            ),
                        )
                    }
                };

                let now = Instant::now();
                let last_emit = last_emit_times
                    .get(&multipv)
                    .copied()
                    .unwrap_or_else(|| {
                        now - Duration::from_secs(
                            1,
                        )
                    });

                // Bypass throttle for mates: Stockfish stops searching after
                // finding a forced mate, so we cannot risk dropping that final line.
                let should_emit = is_mate
                    || now.duration_since(
                        last_emit,
                    ) >= Duration::from_millis(
                        50,
                    );

                if should_emit {
                    let fen = current_fen
                        .lock()
                        .unwrap()
                        .clone();

                    if !fen.is_empty() {
                        let _ = app_handle.emit(
                            "live-engine-info",
                            LivePayload {
                                fen,
                                depth,
                                multipv,
                                evaluation: format_eval(normalized_eval),
                                pv: pv_moves,
                            },
                        );
                        last_emit_times
                            .insert(multipv, now);
                    }
                }
            }
        }
    });
}
