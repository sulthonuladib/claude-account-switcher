use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountMetadata {
    pub saved_at: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountsConfig {
    pub current: Option<String>,
    pub accounts: HashMap<String, AccountMetadata>,
}

impl AccountsConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents =
            fs::read_to_string(path).context("Failed to read accounts configuration file")?;

        serde_json::from_str(&contents).context("Failed to parse accounts configuration")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents =
            serde_json::to_string_pretty(self).context("Failed to serialize configuration")?;

        fs::write(path, contents).context("Failed to write accounts configuration file")
    }

    pub fn get_account(&self, name: &str) -> Option<&AccountMetadata> {
        self.accounts.get(name)
    }

    pub fn add_account(&mut self, name: String, metadata: AccountMetadata) {
        self.accounts.insert(name, metadata);
    }

    pub fn remove_account(&mut self, name: &str) -> Option<AccountMetadata> {
        self.accounts.remove(name)
    }

    pub fn rename_account(&mut self, old_name: &str, new_name: String) -> Result<()> {
        if let Some(metadata) = self.accounts.remove(old_name) {
            self.accounts.insert(new_name.clone(), metadata);

            if self.current.as_deref() == Some(old_name) {
                self.current = Some(new_name);
            }
            Ok(())
        } else {
            anyhow::bail!("Account '{}' not found", old_name)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
}
