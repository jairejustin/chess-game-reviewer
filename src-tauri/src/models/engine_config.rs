use serde::{Deserialize, Serialize};

/// User-facing engine configuration.
///
/// All fields are `Option<T>` so partial updates work and only the fields
/// present in a `configure_engine` call are written to the engine; the
/// rest are left at whatever value Stockfish currently holds.
#[derive(
    Debug, Clone, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    /// Transposition table size in MB.
    /// Stockfish recommends ~25 % of available RAM.
    /// Practical range: 16 – 32768.
    pub hash_mb: Option<u32>,

    /// CPU threads the engine may use.
    pub threads: Option<u32>,

    /// Number of principal variations shown simultaneously.
    /// Higher values slow the engine; 1–3 covers most use-cases.
    pub multi_pv: Option<u8>,

    /// Minimum search time in milliseconds for the batch analysis pipeline.
    /// This is a Tauri-layer setting, not a UCI option.
    pub analysis_time_ms: Option<u32>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            hash_mb: Some(256),
            threads: Some(2),
            multi_pv: Some(2),
            analysis_time_ms: Some(1500),
        }
    }
}

/// Writes every `Some` field in `config` to `stdin` as `setoption` commands.
/// The caller is responsible for flushing and for ensuring no search is running.
pub fn write_config_options(
    stdin: &mut dyn std::io::Write,
    config: &EngineConfig,
) -> std::io::Result<()> {
    if let Some(mb) = config.hash_mb {
        writeln!(
            stdin,
            "setoption name Hash value {}",
            mb
        )?;
    }
    if let Some(t) = config.threads {
        writeln!(
            stdin,
            "setoption name Threads value {}",
            t
        )?;
    }
    if let Some(pv) = config.multi_pv {
        writeln!(
            stdin,
            "setoption name MultiPV value {}",
            pv
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `write_config_options` against an in-memory buffer and returns
    /// the output as a plain string for easy assertion.
    fn capture(config: &EngineConfig) -> String {
        let mut buf: Vec<u8> = Vec::new();
        write_config_options(&mut buf, config)
            .expect("write_config_options failed unexpectedly");
        String::from_utf8(buf)
            .expect("output was not valid UTF-8")
    }

    // EngineConfig::default

    #[test]
    fn default_has_all_fields_as_some() {
        let cfg = EngineConfig::default();
        assert!(cfg.hash_mb.is_some());
        assert!(cfg.threads.is_some());
        assert!(cfg.multi_pv.is_some());
        assert!(cfg.analysis_time_ms.is_some());
    }

    #[test]
    fn default_values_match_documented_defaults()
    {
        let cfg = EngineConfig::default();
        assert_eq!(cfg.hash_mb, Some(256));
        assert_eq!(cfg.threads, Some(2));
        assert_eq!(cfg.multi_pv, Some(2));
        assert_eq!(
            cfg.analysis_time_ms,
            Some(1500)
        );
    }

    #[test]
    fn clone_produces_independent_copy() {
        let original = EngineConfig::default();
        let cloned = EngineConfig {
            hash_mb: Some(999),
            ..original.clone()
        };
        assert_eq!(original.hash_mb, Some(256));
        assert_eq!(cloned.hash_mb, Some(999));
    }

    // Simulates the merge logic in configure_engine to make sure
    // a partial update does not accidentally clear unset fields.
    #[test]
    fn partial_merge_preserves_unset_fields() {
        let mut stored = EngineConfig::default();
        let incoming = EngineConfig {
            threads: Some(8),
            hash_mb: None,
            multi_pv: None,
            analysis_time_ms: None,
        };

        // Replicate the merge logic from commands::configure_engine
        if let Some(v) = incoming.hash_mb {
            stored.hash_mb = Some(v);
        }
        if let Some(v) = incoming.threads {
            stored.threads = Some(v);
        }
        if let Some(v) = incoming.multi_pv {
            stored.multi_pv = Some(v);
        }
        if let Some(v) = incoming.analysis_time_ms
        {
            stored.analysis_time_ms = Some(v);
        }

        assert_eq!(
            stored.threads,
            Some(8),
            "updated field should change"
        );
        assert_eq!(
            stored.hash_mb,
            Some(256),
            "untouched field must be preserved"
        );
        assert_eq!(
            stored.multi_pv,
            Some(2),
            "untouched field must be preserved"
        );
        assert_eq!(
            stored.analysis_time_ms,
            Some(1500),
            "untouched field must be preserved"
        );
    }

    // write_config_options

    #[test]
    fn all_some_emits_all_three_setoption_lines()
    {
        let cfg = EngineConfig::default();
        let out = capture(&cfg);

        assert!(out.contains(
            "setoption name Hash value 256\n"
        ));
        assert!(out.contains(
            "setoption name Threads value 2\n"
        ));
        assert!(out.contains(
            "setoption name MultiPV value 2\n"
        ));
    }

    // Stockfish is sensitive to option order in some builds; confirm the
    // three UCI options are written Hash -> Threads -> MultiPV.
    #[test]
    fn setoption_lines_are_emitted_in_declaration_order(
    ) {
        let cfg = EngineConfig::default();
        let out = capture(&cfg);

        let hash_pos = out.find("Hash").unwrap();
        let threads_pos =
            out.find("Threads").unwrap();
        let multi_pv_pos =
            out.find("MultiPV").unwrap();

        assert!(
            hash_pos < threads_pos,
            "Hash must precede Threads"
        );
        assert!(
            threads_pos < multi_pv_pos,
            "Threads must precede MultiPV"
        );
    }

    #[test]
    fn all_none_produces_no_output() {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: None,
            multi_pv: None,
            analysis_time_ms: None,
        };
        assert_eq!(capture(&cfg), "");
    }

    // analysis_time_ms is a Tauri-layer knob; it must never be forwarded
    // to the engine as a setoption command.
    #[test]
    fn analysis_time_ms_is_never_written_to_engine(
    ) {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: None,
            multi_pv: None,
            analysis_time_ms: Some(9999),
        };
        let out = capture(&cfg);
        assert!(
            out.is_empty(),
            "analysis_time_ms must not produce a setoption line, got: {:?}",
            out
        );
    }

    // write_config_options - partial / single-field updates

    #[test]
    fn only_hash_mb_set_emits_only_hash_line() {
        let cfg = EngineConfig {
            hash_mb: Some(512),
            threads: None,
            multi_pv: None,
            analysis_time_ms: None,
        };
        assert_eq!(
            capture(&cfg),
            "setoption name Hash value 512\n"
        );
    }

    #[test]
    fn only_threads_set_emits_only_threads_line()
    {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: Some(6),
            multi_pv: None,
            analysis_time_ms: None,
        };
        assert_eq!(
            capture(&cfg),
            "setoption name Threads value 6\n"
        );
    }

    #[test]
    fn only_multi_pv_set_emits_only_multipv_line()
    {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: None,
            multi_pv: Some(3),
            analysis_time_ms: None,
        };
        assert_eq!(
            capture(&cfg),
            "setoption name MultiPV value 3\n"
        );
    }

    // write_config_options - boundary values

    #[test]
    fn boundary_u32_max_hash_does_not_panic() {
        let cfg = EngineConfig {
            hash_mb: Some(u32::MAX),
            threads: None,
            multi_pv: None,
            analysis_time_ms: None,
        };
        let out = capture(&cfg);
        assert!(
            out.contains(&u32::MAX.to_string())
        );
    }

    #[test]
    fn boundary_u32_max_threads_does_not_panic() {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: Some(u32::MAX),
            multi_pv: None,
            analysis_time_ms: None,
        };
        let out = capture(&cfg);
        assert!(
            out.contains(&u32::MAX.to_string())
        );
    }

    #[test]
    fn boundary_u8_max_multi_pv_does_not_panic() {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: None,
            multi_pv: Some(u8::MAX),
            analysis_time_ms: None,
        };
        let out = capture(&cfg);
        assert!(
            out.contains(&u8::MAX.to_string())
        );
    }

    #[test]
    fn threads_one_is_written_correctly() {
        let cfg = EngineConfig {
            hash_mb: None,
            threads: Some(1),
            multi_pv: None,
            analysis_time_ms: None,
        };
        assert_eq!(
            capture(&cfg),
            "setoption name Threads value 1\n"
        );
    }

    // write_config_options - I/O error propagation

    /// A writer that always fails, used to verify the `?` propagation.
    struct AlwaysFailWriter;
    impl std::io::Write for AlwaysFailWriter {
        fn write(
            &mut self,
            _buf: &[u8],
        ) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "simulated write failure",
            ))
        }
        fn flush(
            &mut self,
        ) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_error_is_propagated_not_swallowed() {
        let cfg = EngineConfig::default();
        let result = write_config_options(
            &mut AlwaysFailWriter,
            &cfg,
        );
        assert!(
            result.is_err(),
            "write failure must surface as Err, not be silently ignored"
        );
    }

    #[test]
    fn write_error_on_none_config_is_ok() {
        // If every field is None, write_config_options never touches the writer.
        // Even a broken writer must return Ok(()) in this case.
        let cfg = EngineConfig {
            hash_mb: None,
            threads: None,
            multi_pv: None,
            analysis_time_ms: None,
        };
        let result = write_config_options(
            &mut AlwaysFailWriter,
            &cfg,
        );
        assert!(
            result.is_ok(),
            "all-None config must not write anything and must not error"
        );
    }
}
