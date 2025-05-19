#!/bin/bash
set -e

# Install wasm-bindgen-cli with the correct version
echo "Installing wasm-bindgen-cli version 0.2.100..."
cargo install -f wasm-bindgen-cli --version 0.2.100

# Build the project for wasm32-unknown-unknown target
echo "Building for WebAssembly..."
cargo build --target wasm32-unknown-unknown

# Run wasm-bindgen to generate JavaScript bindings
echo "Generating JavaScript bindings..."
wasm-bindgen --out-dir pkg --target web target/wasm32-unknown-unknown/debug/free_mint.wasm

# Run the tests
echo "Running tests..."
wasm-pack test --node

echo "WebAssembly tests completed successfully!"