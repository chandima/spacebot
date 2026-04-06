# Authentication & Integration Status

## ✅ Active Integrations

All authentication and service integrations are working. This document serves as a reference for setup steps and current status.

## LLM Providers

| Provider | Status | Auth Method | Models Available | Notes |
|----------|--------|-------------|------------------|-------|
| GitHub Copilot | ✅ Active | Device flow OAuth | Claude Opus 4.6, GPT-5.3-Codex, GPT-5-mini, etc. | 26 models via dynamic discovery |
| OpenAI ChatGPT | ✅ Active | Device flow OAuth (imported from Codex CLI) | GPT-5.4, GPT-5.3-Codex, GPT-5.2-Codex, GPT-5.1 | 7 models on Codex endpoint |
| LiteLLM Gateway | ✅ Active | API key | GLM-5, GPT-5-mini, Kimi-k2.5, DeepSeek | Custom provider at `litellm.sandbox.iamzone.dev` |
| Anthropic | ⏳ Available | Browser PKCE | Claude 3.5 Sonnet, Haiku | CLI auth works (`spacebot auth login`) |

### Model Routing (Current)

All agents are configured with multi-provider routing to preserve Copilot quota:

| Agent | Channel | Worker | Branch |
|-------|---------|--------|--------|
| default-agent | `github-copilot/claude-opus-4.6` | `github-copilot/gpt-5-mini` | `github-copilot/gpt-5-mini` |
| architect-agent | `github-copilot/claude-opus-4.6` | `github-copilot/claude-opus-4.6` | `github-copilot/gpt-5-mini` |
| coder-agent | `github-copilot/claude-sonnet-4.5` | `github-copilot/claude-sonnet-4.5` | `github-copilot/gpt-5-mini` |
| reviewer-agent | `litellm/glm-5` | `litellm/glm-5` | `github-copilot/gpt-5-mini` |
| slack/notebook/google | `github-copilot/gpt-5-mini` | `github-copilot/gpt-5-mini` | `github-copilot/gpt-5-mini` |
| All compactor/cortex | `openai-chatgpt/gpt-5.1-codex-mini` | — | — |

**Fallback chains** configured across providers for resilience.

## MCP (Model Context Protocol) Integrations

All MCP servers are active and integrated via stdio transport.

| MCP Server | Status | Auth Method | Agents | Purpose |
|------------|--------|-------------|--------|---------|
| **searxng** | ✅ Active | None (public instance) | All | Web search via SearXNG |
| **microsoft** | ✅ Active | OAuth (browser flow) | All | M365 (calendar, mail, contacts, files) |
| **linkedin** | ✅ Active | Browser session (Patchright) | All | LinkedIn profiles, companies, posts, jobs |
| **enterprise-slack** | ✅ Active | Browser tokens (`xoxc`/`xoxd`) | `slack-agent` only | Enterprise Slack search/history |
| **notebooklm** | ✅ Active | Google account | `notebooklm-agent` only | Google NotebookLM |
| **google-workspace** | ✅ Active | OAuth | `google-agent` only | Drive, Docs, Slides, Sheets, Gmail, Calendar |
| **youtube** | ✅ Active | OAuth (Data API) + unauthenticated (transcripts) | `google-agent` only | YouTube subscriptions, search, transcripts |

### MCP Authentication Setup Steps

#### 1. Microsoft 365 (Already Configured)
```bash
# OAuth flow is automatic on first use
# Token cache: ~/.spacebot/ms365/token-cache.json
# Selected account: ~/.spacebot/ms365/selected-account.json
```

#### 2. LinkedIn (New — Setup Complete)
```bash
# One-time browser login
linkedin-scraper-mcp --login

# Verify session
linkedin-scraper-mcp --status
# Expected: ✅ Session is valid (profile: ~/.linkedin-mcp/profile)

# Session persists across restarts, lasts 30-90 days
```

**Session storage:** `~/.linkedin-mcp/profile` (Chromium profile with auth cookies)

#### 3. Enterprise Slack (Already Configured)
```bash
# Manual token extraction from Slack web app
# Tokens stored in config.toml (slack-agent only)
```

