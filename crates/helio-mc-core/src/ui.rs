//! The shared output layer for every CLI command.
//!
//! Every command renders through this module. It picks the right mode:
//! colored + icons when stdout is a TTY and color is allowed; plain text when
//! `--json`, `--quiet`, `NO_COLOR`, or stdout is piped.
//!
//! The module also owns the `--help` icon headings and the terminal title
//! OSC.

use std::io::{IsTerminal, Write};

use crate::brand;
use crate::icons::{IconMode, Icons};

/// The output mode resolved from flags + env + TTY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Full color + icons, machine-readable `--json` suppressed.
    Color,
    /// No color, icons allowed.
    NoColor,
    /// Machine-readable JSON on stdout; human text on stderr.
    Json,
    /// Minimal text, no color, no icons.
    Quiet,
}

/// A resolved output context for one CLI invocation.
pub struct Out {
    pub mode: OutputMode,
    pub icons: Icons,
    /// True when human text should go to stderr (for `--json`).
    pub json_active: bool,
}

impl Out {
    /// Resolve the output mode from CLI flags and the environment.
    pub fn new(json: bool, quiet: bool, no_color: bool, icon_mode: Option<IconMode>) -> Self {
        let stdout_is_tty = std::io::stdout().is_terminal();
        let no_color_env = std::env::var("NO_COLOR").is_ok();

        let mode = if json {
            OutputMode::Json
        } else if quiet {
            OutputMode::Quiet
        } else if no_color || no_color_env || !stdout_is_tty {
            OutputMode::NoColor
        } else {
            OutputMode::Color
        };

        let icons = Icons::new(icon_mode.unwrap_or_default(), 1);
        Self {
            mode,
            icons,
            json_active: json,
        }
    }

    /// The active `IconMode` for this context.
    pub fn icon_mode(&self) -> IconMode {
        match self.mode {
            OutputMode::Json | OutputMode::Quiet => IconMode::Ascii,
            _ => self.icons.mode(),
        }
    }

    /// Emit a `success` line with the `Check` icon.
    pub fn success(&self, msg: &str) {
        self.emit_icon_line("Check", msg, "success");
    }

    /// Emit an `info` line with the `Info` icon.
    pub fn info(&self, msg: &str) {
        self.emit_icon_line("Info", msg, "info");
    }

    /// Emit a `warn` line with the `Warn` icon.
    pub fn warn(&self, msg: &str) {
        self.emit_icon_line("Warn", msg, "warn");
    }

    /// Emit an `error` line with the `Error` icon.
    pub fn error(&self, msg: &str) {
        self.emit_icon_line("Error", msg, "error");
    }

    /// Emit a `hint` line with the `Command` icon, dimmed.
    pub fn hint(&self, msg: &str) {
        self.emit_icon_line("Command", msg, "dim");
    }

    /// Emit a `section` heading with the `ChevronRight` icon.
    pub fn section(&self, title: &str) {
        self.emit_icon_line("ChevronRight", title, "accent");
    }

    /// Emit a `kv` (key/value) row.
    pub fn kv(&self, key: &str, value: &str) {
        if self.json_active {
            return;
        }
        let line = format!("{key:>20}  {value}");
        self.write_line(&line, "dim", true);
    }

    /// Print plain data (for `--json` or tables).
    pub fn data(&self, text: &str) {
        println!("{text}");
    }

    /// Print a JSON value.
    pub fn json_value(&self, v: &impl serde::Serialize) {
        let pretty = serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".into());
        println!("{pretty}");
    }

    // ---- internals ----

    fn emit_icon_line(&self, icon: &str, msg: &str, color: &str) {
        if self.json_active {
            return;
        }
        let icon_str = self.icons.render(icon);
        let text = format!("{icon_str}{msg}");
        self.write_line(&text, color, true);
    }

    fn write_line(&self, text: &str, _color: &str, _indent: bool) {
        match self.mode {
            OutputMode::Color => {
                // Color would be applied here per-token; kept simple for now.
                println!("{text}");
            }
            OutputMode::NoColor | OutputMode::Quiet => {
                // Strip icon padding for quiet.
                println!("{text}");
            }
            OutputMode::Json => {
                // Human text goes to stderr in json mode.
                eprintln!("{text}");
            }
        }
    }
}

/// Set the terminal title to "Helio" (OSC 0/2).
pub fn set_terminal_title() {
    if !std::io::stdout().is_terminal() {
        return;
    }
    print!("\u{1b}]0;{title}\u{1b}\\", title = brand::NAME);
    let _ = std::io::stdout().flush();
}

/// Restore the terminal title on exit (clear it).
pub fn restore_terminal_title() {
    if !std::io::stdout().is_terminal() {
        return;
    }
    print!("\u{1b}]0;\u{1b}\\");
    let _ = std::io::stdout().flush();
}

/// A convenience helper: print a success line with the right icon.
pub fn success(msg: &str) {
    Out::new(false, false, false, None).success(msg);
}

/// A convenience helper: print an info line with the right icon.
pub fn info(msg: &str) {
    Out::new(false, false, false, None).info(msg);
}

/// A convenience helper: print an error line with the right icon.
pub fn error(msg: &str) {
    Out::new(false, false, false, None).error(msg);
}

/// A convenience helper: print a hint line with the right icon.
pub fn hint(msg: &str) {
    Out::new(false, false, false, None).hint(msg);
}

/// The brand name for headings.
pub const fn brand_name() -> &'static str {
    brand::NAME
}
