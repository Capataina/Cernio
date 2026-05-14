//! Append-only JSONL telemetry for Cernio sessions.
//!
//! Every user keystroke, every state transition, every DB write, and every
//! autofill stage emits one structured event. Output lives at
//! `~/Library/Application Support/cernio/telemetry/session-{utc_ts}.jsonl`
//! (or the project's `state/telemetry/` directory as a fallback if `$HOME`
//! is unset).
//!
//! Why this exists: the TUI runs in raw mode, which suppresses stderr, and
//! the autofill spawns tokio tasks whose `Result`s are dropped. Without a
//! durable side-channel, silent failures are unfixable. With per-event
//! JSONL, every failure has a timestamp, a phase, an error string, and a
//! preceding key-press trail that's enough to reproduce the bug.
//!
//! Usage:
//!
//! ```ignore
//! telemetry::init();
//! tel!("key_press", "code": "p", "view": "Jobs", "selected_job": 2558);
//! tel!("autofill_error", "phase": "browser_launch", "error": e.to_string());
//! ```

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

static WRITER: OnceLock<Mutex<BufWriter<std::fs::File>>> = OnceLock::new();
static SESSION_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initialise the telemetry writer. Idempotent: subsequent calls return Ok
/// without re-opening the file. Failures are silent — telemetry must never
/// crash the host program.
pub fn init() {
    if WRITER.get().is_some() {
        return;
    }

    let dir = telemetry_dir();
    if let Err(_) = fs::create_dir_all(&dir) {
        return;
    }

    let session_id = format!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let path = dir.join(format!("session-{session_id}.jsonl"));

    let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let _ = WRITER.set(Mutex::new(BufWriter::new(file)));
    let _ = SESSION_PATH.set(path);

    log(
        "session_start",
        serde_json::json!({
            "session_id": session_id,
            "version": env!("CARGO_PKG_VERSION"),
        }),
    );
}

/// Return the path the current session is writing to, if telemetry is
/// initialised. Used for the startup banner so the user knows where to look.
pub fn current_session_path() -> Option<PathBuf> {
    SESSION_PATH.get().cloned()
}

/// Append one event to the telemetry log. Auto-flushes so the file survives
/// a crash. Silent on every failure — telemetry must not crash the host.
pub fn log(kind: &str, data: serde_json::Value) {
    let Some(writer) = WRITER.get() else {
        return;
    };

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Merge `data` (assumed to be an object) into a top-level event object
    // that always has `ts` and `kind` fields.
    let mut event = serde_json::Map::new();
    event.insert("ts".to_string(), serde_json::Value::String(ts));
    event.insert("kind".to_string(), serde_json::Value::String(kind.to_string()));
    if let serde_json::Value::Object(data_map) = data {
        for (k, v) in data_map {
            event.insert(k, v);
        }
    }

    let Ok(mut w) = writer.lock() else {
        return;
    };
    let line = match serde_json::to_string(&serde_json::Value::Object(event)) {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = writeln!(*w, "{line}");
    let _ = w.flush();
}

/// Where the JSONL files live. On macOS, `~/Library/Application Support/cernio/telemetry/`.
fn telemetry_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("cernio")
            .join("telemetry");
    }
    PathBuf::from("state/telemetry")
}

/// Ergonomic macro: `tel!("kind")` or `tel!("kind", "key": value, ...)`.
#[macro_export]
macro_rules! tel {
    ($kind:expr) => {
        $crate::telemetry::log($kind, ::serde_json::json!({}))
    };
    ($kind:expr, $($key:tt : $value:expr),+ $(,)?) => {
        $crate::telemetry::log($kind, ::serde_json::json!({ $($key: $value),+ }))
    };
}
