# LinkedIn Integration

## Overview

Spacebot integrates with LinkedIn through the `linkedin` MCP server for professional network access, research, and data gathering. The integration uses the [stickerdaniel/linkedin-mcp-server](https://github.com/stickerdaniel/linkedin-scraper-mcp) (1,283 stars) — a Patchright-based (anti-detection Playwright fork) browser automation server that provides authenticated LinkedIn access.

## Architecture

```
                    ┌──────────────┐
                    │ default-agent│ ← Primary access
                    │ (Orchestrator)│
                    └──────┬───────┘
                           │ MCP tools
                           ▼
                    linkedin MCP
                  (Browser-based API)
               Patchright + persistent
                   session storage
```

### MCP Server

| Server | Source | Transport | Purpose |
|--------|--------|-----------|---------|
| `linkedin` | [stickerdaniel/linkedin-scraper-mcp](https://github.com/stickerdaniel/linkedin-scraper-mcp) | stdio | LinkedIn profiles, companies, posts, jobs, messaging, search |

**Key features:**
- Browser automation with anti-detection (Patchright)
- Persistent session storage (survives restarts)
- Full LinkedIn API access via authenticated browser
- Headless operation (production) and visible mode (debugging/auth)

## Tool Surface

Tools are auto-discovered from the MCP server. Key capabilities include:

### Profile & People
- `linkedin_search_profiles` — Search for people by name, title, company
- `linkedin_get_profile` — Fetch detailed profile information
- `linkedin_get_connections` — List user's connections

### Companies
- `linkedin_search_companies` — Search for companies by name, industry
- `linkedin_get_company` — Fetch company page details
- `linkedin_get_company_employees` — List employees at a company

### Posts & Content
- `linkedin_search_posts` — Search posts by keywords, hashtags
- `linkedin_get_post` — Fetch post details and engagement
- `linkedin_get_feed` — View user's LinkedIn feed

### Jobs
- `linkedin_search_jobs` — Search job listings
- `linkedin_get_job` — Fetch job posting details

### Messaging
- `linkedin_get_messages` — Access inbox messages
- `linkedin_send_message` — Send direct messages

Tool names are namespaced as `linkedin_{tool_name}` by Spacebot's MCP adapter.

## Authentication Setup

LinkedIn MCP requires a one-time browser login to create a persistent session. The session is stored at `~/.linkedin-mcp/profile` and reused across all MCP server invocations.

### Initial Setup

1. **Install the LinkedIn MCP server** (via uv):
   ```bash
   uv tool install linkedin-scraper-mcp
   ```

2. **Login with visible browser** (one-time):
   ```bash
   linkedin-scraper-mcp --login
   ```
   
   This will:
   - Open a Chromium browser window
   - Navigate to LinkedIn login
   - Wait for you to complete authentication (email + password + 2FA if enabled)
   - Save the authenticated session to `~/.linkedin-mcp/profile`
   - Close the browser

3. **Verify authentication**:
   ```bash
   linkedin-scraper-mcp --status
   ```
   
   Expected output:
   ```
   ✅ Session is valid (profile: /Users/you/.linkedin-mcp/profile)
   ```

4. **Add to Spacebot config** (`~/.spacebot/config.toml`):
   ```toml
   [[defaults.mcp]]
   name = "linkedin"
   transport = "stdio"
   enabled = true
   command = "/Users/you/.local/bin/linkedin-scraper-mcp"
   args = ["--log-level", "WARNING"]

   [defaults.mcp.env]
   # LinkedIn MCP uses ~/.linkedin-mcp/profile for session storage
   ```

5. **Restart Spacebot**:
   ```bash
   ~/.local/bin/spacebot restart
   ```

### Session Maintenance

The LinkedIn session is stored in a persistent browser profile and survives:
- Spacebot restarts
- System reboots
- MCP server updates

**Session expiration:** LinkedIn sessions typically last 30-90 days. If the session expires:
1. You'll see authentication errors in logs
2. Run `linkedin-scraper-mcp --login` again
3. Restart Spacebot

**Check session status** at any time:
```bash
linkedin-scraper-mcp --status
```

**Logout and clear session**:
```bash
linkedin-scraper-mcp --logout
```

## Configuration

### Per-Agent Override

To disable LinkedIn MCP for specific agents:

```toml
[[agents]]
id = "slack-agent"

[[agents.mcp]]
name = "linkedin"
enabled = false  # Override: disable LinkedIn for this agent
command = "/bin/true"
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `LINKEDIN_MCP_USER_DATA_DIR` | Custom session storage path | `~/.linkedin-mcp/profile` |
| `CHROME_PATH` | Custom Chrome/Chromium binary path | Auto-detected |

### Advanced Options

For debugging or development, you can run the MCP server with visible browser:

```bash
linkedin-scraper-mcp --no-headless
```

Other useful flags:
- `--log-level DEBUG` — Verbose logging
- `--slow-mo 1000` — Slow down browser actions (milliseconds)
- `--timeout 60000` — Custom timeout for operations (milliseconds)

## Usage Patterns

### Research Use Cases

**Find people in a company:**
```
Ask Spacebot: "Find all ML engineers at Anthropic on LinkedIn"
→ linkedin_search_profiles(query="ML engineer", company="Anthropic")
```

**Company analysis:**
```
Ask Spacebot: "Get recent posts from OpenAI's LinkedIn page"
→ linkedin_get_company(name="OpenAI")
→ linkedin_search_posts(company="OpenAI", limit=10)
```

**Job market research:**
```
Ask Spacebot: "What Rust engineering jobs are available in Phoenix?"
→ linkedin_search_jobs(keywords="Rust engineer", location="Phoenix")
```

### Automated Workflows

**Newsletter monitoring** (via cron):
```toml
# Cron job on default-agent
prompt = """
Search LinkedIn for posts from AI safety newsletters in the last 24 hours.
Extract article titles, URLs, and summaries.
Send digest to Slack DM.
"""
interval = "0 9 * * *"  # Daily at 9am
```

**Competitive intelligence**:
```
Ask Spacebot: "Track new hires at competitor companies and notify me weekly"
→ Creates recurring task with linkedin_search_profiles + linkedin_get_company
```

## Rate Limits & Best Practices

LinkedIn has undocumented rate limits. The MCP server handles this by:
- Mimicking human browsing patterns (random delays, realistic scrolling)
- Using persistent sessions (avoids repeated logins)
- Patchright anti-detection (reduces bot detection triggers)

**Recommended practices:**
- Don't make hundreds of requests per minute
- Add delays between batch operations
- Use specific search queries (not broad scraping)
- Monitor for CAPTCHA or session expiration errors

## Troubleshooting

### Session Expired
```
Error: LinkedIn session expired or invalid
```

**Fix:**
```bash
linkedin-scraper-mcp --logout
linkedin-scraper-mcp --login
~/.local/bin/spacebot restart
```

### CAPTCHA Detected
```
Error: CAPTCHA required
```

**Fix:**
1. Run `linkedin-scraper-mcp --login --no-headless`
2. Complete CAPTCHA manually in visible browser
3. Session will be saved with CAPTCHA solved

### Browser Binary Not Found
```
Error: Chrome/Chromium not found
```

**Fix:**
```bash
# macOS
export CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

# Linux
export CHROME_PATH="/usr/bin/chromium-browser"
```

Add to `[defaults.mcp.env]` in config:
```toml
[defaults.mcp.env]
CHROME_PATH = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

### Check MCP Server Logs

Spacebot logs include MCP server stderr:
```bash
tail -f ~/.spacebot/logs/spacebot.log.$(date +%Y-%m-%d) | grep linkedin
```

## Security Considerations

### Session Storage
- LinkedIn session cookies are stored at `~/.linkedin-mcp/profile`
- Permissions: `700` (owner read/write/execute only)
- Contains: Chromium profile with LinkedIn authentication cookies

**Protect the session directory:**
```bash
chmod 700 ~/.linkedin-mcp/profile
```

### Credential Leaks
Spacebot's leak detection (`SpacebotHook`) scans tool arguments and results for:
- Email addresses
- Phone numbers
- Session tokens

LinkedIn MCP data passes through the same leak detection as native tools.

### Account Safety
- Use a professional/test LinkedIn account (not personal)
- Enable 2FA on the LinkedIn account
- Monitor login activity: https://www.linkedin.com/psettings/sessions
- LinkedIn MCP uses read-only operations by default

## Differences from Enterprise Slack MCP

| Feature | LinkedIn MCP | Enterprise Slack MCP |
|---------|--------------|----------------------|
| Auth method | Browser session (Patchright) | Browser tokens (`xoxc`/`xoxd`) |
| Session storage | `~/.linkedin-mcp/profile` | Tokens in config |
| Transport | stdio (subprocess) | stdio (subprocess) |
| Rate limiting | LinkedIn (undocumented) | Slack (well-documented) |
| Enabled on | All agents (by default) | `slack-agent` only |
| Primary use | Research, monitoring, outreach | Workspace search, history |

## Implementation Details

### Config Resolution
1. `defaults.mcp` defines LinkedIn MCP globally
2. Inherited by all agents unless overridden
3. Per-agent `[[agents.mcp]]` can disable or reconfigure

### Tool Registration Flow
```
Spacebot startup
  → Load config
  → McpManager::new(agent_config)
    → For each mcp server in config:
      → Spawn subprocess: linkedin-scraper-mcp
      → MCP initialize handshake
      → tools/list → discover LinkedIn tools
      → Create McpToolAdapter per tool
      → Register on worker ToolServer
```

### Execution Flow
```
Worker prompt with "find people on LinkedIn"
  → LLM decides to call linkedin_search_profiles
  → McpToolAdapter.call(args)
    → Send tools/call JSON-RPC to MCP server
    → MCP server: Patchright automation
    → Return results as JSON
  → SpacebotHook scans for leaks
  → LLM sees results, formats response
```

## References

- **MCP Server**: https://github.com/stickerdaniel/linkedin-scraper-mcp
- **Patchright**: https://github.com/Kaliiiiiiiiii-Vinyzu/patchright (anti-detection Playwright fork)
- **LinkedIn API Limits**: https://www.linkedin.com/help/linkedin/answer/52950 (public search)
- **MCP Protocol**: https://modelcontextprotocol.io/
- **Spacebot MCP Design**: `docs/design-docs/mcp.md`
