# Claude Account Switcher

A command-line tool to manage multiple Claude Code CLI accounts seamlessly.

## Features

- Save and switch between multiple Claude Code accounts
- List all saved accounts with timestamps  
- Delete accounts you no longer need
- Rename accounts for better organization
- Show current active account

## Installation

### Pre-built Binaries

Download the latest release from the [GitHub Releases page](https://github.com/sulthonuladib/claude-account-switcher/releases).

**Supported Platforms:**
- **Linux**: x86_64, aarch64, armv7, i686
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)

**Installation steps:**
1. Download the appropriate binary for your platform
2. Extract the archive: `tar -xzf claude-account-switcher-*.tar.gz`
3. Move the binary to a directory in your PATH: `sudo mv claude-account-switcher /usr/local/bin/`
4. Make it executable: `chmod +x /usr/local/bin/claude-account-switcher`

### Build from Source

```bash
git clone https://github.com/sulthonuladib/claude-account-switcher
cd claude-account-switcher
cargo build --release
sudo cp target/release/claude-account-switcher /usr/local/bin/
```

## Usage

### Basic Commands

```bash
# Save your current Claude Code account with a name
claude-account-switcher save work

# Switch to a different account
claude-account-switcher switch personal

# List all saved accounts
claude-account-switcher list

# Show currently active account
claude-account-switcher current

# Delete an account
claude-account-switcher delete old-account

# Rename an account
claude-account-switcher rename old-name new-name
```

### Example Workflow

```bash
# Save your work account
claude-account-switcher save work

# Save your personal account  
claude-account-switcher save personal

# List accounts to see what's available
claude-account-switcher list
# Output:
# Claude Code Accounts:
# ------------------------------------------------------------
# * personal           (saved: 2024-01-09T10:30)
#   work              (saved: 2024-01-09T09:15)

# Switch to work account
claude-account-switcher switch work

# Check current account
claude-account-switcher current
# Output: work
```

## How It Works

The tool manages your Claude Code CLI configuration by:
- Storing account configurations in `~/.claude-accounts/`
- Backing up your current `~/.config/claude/` directory
- Switching between saved configurations seamlessly
- Tracking which account is currently active

## Requirements

- Claude Code CLI must be installed and authenticated at least once
- Linux or macOS operating system
- No additional dependencies

## Development

Built with:
- Rust 2024 edition
- Latest dependency versions
- GitHub Actions for automated releases
- Comprehensive CI/CD pipeline

### Making a Release

Use the semantic release scripts for easy version management:

```bash
# For bug fixes (0.1.0 -> 0.1.1)
./release.sh patch
# or
./quick-release.sh fix

# For new features (0.1.0 -> 0.2.0)
./release.sh minor
# or
./quick-release.sh feature

# For breaking changes (0.1.0 -> 1.0.0)
./release.sh major
# or
./quick-release.sh breaking
```

The release script will:
1. Run pre-flight checks (git status, tests)
2. Update version in Cargo.toml
3. Run tests and build
4. Commit version changes
5. Create and push git tag
6. Trigger automated GitHub release

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

Made with care for the Claude Code community