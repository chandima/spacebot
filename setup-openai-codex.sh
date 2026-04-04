#!/bin/bash
set -e

echo "=== OpenAI Codex (ChatGPT) OAuth Setup ===" 
echo ""

# Find and stop existing spacebot
OLD_PID=$(ps aux | grep "[s]pacebot start" | awk '{print $2}' | head -1)
if [ -n "$OLD_PID" ]; then
    echo "Stopping existing Spacebot (PID: $OLD_PID)..."
    kill $OLD_PID 2>/dev/null || true
    sleep 2
fi

# Start Spacebot
echo "Starting Spacebot..."
spacebot start --foreground --config ./config.toml > /tmp/spacebot-oauth.log 2>&1 &
SPACEBOT_PID=$!
echo "Spacebot PID: $SPACEBOT_PID"

# Wait for API server
echo "Waiting for API server..."
for i in {1..20}; do
    if curl -s http://127.0.0.1:19898/api/health >/dev/null 2>&1; then
        echo "✓ API server ready"
        break
    fi
    sleep 1
done

# Start OAuth flow with model parameter
echo ""
echo "Initiating OpenAI OAuth flow..."
OAUTH_RESPONSE=$(curl -s -X POST http://127.0.0.1:19898/api/providers/openai/browser-oauth/start \
  -H "Content-Type: application/json" \
  -d '{"model":"openai-chatgpt/gpt-5.4"}')

echo ""
echo "Response:"
echo "$OAUTH_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$OAUTH_RESPONSE"

# Parse response
SUCCESS=$(echo "$OAUTH_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('success', False))" 2>/dev/null || echo "false")

if [ "$SUCCESS" = "True" ] || [ "$SUCCESS" = "true" ]; then
    USER_CODE=$(echo "$OAUTH_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('user_code', ''))")
    VERIFICATION_URL=$(echo "$OAUTH_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('verification_url', ''))")
    STATE=$(echo "$OAUTH_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('state', ''))")
    
    echo ""
    echo "══════════════════════════════════════════"
    echo "  OpenAI Device Authorization Required"
    echo "══════════════════════════════════════════"
    echo ""
    echo "1. Open this URL in your browser:"
    echo "   $VERIFICATION_URL"
    echo ""
    echo "2. Enter this code when prompted:"
    echo "   $USER_CODE"
    echo ""
    echo "3. Sign in with your OpenAI/ChatGPT account"
    echo "   (Must have Codex/ChatGPT Plus/Pro access)"
    echo ""
    echo "Waiting for you to authorize..."
    echo "(This may take a minute or two)"
    
    # Poll for status
    AUTHORIZED=false
    for i in {1..60}; do
        sleep 5
        STATUS_RESPONSE=$(curl -s "http://127.0.0.1:19898/api/providers/openai/browser-oauth/status?state=$STATE" 2>/dev/null)
        STATUS=$(echo "$STATUS_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('status', 'pending'))" 2>/dev/null || echo "pending")
        
        if [ "$STATUS" = "completed" ]; then
            AUTHORIZED=true
            echo ""
            echo "✓✓✓ Authorization Successful! ✓✓✓"
            echo ""
            echo "OAuth credentials have been saved to:"
            echo "  ~/.spacebot/openai_chatgpt_oauth.json"
            echo ""
            echo "Available models:"
            echo "  - openai-chatgpt/gpt-5.4 (frontier agentic coding)"
            echo "  - openai-chatgpt/gpt-5.4-mini (smaller, faster)"
            echo "  - openai-chatgpt/gpt-5.3-codex (codex-optimized)"
            echo "  - openai-chatgpt/gpt-5.2-codex"
            echo "  - openai-chatgpt/gpt-5.2"
            echo "  - openai-chatgpt/gpt-5.1-codex-max"
            echo "  - openai-chatgpt/gpt-5.1-codex-mini"
            echo ""
            echo "Spacebot is now configured and running!"
            echo "Web UI: http://127.0.0.1:19898"
            echo ""
            echo "Test it:"
            echo "  curl -X POST http://127.0.0.1:19898/api/webchat/send \\"
            echo "    -H 'Content-Type: application/json' \\"
            echo "    -d '{\"agent_id\":\"default-agent\",\"session_id\":\"test\",\"message\":\"Hello!\"}'"
            echo ""
            break
        elif [ "$STATUS" = "failed" ]; then
            MESSAGE=$(echo "$STATUS_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('message', 'Unknown error'))" 2>/dev/null || echo "Unknown error")
            echo ""
            echo "✗ Authorization failed: $MESSAGE"
            break
        fi
        
        printf "."
    done
    
    if [ "$AUTHORIZED" = "false" ]; then
        echo ""
        echo "Timeout waiting for authorization."
        echo "You can try again or authorize manually at:"
        echo "  $VERIFICATION_URL"
        echo "  Code: $USER_CODE"
    fi
else
    MESSAGE=$(echo "$OAUTH_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('message', 'Unknown error'))" 2>/dev/null || echo "Unknown error")
    echo ""
    echo "✗ Failed to start OAuth flow"
    echo "Error: $MESSAGE"
    echo ""
    echo "Spacebot logs:"
    tail -30 /tmp/spacebot-oauth.log
fi

echo ""
echo "Press Ctrl+C to stop Spacebot"
wait $SPACEBOT_PID
