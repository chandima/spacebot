# Google + YouTube Integration

## Overview

Spacebot integrates with Google Workspace and YouTube through a dedicated `google-agent` specialist in the multi-agent topology. The integration is MCP-first — all Google and YouTube capabilities are exposed through MCP servers that the agent's workers auto-discover and invoke.

## Architecture

```
                    ┌──────────────┐
                    │ default-agent│ ← Slack Bot adapter
                    │ (Orchestrator)│
                    └──────┬───────┘
                           │ send_agent_message
                           ▼
                    ┌──────────────┐
                    │ google-agent │
                    │ (Specialist) │
                    └──────┬───────┘
                           │ MCP tools
              ┌────────────┼────────────┐
              ▼            ▼            ▼
     google-workspace   youtube      searxng
         MCP             MCP          MCP
    (Workspace)     (Data API +    (web search
                    transcripts +   enrichment)
                    yt-dlp fallback)
```

### MCP Servers

| Server | Source | Transport | Purpose |
|--------|--------|-----------|---------|
| `google-workspace` | [taylorwilsdon/google_workspace_mcp](https://github.com/taylorwilsdon/google_workspace_mcp) | stdio | Google Drive, Docs, Slides, Sheets, Gmail, Calendar |
| `youtube` | Custom (`vendor/youtube-mcp/`) | stdio | YouTube Data API, transcript extraction, yt-dlp fallback |
| `searxng` | Inherited from defaults | stdio | Web search for enrichment |

### Tool Surface

#### Google Workspace (via `google-workspace` MCP)

Tools are auto-discovered from the MCP server. Key capabilities include:

- **Drive**: `google_workspace_drive_search`, `google_workspace_drive_get_file`, etc.
- **Docs**: `google_workspace_docs_create`, `google_workspace_docs_read`, `google_workspace_docs_update`
- **Slides**: `google_workspace_slides_create`, `google_workspace_slides_update`
- **Sheets**: `google_workspace_sheets_read`, `google_workspace_sheets_write`
- **Gmail**: `google_workspace_gmail_search`, `google_workspace_gmail_read`
- **Calendar**: `google_workspace_calendar_list`, `google_workspace_calendar_create`

Tool names are namespaced as `google_workspace_{tool_name}` by Spacebot's MCP adapter.

#### YouTube (via custom `youtube` MCP)

**Data API tools** (OAuth 2.0, read-only):
- `youtube_get_subscriptions` — List user's subscribed channels
- `youtube_search_videos` — Search videos by query, optionally within a channel
- `youtube_get_video_details` — Get video metadata, stats, tags
- `youtube_get_channel_details` — Get channel info and statistics
- `youtube_get_recent_from_subscriptions` — Recent uploads from subscribed channels

**Transcript tools** (no auth needed):
- `youtube_get_transcript` — Extract transcript text from a video (primary path)
- `youtube_get_video_info` — Check available transcript languages

**Fallback** (requires yt-dlp installed):
- `youtube_get_subtitles_fallback` — Extract subtitles via yt-dlp when transcripts are unavailable

### Transcript Extraction Flow

```
User request: "Get transcript for video X"
         │
         ▼
youtube_get_transcript (youtube-transcript-api)
         │
    ┌────┴────┐
    │ Success │──→ Return transcript text
    │         │
    │ Failure │──→ Error message includes suggestion
    └─────────┘    to use fallback
         │
         ▼
youtube_get_subtitles_fallback (yt-dlp)
         │
    ┌────┴────┐
    │ Success │──→ Return subtitle text (source: "yt-dlp")
    │         │
    │ Failure │──→ Report error (no subtitles available)
    └─────────┘
```

The agent decides when to use the fallback based on the error message from the primary tool. The fallback is NOT automatically chained — the agent uses judgment.

## Configuration

### config.toml (google-agent section)

```toml
[[agents]]
id = "google-agent"
display_name = "Google"
role = "Google Workspace and YouTube specialist"
max_concurrent_workers = 3
max_turns = 15

# Google Workspace MCP
[[agents.mcp]]
name = "google-workspace"
transport = "stdio"
enabled = true
command = "/Users/chandima/.local/bin/workspace-mcp"
args = ["--single-user", "--tools", "drive", "docs", "slides", "sheets", "gmail", "calendar"]

[agents.mcp.env]
WORKSPACE_MCP_OAUTH_PATH = "/Users/chandima/.spacebot/google"

# YouTube MCP
[[agents.mcp]]
name = "youtube"
transport = "stdio"
enabled = true
command = "/Users/chandima/.local/bin/youtube-mcp"
args = []
init_timeout_secs = 60

[agents.mcp.env]
YOUTUBE_CREDENTIALS_PATH = "/Users/chandima/.spacebot/google/credentials.json"
YOUTUBE_TOKEN_PATH = "/Users/chandima/.spacebot/google/youtube-token.json"

# Link to orchestrator
[[links]]
from = "default-agent"
to = "google-agent"
direction = "two_way"
kind = "hierarchical"
```

### Environment Variables

| Variable | Used By | Default | Description |
|----------|---------|---------|-------------|
| `WORKSPACE_MCP_OAUTH_PATH` | google-workspace | — | Directory containing Google OAuth credentials |
| `YOUTUBE_CREDENTIALS_PATH` | youtube | `~/.spacebot/google/credentials.json` | OAuth client credentials file |
| `YOUTUBE_TOKEN_PATH` | youtube | `~/.spacebot/google/youtube-token.json` | OAuth token storage path |

### OAuth Setup

Both MCP servers share the same Google Cloud project but use separate token files (different API scopes).

#### 1. Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select existing)
3. Enable these APIs:
   - Google Drive API
   - Google Docs API
   - Google Slides API
   - Google Sheets API
   - Gmail API
   - Google Calendar API
   - YouTube Data API v3

#### 2. Create OAuth Credentials

1. Go to **APIs & Services → Credentials**
2. Click **Create Credentials → OAuth 2.0 Client ID**
3. Application type: **Desktop application**
4. Download the credentials JSON file
5. Save to `~/.spacebot/google/credentials.json`

#### 3. Initial Authorization

On first run of each MCP server, a browser window will open for OAuth consent:

- **Google Workspace**: Consent covers Drive, Docs, Slides, Sheets, Gmail, Calendar scopes
- **YouTube**: Consent covers `youtube.readonly` scope only

Tokens are saved automatically and refreshed on expiry.

### Prerequisites

| Dependency | Install | Required For |
|-----------|---------|-------------|
| `workspace-mcp` | `uv tool install workspace-mcp` | Google Workspace MCP |
| `youtube-mcp` | `uv tool install ./vendor/youtube-mcp` | YouTube MCP |
| `yt-dlp` | `uv tool install yt-dlp` | Subtitle fallback (optional) |

## Per-Agent Scoping

Google and YouTube MCP servers are **only enabled on `google-agent`**. Other agents do not have access — all Google/YouTube requests must go through `google-agent` via the `send_agent_message` delegation from `default-agent`.

This matches the existing pattern:
- `enterprise-slack` → only on `slack-agent`
- `notebooklm` → only on `notebooklm-agent`
- `google-workspace` + `youtube` → only on `google-agent`

## Example Workflows

### Search Drive and summarize a document

1. User (Slack): "Find the Q4 budget spreadsheet and summarize the highlights"
2. `default-agent` → delegates to `google-agent`
3. `google-agent` worker → `google_workspace_drive_search(query="Q4 budget")`
4. Worker → `google_workspace_sheets_read(spreadsheet_id=..., range="Sheet1")`
5. Worker summarizes and returns result
6. `default-agent` → replies to user in Slack

### Get YouTube transcript

1. User: "Get the transcript for https://youtu.be/dQw4w9WgXcQ"
2. `default-agent` → delegates to `google-agent`
3. `google-agent` worker → `youtube_get_transcript(video_url_or_id="dQw4w9WgXcQ")`
4. If failed → `youtube_get_subtitles_fallback(video_url_or_id="dQw4w9WgXcQ")`
5. Returns transcript text

### Daily subscription digest (cron)

A cron job on `google-agent` fires daily:
1. `youtube_get_recent_from_subscriptions(max_channels=15, videos_per_channel=2)`
2. Summarize new videos by topic
3. Deliver digest to Slack DM via delegation to `default-agent`

## Related

- **[Adversarial Research Pipeline](adversarial-research-pipeline.md)** — uses Google Workspace MCP (for output to Docs/Slides/Sheets), YouTube MCP (for video research), and additional research MCP servers (arxiv, paper-search, fetcher, pdf-reader) in a structured multi-agent research workflow.
