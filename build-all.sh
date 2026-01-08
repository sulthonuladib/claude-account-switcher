#!/bin/bash

# Local build script for testing cross-compilation
set -e

echo "Building Claude Account Switcher for all supported platforms..."

# Linux targets
targets=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "armv7-unknown-linux-gnueabihf"
    "i686-unknown-linux-gnu"
)

# macOS targets (only if running on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    targets+=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
    )
fi

# Install targets if needed
for target in "${targets[@]}"; do
    echo "Installing target: $target"
    rustup target add "$target" || echo "Target $target already installed"
done

# Build for each target
for target in "${targets[@]}"; do
    echo "Building for $target..."
    if cargo build --release --target "$target"; then
        echo "✅ Successfully built for $target"
    else
        echo "❌ Failed to build for $target"
    fi
done

echo "Build complete! Binaries are in target/{target}/release/"