#### 4. Google Workspace & YouTube (Already Configured)
```bash
# OAuth flow via workspace-mcp CLI
# Credentials stored in ~/.config/workspace-mcp/
```

#### 5. NotebookLM (Already Configured)
```bash
# Google account auth via notebooklm-mcp-cli
# Credentials stored in NotebookLM MCP config
```

## Messaging Adapters

| Platform | Status | Auth Method | Bound Agent | Notes |
|----------|--------|-------------|-------------|-------|
| **Slack** | ✅ Active | Socket Mode (bot + app tokens) | `default-agent` | 10 cron jobs for monitoring |

**Slack tokens:**
- Bot token (`xoxb-`): For sending/receiving messages
- App token (`xapp-`): For Socket Mode connection
- Stored in `config.toml` (encrypted at rest via secrets store)

## Historical Issues (Resolved)

### Previously: GitHub Copilot Enterprise ❌ → ✅ Resolved

**Old problem:** Standard GitHub PAT didn't have Copilot API access

**Solution:** Switched from PAT exchange to OAuth device flow
- Uses official Copilot OAuth client ID (`Ov23li8tweQw6odWQebz`)
- No PAT needed
- Direct bearer token access to `api.githubcopilot.com`
- Dynamic model discovery via `/models` endpoint

### Previously: OpenAI Codex (ASU Enterprise) ❌ → ✅ Resolved

### 1. GitHub Copilot Enterprise ❌
**Problem:** Standard GitHub PAT doesn't have Copilot API access

**Error:**
```
404 Not Found: GitHub Copilot token exchange failed
Endpoint: https://api.github.com/copilot_internal/v2/token
```

**What you need:**
- Contact your GitHub Enterprise admin
- Request a PAT with `copilot` scope
- Or ask for the correct Copilot API authentication method

### Previously: OpenAI Codex (ASU Enterprise) ❌ → ✅ Resolved

**Old problem:** Device code authentication disabled by workspace admin

**Solution:** Import OAuth tokens from Codex CLI
```bash
spacebot auth openai --from-codex
```
- Reads tokens from `~/.codex/auth.json`
- Reuses existing Codex CLI authentication
- Works with ASU SSO that blocks direct device flow

---

## Quick Reference: All Auth Commands

### LLM Providers
```bash
# GitHub Copilot (OAuth device flow)
spacebot auth copilot

# OpenAI (import from Codex CLI)
spacebot auth openai --from-codex

# Anthropic (browser PKCE)
# Anthropic (browser PKCE)
spacebot auth login

# Check all provider status
spacebot auth status
```

### MCP Services
```bash
# LinkedIn (browser session)
linkedin-scraper-mcp --login
linkedin-scraper-mcp --status

# Microsoft 365 (auto-prompts on first use)
# No manual auth needed

# Google Workspace (via workspace-mcp)
workspace-mcp --login

# Enterprise Slack (manual token extraction)
# See: docs/design-docs/named-messaging-adapters.md
```

## Setup Checklist for New Spacebot Instance

### 1. Install Dependencies
```bash
# Core
brew install sccache  # macOS build cache
export RUSTC_WRAPPER="$(which sccache)"

# MCP servers
uv tool install linkedin-scraper-mcp
npm install -g @softeria/ms-365-mcp-server
npm install -g workspace-mcp
npm install -g youtube-mcp
npm install -g mcp-searxng
```

### 2. LLM Provider Auth
```bash
# Authenticate to at least one provider
spacebot auth copilot          # GitHub Copilot (recommended)
# OR
spacebot auth openai --from-codex  # OpenAI via Codex CLI
# OR
spacebot auth login            # Anthropic Claude
```

### 3. MCP Service Auth
```bash
# LinkedIn (required for LinkedIn integration)
linkedin-scraper-mcp --login

# Microsoft 365 will auto-prompt on first tool use

# Google Workspace (if using google-agent)
workspace-mcp --login
```

### 4. Messaging Platform (Optional)
```bash
# Slack Socket Mode setup
# 1. Create Slack app at api.slack.com/apps
# 2. Enable Socket Mode
# 3. Add bot and app tokens to config.toml
# See: docs/design-docs/named-messaging-adapters.md
```

