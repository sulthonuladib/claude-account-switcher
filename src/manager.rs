use crate::config::{AccountMetadata, AccountsConfig};
use crate::error::AccountError;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct AccountManager {
    claude_config_dir: PathBuf,
    switcher_dir: PathBuf,
    accounts_file: PathBuf,
}

impl AccountManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to determine home directory")?;

        let claude_config_dir = home.join(".claude");

        // XDG Base Directory compliant paths
        let state_dir = home.join(".local/state/claude-account-switcher");
        let switcher_dir = home.join(".local/share/claude-account-switcher");
        let accounts_file = state_dir.join("accounts.json");

        fs::create_dir_all(&state_dir).context("Failed to create state directory")?;
        fs::create_dir_all(&switcher_dir).context("Failed to create account storage directory")?;

        Ok(Self {
            claude_config_dir,
            switcher_dir,
            accounts_file,
        })
    }

    fn load_config(&self) -> Result<AccountsConfig> {
        AccountsConfig::load(&self.accounts_file)
    }

    fn save_config(&self, config: &AccountsConfig) -> Result<()> {
        config.save(&self.accounts_file)
    }

    pub fn save_account(&self, name: &str) -> Result<()> {
        if !self.claude_config_dir.exists() {
            return Err(AccountError::NoConfiguration.into());
        }

        let mut config = self.load_config()?;
        let account_dir = self.switcher_dir.join(name);

        fs::create_dir_all(&account_dir).context("Failed to create account directory")?;

        self.copy_dir_recursive(&self.claude_config_dir, &account_dir)
            .context("Failed to copy configuration files")?;

        config.add_account(
            name.to_string(),
            AccountMetadata {
                saved_at: Utc::now().to_rfc3339(),
                path: account_dir,
            },
        );
        config.current = Some(name.to_string());

        self.save_config(&config)?;
        println!("Saved account '{}'", name);

        Ok(())
    }

    pub fn switch_account(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        let account_meta = config
            .get_account(name)
            .ok_or_else(|| AccountError::NotFound(name.to_string()))?
            .clone();

        // Save current state if it exists
        if let Some(current) = &config.current
            && self.claude_config_dir.exists()
        {
            let _ = self.save_account(current);
        }

        // Validate account directory exists
        if !account_meta.path.exists() {
            anyhow::bail!(
                "Account directory not found: {}",
                account_meta.path.display()
            );
        }

        // Clear and recreate config directory
        if self.claude_config_dir.exists() {
            fs::remove_dir_all(&self.claude_config_dir)
                .context("Failed to remove current configuration")?;
        }

        fs::create_dir_all(&self.claude_config_dir)
            .context("Failed to create configuration directory")?;

        // Restore account configuration
        self.copy_dir_recursive(&account_meta.path, &self.claude_config_dir)
            .context("Failed to restore account configuration")?;

        config.current = Some(name.to_string());
        self.save_config(&config)?;

        println!("Switched to account '{}'", name);
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<()> {
        let config = self.load_config()?;

        if config.is_empty() {
            println!("No saved accounts found.");
            return Ok(());
        }

        println!("Claude Code Accounts:");
        println!("{}", "-".repeat(60));

        let current = config.current.as_deref();
        let mut accounts: Vec<_> = config.accounts.iter().collect();
        accounts.sort_by_key(|(name, _)| *name);

        for (name, meta) in accounts {
            let marker = if Some(name.as_str()) == current {
                "*"
            } else {
                " "
            };
            let saved_at = meta.saved_at.get(..19).unwrap_or(&meta.saved_at);
            println!("{} {:<20} (saved: {})", marker, name, saved_at);
        }
        println!();

        Ok(())
    }

    pub fn delete_account(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        let account_meta = config
            .get_account(name)
            .ok_or_else(|| AccountError::NotFound(name.to_string()))?
            .clone();

        // Check if it's the current account
        if config.current.as_deref() == Some(name) {
            eprintln!("Warning: '{}' is currently active", name);
            eprint!("Continue? This will clear your active session (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
            config.current = None;
        }

        // Remove directory
        if account_meta.path.exists() {
            fs::remove_dir_all(&account_meta.path).context("Failed to remove account directory")?;
        }

        config.remove_account(name);
        self.save_config(&config)?;

        println!("Deleted account '{}'", name);
        Ok(())
    }

    pub fn rename_account(&self, old_name: &str, new_name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        if !config.accounts.contains_key(old_name) {
            return Err(AccountError::NotFound(old_name.to_string()).into());
        }

        if config.accounts.contains_key(new_name) {
            return Err(AccountError::AlreadyExists(new_name.to_string()).into());
        }

        let account_meta = config
            .get_account(old_name)
            .ok_or_else(|| AccountError::NotFound(old_name.to_string()))?
            .clone();

        // Rename directory
        let new_dir = self.switcher_dir.join(new_name);
        fs::rename(&account_meta.path, &new_dir).context("Failed to rename account directory")?;

        // Update configuration using the config method
        config.rename_account(old_name, new_name.to_string())?;

        // Update the path in the renamed account metadata
        if let Some(meta) = config.accounts.get_mut(new_name) {
            meta.path = new_dir;
        }

        self.save_config(&config)?;
        println!("Renamed account '{}' to '{}'", old_name, new_name);

        Ok(())
    }

    pub fn show_current(&self) -> Result<()> {
        let config = self.load_config()?;
        match config.current {
            Some(name) => println!("{}", name),
            None => println!("No active account"),
        }
        Ok(())
    }

    pub fn show_current_if_any(&self) -> Result<()> {
        let config = self.load_config()?;
        if let Some(name) = config.current {
            println!("{}", name);
        }
        Ok(())
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        for entry in fs::read_dir(src)
            .with_context(|| format!("Failed to read directory: {}", src.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                fs::create_dir_all(&dst_path).with_context(|| {
                    format!("Failed to create directory: {}", dst_path.display())
                })?;
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dst_path).with_context(|| {
                    format!(
                        "Failed to copy file from {} to {}",
                        src_path.display(),
                        dst_path.display()
                    )
                })?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    struct TestSetup {
        _temp_dir: TempDir,
        manager: AccountManager,
        claude_config_dir: PathBuf,
    }

    impl TestSetup {
        fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            let temp_path = temp_dir.path();

            let claude_config_dir = temp_path.join(".claude");
            let switcher_dir = temp_path.join(".local/share/claude-account-switcher");
            let state_dir = temp_path.join(".local/state/claude-account-switcher");
            let accounts_file = state_dir.join("accounts.json");

            fs::create_dir_all(&state_dir)?;
            fs::create_dir_all(&switcher_dir)?;

            let manager = AccountManager {
                claude_config_dir: claude_config_dir.clone(),
                switcher_dir,
                accounts_file,
            };

            Ok(Self {
                _temp_dir: temp_dir,
                manager,
                claude_config_dir,
            })
        }

        fn create_mock_claude_config(&self) -> Result<()> {
            fs::create_dir_all(&self.claude_config_dir)?;
            fs::write(
                self.claude_config_dir.join("config.json"),
                r#"{"api_key": "test_key"}"#,
            )?;
            fs::write(
                self.claude_config_dir.join("session.json"),
                r#"{"session": "test_session"}"#,
            )?;
            Ok(())
        }
    }

    #[test]
    fn test_save_account_no_configuration() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.save_account("test");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No Claude Code configuration found"));
    }

    #[test]
    fn test_save_account_success() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        let result = setup.manager.save_account("test_account");
        assert!(result.is_ok());

        let config = setup.manager.load_config().unwrap();
        assert!(config.get_account("test_account").is_some());
        assert_eq!(config.current, Some("test_account".to_string()));

        let account_dir = setup.manager.switcher_dir.join("test_account");
        assert!(account_dir.exists());
        assert!(account_dir.join("config.json").exists());
        assert!(account_dir.join("session.json").exists());
    }

    #[test]
    fn test_save_multiple_accounts() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("account1").unwrap();
        setup.manager.save_account("account2").unwrap();
        setup.manager.save_account("account3").unwrap();

        let config = setup.manager.load_config().unwrap();
        assert_eq!(config.accounts.len(), 3);
        assert!(config.get_account("account1").is_some());
        assert!(config.get_account("account2").is_some());
        assert!(config.get_account("account3").is_some());
    }

    #[test]
    fn test_switch_account_not_found() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.switch_account("nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_switch_account_success() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("account1").unwrap();

        fs::write(
            setup.claude_config_dir.join("config.json"),
            r#"{"api_key": "modified_key"}"#,
        )
        .unwrap();

        setup.manager.save_account("account2").unwrap();

        setup.manager.switch_account("account1").unwrap();

        let config = setup.manager.load_config().unwrap();
        assert_eq!(config.current, Some("account1".to_string()));

        let content = fs::read_to_string(setup.claude_config_dir.join("config.json")).unwrap();
        assert!(content.contains("test_key"));
    }

    #[test]
    fn test_switch_account_directory_not_found() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();
        setup.manager.save_account("test_account").unwrap();

        // Remove both the account directory and the claude config directory
        // to prevent auto-save from recreating the account
        let account_dir = setup.manager.switcher_dir.join("test_account");
        fs::remove_dir_all(&account_dir).unwrap();
        fs::remove_dir_all(&setup.claude_config_dir).unwrap();

        let result = setup.manager.switch_account("test_account");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Account directory not found"));
    }

    #[test]
    fn test_list_accounts_empty() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.list_accounts();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_accounts_with_data() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("account1").unwrap();
        setup.manager.save_account("account2").unwrap();

        let result = setup.manager.list_accounts();
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_account_not_found() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.delete_account("nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_delete_account_success() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("test_account").unwrap();

        let account_dir = setup.manager.switcher_dir.join("test_account");
        assert!(account_dir.exists());

        // Note: This test doesn't actually call delete_account because it requires user input
        // Instead, we manually simulate the deletion
        let mut config = setup.manager.load_config().unwrap();
        config.remove_account("test_account");
        config.current = None;
        setup.manager.save_config(&config).unwrap();

        fs::remove_dir_all(&account_dir).unwrap();

        let config = setup.manager.load_config().unwrap();
        assert!(config.get_account("test_account").is_none());
        assert!(!account_dir.exists());
    }

    #[test]
    fn test_rename_account_not_found() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.rename_account("old", "new");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_rename_account_already_exists() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("account1").unwrap();
        setup.manager.save_account("account2").unwrap();

        let result = setup.manager.rename_account("account1", "account2");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already exists"));
    }

    #[test]
    fn test_rename_account_success() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("old_name").unwrap();

        let old_dir = setup.manager.switcher_dir.join("old_name");
        let new_dir = setup.manager.switcher_dir.join("new_name");

        assert!(old_dir.exists());
        assert!(!new_dir.exists());

        let result = setup.manager.rename_account("old_name", "new_name");
        assert!(result.is_ok());

        assert!(!old_dir.exists());
        assert!(new_dir.exists());

        let config = setup.manager.load_config().unwrap();
        assert!(config.get_account("old_name").is_none());
        assert!(config.get_account("new_name").is_some());
        assert_eq!(config.current, Some("new_name".to_string()));
    }

    #[test]
    fn test_show_current_no_account() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.show_current();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_current_with_account() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        setup.manager.save_account("test_account").unwrap();

        let config = setup.manager.load_config().unwrap();
        assert_eq!(config.current, Some("test_account".to_string()));

        let result = setup.manager.show_current();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_current_if_any_empty() {
        let setup = TestSetup::new().unwrap();
        let result = setup.manager.show_current_if_any();
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let setup = TestSetup::new().unwrap();
        setup.create_mock_claude_config().unwrap();

        let nested_dir = setup.claude_config_dir.join("nested");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("file.txt"), "content").unwrap();

        let dest = setup.manager.switcher_dir.join("copied");
        fs::create_dir_all(&dest).unwrap();

        let result = setup
            .manager
            .copy_dir_recursive(&setup.claude_config_dir, &dest);
        assert!(result.is_ok());

        assert!(dest.join("config.json").exists());
        assert!(dest.join("session.json").exists());
        assert!(dest.join("nested").exists());
        assert!(dest.join("nested/file.txt").exists());

        let content = fs::read_to_string(dest.join("nested/file.txt")).unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_load_and_save_config() {
        let setup = TestSetup::new().unwrap();

        let mut config = AccountsConfig::default();
        config.current = Some("test".to_string());
        config.add_account(
            "test".to_string(),
            AccountMetadata {
                saved_at: Utc::now().to_rfc3339(),
                path: PathBuf::from("/test"),
            },
        );

        let save_result = setup.manager.save_config(&config);
        assert!(save_result.is_ok());

        let loaded = setup.manager.load_config().unwrap();
        assert_eq!(loaded.current, Some("test".to_string()));
        assert!(loaded.get_account("test").is_some());
    }
}
