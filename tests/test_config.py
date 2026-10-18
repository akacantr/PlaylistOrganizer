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

    def test_get_base_dir_development(self, monkeypatch):
        monkeypatch.delattr(config.sys, "frozen", raising=False)
        base = config.get_base_dir()
        assert base.exists()
        assert (base / "app").exists()

    def test_get_base_dir_frozen_standard(self, monkeypatch, tmp_path):
        monkeypatch.setattr(config.sys, "frozen", True, raising=False)
        monkeypatch.delenv("APPDIR", raising=False)
        monkeypatch.delenv("APPIMAGE", raising=False)
        dummy_exe = tmp_path / "bin" / "playlist-organizer"
        dummy_exe.parent.mkdir(parents=True, exist_ok=True)
        dummy_exe.touch()
        monkeypatch.setattr(config.sys, "executable", str(dummy_exe))
        assert config.get_base_dir() == dummy_exe.parent

    def test_get_base_dir_frozen_appimage(self, monkeypatch, tmp_path):
        monkeypatch.setattr(config.sys, "frozen", True, raising=False)
        monkeypatch.setenv("APPDIR", str(tmp_path / "mount"))
        monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / ".config"))
        base = config.get_base_dir()
        assert base == tmp_path / ".config" / "playlist-organizer"
        assert base.exists()

    def test_get_base_dir_frozen_appimage_portable(self, monkeypatch, tmp_path):
        monkeypatch.setattr(config.sys, "frozen", True, raising=False)
        appimage_file = tmp_path / "PlaylistOrganizer.AppImage"
        appimage_file.touch()
        (tmp_path / ".env").touch()
        monkeypatch.setenv("APPDIR", str(tmp_path / "mount"))
        monkeypatch.setenv("APPIMAGE", str(appimage_file))
        assert config.get_base_dir() == tmp_path
