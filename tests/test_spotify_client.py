import pytest
from app.spotify_client import deduplicate_track_uris

class TestTrackDeduplication:
    def test_empty_list(self):
        assert deduplicate_track_uris([]) == []

    def test_removes_duplicates_and_preserves_order(self):
        uris = [
            "spotify:track:1",
            "spotify:track:2",
            "spotify:track:1",
            "spotify:track:3",
            "spotify:track:2",
        ]
        expected = [
            "spotify:track:1",
            "spotify:track:2",
            "spotify:track:3",
        ]
        assert deduplicate_track_uris(uris) == expected

    def test_filters_invalid_or_non_track_uris(self):
        uris = [
            "spotify:track:valid1",
            None,
            "",
            "spotify:album:invalid1",
            "spotify:track:valid2",
            123,  # non-string
        ]
        expected = [
            "spotify:track:valid1",
            "spotify:track:valid2",
        ]
        assert deduplicate_track_uris(uris) == expected
