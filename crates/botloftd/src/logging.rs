//! Logs to `logs\botloftd.YYYY-MM-DD.log` (daily, 14 files kept) and stderr.
//!
//! Never log tokens, message bodies or terminal output at `info` or above:
//! bots may handle personal and health data (spec 13).

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

const KEEP_FILES: usize = 14;

/// Installs the global subscriber. `level` applies to Botloft's own crates;
/// dependencies stay at `warn` because at debug/trace the WebSocket stack
/// logs frames, which can carry message bodies. `RUST_LOG` overrides both.
/// Keep the returned guard alive: dropping it flushes the file writer.
pub fn init(logs_dir: &Path, level: &str) -> anyhow::Result<WorkerGuard> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("botloftd")
        .filename_suffix("log")
        .max_log_files(KEEP_FILES)
        .build(logs_dir)?;
    let (file, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "warn,botloftd={level},botloft_core={level},botloft_store={level}"
        ))
    });
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_ansi(false).with_writer(file))
        .with(fmt::layer().with_writer(std::io::stderr))
        .try_init()?;
    Ok(guard)
}
