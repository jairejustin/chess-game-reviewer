use crate::models::engine_config::{
    write_config_options, EngineConfig,
};
use crate::uci::evaluation::{
    engine_to_white_pov, format_eval,
};
use crate::uci::uci_engine::{
    Evaluation, UciEngine,
};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;

use tauri::{AppHandle, Emitter};

#[derive(Debug)]
pub enum LiveCommand {
    Start { binary_path: String },
    Stop,
    Analyze { fen: String, multipv: u8 },
    Terminate,
    Configure { config: EngineConfig },
}

pub struct LiveEngineManager {
    pub tx: mpsc::UnboundedSender<LiveCommand>,
}

#[derive(Clone, Serialize)]
pub struct LivePayload {
    pub fen: String,
    pub depth: usize,
    pub multipv: usize,
    pub evaluation: String,
    pub pv: Vec<String>,
}

pub fn init_live_manager(
    app_handle: AppHandle,
    initial_config: Arc<Mutex<EngineConfig>>,
) -> LiveEngineManager {
    let (boot_tx, boot_rx) =
        std::sync::mpsc::channel::<
            mpsc::UnboundedSender<LiveCommand>,
        >();

    std::thread::spawn(move || {
        let rt = Runtime::new().expect(
            "failed to build Tokio runtime for live engine",
        );

        rt.block_on(async move {
            let (tx, mut rx) =
                mpsc::unbounded_channel::<LiveCommand>();
            boot_tx
                .send(tx)
                .expect("failed to send engine tx to main thread");

            // engine_tx: the channel into the stdin loop for the currently live engine.
            // None means no engine is running.
            let mut engine_tx: Option<
                mpsc::UnboundedSender<LiveCommand>,
            > = None;

            // Written by the stdin loop when it sends a position, read by the stdout loop
            // when it normalises evals. Both loops reference the same Arc.
            let is_white_turn =
                Arc::new(AtomicBool::new(true));

            // The binary path we used to start the engine,
            // tracked in case we have to restart it.
            let mut current_binary: Option<String> =
                None;

            let current_fen: Arc<Mutex<String>> =
                Arc::new(Mutex::new(String::new()));

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    // Spawn the engine process if not already running
                    LiveCommand::Start {
                        binary_path,
                    } => {
                        if engine_tx.is_some() {
                            continue; // already running
                        }
                        current_binary =
                            Some(binary_path.clone());
                        let config = initial_config
                            .lock()
                            .unwrap()
                            .clone();
                        engine_tx = Some(spawn_engine(
                            &binary_path,
                            app_handle.clone(),
                            is_white_turn.clone(),
                            current_fen.clone(),
                            &config,
                        ));
                    }

                    // Pauses searching whilst keeping the engine process alive
                    LiveCommand::Stop => {
                        if let Some(tx) = &engine_tx {
                            let _ = tx
                                .send(LiveCommand::Stop);
                        }
                    }

                    // Sends a new position to search
                    LiveCommand::Analyze {
                        fen: _,
                        multipv: _,
                    } => {
                        // If the engine died (stdout loop exited -> tx is closed), restart it
                        // transparently before forwarding the command.
                        if let Some(tx) = &engine_tx {
                            if tx.is_closed() {
                                eprintln!("[live_manager] engine channel closed — restarting");
                                engine_tx = None;
                            }
                        }

                        if engine_tx.is_none() {
                            if let Some(ref path) =
                                current_binary.clone()
                            {
                                let config =
                                    initial_config
                                        .lock()
                                        .unwrap()
                                        .clone();
                                engine_tx =
                                    Some(spawn_engine(
                                        path,
                                        app_handle.clone(),
                                        is_white_turn
                                            .clone(),
                                        current_fen
                                            .clone(),
                                        &config,
                                    ));
                            } else {
                                eprintln!("[live_manager] Analyze received but no engine binary known");
                                continue;
                            }
                        }

                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(cmd);
                        }
                    }

                    // Applies new options to a running engine without restarting it.
                    // If the engine is not running, the updated config will be picked up
                    // automatically the next time it is started.
                    LiveCommand::Configure { config } => {
                        // Persist so restarts pick up the new values
                        *initial_config.lock().unwrap() =
                            config.clone();

                        if let Some(tx) = &engine_tx {
                            if !tx.is_closed() {
                                let _ = tx.send(
                                    LiveCommand::Configure {
                                        config,
                                    },
                                );
                            }
                        }
                    }

                    // Terminates the engine process entirely
                    LiveCommand::Terminate => {
                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(
                                LiveCommand::Terminate,
                            );
                        }
                        engine_tx = None;
                    }
                }
            }
        });
    });

    let tx = boot_rx.recv().expect(
        "live engine thread failed to send tx",
    );
    LiveEngineManager { tx }
}

