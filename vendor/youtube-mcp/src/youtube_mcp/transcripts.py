"""YouTube transcript extraction using youtube-transcript-api."""

from __future__ import annotations

import re

from youtube_transcript_api import YouTubeTranscriptApi
from youtube_transcript_api._errors import (
    NoTranscriptFound,
    TranscriptsDisabled,
    VideoUnavailable,
)


def _extract_video_id(video_url_or_id: str) -> str:
    """Extract video ID from a URL or return as-is if already an ID."""
    if len(video_url_or_id) == 11 and re.match(r"^[a-zA-Z0-9_-]+$", video_url_or_id):
        return video_url_or_id

    patterns = [
        r"(?:v=|/v/)([a-zA-Z0-9_-]{11})",
        r"(?:youtu\.be/)([a-zA-Z0-9_-]{11})",
        r"(?:embed/)([a-zA-Z0-9_-]{11})",
        r"(?:shorts/)([a-zA-Z0-9_-]{11})",
    ]
    for pattern in patterns:
        match = re.search(pattern, video_url_or_id)
        if match:
            return match.group(1)

    return video_url_or_id


def get_transcript(
    video_url_or_id: str,
    language: str | None = None,
    with_timestamps: bool = False,
) -> dict:
    """Extract transcript text from a YouTube video.

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.
        language: Preferred language code (e.g., 'en'). Falls back to any available.
        with_timestamps: Include start time and duration per segment.

    Returns:
        Dict with video_id, language, and transcript text.
    """
    video_id = _extract_video_id(video_url_or_id)

    try:
        transcript_list = YouTubeTranscriptApi.list_transcripts(video_id)

        transcript = None
        if language:
            try:
                transcript = transcript_list.find_transcript([language])
            except NoTranscriptFound:
                try:
                    transcript = transcript_list.find_generated_transcript([language])
                except NoTranscriptFound:
                    pass

        if transcript is None:
            try:
                transcript = transcript_list.find_manually_created_transcript(
                    [t.language_code for t in transcript_list]
                )
            except NoTranscriptFound:
                transcript = transcript_list.find_generated_transcript(
                    [t.language_code for t in transcript_list]
                )

        entries = transcript.fetch()
        actual_language = transcript.language_code

        if with_timestamps:
            segments = []
            for entry in entries:
                segments.append(
                    {
                        "start": round(entry.start, 2),
                        "duration": round(entry.duration, 2),
                        "text": entry.text,
                    }
                )
            return {
                "video_id": video_id,
                "language": actual_language,
                "segments": segments,
            }
        else:
            text = " ".join(entry.text for entry in entries)
            return {
                "video_id": video_id,
                "language": actual_language,
                "transcript": text,
            }

    except TranscriptsDisabled:
        return {
            "video_id": video_id,
            "error": "Transcripts are disabled for this video.",
            "suggestion": "Try the youtube_get_subtitles_fallback tool which uses yt-dlp.",
        }
    except VideoUnavailable:
        return {
            "video_id": video_id,
            "error": f"Video unavailable: {video_id}",
        }
    except NoTranscriptFound:
        return {
            "video_id": video_id,
            "error": "No transcript found in any language.",
            "suggestion": "Try the youtube_get_subtitles_fallback tool which uses yt-dlp.",
        }
    except Exception as exc:
        return {
            "video_id": video_id,
            "error": f"Transcript extraction failed: {exc}",
            "suggestion": "Try the youtube_get_subtitles_fallback tool which uses yt-dlp.",
        }


def get_video_info(video_url_or_id: str) -> dict:
    """Get video metadata and available transcript languages.

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.

    Returns:
        Dict with video_id and available transcript languages.
    """
    video_id = _extract_video_id(video_url_or_id)

    try:
        transcript_list = YouTubeTranscriptApi.list_transcripts(video_id)
        languages = []
        for transcript in transcript_list:
            languages.append(
                {
                    "language": transcript.language,
                    "language_code": transcript.language_code,
                    "is_generated": transcript.is_generated,
                    "is_translatable": transcript.is_translatable,
                }
            )

        return {
            "video_id": video_id,
            "available_transcripts": languages,
        }

    except TranscriptsDisabled:
        return {
            "video_id": video_id,
            "available_transcripts": [],
            "note": "Transcripts are disabled for this video.",
        }
    except VideoUnavailable:
        return {
            "video_id": video_id,
            "error": f"Video unavailable: {video_id}",
        }
    except Exception as exc:
        return {
            "video_id": video_id,
            "error": f"Failed to get video info: {exc}",
        }
