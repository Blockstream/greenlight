#!/bin/bash

# Setup script for gl-sdk Node.js bindings
# This script helps you integrate the napi-rs bindings into your Greenlight repository

set -e

echo "========================================="
echo "Greenlight SDK - Node.js Bindings Setup"
echo "========================================="
echo ""

# Check if we're in the greenlight repository
if [ ! -f "../gl-sdk/Cargo.toml" ]; then
    echo "Error: Please run this script from the root of the greenlight repository"
    exit 1
fi

echo "✓ Found greenlight repository"

# Check for required tools
echo ""
echo "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "✗ Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "✓ Rust/Cargo installed"

if ! command -v node &> /dev/null; then
    echo "✗ Node.js not found. Please install Node.js >= 16"
    exit 1
fi
echo "✓ Node.js installed ($(node --version))"

if ! command -v npm &> /dev/null; then
    echo "✗ npm not found. Please install npm"
    exit 1
fi
echo "✓ npm installed ($(npm --version))"

if ! command -v protoc &> /dev/null; then
    echo "⚠ Warning: protoc (Protocol Buffers compiler) not found"
    echo "  Install it with:"
    echo "    Ubuntu/Debian: sudo apt-get install protobuf-compiler"
    echo "    macOS: brew install protobuf"
    echo "    Windows: Download from https://github.com/protocolbuffers/protobuf/releases"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✓ protoc installed ($(protoc --version))"
fi

echo ""
echo "========================================="
echo "Next Steps:"
echo "========================================="
echo ""
echo "1. Install napi-rs CLI globally: npm install -g @napi-rs/cli"
echo ""
echo "2. Navigate to the directory: cd libs/gl-sdk-napi"
echo ""
echo "3. Install Node.js dependencies: npm install"
echo ""
echo "4. Build the native module: npm run build"
echo ""
echo "5. Test it: node -e \"const gl = require('./index'); console.log(gl);\""
echo ""
echo "6. Run Tests: npm test"
echo ""
echo "7. Or run the example: npx ts-node example.ts"
echo ""
