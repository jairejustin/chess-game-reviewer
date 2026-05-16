use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead, Write};

pub fn test_engine_communication() {
    let binary = "/home/j/Documents/Personal Projects/chess-analyze/src-tauri/binaries/theoria-x86_64-unknown-linux-gnu";

    let mut engine = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn engine");

    let stdin = engine.stdin.as_mut().unwrap();
    
    let mut stdout = BufReader::new(engine.stdout.take().unwrap());

    writeln!(stdin, "uci").unwrap();
    for line in (&mut stdout).lines() {
        let line = line.unwrap();
        println!("engine: {}", line);
        if line == "uciok" { break; }
    }
    
    writeln!(stdin, "isready").unwrap();
    for line in (&mut stdout).lines() {
        let line = line.unwrap();
        println!("engine: {}", line);
        if line == "readyok" { break; }
    }

    writeln!(stdin, "position startpos").unwrap();
    writeln!(stdin, "go depth 10").unwrap();
    for line in (&mut stdout).lines() {
        let line = line.unwrap();
        println!("engine: {}", line);
        if line.starts_with("bestmove") { break; }
    }

    writeln!(stdin, "quit").unwrap();
    engine.wait().unwrap();
}