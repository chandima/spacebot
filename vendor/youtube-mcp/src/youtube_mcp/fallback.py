"""yt-dlp fallback for subtitle extraction when youtube-transcript-api fails."""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


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


def is_available() -> bool:
    """Check if yt-dlp is installed and accessible."""
    return shutil.which("yt-dlp") is not None


def get_subtitles_fallback(
    video_url_or_id: str,
    language: str = "en",
    format: str = "json3",
) -> dict:
    """Extract subtitles using yt-dlp as a fallback.

    Args:
        video_url_or_id: YouTube video URL or 11-character video ID.
        language: Subtitle language code (default: 'en').
        format: Subtitle format — 'json3', 'vtt', or 'srt' (default: 'json3').

    Returns:
        Dict with video_id and extracted subtitle text.
    """
    if not is_available():
        return {
            "error": "yt-dlp is not installed. Install via: brew install yt-dlp",
        }

    video_id = _extract_video_id(video_url_or_id)
    url = f"https://www.youtube.com/watch?v={video_id}"

    with tempfile.TemporaryDirectory() as tmpdir:
        output_template = str(Path(tmpdir) / "%(id)s")

        cmd = [
            "yt-dlp",
            "--write-subs",
            "--write-auto-subs",
            f"--sub-lang={language}",
            f"--sub-format={format}",
            "--skip-download",
            "--no-warnings",
            "--quiet",
            "-o",
            output_template,
            url,
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30,
            )

            if result.returncode != 0:
                return {
                    "video_id": video_id,
                    "error": f"yt-dlp failed: {result.stderr.strip() or 'unknown error'}",
                }

            sub_files = list(Path(tmpdir).glob(f"{video_id}*"))
            if not sub_files:
                return {
                    "video_id": video_id,
                    "error": f"No subtitles found for language '{language}'.",
                }

            sub_file = sub_files[0]
            content = sub_file.read_text(encoding="utf-8")

            if format == "json3":
                return _parse_json3(video_id, language, content)
            elif format == "vtt":
                return _parse_vtt(video_id, language, content)
            else:
                return {
                    "video_id": video_id,
                    "language": language,
                    "format": format,
                    "raw_content": content[:50000],
                }

        except subprocess.TimeoutExpired:
            return {
                "video_id": video_id,
                "error": "yt-dlp timed out after 30 seconds.",
            }
        except Exception as exc:
            return {
                "video_id": video_id,
                "error": f"yt-dlp fallback failed: {exc}",
            }


def _parse_json3(video_id: str, language: str, content: str) -> dict:
    """Parse json3 subtitle format into structured text."""
    try:
        data = json.loads(content)
        segments = []
        for event in data.get("events", []):
            start_ms = event.get("tStartMs", 0)
            text_parts = []
            for seg in event.get("segs", []):
                text = seg.get("utf8", "").strip()
                if text and text != "\n":
                    text_parts.append(text)
            if text_parts:
                segments.append(
                    {
                        "start": round(start_ms / 1000, 2),
                        "text": " ".join(text_parts),
                    }
                )

        full_text = " ".join(seg["text"] for seg in segments)
        return {
            "video_id": video_id,
            "language": language,
            "source": "yt-dlp",
            "transcript": full_text,
            "segment_count": len(segments),
        }
    except json.JSONDecodeError:
        return {
            "video_id": video_id,
            "error": "Failed to parse json3 subtitle data.",
        }


def _parse_vtt(video_id: str, language: str, content: str) -> dict:
    """Parse VTT subtitle format into plain text."""
    lines = content.split("\n")
    text_lines = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith("WEBVTT") or "-->" in line or line.isdigit():
            continue
        clean = re.sub(r"<[^>]+>", "", line)
        if clean:
            text_lines.append(clean)

    text_lines = list(dict.fromkeys(text_lines))

    return {
        "video_id": video_id,
        "language": language,
        "source": "yt-dlp",
        "transcript": " ".join(text_lines),
    }
