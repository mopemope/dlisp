use std::fs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn get_state_dir() -> Option<PathBuf> {
    let mut path = dirs::state_dir().or_else(dirs::home_dir)?;
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
    let file_appender = tracing_appender::rolling::never(&log_dir, "debug.log");
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
