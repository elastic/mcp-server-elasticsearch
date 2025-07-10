#!/usr/bin/env bash

PRINT_TARGET=false
if [[ $1 == "--print-target" ]]; then
  PRINT_TARGET=true
  shift
fi

# build Rust toolchain name
if [[ "$RUST_ARCH" == "x64" ]]; then
  ARCH="x86_64"
elif [[ "$RUST_ARCH" == "arm64" ]]; then
  ARCH="aarch64"
else
  echo "Unsupported architecture: $RUST_ARCH"
  exit 1
fi

if [[ "$RUST_OS" == "linux" ]]; then
  OS="unknown-linux-gnu"
elif [[ "$RUST_OS" == "macos" ]]; then
  OS="apple-darwin"
elif [[ "$RUST_OS" == "windows" ]]; then
  OS="pc-windows-msvc"
else
  echo "Unsupported OS: $RUST_OS"
  exit 1
fi

RUST_TOOLCHAIN="$ARCH-$OS"

if [[ "$PRINT_TARGET" == true ]]; then
  echo "$RUST_TOOLCHAIN"
  exit 0
else
  cargo build --release --target "$RUST_TOOLCHAIN"
fi
