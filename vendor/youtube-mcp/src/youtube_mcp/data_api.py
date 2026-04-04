"""YouTube Data API v3 client with OAuth 2.0 authentication."""

from __future__ import annotations

import json
import os
from pathlib import Path

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/youtube.readonly"]


def _get_credentials(credentials_path: str, token_path: str) -> Credentials:
    """Load or refresh OAuth 2.0 credentials."""
    creds = None
    token_file = Path(token_path)

    if token_file.exists():
        creds = Credentials.from_authorized_user_file(str(token_file), SCOPES)

    if creds and creds.expired and creds.refresh_token:
        creds.refresh(Request())
        token_file.write_text(creds.to_json())
    elif not creds or not creds.valid:
        if not Path(credentials_path).exists():
            raise FileNotFoundError(
                f"OAuth credentials file not found at {credentials_path}. "
                "Download it from Google Cloud Console → APIs & Services → Credentials."
            )
        flow = InstalledAppFlow.from_client_secrets_file(credentials_path, SCOPES)
        creds = flow.run_local_server(port=0)
        token_file.parent.mkdir(parents=True, exist_ok=True)
        token_file.write_text(creds.to_json())

    return creds


def _build_service(credentials_path: str, token_path: str):
    """Build an authenticated YouTube Data API v3 service."""
    creds = _get_credentials(credentials_path, token_path)
    return build("youtube", "v3", credentials=creds)


def get_subscriptions(
    credentials_path: str,
    token_path: str,
    max_results: int = 25,
) -> list[dict]:
    """List the authenticated user's YouTube subscriptions."""
    service = _build_service(credentials_path, token_path)
    results = []
    request = service.subscriptions().list(
        part="snippet,contentDetails",
        mine=True,
        maxResults=min(max_results, 50),
        order="relevance",
    )

    while request and len(results) < max_results:
        response = request.execute()
        for item in response.get("items", []):
            snippet = item["snippet"]
            results.append(
                {
                    "channel_id": snippet["resourceId"]["channelId"],
                    "title": snippet["title"],
                    "description": snippet.get("description", "")[:200],
                    "thumbnail": snippet.get("thumbnails", {})
                    .get("default", {})
                    .get("url"),
                    "total_videos": item.get("contentDetails", {}).get(
                        "totalItemCount"
                    ),
                }
            )
        request = service.subscriptions().list_next(request, response)

    return results[:max_results]


def search_videos(
    credentials_path: str,
    token_path: str,
    query: str,
    channel_id: str | None = None,
    max_results: int = 10,
    order: str = "relevance",
) -> list[dict]:
    """Search YouTube videos."""
    service = _build_service(credentials_path, token_path)
    params = {
        "part": "snippet",
        "q": query,
        "type": "video",
        "maxResults": min(max_results, 50),
        "order": order,
    }
    if channel_id:
        params["channelId"] = channel_id

    response = service.search().list(**params).execute()
    results = []
    for item in response.get("items", []):
        snippet = item["snippet"]
        results.append(
            {
                "video_id": item["id"]["videoId"],
                "title": snippet["title"],
                "description": snippet.get("description", "")[:200],
                "channel_title": snippet.get("channelTitle"),
                "published_at": snippet.get("publishedAt"),
                "thumbnail": snippet.get("thumbnails", {})
                .get("medium", {})
                .get("url"),
            }
        )
    return results


def get_video_details(
    credentials_path: str,
    token_path: str,
    video_id: str,
) -> dict:
    """Get detailed info for a single video."""
    service = _build_service(credentials_path, token_path)
    response = (
        service.videos()
        .list(part="snippet,contentDetails,statistics", id=video_id)
        .execute()
    )

    items = response.get("items", [])
    if not items:
        return {"error": f"Video not found: {video_id}"}

    item = items[0]
    snippet = item["snippet"]
    stats = item.get("statistics", {})
    content = item.get("contentDetails", {})

    return {
        "video_id": video_id,
        "title": snippet["title"],
        "description": snippet.get("description", ""),
        "channel_title": snippet.get("channelTitle"),
        "channel_id": snippet.get("channelId"),
        "published_at": snippet.get("publishedAt"),
        "duration": content.get("duration"),
        "view_count": stats.get("viewCount"),
        "like_count": stats.get("likeCount"),
        "comment_count": stats.get("commentCount"),
        "tags": snippet.get("tags", []),
        "category_id": snippet.get("categoryId"),
    }


def get_channel_details(
    credentials_path: str,
    token_path: str,
    channel_id: str,
) -> dict:
    """Get channel info and statistics."""
    service = _build_service(credentials_path, token_path)
    response = (
        service.channels()
        .list(part="snippet,statistics,contentDetails", id=channel_id)
        .execute()
    )

    items = response.get("items", [])
    if not items:
        return {"error": f"Channel not found: {channel_id}"}

    item = items[0]
    snippet = item["snippet"]
    stats = item.get("statistics", {})

    return {
        "channel_id": channel_id,
        "title": snippet["title"],
        "description": snippet.get("description", "")[:500],
        "custom_url": snippet.get("customUrl"),
        "published_at": snippet.get("publishedAt"),
        "subscriber_count": stats.get("subscriberCount"),
        "video_count": stats.get("videoCount"),
        "view_count": stats.get("viewCount"),
        "uploads_playlist": item.get("contentDetails", {})
        .get("relatedPlaylists", {})
        .get("uploads"),
    }


def get_recent_from_subscriptions(
    credentials_path: str,
    token_path: str,
    max_channels: int = 10,
    videos_per_channel: int = 3,
) -> list[dict]:
    """Get recent uploads from subscribed channels."""
    service = _build_service(credentials_path, token_path)

    subs_response = (
        service.subscriptions()
        .list(
            part="snippet,contentDetails",
            mine=True,
            maxResults=min(max_channels, 50),
            order="relevance",
        )
        .execute()
    )

    results = []
    for sub in subs_response.get("items", []):
        channel_id = sub["snippet"]["resourceId"]["channelId"]
        channel_title = sub["snippet"]["title"]

        channel_response = (
            service.channels()
            .list(part="contentDetails", id=channel_id)
            .execute()
        )
        channel_items = channel_response.get("items", [])
        if not channel_items:
            continue

        uploads_id = (
            channel_items[0]
            .get("contentDetails", {})
            .get("relatedPlaylists", {})
            .get("uploads")
        )
        if not uploads_id:
            continue

        playlist_response = (
            service.playlistItems()
            .list(
                part="snippet",
                playlistId=uploads_id,
                maxResults=videos_per_channel,
            )
            .execute()
        )

        for video in playlist_response.get("items", []):
            snippet = video["snippet"]
            results.append(
                {
                    "channel_title": channel_title,
                    "channel_id": channel_id,
                    "video_id": snippet.get("resourceId", {}).get("videoId"),
                    "title": snippet["title"],
                    "description": snippet.get("description", "")[:200],
                    "published_at": snippet.get("publishedAt"),
                    "thumbnail": snippet.get("thumbnails", {})
                    .get("medium", {})
                    .get("url"),
                }
            )

    results.sort(key=lambda x: x.get("published_at", ""), reverse=True)
    return results
