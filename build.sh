#!/bin/bash

# Build script for kpasscli Rust application
# Cross-compilation for multiple platforms

set -e

# cd kpasscli-rs
echo "Build linux local binary of kpasscli"
cargo build --release -v

echo
echo "Build linux x86_64 binary of kpasscli"
mkdir -p dist/linux-amd64
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/kpasscli dist/linux-amd64/kpasscli

echo
echo "Build windows x86_64 binary of kpasscli"
mkdir -p dist/windows-amd64
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/kpasscli.exe dist/windows-amd64/kpasscli.exe

# echo
# echo "Build darwin x86_64 binary of kpasscli"
# mkdir -p ../dist/darwin-amd64
# cargo build --release --target x86_64-apple-darwin
# cp target/x86_64-apple-darwin/release/kpasscli ../dist/darwin-amd64/kpasscli

# echo
# echo "Build darwin arm64 binary of kpasscli"
# mkdir -p ../dist/darwin-arm64
# cargo build --release --target aarch64-apple-darwin
# cp target/aarch64-apple-darwin/release/kpasscli ../dist/darwin-arm64/kpasscli

cd ..

# echo
# echo "Build process finished."
# echo
# echo "Note: Cross-compilation requires the appropriate Rust targets installed:"
# echo "  rustup target add x86_64-unknown-linux-gnu"
# echo "  rustup target add x86_64-pc-windows-gnu"
# echo "  rustup target add x86_64-apple-darwin"
# echo "  rustup target add aarch64-apple-darwin"
# echo
# echo "Additionally, cross-compilation may require platform-specific toolchains:"
# echo "  - For Windows: mingw-w64"
# echo "  - For macOS (on Linux): osxcross or similar"
# echo