/// Spawns a fresh engine process and returns the sender for its stdin loop.
fn spawn_engine(
    binary_path: &str,
    app_handle: AppHandle,
    is_white_turn: Arc<AtomicBool>,
    current_fen: Arc<Mutex<String>>,
    config: &EngineConfig,
) -> mpsc::UnboundedSender<LiveCommand> {
    let engine =
        UciEngine::new(binary_path, config);
    let (_child, stdin, stdout) = engine.unpack();
    let (internal_tx, internal_rx) =
        mpsc::unbounded_channel();

    // The atomic barrier ensures synchronization between stdin/stdout loops
    let ready_barrier =
        Arc::new(AtomicBool::new(true));

    spawn_stdin_loop(
        stdin,
        internal_rx,
        is_white_turn.clone(),
        ready_barrier.clone(),
        current_fen.clone(),
    );
    spawn_stdout_loop(
        stdout,
        app_handle,
        is_white_turn,
        current_fen,
        ready_barrier,
    );

    internal_tx
}

fn spawn_stdin_loop(
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
                    // Captures whose turn it is before writing to stdin.
                    // The stdout loop reads this flag while processing the
                    // replies that come back for this position, so we must
                    // set it before "go infinite" is flushed.
                    let is_white = fen
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("w")
                        == "w";
                    is_white_turn.store(
                        is_white,
                        Ordering::SeqCst,
                    );

                    // Lock the barrier
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

                    // Spin-wait until the stdout loop unlocks the barrier upon seeing "readyok"
                    while !ready_barrier
                        .load(Ordering::SeqCst)
                    {
                        std::thread::sleep(
                            Duration::from_millis(
                                1,
                            ),
                        );
                    }

                    // Update the FEN lock safely
                    *current_fen
                        .lock()
                        .unwrap() = fen.clone();

                    writeln!(
                        stdin,
                        "setoption name MultiPV value {}",
                        multipv
                    )
                    .and_then(|_| {
                        writeln!(
                            stdin,
                            "position fen {}",
                            fen
                        )
                    })
                    .and_then(|_| {
                        writeln!(stdin, "go infinite")
                    })
                    .and_then(|_| stdin.flush())
                }

                LiveCommand::Configure {
                    config,
                } => {
                    // Stop any running search first, setoption is invalid mid-search
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

                    // Wait for engine to confirm it is idle
                    while !ready_barrier
                        .load(Ordering::SeqCst)
                    {
                        std::thread::sleep(
                            Duration::from_millis(
                                1,
                            ),
                        );
                    }

                    // Apply the new options
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

                LiveCommand::Start { .. } => {
                    Ok(())
                }
            };

            if let Err(e) = result {
                eprintln!(
                    "[live_manager] stdin write error: {}",
                    e
                );
                break;
            }
        }
    });
}

fn spawn_stdout_loop(
    stdout: BufReader<std::process::ChildStdout>,
    app_handle: AppHandle,
    is_white_turn: Arc<AtomicBool>,
    current_fen: Arc<Mutex<String>>,
    ready_barrier: Arc<AtomicBool>,
) {
    task::spawn_blocking(move || {
        let mut reader = stdout;
        let mut line = String::new();
        let mut last_emit_times: HashMap<
            usize,
            Instant,
        > = HashMap::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    eprintln!("[live_manager] engine stdout closed (EOF)");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "[live_manager] stdout read error: {}",
                        e
                    );
                    break;
                }
            }

            let trimmed = line.trim();

            if trimmed == "readyok" {
                // Instantly unblock the stdin loop
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

                // Ignores bound updates with no PVs, unless it's a mate.
                // When Stockfish finds the same pos on transposition table,
                // it's known to just omit the PV and answer instantly from cache.
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

                // Bypasses throttle for mates. Forced mates cause Stockfish
                // to stop searching, so we cannot risk dropping its final output.
                if is_mate
                    || now
                        .duration_since(last_emit)
                        >= Duration::from_millis(
                            50,
                        )
                {
                    let fen = current_fen
                        .lock()
                        .unwrap()
                        .clone();

                    if !fen.is_empty() {
                        let payload = LivePayload {
                            fen: fen.clone(),
                            depth,
                            multipv,
                            evaluation: format_eval(
                                normalized_eval,
                            ),
                            pv: pv_moves,
                        };

                        let _ = app_handle.emit(
                            "live-engine-info",
                            payload,
                        );
                        last_emit_times
                            .insert(multipv, now);
                    }
                }
            }
        }
    });
}
