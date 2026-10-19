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


class TestSpotifyClientRobustness:
    def test_deezer_fallback_null_genres(self, monkeypatch):
        from unittest.mock import MagicMock
        from app import spotify_client

        mock_search_res = MagicMock()
        mock_search_res.status_code = 200
        mock_search_res.json.return_value = {"data": [{"album": {"id": 999999}}]}

        mock_album_res = MagicMock()
        mock_album_res.status_code = 200
        mock_album_res.json.return_value = {"genres": None}

        def mock_get(url, *args, **kwargs):
            if "album/999999" in url:
                return mock_album_res
            return mock_search_res

        monkeypatch.setattr(spotify_client.HTTP_SESSION, "get", mock_get)
        monkeypatch.setattr(spotify_client, "save_to_disk_cache", lambda *args: None)
        genres = spotify_client.fetch_artist_genre_fallback("SpecialTestArtistNullGenres")
        assert genres == []

    def test_playlists_sort_with_none_name(self, monkeypatch):
        from unittest.mock import MagicMock
        from app import spotify_client

        mock_sp = MagicMock()
        mock_sp.current_user.return_value = {"id": "user1"}
        mock_sp.current_user_playlists.return_value = {
            "items": [
                {"id": "p1", "name": None, "owner": {"id": "user1"}},
                {"id": "p2", "name": "Rock Classics", "owner": {"id": "user1"}},
            ]
        }
        monkeypatch.setattr(spotify_client, "get_spotify_client", lambda: mock_sp)
        playlists = spotify_client.get_user_playlists()
        assert len(playlists) == 2
        names = [p["name"] for p in playlists]
        assert None in names
