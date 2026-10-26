//! Brand constants. Every user-visible name in Helio derives from here.

pub const NAME: &str = "Helio";
pub const BIN: &str = "helio";
pub const DATA_DIR_NAME: &str = "Helio";
pub const ENV_PREFIX: &str = "HELIO_";
pub const KEYCHAIN_SERVICE: &str = "Helio";
pub const TAGLINE: &str = "Minecraft in your terminal, bathed in sunlight.";

/// Repository URL: read at build time from `git remote get-url origin`
/// (see `build.rs`), else the `repository` field of `Cargo.toml`. Never invented.
pub const REPO_URL: &str = env!("HELIO_REPO_URL");

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent for all network requests: `Helio/<version> (+<repo url>)`.
pub fn user_agent() -> String {
    format!("{NAME}/{VERSION} (+{REPO_URL})")
}
