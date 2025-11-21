use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tracing::Dispatch;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const LOG_DIR_ENV: &str = "CONTEXT_LOG_DIR";

pub struct TelemetryGuard {
    log_path: PathBuf,
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl TelemetryGuard {
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
}

fn resolve_log_dir() -> Result<PathBuf> {
    let log_dir = match env::var(LOG_DIR_ENV) {
        Ok(dir) if Path::new(&dir).is_absolute() => PathBuf::from(dir),
        Ok(dir) => env::current_dir()?.join(dir),
        Err(_) => env::current_dir()?.join(".context").join("logs"),
    };

    fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

fn default_env_filter(default_directives: &[&str]) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if default_directives.is_empty() {
            EnvFilter::new("info")
        } else {
            let fallback = default_directives
                .iter()
                .map(|directive| format!("{directive}=info"))
                .collect::<Vec<_>>()
                .join(",");
            EnvFilter::new(fallback)
        }
    })
}

fn build_dispatch(
    app_name: &str,
    log_dir: PathBuf,
    env_filter: EnvFilter,
    console_writer: fmt::writer::BoxMakeWriter,
) -> Result<(Dispatch, TelemetryGuard)> {
    fs::create_dir_all(&log_dir)?;
    let log_file_name = format!("{app_name}.jsonl");
    let log_path = log_dir.join(&log_file_name);

    let file_appender = tracing_appender::rolling::never(&log_dir, log_file_name);
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let json_layer = fmt::layer()
        .json()
        .with_ansi(false)
        .with_writer(file_writer)
        .with_target(true);

    let console_layer = fmt::layer()
        .pretty()
        .with_writer(console_writer)
        .with_target(true);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .with(console_layer);

    let dispatch = Dispatch::new(subscriber);
    let guard = TelemetryGuard {
        log_path,
        _file_guard: file_guard,
    };

    Ok((dispatch, guard))
}

pub fn init_tracing(app_name: &str, default_directives: &[&str]) -> Result<TelemetryGuard> {
    let log_dir = resolve_log_dir()?;
    let env_filter = default_env_filter(default_directives);
    let console_writer = fmt::writer::BoxMakeWriter::new(std::io::stderr);

    let (dispatch, guard) = build_dispatch(app_name, log_dir, env_filter, console_writer)?;
    tracing::dispatcher::set_global_default(dispatch)?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };

    #[derive(Clone, Default)]
    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn make_writer(&self) -> fmt::writer::BoxMakeWriter {
            let buffer = self.buffer.clone();
            fmt::writer::BoxMakeWriter::new(move || BufferWriter {
                buffer: buffer.clone(),
            })
        }

        fn contents(&self) -> String {
            let guard = self.buffer.lock().unwrap();
            String::from_utf8_lossy(&guard[..]).to_string()
        }
    }

    struct BufferWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for BufferWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut guard = self.buffer.lock().unwrap();
            guard.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn writes_json_logs_to_file() {
        let temp = tempfile::tempdir().unwrap();
        let writer = TestWriter::default();

        let (dispatch, guard) = build_dispatch(
            "context-cli",
            temp.path().to_path_buf(),
            EnvFilter::new("info"),
            writer.make_writer(),
        )
        .unwrap();

        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!("file-log");
        });

        drop(guard);

        let log_path = temp.path().join("context-cli.jsonl");
        let contents = std::fs::read_to_string(log_path).unwrap();
        let first = contents.lines().next().unwrap();
        let json: Value = serde_json::from_str(first).unwrap();

        assert_eq!(json["fields"]["message"], "file-log");
        assert_eq!(json["level"], "INFO");
    }

    #[test]
    fn writes_pretty_console_logs() {
        let temp = tempfile::tempdir().unwrap();
        let writer = TestWriter::default();

        let (dispatch, guard) = build_dispatch(
            "context-web",
            temp.path().to_path_buf(),
            EnvFilter::new("info"),
            writer.make_writer(),
        )
        .unwrap();

        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!("console-log");
        });

        drop(guard);

        let output = writer.contents();
        assert!(output.contains("console-log"));
        assert!(output.contains("INFO"));
        assert!(!output.trim_start().starts_with('{'));
    }
}
