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

        let claude_config_dir = home.join(".config").join("claude");
        let switcher_dir = home.join(".claude-accounts");
        let accounts_file = switcher_dir.join("accounts.json");

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
