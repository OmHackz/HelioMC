//! Global launcher configuration, stored as `config.json` in the data dir.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{HelioError, HelioResult};
use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Schema version of this file. Unknown schema is migrated best-effort.
    pub schema: u32,
    /// Explicit data dir (overrides the env/default). `null` means "use default".
    pub data_dir: Option<PathBuf>,
    /// Globally preferred Java path (a full path to `java`).
    pub java_path: Option<PathBuf>,
    /// Minimum RAM to give the game, in MB.
    pub min_ram_mb: u32,
    /// Maximum RAM to give the game, in MB.
    pub max_ram_mb: u32,
    /// Extra JVM args applied to every launch.
    pub jvm_args: Vec<String>,
    /// Window resolution, e.g. `{"w": 1280, "h": 720}`.
    pub resolution: Option<Resolution>,
    /// Number of parallel downloads (1..=16).
    pub concurrent_downloads: u32,
    /// Close the launcher after a successful launch.
    pub close_on_launch: bool,
    /// Active theme name: `Sun` (default), `Midnight` or `Daylight`.
    pub theme: String,
    /// Force ASCII icon mode.
    pub ascii_icons: bool,
    /// Icon mode when not ascii: `nerd` or `unicode`.
    pub icons: String,
    /// Id of the active account.
    pub active_account_id: Option<String>,
    /// Ely.by OAuth client id.
    pub elyby_client_id: Option<String>,
    /// Ely.by OAuth client secret (for refresh flows; also stored in the
    /// keychain when available).
    pub elyby_client_secret: Option<String>,
    /// Check for Helio updates on startup (Phase 4).
    pub check_updates: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub w: u32,
    pub h: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: 1,
            data_dir: None,
            java_path: None,
            min_ram_mb: 512,
            max_ram_mb: 4096,
            jvm_args: vec![],
            resolution: None,
            concurrent_downloads: 8,
            close_on_launch: false,
            theme: "Sun".into(),
            ascii_icons: false,
            icons: "unicode".into(),
            active_account_id: None,
            elyby_client_id: None,
            elyby_client_secret: None,
            check_updates: false,
        }
    }
}

impl Config {
    /// The path of `config.json` inside a data dir.
    pub fn path(data_dir: &Path) -> PathBuf {
        data_dir.join("config.json")
    }

    /// Run migration on loaded data (in-place; bump `schema`).
    /// Schema 0/missing is treated as an early save; fields are already
    /// defaulted by serde, so migration is just bumping the version.
    pub fn migrate(mut self) -> Self {
        if self.schema == 0 {
            self.schema = 1;
        }
        self
    }

    /// Load config from the data dir, or the defaults when missing/corrupt.
    pub fn load(data_dir: &Path) -> Self {
        let c = paths::read_json_or_default(&Self::path(data_dir), Self::default());
        c.migrate().validate()
    }

    /// Save config atomically, with a `.bak` of the previous file.
    pub fn save(&self, data_dir: &Path) -> HelioResult<()> {
        paths::write_json_atomic(&Self::path(data_dir), self)
    }

    /// Validate constraints, returning a defaulted-but-sane fixup.
    fn validate(mut self) -> Self {
        if self.schema == 0 {
            self.schema = 1;
        }
        self.concurrent_downloads = self.concurrent_downloads.clamp(1, 16);
        if self.max_ram_mb < self.min_ram_mb {
            self.max_ram_mb = self.min_ram_mb;
        }
        self
    }

    /// Validate a specific key/value pair before `config set` writes it.
    /// Returns an error with a clear message on bad input.
    pub fn validate_key_value(key: &str, value: &str) -> HelioResult<()> {
        match key {
            "concurrent_downloads" => {
                let n: u32 = value.parse().map_err(|_| {
                    HelioError::Config("concurrent_downloads must be a number".into())
                })?;
                if !(1..=16).contains(&n) {
                    return Err(HelioError::Config(
                        "concurrent_downloads must be between 1 and 16".into(),
                    ));
                }
            }
            "min_ram_mb" | "max_ram_mb" => {
                let _n: u32 = value
                    .parse()
                    .map_err(|_| HelioError::Config("RAM must be a number (MB)".to_string()))?;
            }
            "theme" => {
                if !matches!(value, "Sun" | "Midnight" | "Daylight") {
                    return Err(HelioError::Config(
                        "theme must be one of: Sun, Midnight, Daylight".into(),
                    ));
                }
            }
            "icons" => {
                if !matches!(value, "nerd" | "unicode" | "ascii") {
                    return Err(HelioError::Config(
                        "icons must be one of: nerd, unicode, ascii".into(),
                    ));
                }
            }
            _ => {
                return Err(HelioError::Config(format!(
                    "unknown config key: {key:?}. Known keys: concurrent_downloads, min_ram_mb, \
                     max_ram_mb, theme, icons, elyby_client_id, elyby_client_secret, active_account_id, \
                     close_on_launch, check_updates, java_path, jvm_args, resolution"
                )));
            }
        }
        Ok(())
    }

    /// The data dir this config points to, or the platform default.
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir
            .clone()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(crate::paths::default_data_dir)
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brand;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.concurrent_downloads, 8);
        assert_eq!(c.min_ram_mb, 512);
        assert!(c.max_ram_mb >= c.min_ram_mb);
        assert_eq!(c.theme, "Sun");
    }

    #[test]
    fn concurrent_downloads_is_clamped() {
        let mut c = Config::default();
        c.concurrent_downloads = 99;
        c = c.validate();
        assert_eq!(c.concurrent_downloads, 16);
        c.concurrent_downloads = 0;
        c = c.validate();
        assert_eq!(c.concurrent_downloads, 1);
    }

    #[test]
    fn ram_ordering_is_fixed() {
        let mut c = Config::default();
        c.min_ram_mb = 8192;
        c.max_ram_mb = 1024;
        c = c.validate();
        assert!(c.max_ram_mb >= c.min_ram_mb);
    }

    #[test]
    fn key_value_validation() {
        assert!(Config::validate_key_value("concurrent_downloads", "16").is_ok());
        assert!(Config::validate_key_value("concurrent_downloads", "17").is_err());
        assert!(Config::validate_key_value("theme", "Nope").is_err());
        assert!(Config::validate_key_value("theme", "Midnight").is_ok());
        assert!(Config::validate_key_value("bogus", "1").is_err());
    }

    #[test]
    fn env_prefix_is_brand() {
        assert_eq!(brand::ENV_PREFIX, "HELIO_");
    }
}
