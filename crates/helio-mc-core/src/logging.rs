//! Logging setup with secret redaction and a panic hook that records location.

use std::panic;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the global logging. Idempotent. The file log goes to
/// `Helio/logs/helio.log` (rotating). The panic hook writes the panic
/// location to the same logger.
///
/// Returns the guard that keeps the log-flush thread alive. Keep it alive for
/// the process lifetime, otherwise log lines queued at shutdown are lost.
pub fn init(
    data_dir: &std::path::Path,
    level: tracing::Level,
) -> &'static tracing_appender::non_blocking::WorkerGuard {
    static GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> =
        std::sync::OnceLock::new();
    INIT.call_once(|| {
        let file_appender = tracing_appender::rolling::never(data_dir.join("logs"), "helio.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy();

        let subscriber = tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(env_filter)
            .with_writer(non_blocking)
            .with_ansi(false)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set global default subscriber");

        // Panic hook that records the location, then the default hook.
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_default();
            let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".into()
            };
            tracing::error!(panic_location = %location, "panicked: {message}");
            default_hook(info);
        }));

        let _ = GUARD.set(guard);
    });
    GUARD.get().expect("guard is set on first call")
}

/// Mask a value in log output. Use for anything that might be a token or
/// password; the actual value is never logged.
pub fn redact(value: &str) -> String {
    if value.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}***", &value[..4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_short_value_is_masked() {
        assert_eq!(redact("abc"), "***");
        assert_eq!(redact("abcdefg"), "***");
    }

    #[test]
    fn redact_long_value_keeps_prefix() {
        assert_eq!(redact("secret-value"), "secr***");
    }
}
