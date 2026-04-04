#!/usr/bin/env bash
# Crash-loop protection wrapper for launchd.
# Stops restarting spacebot after MAX_FAILURES consecutive rapid failures.
# A "rapid failure" is an exit within GRACE_PERIOD seconds of launch.
# If the process runs longer than GRACE_PERIOD, the failure counter resets.

set -euo pipefail

MAX_FAILURES=5
GRACE_PERIOD=30
STATE_DIR="${HOME}/.spacebot"
FAILURE_FILE="${STATE_DIR}/crash_count"
BINARY="${HOME}/.local/bin/spacebot"

mkdir -p "$STATE_DIR"

# Read current failure count
failures=0
if [ -f "$FAILURE_FILE" ]; then
    failures=$(cat "$FAILURE_FILE" 2>/dev/null || echo 0)
fi

if [ "$failures" -ge "$MAX_FAILURES" ]; then
    echo "$(date -Iseconds) CRASH LOOP DETECTED: $failures consecutive rapid failures (limit: $MAX_FAILURES)." >&2
    echo "Spacebot will not restart until the crash counter is reset." >&2
    echo "To reset: rm ${FAILURE_FILE} && launchctl unload ~/Library/LaunchAgents/com.spacebot.agent.plist && launchctl load ~/Library/LaunchAgents/com.spacebot.agent.plist" >&2
    # Exit 0 so launchd's SuccessfulExit=false does NOT restart us
    exit 0
fi

start_time=$(date +%s)

# Run spacebot, forwarding all signals
"$BINARY" start "$@"
exit_code=$?

elapsed=$(( $(date +%s) - start_time ))

if [ "$exit_code" -ne 0 ] && [ "$elapsed" -lt "$GRACE_PERIOD" ]; then
    failures=$((failures + 1))
    echo "$failures" > "$FAILURE_FILE"
    echo "$(date -Iseconds) Rapid failure #${failures}/${MAX_FAILURES} (exited ${exit_code} after ${elapsed}s)" >&2
else
    # Ran long enough — reset counter
    rm -f "$FAILURE_FILE"
fi

exit "$exit_code"
