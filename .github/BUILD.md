# Build Scripts

This directory contains scripts for building and releasing the Claude Account Switcher.

## Local Testing

To test the release process locally:

```bash
# Test building for all targets (requires cross-compilation setup)
./build-all.sh

# Build for current platform only
cargo build --release
```

## Release Process

1. **Create a new tag:**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **The GitHub Actions workflow will automatically:**
   - Build binaries for all supported platforms and architectures
   - Create a GitHub release
   - Upload all binary assets
   - Generate release notes

3. **Manual release (if needed):**
   - Go to GitHub Actions
   - Run the "Release" workflow manually
   - Provide a tag name (e.g., v0.1.0)

## Supported Platforms

- **Linux:**
  - x86_64 (Intel/AMD 64-bit)
  - aarch64 (ARM 64-bit)
  - armv7 (ARM 32-bit)
  - i686 (Intel/AMD 32-bit)

- **macOS:**
  - x86_64 (Intel Macs)
  - aarch64 (Apple Silicon Macs)

## Binary Naming Convention

- Linux: `claude-account-switcher-linux-{arch}.tar.gz`
- macOS: `claude-account-switcher-macos-{arch}.tar.gz`

Where `{arch}` is one of: x86_64, aarch64, armv7, i686