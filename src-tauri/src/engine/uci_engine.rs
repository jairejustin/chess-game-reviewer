use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::io::{BufReader, BufRead, Write };

#[derive(Debug, Clone, PartialEq)]
pub enum Evaluation {
    Cp(i32),
    Mate(i32),
}

pub struct UciEngine {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl UciEngine {
    pub fn new(binary_path: &str) -> Self {
        let mut process = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn engine");

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        let mut engine = Self { process, stdin, stdout };
        engine.init();
        engine
    }

    fn init(&mut self) {
        self.send_command("uci");
        self.wait_for("uciok");
        self.send_command("isready");
        self.wait_for("readyok");
    }

    pub fn send_command(&mut self, cmd: &str) {
        writeln!(self.stdin, "{}", cmd).unwrap();
        self.stdin.flush().unwrap();
    }

    fn wait_for(&mut self, target: &str) {
        let mut line = String::new();
        while self.stdout.read_line(&mut line).unwrap() > 0 {
            if line.trim() == target { break; }
            line.clear();
        }
    }

    pub fn analyze_position(&mut self, fen: &str, depth: u8) -> (Evaluation, String, Vec<String>) {
        self.send_command(&format!("position fen {}", fen));
        self.send_command(&format!("go depth {}", depth));

        let mut last_eval = Evaluation::Cp(0);
        let mut last_pv = Vec::new();
        let mut best_move = String::new();

        let mut line = String::new();
        while self.stdout.read_line(&mut line).unwrap() > 0 {
            let trimmed = line.trim();
            
            if let Some((eval, pv)) = Self::parse_info_line(trimmed) {
                last_eval = eval;
                last_pv = pv;
            } else if let Some(bm) = Self::parse_bestmove(trimmed) {
                best_move = bm;
                break;
            }
            line.clear();
        }

        (last_eval, best_move, last_pv)
    }

    pub fn quit(mut self) {
        self.send_command("quit");
        self.process.wait().unwrap();
    }


    fn parse_info_line(line: &str) -> Option<(Evaluation, Vec<String>)> {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.first() != Some(&"info") { return None; }

        let eval = if let Some(cp_idx) = words.iter().position(|&w| w == "cp") {
            Evaluation::Cp(words.get(cp_idx + 1)?.parse().ok()?)
        } else if let Some(mate_idx) = words.iter().position(|&w| w == "mate") {
            Evaluation::Mate(words.get(mate_idx + 1)?.parse().ok()?)
        } else {
            return None;
        };

        let pv_moves = if let Some(pv_idx) = words.iter().position(|&w| w == "pv") {
            words[pv_idx + 1..]
                .iter()
                .map(|&s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        Some((eval, pv_moves))
    }

    fn parse_bestmove(line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.first() != Some(&"bestmove") { return None; }
        words.get(1).map(|&s| s.to_string())
    }
}