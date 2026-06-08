use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;

use tauri::{AppHandle, Emitter};

use crate::uci::evaluation::{format_eval, engine_to_white_pov};
use crate::uci::uci_engine::{Evaluation, UciEngine};

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

pub fn init_live_manager(app_handle: AppHandle) -> LiveEngineManager {
    let (boot_tx, boot_rx) = std::sync::mpsc::channel::<mpsc::UnboundedSender<LiveCommand>>();

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("failed to build Tokio runtime for live engine");

        rt.block_on(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<LiveCommand>();
            boot_tx.send(tx).expect("failed to send engine tx to main thread");

            let mut stdin_tx: Option<mpsc::UnboundedSender<LiveCommand>> = None;
            
            // Shared state to track whose turn it is for the active analysis
            let is_white_turn = Arc::new(AtomicBool::new(true));

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    LiveCommand::Start { binary_path } => {
                        if stdin_tx.is_some() {
                            continue;
                        }

                        let engine = task::spawn_blocking(move || UciEngine::new(&binary_path))
                            .await
                            .expect("UCI engine boot task panicked");

                        let (_child, stdin, stdout) = engine.unpack();
                        let (internal_tx, internal_rx) = mpsc::unbounded_channel();
                        stdin_tx = Some(internal_tx);

                        spawn_stdin_loop(stdin, internal_rx, is_white_turn.clone());
                        spawn_stdout_loop(stdout, app_handle.clone(), is_white_turn.clone());
                    }
                    LiveCommand::Stop => {
                        if let Some(tx) = &stdin_tx {
                            let _ = tx.send(LiveCommand::Stop);
                        }
                    }
                    LiveCommand::Analyze { fen, multipv } => {
                        if let Some(tx) = &stdin_tx {
                            let _ = tx.send(LiveCommand::Analyze { fen, multipv });
                        }
                    }
                    LiveCommand::Terminate => {
                        if let Some(tx) = &stdin_tx {
                            let _ = tx.send(LiveCommand::Terminate);
                        }
                        stdin_tx = None;
                    }
                }
            }
        });
    });

    let tx = boot_rx.recv().expect("live engine thread failed to send tx");
    LiveEngineManager { tx }
}

fn spawn_stdin_loop(
    mut stdin: std::process::ChildStdin,
    mut rx: mpsc::UnboundedReceiver<LiveCommand>,
    is_white_turn: Arc<AtomicBool>,
) {
    task::spawn_blocking(move || {
        while let Some(cmd) = rx.blocking_recv() {
            let result = match cmd {
                LiveCommand::Analyze { fen, multipv } => {
                    // Update the turn tracker for the stdout loop normalization
                    let is_white = fen.split_whitespace().nth(1).unwrap_or("w") == "w";
                    is_white_turn.store(is_white, Ordering::Relaxed);

                    writeln!(stdin, "stop")
                        .and_then(|_| writeln!(stdin, "setoption name MultiPV value {}", multipv))
                        .and_then(|_| writeln!(stdin, "position fen {}", fen))
                        .and_then(|_| writeln!(stdin, "go infinite"))
                        .and_then(|_| stdin.flush())
                }
                LiveCommand::Stop => writeln!(stdin, "stop").and_then(|_| stdin.flush()),
                LiveCommand::Terminate => {
                    let _ = writeln!(stdin, "quit").and_then(|_| stdin.flush());
                    break;
                }
                LiveCommand::Start { .. } => Ok(()),
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
        
        // Track the last emit time individually for each MultiPV index
        let mut last_emit_times = std::collections::HashMap::new();

        loop {
            match reader.read_line(&mut line) {
                Ok(0) => break, 
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[live_manager] stdout read error: {}", e);
                    break;
                }
            }

            let trimmed = line.trim();

            if let Some((multipv, eval, pv_moves)) = UciEngine::parse_info_line(trimmed) {
                // Ignore engine bounds updates that do not contain PV lines
                if pv_moves.is_empty() {
                    line.clear();
                    continue;
                }

                // Lock the engine score strictly to Absolute White POV
                let is_white = is_white_turn.load(Ordering::Relaxed);
                let normalized_eval = match eval {
                    Evaluation::Cp(cp) => Evaluation::Cp(engine_to_white_pov(cp, is_white)),
                    Evaluation::Mate(m) => Evaluation::Mate(engine_to_white_pov(m, is_white)),
                };

                let now = Instant::now();
                let last_emit = last_emit_times
                    .get(&multipv)
                    .copied()
                    .unwrap_or_else(|| now - Duration::from_secs(1));

                if now.duration_since(last_emit) >= Duration::from_millis(50) {
                    let payload = LivePayload {
                        multipv,
                        evaluation: format_eval(normalized_eval),
                        pv: pv_moves,
                    };

                    let _ = app_handle.emit("live-engine-info", payload);
                    last_emit_times.insert(multipv, now);
                }
            }

            line.clear();
        }
    });
}