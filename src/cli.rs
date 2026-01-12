use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-account")]
#[command(version, about = "Manage multiple Claude Code CLI accounts")]
#[command(author = "")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Save { name: String },
    Switch { name: String },
    List,
    Delete { name: String },
    Rename { old_name: String, new_name: String },
    Current,
}
