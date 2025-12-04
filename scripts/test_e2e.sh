#!/bin/bash
source $HOME/.cargo/env

# Note: This script relies on GEMINI_API_KEY being set in the environment
# OR in a .env file in the working directory.

echo "🔨 Building Workspace..."
cd ~/Projects/sensei-core
cargo build --quiet

echo "🚀 Starting Server..."
./target/debug/sensei-server &
SERVER_PID=$!

echo "⏳ Waiting for server..."
sleep 5

echo "📡 Running Client..."
./target/debug/sensei-client --ask "Hello Rust, are you connected to Gemini?"

echo "🛑 Stopping Server..."
kill $SERVER_PID
