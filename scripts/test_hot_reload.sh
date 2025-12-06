#!/bin/bash
# Test Hot Reloading Logic

CONFIG_FILE="mcp_reload_test.json"
SERVER_BIN="./target/release/sensei-server"
LOG_FILE="server_reload.log"

# 1. Setup initial empty config
echo '{"mcpServers": {}}' > $CONFIG_FILE

# 2. Start Server
echo "🚀 Starting Server..."
export SENSEI_MCP_CONFIG=$CONFIG_FILE
# Force TCP just in case
export SENSEI_LISTEN_ADDR="0.0.0.0:3005" 

$SERVER_BIN > $LOG_FILE 2>&1 &
SERVER_PID=$!

echo "⏳ Waiting for startup..."
sleep 2

# 3. Modify Config (Add a dummy agent)
echo "📝 Updating config (Hot Reload trigger)..."
# We add a 'filesystem' agent pointing to 'ls' just as a dummy command that runs and exits (it will fail to connect as MCP but will trigger the add logic)
cat <<EOF > $CONFIG_FILE
{
  "mcpServers": {
    "hot_agent": {
      "command": "echo",
      "args": ["hello"]
    }
  }
}
EOF

echo "⏳ Waiting for watcher (6s)..."
sleep 7

# 4. Check Logs
echo "🔍 Checking logs..."
if grep -q "Configuration change detected" $LOG_FILE; then
    echo "✅ Change detected!"
else
    echo "❌ Change NOT detected."
    cat $LOG_FILE
    kill $SERVER_PID
    rm $CONFIG_FILE $LOG_FILE
    exit 1
fi

if grep -q "Adding new agent 'hot_agent'" $LOG_FILE; then
    echo "✅ Agent 'hot_agent' addition triggered!"
else
    echo "❌ Agent addition NOT triggered."
    cat $LOG_FILE
    kill $SERVER_PID
    rm $CONFIG_FILE $LOG_FILE
    exit 1
fi

# Cleanup
kill $SERVER_PID
rm $CONFIG_FILE $LOG_FILE
echo "🎉 Hot Reload Test Passed!"
