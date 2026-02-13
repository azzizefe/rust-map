#!/bin/bash

# GeoPic Linux Setup Script
# Automatically installs dependencies for Ubuntu/Debian, Fedora, and Arch.

set -e

echo "📍 Starting GeoPic Setup for Linux..."

# Check Internet Connectivity
if ! ping -c 1 google.com &> /dev/null; then
    echo "⚠️ No internet connection detected. Please check your network."
    exit 1
fi

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "❌ Could not detect OS. Please install dependencies manually."
    exit 1
fi

case $OS in
    ubuntu|debian|pop|mint)
        echo "📦 detected $OS. Installing dependencies via apt..."
        sudo apt update
        sudo apt install -y pkg-config libssl-dev build-essential ca-certificates
        ;;
    fedora)
        echo "📦 detected $OS. Installing dependencies via dnf..."
        sudo dnf install -y pkg-config openssl-devel
        ;;
    arch)
        echo "📦 detected $OS. Installing dependencies via pacman..."
        sudo pacman -S --noconfirm pkgconf openssl base-devel
        ;;
    *)
        echo "⚠️ OS not recognized. Please ensure pkg-config and openssl-dev are installed."
        ;;
esac

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "🦀 Rust not found. Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "✅ Rust is already installed."
fi

echo "🚀 Building GeoPic..."
cargo build --release

echo "✨ Setup complete! You can now run GeoPic using: ./target/release/rust-map <photo_path>"
