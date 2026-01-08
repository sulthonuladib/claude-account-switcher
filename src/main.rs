mod cli;
mod config;
mod error;
mod manager;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use manager::AccountManager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manager = AccountManager::new()?;

    match cli.command {
        Commands::Save { name } => manager.save_account(&name),
        Commands::Switch { name } => manager.switch_account(&name),
        Commands::List => manager.list_accounts(),
        Commands::Delete { name } => manager.delete_account(&name),
        Commands::Rename { old_name, new_name } => manager.rename_account(&old_name, &new_name),
        Commands::Current => manager.show_current(),
    }
}
