"""YouTube MCP server — Data API, transcripts, and yt-dlp fallback."""

from __future__ import annotations

import json
import os
import sys

from fastmcp import FastMCP

from youtube_mcp import data_api, fallback, transcripts

mcp = FastMCP(
    "YouTube",
    description="YouTube Data API, transcript extraction, and subtitle fallback for Spacebot",
)

CREDENTIALS_PATH = os.environ.get(
    "YOUTUBE_CREDENTIALS_PATH", os.path.expanduser("~/.spacebot/google/credentials.json")
)
TOKEN_PATH = os.environ.get(
    "YOUTUBE_TOKEN_PATH", os.path.expanduser("~/.spacebot/google/youtube-token.json")
)


# --- YouTube Data API tools ---


@mcp.tool()
def youtube_get_subscriptions(max_results: int = 25) -> str:
    """List the authenticated user's YouTube subscriptions.

    Args:
        max_results: Maximum number of subscriptions to return (default: 25, max: 50).

    Returns:
        JSON list of subscribed channels with title, channel_id, and description.
    """
    result = data_api.get_subscriptions(CREDENTIALS_PATH, TOKEN_PATH, max_results)
    return json.dumps(result, indent=2)


@mcp.tool()
def youtube_search_videos(
    query: str,
    channel_id: str | None = None,
    max_results: int = 10,
    order: str = "relevance",
) -> str:
    """Search YouTube videos by query.

    Args:
        query: Search query string.
        channel_id: Optional channel ID to search within a specific channel.
        max_results: Maximum number of results (default: 10, max: 50).
        order: Sort order — 'relevance', 'date', 'rating', 'viewCount', or 'title'.

    Returns:
        JSON list of matching videos with title, video_id, channel, and publish date.
    """
    result = data_api.search_videos(
        CREDENTIALS_PATH, TOKEN_PATH, query, channel_id, max_results, order
    )
    return json.dumps(result, indent=2)


@mcp.tool()
def youtube_get_video_details(video_id: str) -> str:
    """Get detailed information about a specific YouTube video.

    Args:
        video_id: The 11-character YouTube video ID.

    Returns:
        JSON with title, description, stats (views, likes, comments), duration, tags, and more.
    """
    result = data_api.get_video_details(CREDENTIALS_PATH, TOKEN_PATH, video_id)
    return json.dumps(result, indent=2)


@mcp.tool()
def youtube_get_channel_details(channel_id: str) -> str:
    """Get information about a YouTube channel.

    Args:
        channel_id: The YouTube channel ID.

    Returns:
        JSON with channel title, description, subscriber count, video count, and upload playlist ID.
    """
    result = data_api.get_channel_details(CREDENTIALS_PATH, TOKEN_PATH, channel_id)
    return json.dumps(result, indent=2)


@mcp.tool()
def youtube_get_recent_from_subscriptions(
    max_channels: int = 10,
    videos_per_channel: int = 3,
) -> str:
    """Get recent video uploads from the user's subscribed channels.

    Args:
        max_channels: Maximum number of subscription channels to check (default: 10).
        videos_per_channel: Maximum recent videos per channel (default: 3).

    Returns:
        JSON list of recent videos sorted by publish date, with channel, title, and description.
    """
    result = data_api.get_recent_from_subscriptions(
        CREDENTIALS_PATH, TOKEN_PATH, max_channels, videos_per_channel
    )
    return json.dumps(result, indent=2)


# --- Transcript tools ---


@mcp.tool()
def youtube_get_transcript(
    video_url_or_id: str,
    language: str | None = None,
    with_timestamps: bool = False,
) -> str:
    """Extract transcript text from a YouTube video.

    This is the primary transcript extraction tool. It uses YouTube's native transcript
    data (manual captions and auto-generated captions).

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.
        language: Preferred language code (e.g., 'en'). Falls back to any available language.
        with_timestamps: If true, include start time and duration per segment.

    Returns:
        JSON with video_id, language, and transcript text (or segments with timestamps).
        If transcripts are unavailable, suggests using the fallback tool.
    """
    result = transcripts.get_transcript(video_url_or_id, language, with_timestamps)
    return json.dumps(result, indent=2)


@mcp.tool()
def youtube_get_video_info(video_url_or_id: str) -> str:
    """Get video metadata and available transcript languages.

    Use this to check which transcript languages are available before extracting.

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.

    Returns:
        JSON with video_id and list of available transcript languages (manual and auto-generated).
    """
    result = transcripts.get_video_info(video_url_or_id)
    return json.dumps(result, indent=2)


# --- yt-dlp fallback tools ---


@mcp.tool()
def youtube_get_subtitles_fallback(
    video_url_or_id: str,
    language: str = "en",
    format: str = "json3",
) -> str:
    """Extract subtitles using yt-dlp as a fallback when primary transcript extraction fails.

    Only use this tool when youtube_get_transcript reports that transcripts are unavailable
    or disabled. This tool uses yt-dlp which can access auto-generated subtitles that may
    not be available through the standard transcript API.

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.
        language: Subtitle language code (default: 'en').
        format: Subtitle format — 'json3' or 'vtt' (default: 'json3').

    Returns:
        JSON with video_id, language, source ('yt-dlp'), and extracted transcript text.
    """
    result = fallback.get_subtitles_fallback(video_url_or_id, language, format)
    return json.dumps(result, indent=2)


def main():
    """Entry point for the YouTube MCP server."""
    if not fallback.is_available():
        print(
            "Warning: yt-dlp is not installed. Fallback subtitle extraction will be unavailable.",
            file=sys.stderr,
        )
    mcp.run(transport="stdio")


if __name__ == "__main__":
    main()
