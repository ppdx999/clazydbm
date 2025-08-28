use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static LEVEL: OnceLock<LogLevel> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_env() -> LogLevel {
        match std::env::var("CLAZYDBM_LOG").unwrap_or_default().to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }
}

pub fn init(log_path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = log_path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let _ = LOG_FILE.set(Mutex::new(file));
    let _ = LEVEL.set(LogLevel::from_env());
    info(&format!("logging initialized: {}", path.display()));
    Ok(path.to_path_buf())
}

fn now_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    format!("{}.{}", secs, millis)
}

fn rank(level: LogLevel) -> u8 {
    match level {
        LogLevel::Trace => 0,
        LogLevel::Debug => 1,
        LogLevel::Info => 2,
        LogLevel::Warn => 3,
        LogLevel::Error => 4,
    }
}

fn enabled(level: LogLevel) -> bool {
    let min = *LEVEL.get_or_init(LogLevel::from_env);
    rank(level) >= rank(min)
}

fn write_line(level: &str, msg: &str) {
    if let Some(m) = LOG_FILE.get() {
        if let Ok(mut f) = m.lock() {
            let _ = writeln!(f, "{} [{}] {}", now_ts(), level, msg);
            let _ = f.flush();
        }
    }
}

pub fn error(msg: &str) {
    if enabled(LogLevel::Error) { write_line("ERROR", msg); }
}
pub fn warn(msg: &str) {
    if enabled(LogLevel::Warn) { write_line("WARN", msg); }
}
pub fn info(msg: &str) {
    if enabled(LogLevel::Info) { write_line("INFO", msg); }
}
pub fn debug(msg: &str) {
    if enabled(LogLevel::Debug) { write_line("DEBUG", msg); }
}
pub fn trace(msg: &str) {
    if enabled(LogLevel::Trace) { write_line("TRACE", msg); }
}
