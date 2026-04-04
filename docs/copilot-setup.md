# Copilot CLI Configuration

Development on this repo uses GitHub Copilot CLI. The configuration lives across several locations — this document records the setup so it can be reconstructed after a crash or fresh machine.

## Spacebot Daemon (launchd)

Spacebot runs as a macOS LaunchAgent managed by `launchd`.

**Plist location:** `~/Library/LaunchAgents/com.spacebot.agent.plist`

```xml
<key>ProgramArguments</key>
<array>
    <string>/Users/chandima/.local/bin/spacebot-wrapper.sh</string>
</array>

<key>KeepAlive</key>
<dict>
    <key>SuccessfulExit</key>
    <false/>
    <key>Crashed</key>
    <true/>
</dict>

<key>ThrottleInterval</key>
<integer>10</integer>
```

Key behaviour:
- `Crashed: true` — launchd restarts spacebot after any non-zero exit
- `SuccessfulExit: false` — launchd does NOT restart after a clean exit (exit 0)
- `ThrottleInterval: 10` — minimum 10 seconds between restart attempts

### Crash-Loop Protection Wrapper

The plist runs `~/.local/bin/spacebot-wrapper.sh` (source: `scripts/launchd-wrapper.sh`) instead of the binary directly. The wrapper tracks consecutive rapid failures (exit within 30 seconds of launch). After 5 rapid failures, it exits 0 to stop the restart cycle.

- **Crash counter file:** `~/.spacebot/crash_count`
- **Grace period:** 30 seconds (exits before this count as rapid failures)
- **Max rapid failures:** 5

**To reset after a crash loop:**
```bash
rm ~/.spacebot/crash_count
launchctl unload ~/Library/LaunchAgents/com.spacebot.agent.plist
launchctl load ~/Library/LaunchAgents/com.spacebot.agent.plist
```

**Note:** `just deploy` automatically resets the crash counter, copies the wrapper, and restarts the service.

### Common launchd Commands

```bash
# Quick restart (handles stale PIDs, force-kills, waits for startup)
spacebot-restart

# Check service status
launchctl list com.spacebot.agent

# Manual restart
launchctl unload ~/Library/LaunchAgents/com.spacebot.agent.plist
rm -f ~/.spacebot/spacebot.pid ~/.spacebot/spacebot.sock
launchctl load ~/Library/LaunchAgents/com.spacebot.agent.plist

# View logs
tail -f ~/.spacebot/launchd.err.log
tail -f ~/.spacebot/launchd.out.log
```

The `spacebot-restart` script lives at `~/.local/bin/spacebot-restart`. It force-kills any running process, cleans stale PID/socket files, reloads launchd, and waits up to 20 seconds to verify startup. Use it for quick restarts during development.

## Model & Core Settings

- **Default model:** `claude-opus-4.6`
- **Trusted folders:** `/Users/chandima/Documents/Projects`, `/private/tmp`
- **Config file:** `~/.copilot/config.json`

## Plugin: context-mode (MCP)

Context-window optimization plugin. Provides sandboxed code execution, FTS5 knowledge base, and intent-driven search. Saves ~98% of context tokens.

- **Source:** `mksglu/context-mode` (GitHub)
- **Install:** `copilot install mksglu/context-mode` (or via plugin marketplace)
- **Config lives at:** `~/.copilot/installed-plugins/_direct/mksglu--context-mode`

## Plugin: copilot-ntfy (notifications)

Sends push notifications via [ntfy](https://ntfy.sh) when an agent turn completes. Cross-platform (Copilot CLI, Claude Code, VS Code Copilot).

- **Source:** `~/Documents/Projects/opencode-config/.copilot/plugins/copilot-ntfy/`
- **Hook:** `agentStop` → runs `ntfy_notify.sh` (15s timeout)
- **Config lives at:** `~/.copilot/hooks/copilot-ntfy.json`

## Skills

All skills are symlinked from `~/Documents/Projects/opencode-config/skills/` to `~/.copilot/skills/`. To restore after a fresh setup:

```bash
cd ~/Documents/Projects/opencode-config
for skill in skills/*/; do
    name=$(basename "$skill")
    ln -sf "$(pwd)/$skill" ~/.copilot/skills/"$name"
done
```

| Skill | Purpose |
|-------|---------|
| `agent-browser` | Browser automation (Playwright-based page interaction, screenshots) |
| `context7-docs` | Fetch official library docs via Context7 MCP (React, Tailwind, etc.) |
| `github-ops` | GitHub operations via `gh` CLI — full MCP Server parity without token overhead |
| `mcporter` | Direct MCP server access — discover servers, list tools, call tools |
| `planning-doc` | Create/update PLAN.md planning documents for multi-phase work |
| `production-hardening` | Scan for resilience anti-patterns; add retries, circuit breakers, timeouts |
| `security-auditor` | Pre-deployment security audit for production releases |
| `skill-creator` | Scaffold, test, and optimize OpenCode/Copilot skills |

### Project-Level Skills (Spacebot's own)

These are skills for the spacebot agent itself (not the Copilot CLI). They live in `agents/skills/` and are loaded by spacebot at runtime. Do not confuse with Copilot CLI skills above.

## MCP Servers

The Copilot CLI automatically connects to a hosted `github-mcp-server` (remote, via Streamable HTTP to `api.enterprise.githubcopilot.com`). This is not user-configured — it's built into the CLI. If it fails to connect on startup (network timeout), the session continues without it; the `github-ops` skill covers the same functionality via `gh` CLI.
