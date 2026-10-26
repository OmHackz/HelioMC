//! Account data models: `Account`, `AccountKind`, `AccountsFile`,
//! and the `AccountStore` that owns the on-disk file.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::brand;
use crate::error::{HelioError, HelioResult};
use crate::paths;

use super::keychain;
use super::offline;

/// The kind of account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    /// An offline (cracked) account.
    Offline,
    /// An Ely.by OAuth account.
    Elyby,
}

/// The account metadata stored in `accounts.json`. **No tokens here.**
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// Stable id (uuid).
    pub id: String,
    pub kind: AccountKind,
    /// Display username.
    pub username: String,
    /// Player UUID (hyphenated), e.g. for skins.
    pub uuid: String,
    /// When the account was added.
    pub added_at: DateTime<Utc>,
    /// Ely.by-specific: the client_id used to log in (for refresh).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elyby_client_id: Option<String>,
}

/// The on-disk `accounts.json` document.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsFile {
    pub schema: u32,
    pub accounts: Vec<Account>,
}

impl AccountsFile {
    /// The path of `accounts.json` inside a data dir.
    pub fn path(data_dir: &Path) -> std::path::PathBuf {
        data_dir.join("accounts.json")
    }

    /// Load from disk, or an empty file when missing/corrupt.
    pub fn load(data_dir: &Path) -> Self {
        let mut f = paths::read_json_or_default(&Self::path(data_dir), Self::default());
        if f.schema == 0 {
            f.schema = 1;
        }
        f
    }

    /// Save atomically (with a `.bak`).
    pub fn save(&self, data_dir: &Path) -> HelioResult<()> {
        let mut self_ = self.clone();
        self_.schema = 1;
        paths::write_json_atomic(&Self::path(data_dir), &self_)
    }

    /// Find an account by id, username (case-insensitive), or uuid.
    pub fn find(&self, query: &str) -> Option<&Account> {
        let q = query.to_lowercase();
        self.accounts.iter().find(|a| {
            a.id.to_lowercase() == q || a.username.to_lowercase() == q || a.uuid.to_lowercase() == q
        })
    }

    /// True when an account with the same username already exists
    /// (case-insensitive).
    pub fn username_taken(&self, username: &str) -> bool {
        let u = username.to_lowercase();
        self.accounts.iter().any(|a| a.username.to_lowercase() == u)
    }

    /// Add an account. Rejects duplicate usernames (case-insensitive).
    pub fn add(&mut self, acc: Account) -> HelioResult<()> {
        if self.username_taken(&acc.username) {
            return Err(HelioError::InvalidInput(format!(
                "account name {} is already in use (duplicates rejected case-insensitively)",
                acc.username
            )));
        }
        self.accounts.push(acc);
        Ok(())
    }

    /// Remove an account by id.
    pub fn remove(&mut self, id: &str) -> Option<Account> {
        if let Some(i) = self.accounts.iter().position(|a| a.id == id) {
            Some(self.accounts.swap_remove(i))
        } else {
            None
        }
    }
}

/// The file-based account store. Owns the data-dir path and the file.
pub struct AccountStore {
    pub data_dir: std::path::PathBuf,
    pub file: AccountsFile,
}

impl AccountStore {
    /// Load (or create) the store at a data dir.
    pub fn open(data_dir: &Path) -> Self {
        let data_dir = data_dir.to_path_buf();
        let file = AccountsFile::load(&data_dir);
        Self { data_dir, file }
    }

    /// Persist.
    pub fn save(&self) -> HelioResult<()> {
        self.file.save(&self.data_dir)
    }

    /// Add an account.
    pub fn add(&mut self, acc: Account) -> HelioResult<()> {
        self.file.add(acc)?;
        self.save()
    }

    /// Remove an account by id, also removing any keychain entries.
    pub fn remove(&mut self, id: &str) -> HelioResult<Account> {
        let acc = self
            .file
            .remove(id)
            .ok_or_else(|| HelioError::NotFound(format!("no account with id {id:?}")))?;
        // Drop keychain tokens for this account.
        if acc.kind == AccountKind::Elyby {
            let _ = keychain::delete_elyby(&acc.uuid);
        }
        self.save()?;
        Ok(acc)
    }

    /// Find an account by id, username, or uuid.
    pub fn find(&self, query: &str) -> Option<&Account> {
        self.file.find(query)
    }

    /// The active account (from config), or None.
    pub fn active(&self) -> Option<&Account> {
        let config = crate::config::Config::load(&self.data_dir);
        let id = config.active_account_id?;
        self.file.find(&id)
    }

    /// Set the active account id (in config), and persist both files.
    pub fn set_active(&mut self, account_id: &str) -> HelioResult<()> {
        let mut config = crate::config::Config::load(&self.data_dir);
        if config.active_account_id.as_deref() != Some(account_id) {
            config.active_account_id = Some(account_id.to_string());
            config.save(&self.data_dir)?;
        }
        Ok(())
    }

    /// Create an offline account, storing nothing but metadata.
    pub fn add_offline(&mut self, username: &str) -> HelioResult<Account> {
        offline::validate_username(username)?;
        let uuid = offline::offline_uuid(username);
        let acc = Account {
            id: Uuid::new_v4().to_string(),
            kind: AccountKind::Offline,
            username: username.to_string(),
            uuid,
            added_at: Utc::now(),
            elyby_client_id: None,
        };
        self.add(acc.clone())?;
        // Set active if it is the first account.
        if self.file.accounts.len() == 1 {
            self.set_active(&acc.id)?;
        }
        Ok(acc)
    }

    /// Create an Ely.by account from an OAuth result, storing tokens in the
    /// keychain (never in accounts.json).
    pub fn add_elyby(
        &mut self,
        profile: &super::elyby::UserProfile,
        client_id: &str,
    ) -> HelioResult<Account> {
        let uuid = profile.uuid.clone();
        let acc = Account {
            id: Uuid::new_v4().to_string(),
            kind: AccountKind::Elyby,
            username: profile.username.clone(),
            uuid,
            added_at: Utc::now(),
            elyby_client_id: Some(client_id.to_string()),
        };
        self.add(acc.clone())?;
        if self.file.accounts.len() == 1 {
            self.set_active(&acc.id)?;
        }
        Ok(acc)
    }

    /// The keychain service name (from brand, single source of truth).
    pub fn keychain_service(&self) -> &str {
        brand::KEYCHAIN_SERVICE
    }
}
