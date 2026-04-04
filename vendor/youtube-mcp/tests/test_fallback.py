"""Tests for yt-dlp fallback module."""

import json
from unittest.mock import patch

from youtube_mcp.fallback import (
    _extract_video_id,
    _parse_json3,
    _parse_vtt,
    is_available,
)


class TestFallbackParsing:
    def test_parse_json3(self):
        json3_data = json.dumps(
            {
                "events": [
                    {
                        "tStartMs": 0,
                        "segs": [{"utf8": "Hello world"}],
                    },
                    {
                        "tStartMs": 5000,
                        "segs": [{"utf8": "This is a test"}],
                    },
                ]
            }
        )
        result = _parse_json3("test123", "en", json3_data)
        assert result["video_id"] == "test123"
        assert result["language"] == "en"
        assert result["source"] == "yt-dlp"
        assert "Hello world" in result["transcript"]
        assert "This is a test" in result["transcript"]
        assert result["segment_count"] == 2

    def test_parse_json3_invalid(self):
        result = _parse_json3("test123", "en", "not json")
        assert "error" in result

    def test_parse_vtt(self):
        vtt_content = """WEBVTT

00:00:00.000 --> 00:00:05.000
Hello world

00:00:05.000 --> 00:00:10.000
This is a test
"""
        result = _parse_vtt("test123", "en", vtt_content)
        assert result["video_id"] == "test123"
        assert "Hello world" in result["transcript"]
        assert "This is a test" in result["transcript"]

    def test_parse_vtt_strips_tags(self):
        vtt_content = """WEBVTT

00:00:00.000 --> 00:00:05.000
<c.colorE5E5E5>Hello</c> world
"""
        result = _parse_vtt("test123", "en", vtt_content)
        assert "<c" not in result["transcript"]
        assert "Hello" in result["transcript"]


class TestIsAvailable:
    @patch("shutil.which", return_value="/usr/local/bin/yt-dlp")
    def test_available(self, mock_which):
        assert is_available() is True

    @patch("shutil.which", return_value=None)
    def test_not_available(self, mock_which):
        assert is_available() is False


class TestExtractVideoId:
    def test_plain_id(self):
        assert _extract_video_id("dQw4w9WgXcQ") == "dQw4w9WgXcQ"

    def test_url(self):
        assert (
            _extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            == "dQw4w9WgXcQ"
        )
