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
        Some(Commands::Save { name }) => manager.save_account(&name),
        Some(Commands::Switch { name }) => manager.switch_account(&name),
        Some(Commands::List) => manager.list_accounts(),
        Some(Commands::Delete { name }) => manager.delete_account(&name),
        Some(Commands::Rename { old_name, new_name }) => manager.rename_account(&old_name, &new_name),
        Some(Commands::Current) => manager.show_current(),
        None => manager.show_current_if_any(),
    }
}
