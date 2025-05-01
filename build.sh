#!/bin/bash

# Build script for compiling Rust to WebAssembly

echo "🦀 Building Rust WebAssembly Shooting Game 🚀"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️ wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Build the WebAssembly module
echo "🔧 Building WebAssembly module..."
wasm-pack build --target web --out-dir pkg

# Check if the build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "🌐 Open index.html in a browser to play the game"
else
    echo "❌ Build failed!"
    exit 1
fi 