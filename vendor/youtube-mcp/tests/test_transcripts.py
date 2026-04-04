"""Tests for YouTube transcript extraction."""

from youtube_mcp.transcripts import _extract_video_id, get_transcript, get_video_info


class TestExtractVideoId:
    def test_plain_id(self):
        assert _extract_video_id("dQw4w9WgXcQ") == "dQw4w9WgXcQ"

    def test_standard_url(self):
        assert (
            _extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            == "dQw4w9WgXcQ"
        )

    def test_short_url(self):
        assert (
            _extract_video_id("https://youtu.be/dQw4w9WgXcQ") == "dQw4w9WgXcQ"
        )

    def test_embed_url(self):
        assert (
            _extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ")
            == "dQw4w9WgXcQ"
        )

    def test_shorts_url(self):
        assert (
            _extract_video_id("https://www.youtube.com/shorts/dQw4w9WgXcQ")
            == "dQw4w9WgXcQ"
        )

    def test_url_with_params(self):
        assert (
            _extract_video_id(
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s&list=PLxyz"
            )
            == "dQw4w9WgXcQ"
        )


class TestGetTranscriptErrorHandling:
    def test_invalid_video_returns_error(self):
        result = get_transcript("XXXXXXXXXXX_invalid")
        assert "error" in result

    def test_video_info_invalid_returns_error(self):
        result = get_video_info("XXXXXXXXXXX_invalid")
        assert "error" in result or result.get("available_transcripts") == []
