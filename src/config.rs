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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AccountsConfig::default();
        assert!(config.current.is_none());
        assert!(config.accounts.is_empty());
        assert!(config.is_empty());
    }

    #[test]
    fn test_add_account() {
        let mut config = AccountsConfig::default();
        let metadata = AccountMetadata {
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            path: PathBuf::from("/test/path"),
        };

        config.add_account("test_account".to_string(), metadata);
        assert_eq!(config.accounts.len(), 1);
        assert!(config.get_account("test_account").is_some());
        assert!(!config.is_empty());
    }

    #[test]
    fn test_get_account() {
        let mut config = AccountsConfig::default();
        let metadata = AccountMetadata {
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            path: PathBuf::from("/test/path"),
        };

        config.add_account("test_account".to_string(), metadata);

        let retrieved = config.get_account("test_account");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().saved_at, "2024-01-01T00:00:00Z");

        assert!(config.get_account("nonexistent").is_none());
    }

    #[test]
    fn test_remove_account() {
        let mut config = AccountsConfig::default();
        let metadata = AccountMetadata {
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            path: PathBuf::from("/test/path"),
        };

        config.add_account("test_account".to_string(), metadata);
        assert_eq!(config.accounts.len(), 1);

        let removed = config.remove_account("test_account");
        assert!(removed.is_some());
        assert_eq!(config.accounts.len(), 0);
        assert!(config.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_account() {
        let mut config = AccountsConfig::default();
        let removed = config.remove_account("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_rename_account() {
        let mut config = AccountsConfig::default();
        let metadata = AccountMetadata {
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            path: PathBuf::from("/test/path"),
        };

        config.add_account("old_name".to_string(), metadata);
        config.current = Some("old_name".to_string());

        let result = config.rename_account("old_name", "new_name".to_string());
        assert!(result.is_ok());
        assert!(config.get_account("old_name").is_none());
        assert!(config.get_account("new_name").is_some());
        assert_eq!(config.current, Some("new_name".to_string()));
    }

    #[test]
    fn test_rename_nonexistent_account() {
        let mut config = AccountsConfig::default();
        let result = config.rename_account("old_name", "new_name".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_account_not_current() {
        let mut config = AccountsConfig::default();
        let metadata = AccountMetadata {
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            path: PathBuf::from("/test/path"),
        };

        config.add_account("old_name".to_string(), metadata);
        config.current = Some("other_account".to_string());

        let result = config.rename_account("old_name", "new_name".to_string());
        assert!(result.is_ok());
        assert_eq!(config.current, Some("other_account".to_string()));
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();

        let mut config = AccountsConfig::default();
        config.current = Some("test_account".to_string());
        config.add_account(
            "test_account".to_string(),
            AccountMetadata {
                saved_at: "2024-01-01T00:00:00Z".to_string(),
                path: PathBuf::from("/test/path"),
            },
        );

        config.save(temp_path)?;

        let loaded_config = AccountsConfig::load(temp_path)?;
        assert_eq!(loaded_config.current, Some("test_account".to_string()));
        assert_eq!(loaded_config.accounts.len(), 1);
        assert!(loaded_config.get_account("test_account").is_some());

        Ok(())
    }

    #[test]
    fn test_load_nonexistent_config() -> Result<()> {
        let config = AccountsConfig::load(Path::new("/nonexistent/path.json"))?;
        assert!(config.current.is_none());
        assert!(config.is_empty());
        Ok(())
    }

    #[test]
    fn test_save_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid json {{").unwrap();
        temp_file.flush().unwrap();

        let result = AccountsConfig::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_accounts() {
        let mut config = AccountsConfig::default();

        for i in 1..=5 {
            config.add_account(
                format!("account_{}", i),
                AccountMetadata {
                    saved_at: format!("2024-01-{:02}T00:00:00Z", i),
                    path: PathBuf::from(format!("/test/path_{}", i)),
                },
            );
        }

        assert_eq!(config.accounts.len(), 5);
        assert!(!config.is_empty());

        for i in 1..=5 {
            assert!(config.get_account(&format!("account_{}", i)).is_some());
        }
    }
}
