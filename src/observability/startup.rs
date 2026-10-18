//! Opt-in, low-volume startup/lifecycle trace for bounded field diagnosis.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const TRACE_FILE_ENV: &str = "CLEAN_CTX_STARTUP_TRACE_FILE";
static START: OnceLock<Instant> = OnceLock::new();

pub(crate) fn event(name: &str) {
    event_with(name, None);
}

pub(crate) fn event_detail(name: &str, detail: impl Into<String>) {
    event_with(name, Some(detail.into()));
}

fn event_with(name: &str, detail: Option<String>) {
    let Some(path) = trace_path() else {
        return;
    };
    let elapsed_ms = START.get_or_init(Instant::now).elapsed().as_millis();
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let thread = std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_string();
    let record = serde_json::json!({
        "unix_ms": unix_ms,
        "elapsed_ms": elapsed_ms,
        "pid": std::process::id(),
        "thread": thread,
        "event": name,
        "detail": detail,
    });
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    if let Ok(line) = serde_json::to_string(&record) {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn trace_path() -> Option<PathBuf> {
    let configured = std::env::var_os(TRACE_FILE_ENV)?;
    let rendered = configured
        .to_string_lossy()
        .replace("{pid}", &std::process::id().to_string());
    if rendered.trim().is_empty() {
        return None;
    }
    Some(PathBuf::from(rendered))
}
