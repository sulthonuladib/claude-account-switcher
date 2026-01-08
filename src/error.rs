use std::fmt;

#[derive(Debug)]
pub enum AccountError {
    NotFound(String),
    AlreadyExists(String),
    NoConfiguration,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Account '{}' not found", name),
            Self::AlreadyExists(name) => write!(f, "Account '{}' already exists", name),
            Self::NoConfiguration => write!(f, "No Claude Code configuration found. Please authenticate first with: claude-code auth"),
        }
    }
}

impl std::error::Error for AccountError {}
