import os
import pytest
from app import config

class TestConfig:
    def test_default_redirect_uri(self):
        assert config.DEFAULT_REDIRECT_URI == "http://127.0.0.1:8888/callback"

    def test_has_valid_credentials_false_when_empty(self, monkeypatch):
        monkeypatch.setenv("SPOTIPY_CLIENT_ID", "")
        monkeypatch.setenv("SPOTIPY_CLIENT_SECRET", "")
        assert not config.has_valid_credentials()

    def test_has_valid_credentials_false_when_too_short(self, monkeypatch):
        monkeypatch.setenv("SPOTIPY_CLIENT_ID", "12345")
        monkeypatch.setenv("SPOTIPY_CLIENT_SECRET", "12345")
        assert not config.has_valid_credentials()

    def test_has_valid_credentials_true_when_valid(self, monkeypatch):
        monkeypatch.setenv("SPOTIPY_CLIENT_ID", "a" * 32)
        monkeypatch.setenv("SPOTIPY_CLIENT_SECRET", "b" * 32)
        assert config.has_valid_credentials()
