use std::fs;
use std::path::{Path, PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn get_state_dir() -> Option<PathBuf> {
    // XDG spec: relative XDG_STATE_HOME values must be ignored. `dirs`
    // follows this on Linux but returns None for state_dir() on
    // macOS/Windows, so apply the same rule here for consistent behavior.
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(dirs::state_dir)
        .or_else(dirs::home_dir)?;
    let mut path = base;
    path.push("dlisp");
    if fs::create_dir_all(&path).is_err() {
        return None;
    }
    Some(path)
}

pub fn get_history_path() -> Option<PathBuf> {
    let mut path = get_state_dir()?;
    path.push("history.txt");
    Some(path)
}

pub fn setup_logging() -> Option<WorkerGuard> {
    let log_dir = get_state_dir()?;
    setup_logging_in(log_dir)
}

/// Opens the debug log under an explicit directory; returns `None` when the
/// log file cannot be created so the CLI can run without file logging.
fn open_log_appender(log_dir: &Path) -> Option<RollingFileAppender> {
    tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::NEVER)
        .filename_prefix("debug.log")
        .build(log_dir)
        .ok()
}

/// Initializes file logging; returns `None` when logging is unavailable.
fn setup_logging_in(log_dir: PathBuf) -> Option<WorkerGuard> {
    let file_appender = open_log_appender(&log_dir)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    Some(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Documents WHY `open_log_appender` uses the Result-returning builder:
    /// the legacy `rolling::never()` appender panics on an unopenable log
    /// file, so that shape must never be wired back into startup. Runs the
    /// probe on its own thread; a panic there is the expected observation.
    #[test]
    fn test_legacy_never_appender_panics_on_unwritable_log() {
        let _guard = crate::TEST_PROCESS_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("dlisp");
        std::fs::create_dir_all(&dir).expect("state dir");
        std::fs::create_dir(dir.join("debug.log")).expect("poison debug.log");

        let poisoned = dir;
        let handle = std::thread::Builder::new()
            .spawn(move || {
                // Must not be used in production paths; expected to panic.
                let _ = tracing_appender::rolling::never(&poisoned, "debug.log");
            })
            .expect("probe thread");

        assert!(
            handle.join().is_err(),
            "legacy never() appender unexpectedly stopped panicking; \
             open_log_appender's builder-based guard would be dead code"
        );
    }

    #[test]
    fn test_open_log_appender_unwritable_log_file_is_none() {
        let _guard = crate::TEST_PROCESS_LOCK.lock().unwrap();
        // A directory named debug.log makes opening the log file fail; the
        // CLI must run without file logging instead of panicking.
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("dlisp");
        std::fs::create_dir_all(&dir).expect("state dir");
        std::fs::create_dir(dir.join("debug.log")).expect("poison debug.log");

        assert!(open_log_appender(&dir).is_none());
    }

    #[test]
    fn test_open_log_appender_writable_log_file_is_some() {
        let _guard = crate::TEST_PROCESS_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("dlisp");
        std::fs::create_dir_all(&dir).expect("state dir");

        assert!(open_log_appender(&dir).is_some());
    }
}