### 5. Build & Deploy
```bash
cd ~/Documents/Projects/spacebot

# Build release binary (20-25 min first time, faster with sccache)
cargo build --release

# Deploy
just deploy
# Or manually:
# cp target/release/spacebot ~/.local/bin/spacebot
# codesign -s - -f ~/.local/bin/spacebot  # macOS only, required
# ~/.local/bin/spacebot restart
```

### 6. Verify
```bash
# Check daemon status
~/.local/bin/spacebot status

# Check provider auth
spacebot auth status

# Check MCP servers loaded
curl -s http://localhost:19898/api/agents | python3 -m json.tool | grep -A 5 mcp_servers

# Check web UI
open http://localhost:19898  # or spacebot.chandis.casa if tunneled
```

## Session Persistence

| Service | Session Storage | Lifetime | Renewal |
|---------|-----------------|----------|---------|
| GitHub Copilot | `~/.spacebot/data/github_copilot_token.json` | No expiry (device flow token) | Auto-refresh |
| OpenAI ChatGPT | `~/.spacebot/data/openai_token.json` | Refresh token (90 days) | Auto-refresh |
| Anthropic | `~/.spacebot/data/anthropic_token.json` | Refresh token (90 days) | Auto-refresh |
| LinkedIn | `~/.linkedin-mcp/profile` | 30-90 days | Manual re-login |
| Microsoft 365 | `~/.spacebot/ms365/token-cache.json` | Refresh token | Auto-refresh via MSAL |
| Google Workspace | `~/.config/workspace-mcp/` | Refresh token | Auto-refresh |
| Slack Bot | `config.toml` (encrypted) | No expiry | Manual token rotation |

## Secrets Management

All secrets are stored in `~/.spacebot/data/secrets.redb` (encrypted AES-256-GCM):
- LLM provider tokens
- OAuth refresh tokens
- MCP server credentials

**Config file** (`~/.spacebot/config.toml`):
- References secrets by ID (e.g., `api_key = "env:OPENAI_API_KEY"`)
- No plaintext secrets in config
- Exception: Slack tokens (stored encrypted in config for now)

## Backup Strategy

**What to back up:**
```bash
# Identity files (version-controlled in deploy/)
~/.spacebot/agents/*/identity/*.md

# Skills (version-controlled in deploy/)
~/.spacebot/agents/*/skills/

# Config (version-controlled in deploy/, secrets redacted)
~/.spacebot/config.toml → deploy/config.toml.template

# Secrets (use Time Machine, DO NOT commit)
~/.spacebot/data/secrets.redb
~/.linkedin-mcp/profile
~/.spacebot/ms365/
~/.config/workspace-mcp/
```

**What NOT to commit:**
- `~/.spacebot/data/` — SQLite DBs, secrets, embeddings
- `~/.linkedin-mcp/profile` — LinkedIn session cookies
- Any file with actual tokens/keys

## Troubleshooting

### Provider Auth Failed
```bash
# Re-authenticate
spacebot auth <provider>

# Check token expiry
spacebot auth status

# Check logs
tail -f ~/.spacebot/logs/spacebot.log.$(date +%Y-%m-%d) | grep -i auth
```

### MCP Server Not Loading
```bash
# Check MCP status
curl -s http://localhost:19898/api/mcp/status | python3 -m json.tool

# Check logs for errors
tail -f ~/.spacebot/logs/spacebot.log.$(date +%Y-%m-%d) | grep -i mcp

# Test MCP server directly
linkedin-scraper-mcp --status
```

### Slack Not Connecting
```bash
# Check Socket Mode connection
tail -f ~/.spacebot/logs/spacebot.log.$(date +%Y-%m-%d) | grep -i slack

# Verify tokens haven't expired (regenerate in Slack app settings)
# Restart after token update
~/.local/bin/spacebot restart
```

## Additional Documentation

- **Architecture**: `README.md`
- **Agent System**: `AGENTS.md`
- **MCP Integration**: `docs/design-docs/mcp.md`
- **Google/YouTube**: `docs/design-docs/google-youtube-integration.md`
- **LinkedIn**: `docs/design-docs/linkedin-integration.md`
- **Slack Adapter**: `docs/design-docs/named-messaging-adapters.md`
- **Copilot Setup**: `docs/copilot-setup.md`
