#!/bin/bash
# Test Spacebot GitHub Copilot integration

echo "Starting Spacebot..."
spacebot start --foreground --config ./config.toml > spacebot-test.log 2>&1 &
SPACEBOT_PID=$!
echo "Spacebot PID: $SPACEBOT_PID"

# Wait for server to start
echo "Waiting for API server to start..."
sleep 15

# Check if server is running
if curl -s http://127.0.0.1:19898/api/health > /dev/null 2>&1; then
    echo "✓ API server is running"
else
    echo "✗ API server not responding"
    echo "Last 20 lines of log:"
    tail -20 spacebot-test.log
    kill $SPACEBOT_PID 2>/dev/null
    exit 1
fi

# Test GitHub Copilot provider
echo ""
echo "Testing GitHub Copilot provider..."
curl -s http://127.0.0.1:19898/api/settings 2>&1 | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'GitHub Copilot configured: {bool(data.get(\"llm_config\", {}).get(\"github_copilot_key\"))}')" 2>/dev/null || echo "Could not check settings"

echo ""
echo "Server logs (last 30 lines):"
tail -30 spacebot-test.log

# Keep running for manual testing
echo ""
echo "Spacebot is running. Press Ctrl+C to stop."
echo "Web UI: http://127.0.0.1:19898"
wait $SPACEBOT_PID
