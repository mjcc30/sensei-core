#!/bin/bash
set -e
# source $HOME/.cargo/env # Not needed inside agent env usually

echo "🔨 Using pre-built Release binaries..."
# cd ~/Projects/sensei-core # Path dependent, removing

SERVER_BIN="./target/release/sensei-server"
CLIENT_BIN="./target/release/sensei-client"

# Setup DB
export DATABASE_URL="sqlite://$(pwd)/sensei_e2e.db?mode=rwc"
rm sensei_e2e.db 2>/dev/null || true
sqlite3 sensei_e2e.db < crates/sensei-lib/migrations/20250101_init.sql
sqlite3 sensei_e2e.db < crates/sensei-lib/migrations/20250102_vectors.sql || true

echo "🚀 Starting Server..."
$SERVER_BIN &
SERVER_PID=$!

echo "⏳ Waiting for server..."
sleep 3

echo "📡 Running Client..."
$CLIENT_BIN --ask "Hello Rust, are you connected to Gemini?"

echo "🛑 Stopping Server..."
kill $SERVER_PID
rm sensei_e2e.db