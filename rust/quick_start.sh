#!/bin/bash

# Quick Start Script - Development Mode
# For testing without full release build

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

export DATABASE_URL="sqlite://opensim.db"
export RUST_LOG="${RUST_LOG:-info}"

echo "🚀 Quick Start - Development Mode"
echo "📁 $(pwd)"
echo "💾 Database: $(pwd)/opensim.db"

# Ensure database exists
touch opensim.db

echo "🔨 Building (debug mode)..."
cargo build --bin opensim-next

echo "🌐 Starting server..."
exec ./target/debug/opensim-next start