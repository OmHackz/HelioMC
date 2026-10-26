//! The account system: models, offline accounts, keychain storage, and the
//! Ely.by OAuth flow.
//!
//! `accounts.json` holds **metadata only** — no tokens. Tokens live in the
//! OS keychain (service "Helio"), with an encrypted-file fallback when the
//! keychain is unavailable.

pub mod elyby;
pub mod keychain;
mod models;
pub mod offline;
pub mod skins;

pub use keychain::{delete_elyby, get_elyby, set_elyby, KeychainResult};
pub use models::{Account, AccountKind, AccountStore, AccountsFile};
pub use offline::{offline_uuid, validate_username};
