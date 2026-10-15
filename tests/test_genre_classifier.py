import pytest
from app.genre_classifier import (
    map_raw_genres_to_macro,
    infer_genres_from_track_metadata,
    analyze_playlist_tracks,
    MACRO_RULES,
)

class TestMacroGenreMapping:
    def test_empty_raw_genres(self):
        assert map_raw_genres_to_macro([]) == []

    def test_specific_turkish_pop_discards_generic_turkish(self):
        # When a specific Turkish category is matched, the generic category should be discarded
        raw = ["turkish pop", "turkish"]
        result = map_raw_genres_to_macro(raw)
        assert "Türkçe Pop" in result
        assert "Türkçe Halk & Sanat & Diğer" not in result

    def test_specific_turkish_rap_discards_generic_turkish(self):
        raw = ["turkish hip hop", "turkish rap", "turkish"]
        result = map_raw_genres_to_macro(raw)
        assert "Türkçe Rap / Hip-Hop" in result
        assert "Türkçe Halk & Sanat & Diğer" not in result

    def test_generic_turkish_kept_when_no_specific_match(self):
        raw = ["halk müziği", "anadolu"]
        result = map_raw_genres_to_macro(raw)
        assert result == ["Türkçe Halk & Sanat & Diğer"]

    def test_global_metal_and_rock(self):
        raw = ["death metal", "hard rock"]
        result = map_raw_genres_to_macro(raw)
        assert "Metal" in result
        assert "Rock" in result

    def test_phonk_and_electronic(self):
        raw = ["drift phonk", "synthwave"]
        result = map_raw_genres_to_macro(raw)
        assert "Phonk / Drift & Bass" in result
        assert "Elektronik / Dance" in result


class TestInferencePatterns:
    def test_international_rap_not_classified_as_turkish(self):
        # International rap cues must map to "Hip-Hop & Rap", NOT "Türkçe Rap / Hip-Hop"
        result1 = infer_genres_from_track_metadata("SICKO MODE (Freestyle)", ["Travis Scott"])
        assert result1 == ["Hip-Hop & Rap"]

        result2 = infer_genres_from_track_metadata("Drill Beat", ["Central Cee"])
        assert result2 == ["Hip-Hop & Rap"]

        result3 = infer_genres_from_track_metadata("Trap God", ["Gucci Mane"])
        assert result3 == ["Hip-Hop & Rap"]

        result4 = infer_genres_from_track_metadata("Rap God", ["Eminem"])
        assert result4 == ["Hip-Hop & Rap"]

    def test_explicit_turkish_rap_cues(self):
        result1 = infer_genres_from_track_metadata("Türkçe Rap Cypher", ["Çeşitli Sanatçılar"])
        assert result1 == ["Türkçe Rap / Hip-Hop"]

        result2 = infer_genres_from_track_metadata("Turkce Trap Beat", ["Sanatçı"])
        assert result2 == ["Türkçe Rap / Hip-Hop"]

    def test_phonk_inference(self):
        result = infer_genres_from_track_metadata("Automotivo Montagem Slowed", ["DJ Phonk"])
        assert result == ["Phonk / Drift & Bass"]

    def test_electronic_inference(self):
        result = infer_genres_from_track_metadata("Midnight City (Club Mix)", ["M83"])
        assert result == ["Elektronik / Dance"]

    def test_classical_soundtrack_inference(self):
        result = infer_genres_from_track_metadata("Main Theme OST Piano Instrumental", ["Composer"])
        assert result == ["Klasik & Soundtrack"]

    def test_unmatched_track(self):
        result = infer_genres_from_track_metadata("Random Song Title", ["Some Band"])
        assert result == []


class TestAnalyzePlaylistTracks:
    def test_empty_playlist(self):
        analysis = analyze_playlist_tracks([], {})
        assert analysis["total_tracks"] == 0
        assert analysis["unique_artists"] == 0
        assert analysis["macro_genres"] == []
        assert analysis["tracks"] == []

    def test_curated_artist_enrichment(self):
        raw_tracks = [
            {
                "track": {
                    "id": "trk1",
                    "name": "Geceler",
                    "uri": "spotify:track:trk1",
                    "duration_ms": 180000,
                    "preview_url": None,
                    "popularity": 75,
                    "artists": [{"id": "ezhel_id", "name": "Ezhel"}],
                    "album": {"name": "Müptezhel", "images": []},
                    "external_urls": {"spotify": "https://open.spotify.com/track/trk1"}
                }
            }
        ]
        # Ezhel is in CURATED_ARTISTS
        analysis = analyze_playlist_tracks(raw_tracks, {})
        assert analysis["total_tracks"] == 1
        assert analysis["unique_artists"] == 1
        assert analysis["categorized_count"] == 1
        assert analysis["uncategorized_count"] == 0

        track = analysis["tracks"][0]
        assert "Türkçe Rap / Hip-Hop" in track["macro_genres"]
        # Generic category should have been discarded
        assert "Türkçe Halk & Sanat & Diğer" not in track["macro_genres"]

    def test_untagged_track_falls_back_to_metadata_inference(self):
        raw_tracks = [
            {
                "track": {
                    "id": "trk2",
                    "name": "Tokyo Drift Phonk (Slowed)",
                    "uri": "spotify:track:trk2",
                    "duration_ms": 120000,
                    "artists": [{"id": "unknown_id", "name": "Unknown Underground Artist"}],
                    "album": {"name": "Phonk Single", "images": []}
                }
            }
        ]
        analysis = analyze_playlist_tracks(raw_tracks, {})
        assert analysis["total_tracks"] == 1
        assert analysis["categorized_count"] == 1
        track = analysis["tracks"][0]
        assert "Phonk / Drift & Bass" in track["macro_genres"]
