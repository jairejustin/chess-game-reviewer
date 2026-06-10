use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;

use tauri::{AppHandle, Emitter};

use crate::uci::evaluation::{
    engine_to_white_pov, format_eval,
};
use crate::uci::uci_engine::{
    Evaluation, UciEngine,
};

#[derive(Debug)]
pub enum LiveCommand {
    Start { binary_path: String },
    Stop,
    Analyze { fen: String, multipv: u8 },
    Terminate,
}

pub struct LiveEngineManager {
    pub tx: mpsc::UnboundedSender<LiveCommand>,
}

#[derive(Clone, Serialize)]
pub struct LivePayload {
    pub multipv: usize,
    pub evaluation: String,
    pub pv: Vec<String>,
}

pub fn init_live_manager(
    app_handle: AppHandle,
) -> LiveEngineManager {
    let (boot_tx, boot_rx) =
        std::sync::mpsc::channel::<
            mpsc::UnboundedSender<LiveCommand>,
        >();

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("failed to build Tokio runtime for live engine");

        rt.block_on(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<LiveCommand>();
            boot_tx.send(tx).expect("failed to send engine tx to main thread");

            // engine_tx: the channel into the stdin loop for the currently live engine.
            // None means no engine is running.
            let mut engine_tx: Option<mpsc::UnboundedSender<LiveCommand>> = None;

            // Written by the stdin loop when it sends a position, read by the stdout loop
            // when it normalises evals. Both loops reference the same Arc.
            let is_white_turn = Arc::new(AtomicBool::new(true));

            // The binary path we used to start the engine, 
            // it's tracked in case we have to restart it.
            let mut current_binary: Option<String> = None;

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    // Spawn the engine process if not already running
                    LiveCommand::Start { binary_path } => {
                        if engine_tx.is_some() {
                            continue; // already running
                        }

                        current_binary = Some(binary_path.clone());
                        engine_tx = Some(spawn_engine(
                            &binary_path,
                            app_handle.clone(),
                            is_white_turn.clone(),
                        ));
                    }

                    // Pauses searching whilt keeping the engine process alive
                    LiveCommand::Stop => {
                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(LiveCommand::Stop);
                        }
                    }

                    // Sends a new position to search
                    LiveCommand::Analyze { fen, multipv } => {
                        // If the engine died (stdout loop exited -> tx is closed), restart it
                        // transparently before forwarding the command.
                        if let Some(tx) = &engine_tx {
                            if tx.is_closed() {
                                eprintln!("[live_manager] engine channel closed — restarting");
                                engine_tx = None;
                            }
                        }

                        if engine_tx.is_none() {
                            if let Some(ref path) = current_binary.clone() {
                                engine_tx = Some(spawn_engine(
                                    path,
                                    app_handle.clone(),
                                    is_white_turn.clone(),
                                ));
                            } else {
                                eprintln!("[live_manager] Analyze received but no engine binary known");
                                continue;
                            }
                        }

                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(LiveCommand::Analyze { fen, multipv });
                        }
                    }

                    // Terminates the engine process entirely
                    LiveCommand::Terminate => {
                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(LiveCommand::Terminate);
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
) -> mpsc::UnboundedSender<LiveCommand> {
    let engine = UciEngine::new(binary_path);
    let (_child, stdin, stdout) = engine.unpack();
    let (internal_tx, internal_rx) =
        mpsc::unbounded_channel();

    spawn_stdin_loop(
        stdin,
        internal_rx,
        is_white_turn.clone(),
    );
    spawn_stdout_loop(
        stdout,
        app_handle,
        is_white_turn,
    );

    internal_tx
}

fn spawn_stdin_loop(
    mut stdin: std::process::ChildStdin,
    mut rx: mpsc::UnboundedReceiver<LiveCommand>,
    is_white_turn: Arc<AtomicBool>,
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
                    ); // SeqCst: visible to stdout thread immediately

                    writeln!(stdin, "stop")
                        .and_then(|_| writeln!(stdin, "setoption name MultiPV value {}", multipv))
                        .and_then(|_| writeln!(stdin, "position fen {}", fen))
                        .and_then(|_| writeln!(stdin, "go infinite"))
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
                eprintln!("[live_manager] stdin write error: {}", e);
                break;
            }
        }
    });
}

fn spawn_stdout_loop(
    stdout: BufReader<std::process::ChildStdout>,
    app_handle: AppHandle,
    is_white_turn: Arc<AtomicBool>,
) {
    task::spawn_blocking(move || {
        let mut reader = stdout;
        let mut line = String::new();
        let mut last_emit_times: std::collections::HashMap<usize, Instant> =
            std::collections::HashMap::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF: engine process has exited on a
                    // terminal position). This is expected; just exit the loop cleanly.
                    eprintln!("[live_manager] engine stdout closed (EOF)");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[live_manager] stdout read error: {}", e);
                    break;
                }
            }

            let trimmed = line.trim();

            // Skip "bestmove (none)", ut just means a terminal position, engine has nothing to say.
            // Its where mate is already played or calculated such that engine doesn't even bother
            // analyzing the position then it just completely pauses.
            if trimmed.starts_with("bestmove") {
                continue;
            }

            if let Some((
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

                // Ignores bound updates with no PVs, unless its a mate.
                // When Stockfish finds the same pos on transposition table,
                // its known to just omit the PV and answer instantly from cache.
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
                    let payload = LivePayload {
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
    });
}
