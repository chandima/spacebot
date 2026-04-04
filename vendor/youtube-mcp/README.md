# YouTube MCP Server

Custom YouTube MCP server for Spacebot — YouTube Data API, transcript extraction, and yt-dlp fallback.

## Tools

### YouTube Data API (OAuth 2.0)
- `youtube_get_subscriptions` — List authenticated user's subscriptions
- `youtube_search_videos` — Search YouTube videos
- `youtube_get_video_details` — Get video metadata and stats
- `youtube_get_channel_details` — Get channel info
- `youtube_get_recent_from_subscriptions` — Recent uploads from subscribed channels

### Transcript Extraction
- `youtube_get_transcript` — Extract transcript text (primary, via youtube-transcript-api)
- `youtube_get_video_info` — Get available transcript languages

### Fallback
- `youtube_get_subtitles_fallback` — Extract subtitles via yt-dlp (fallback only)

## Setup

### Prerequisites
- Python 3.11+
- Google Cloud project with YouTube Data API v3 enabled
- OAuth 2.0 client credentials (`credentials.json`)
- `yt-dlp` installed for fallback (`brew install yt-dlp`)

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `YOUTUBE_CREDENTIALS_PATH` | `~/.spacebot/google/credentials.json` | OAuth client credentials file |
| `YOUTUBE_TOKEN_PATH` | `~/.spacebot/google/youtube-token.json` | OAuth token storage |

### OAuth Setup
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create or select a project
3. Enable **YouTube Data API v3**
4. Create **OAuth 2.0 Client ID** (Desktop application)
5. Download `credentials.json` to the path above
6. On first run, a browser window opens for OAuth consent
7. Token is saved automatically for future use

### Installation
```bash
uv tool install /path/to/vendor/youtube-mcp
```

### Running
```bash
youtube-mcp  # Starts stdio MCP server
```
