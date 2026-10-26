//! Typed errors for the whole Helio core.
//!
//! Every error has a stable `exit_code()` and a human `hint()`. The CLI turns
//! these into friendly messages; the TUI renders them as toasts. No error is
//! silent.

use std::path::PathBuf;

/// A result type that uses [`HelioError`].
pub type HelioResult<T> = Result<T, HelioError>;

/// The one error type of the core. Never use `panic!` on user-data problems.
#[derive(thiserror::Error, Debug)]
pub enum HelioError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error ({status}): {message}")]
    Http {
        status: u16,
        message: String,
        host: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Java problem: {0}")]
    Java(String),

    #[error("Launch problem: {0}")]
    Launch(String),

    #[error("Download problem: {0}")]
    Download(String),

    #[error("Hash mismatch: {0}")]
    Hash(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for HelioError {
    fn from(e: reqwest::Error) -> Self {
        HelioError::Http {
            status: 0,
            message: e.to_string(),
            host: None,
        }
    }
}

impl From<serde_urlencoded::ser::Error> for HelioError {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        HelioError::Http {
            status: 0,
            message: e.to_string(),
            host: None,
        }
    }
}

impl HelioError {
    /// Stable process exit code (see the spec's exit-code table).
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Cancelled => 130,
            Self::InvalidInput(_) | Self::NotFound(_) => 2,
            Self::Auth(_) => 4,
            Self::Http { .. } | Self::Download(_) => 5,
            Self::Java(_) => 6,
            Self::Launch(_) => 7,
            _ => 1,
        }
    }

    /// An actionable hint shown to the user.
    pub fn hint(&self) -> Option<String> {
        match self {
            Self::Auth(_) => Some("run: helio login --elyby".into()),
            Self::Java(msg) => Some(format!(
                "run: helio java install 21 (or fix the problem: {msg})"
            )),
            Self::Http { host, .. } => match host {
                Some(h) => Some(format!(
                    "the host {h} is not on Helio's allowlist; network access is restricted by design"
                )),
                None => Some("check your connection and try again".into()),
            },
            _ => None,
        }
    }

    /// Convenience: a [`HelioError::NotFound`] wrapping a path.
    pub fn not_found_path(p: PathBuf) -> Self {
        Self::NotFound(p.display().to_string())
    }
}
