use crate::models::engine_config::EngineConfig;
use crate::uci::live_io::{
    spawn_stdin_loop, spawn_stdout_loop,
};
use crate::uci::uci_engine::UciEngine;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum LiveCommand {
    Start { binary_path: String },
    Stop,
    Analyze { fen: String, multipv: u8 },
    Terminate,
    Configure { config: EngineConfig },
}

/// Handle to the live engine manager.
/// Clone `tx` to send `LiveCommand`s from any Tauri command handler.
pub struct LiveEngineManager {
    pub tx: mpsc::UnboundedSender<LiveCommand>,
}

/// Starts the background Tokio runtime and event loop that owns the live engine.
///
/// Returns a `LiveEngineManager` whose `tx` the Tauri command layer uses to
/// send `LiveCommand`s. The manager loop is the only place that spawns or
/// terminates engine processes; all callers go through the channel.
pub fn init_live_manager(
    app_handle: tauri::AppHandle,
    initial_config: Arc<Mutex<EngineConfig>>,
) -> LiveEngineManager {
    // One-shot channel: the manager thread sends back its command sender once
    // the Tokio runtime is up, so `init_live_manager` can return it synchronously.
    let (boot_tx, boot_rx) =
        std::sync::mpsc::channel::<
            mpsc::UnboundedSender<LiveCommand>,
        >();

    std::thread::spawn(move || {
        let rt = Runtime::new()
            .expect("failed to build Tokio runtime for live engine");

        rt.block_on(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<LiveCommand>();
            boot_tx
                .send(tx)
                .expect("failed to send engine tx to main thread");

            // `engine_tx`: command sender into the stdin loop of the running engine.
            // `None` means no engine process is currently alive.
            let mut engine_tx: Option<mpsc::UnboundedSender<LiveCommand>> = None;

            // Shared between the stdin and stdout loops so each knows whose turn it is.
            let is_white_turn = Arc::new(AtomicBool::new(true));

            // Path used to start the current engine; retained so a crashed engine
            // can be transparently restarted on the next Analyze command.
            let mut current_binary: Option<String> = None;

            // The FEN currently loaded in the engine, shared with the stdout loop
            // so emitted payloads carry the correct position.
            let current_fen: Arc<Mutex<String>> =
                Arc::new(Mutex::new(String::new()));

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    LiveCommand::Start { binary_path } => {
                        if engine_tx.is_some() {
                            continue; // already running
                        }
                        current_binary = Some(binary_path.clone());
                        let config = initial_config.lock().unwrap().clone();
                        engine_tx = Some(spawn_engine(
                            &binary_path,
                            app_handle.clone(),
                            is_white_turn.clone(),
                            current_fen.clone(),
                            &config,
                        ));
                    }

                    LiveCommand::Stop => {
                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(LiveCommand::Stop);
                        }
                    }

                    LiveCommand::Analyze { .. } => {
                        // Restart transparently if the engine crashed (channel closed).
                        if engine_tx.as_ref().map(|t| t.is_closed()).unwrap_or(false) {
                            eprintln!(
                                "[live_manager] engine channel closed — restarting"
                            );
                            engine_tx = None;
                        }

                        if engine_tx.is_none() {
                            match &current_binary.clone() {
                                Some(path) => {
                                    let config =
                                        initial_config.lock().unwrap().clone();
                                    engine_tx = Some(spawn_engine(
                                        path,
                                        app_handle.clone(),
                                        is_white_turn.clone(),
                                        current_fen.clone(),
                                        &config,
                                    ));
                                }
                                None => {
                                    eprintln!(
                                        "[live_manager] Analyze received but no engine binary known"
                                    );
                                    continue;
                                }
                            }
                        }

                        if let Some(tx) = &engine_tx {
                            let _ = tx.send(cmd);
                        }
                    }

                    LiveCommand::Configure { config } => {
                        // Persist so any future restart picks up the new values.
                        *initial_config.lock().unwrap() = config.clone();

                        if let Some(tx) = &engine_tx {
                            if !tx.is_closed() {
                                let _ =
                                    tx.send(LiveCommand::Configure { config });
                            }
                        }
                    }

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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Spawns a fresh engine process and its I/O loops, returning the command sender
/// that the manager uses to drive that engine instance.
fn spawn_engine(
    binary_path: &str,
    app_handle: tauri::AppHandle,
    is_white_turn: Arc<AtomicBool>,
    current_fen: Arc<Mutex<String>>,
    config: &EngineConfig,
) -> mpsc::UnboundedSender<LiveCommand> {
    let engine =
        UciEngine::new(binary_path, config);
    let (_child, stdin, stdout) = engine.unpack();
    let (internal_tx, internal_rx) =
        mpsc::unbounded_channel();

    // The barrier starts `true` (engine is idle and ready).
    // The stdin loop flips it to `false` before sending "isready",
    // and the stdout loop flips it back to `true` on "readyok".
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
