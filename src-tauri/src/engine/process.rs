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
        if let Some((score, pv)) = parse_info_line(&line) {
            println!("score: {} pv: {:?}", score, pv);
        }
        if let Some(best) = parse_bestmove(&line) {
            println!("bestmove: {}", best);
            break;
        }
    }

    writeln!(stdin, "quit").unwrap();
    engine.wait().unwrap();
}

/// Parses an info line to extract the centipawn score and principal variation
pub fn parse_info_line(line: &str) -> Option<(i32, Vec<String>)> {
    let words: Vec<&str> = line.split_whitespace().collect();

    // check for "info"
    if words.first() != Some(&"info") {
        return None;
    }

    // get index of "cp" and parse the next word as the score
    let cp_idx = words.iter().position(|&w| w == "cp")?;
    let score = words.get(cp_idx + 1)?.parse::<i32>().ok()?;

    // get index of "pv" and grab everything after it as the variation
    let pv_idx = words.iter().position(|&w| w == "pv")?;
    let pv_moves: Vec<String> = words[pv_idx + 1..]
        .iter()
        .map(|&s| s.to_string())
        .collect();

    Some((score, pv_moves))
}

/// Parses a bestmove line to extract the suggested move
pub fn parse_bestmove(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split_whitespace().collect();

    // check for "bestmove"
    if words.first() != Some(&"bestmove") {
        return None;
    }

    // extract the second word if it exists
    words.get(1).map(|&s| s.to_string())
}