use crate::models::engine_config::{
    write_config_options, EngineConfig,
};
use std::io::{BufRead, BufReader, Write};
use std::process::{
    Child, ChildStdin, ChildStdout, Command,
    Stdio,
};

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
    pub fn new(
        binary_path: &str,
        config: &EngineConfig,
    ) -> Self {
        let mut process =
            Command::new(binary_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn engine");

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(
            process.stdout.take().unwrap(),
        );

        let mut engine = Self {
            process,
            stdin,
            stdout,
        };
        engine.init(config);
        engine
    }

    fn init(&mut self, config: &EngineConfig) {
        self.send_command("uci");
        self.wait_for("uciok");

        // explicitly disable ponder
        self.send_command(
            "setoption name Ponder value false",
        );

        // Apply all user-configured options
        self.apply_config(config);

        self.send_command("isready");
        self.wait_for("readyok");
    }

    /// Applies every `Some` field in `config` to a running engine.
    ///
    /// The caller must ensure no search is active before calling this
    /// (send "stop" / wait for "readyok" first).
    pub fn apply_config(
        &mut self,
        config: &EngineConfig,
    ) {
        write_config_options(
            &mut self.stdin,
            config,
        )
        .expect("failed to write engine config");
        self.stdin.flush().expect(
            "failed to flush stdin after config",
        );
    }

    pub fn send_command(&mut self, cmd: &str) {
        writeln!(self.stdin, "{}", cmd).unwrap();
        self.stdin.flush().unwrap();
    }

    fn wait_for(&mut self, target: &str) {
        let mut line = String::new();
        while self
            .stdout
            .read_line(&mut line)
            .unwrap()
            > 0
        {
            if line.trim() == target {
                break;
            }
            line.clear();
        }
    }

    /// Destructures the initialized UciEngine, giving up ownership of the underlying
    /// process and piped streams to be managed by asynchronous tokio tasks.
    pub fn unpack(
        self,
    ) -> (Child, ChildStdin, BufReader<ChildStdout>)
    {
        (self.process, self.stdin, self.stdout)
    }

    pub fn analyze_position(
        &mut self,
        position_cmd: &str,
        go_cmd: &str,
    ) -> (Evaluation, String, Vec<String>, Vec<i32>)
    {
        self.send_command(position_cmd);
        self.send_command(go_cmd);

        let mut last_eval = Evaluation::Cp(0);
        let mut last_pv = Vec::new();
        let mut best_move = String::new();
        let mut multi_pv_evals = vec![0; 2];

        let mut line = String::new();
        while self
            .stdout
            .read_line(&mut line)
            .unwrap()
            > 0
        {
            let trimmed = line.trim();

            if let Some((
                _depth,
                multipv,
                eval,
                pv,
            )) = Self::parse_info_line(trimmed)
            {
                let cp = match eval {
                    Evaluation::Cp(c) => c,
                    Evaluation::Mate(m) => {
                        if m > 0 {
                            10000
                        } else {
                            -10000
                        }
                    }
                };

                // Store the evaluation for the respective PV line
                if multipv > 0 && multipv <= 2 {
                    multi_pv_evals[multipv - 1] =
                        cp;
                }

                // Keep the primary variation (multipv 1) as the main return
                if multipv == 1 {
                    last_eval = eval;
                    last_pv = pv;
                }
            } else if let Some(bm) =
                Self::parse_bestmove(trimmed)
            {
                best_move = bm;
                break;
            }
            line.clear();
        }

        (
            last_eval,
            best_move,
            last_pv,
            multi_pv_evals,
        )
    }

    pub fn quit(mut self) {
        self.send_command("quit");
        self.process.wait().unwrap();
    }

    pub fn parse_info_line(
        line: &str,
    ) -> Option<(
        usize,
        usize,
        Evaluation,
        Vec<String>,
    )> {
        let words: Vec<&str> =
            line.split_whitespace().collect();
        if words.first() != Some(&"info") {
            return None;
        }

        // Extract Depth
        let depth = if let Some(idx) = words
            .iter()
            .position(|&w| w == "depth")
        {
            words.get(idx + 1)?.parse().ok()?
        } else {
            0 // Default to 0 since transposition table might also omit this
        };
        let multipv = if let Some(idx) = words
            .iter()
            .position(|&w| w == "multipv")
        {
            words.get(idx + 1)?.parse().ok()?
        } else {
            1
        };

        let eval = if let Some(cp_idx) =
            words.iter().position(|&w| w == "cp")
        {
            Evaluation::Cp(
                words
                    .get(cp_idx + 1)?
                    .parse()
                    .ok()?,
            )
        } else if let Some(mate_idx) = words
            .iter()
            .position(|&w| w == "mate")
        {
            Evaluation::Mate(
                words
                    .get(mate_idx + 1)?
                    .parse()
                    .ok()?,
            )
        } else {
            return None;
        };

        let pv_moves = if let Some(pv_idx) =
            words.iter().position(|&w| w == "pv")
        {
            words[pv_idx + 1..]
                .iter()
                .map(|&s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        Some((depth, multipv, eval, pv_moves))
    }

    pub fn parse_bestmove(
        line: &str,
    ) -> Option<String> {
        let words: Vec<&str> =
            line.split_whitespace().collect();
        if words.first() != Some(&"bestmove") {
            return None;
        }
        words.get(1).map(|&s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cp_score_and_pv_from_info_line() {
        let line = "info depth 10 seldepth 11 multipv 1 score cp 30 nodes 32656 pv d2d4 e7e6";
        let result =
            UciEngine::parse_info_line(line);
        assert!(result.is_some());
        let (_, multipv, eval, pv) =
            result.unwrap();
        assert_eq!(multipv, 1);
        assert_eq!(eval, Evaluation::Cp(30));
        assert_eq!(pv, vec!["d2d4", "e7e6"]);
    }

    #[test]
    fn parses_mate_score_from_info_line() {
        let line = "info depth 5 multipv 1 score mate 3 pv e1e8 d8e8 f1e1";
        let result =
            UciEngine::parse_info_line(line);
        assert!(result.is_some());
        let (_, _, eval, _) = result.unwrap();
        assert_eq!(eval, Evaluation::Mate(3));
    }

    #[test]
    fn returns_none_for_info_line_without_score()
    {
        let line = "info depth 10 nodes 32656 nps 375356";
        assert!(UciEngine::parse_info_line(line)
            .is_none());
    }

    #[test]
    fn returns_none_for_non_info_line() {
        let line = "bestmove e2e4 ponder c7c5";
        assert!(UciEngine::parse_info_line(line)
            .is_none());
    }

    #[test]
    fn parses_bestmove_from_valid_line() {
        let line = "bestmove e2e4 ponder c7c5";
        assert_eq!(
            UciEngine::parse_bestmove(line),
            Some("e2e4".to_string())
        );
    }

    #[test]
    fn returns_none_for_non_bestmove_line() {
        let line = "info depth 10 score cp 30";
        assert_eq!(
            UciEngine::parse_bestmove(line),
            None
        );
    }

    #[test]
    fn parses_bestmove_without_ponder() {
        let line = "bestmove d2d4";
        assert_eq!(
            UciEngine::parse_bestmove(line),
            Some("d2d4".to_string())
        );
    }
}
