//! Event types emitted by the core so any front end can render progress.
//!
//! These are the *only* thing that flows out of the core to a UI. No
//! `println!` anywhere in the core.

/// An event emitted by the core, either as progress or as a log line.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A progress report for a task.
    Progress {
        task: String,
        stage: String,
        current: u64,
        total: u64,
        bytes_done: u64,
        bytes_total: u64,
        message: Option<String>,
    },
    /// A log line from the game or an internal action.
    Log { level: String, message: String },
    /// An authentication stage change.
    AuthStage { stage: String },
    /// Informational message.
    Info { message: String },
    /// Warning.
    Warn { message: String },
}

/// A throttled emitter that limits events to ~10 per second per task while
/// always emitting the last one.
pub struct ThrottledEmitter {
    last_by_task: std::collections::HashMap<String, std::time::Instant>,
}

impl ThrottledEmitter {
    pub fn new() -> Self {
        Self {
            last_by_task: std::collections::HashMap::new(),
        }
    }

    /// Returns true when the event should be emitted now, false when it should
    /// be suppressed. Always emits when `force` is true.
    pub fn should_emit(&mut self, task: &str, force: bool) -> bool {
        let now = std::time::Instant::now();
        let entry = self.last_by_task.entry(task.to_string()).or_insert(now);
        if force {
            *entry = now;
            return true;
        }
        if now.duration_since(*entry).as_secs_f64() >= 0.1 {
            *entry = now;
            true
        } else {
            false
        }
    }
}

impl Default for ThrottledEmitter {
    fn default() -> Self {
        Self::new()
    }
}
