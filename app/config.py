import os
import sys
from pathlib import Path
from dotenv import load_dotenv, set_key

def get_base_dir() -> Path:
    if getattr(sys, "frozen", False):
        # When running inside an AppImage, the bundle mount is read-only.
        # Store persistent configuration (.env, data cache) in user config directory,
        # or alongside the AppImage if a portable .env already exists there.
        if os.environ.get("APPDIR") or os.environ.get("APPIMAGE"):
            appimage_path = os.environ.get("APPIMAGE")
            if appimage_path:
                portable_dir = Path(appimage_path).resolve().parent
                if (portable_dir / ".env").exists():
                    return portable_dir
            xdg_config = Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config"))
            config_dir = xdg_config / "playlist-organizer"
            config_dir.mkdir(parents=True, exist_ok=True)
            return config_dir
        return Path(sys.executable).resolve().parent
    return Path(__file__).resolve().parent.parent

BASE_DIR = get_base_dir()
ENV_PATH = BASE_DIR / ".env"

if ENV_PATH.exists():
    load_dotenv(dotenv_path=ENV_PATH, override=True)
else:
    load_dotenv(override=True)

DEFAULT_REDIRECT_URI = "http://127.0.0.1:8888/callback"
SPOTIFY_SCOPES = [
    "user-read-private",
    "user-read-email",
    "user-library-read",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-public",
    "playlist-modify-private"
]

def get_client_id() -> str:
    return os.getenv("SPOTIPY_CLIENT_ID", "").strip()

def get_client_secret() -> str:
    return os.getenv("SPOTIPY_CLIENT_SECRET", "").strip()

def get_redirect_uri() -> str:
    return os.getenv("SPOTIPY_REDIRECT_URI", DEFAULT_REDIRECT_URI).strip() or DEFAULT_REDIRECT_URI

def has_valid_credentials() -> bool:
    cid = get_client_id()
    csec = get_client_secret()
    return bool(cid and csec and len(cid) > 10 and len(csec) > 10)

def save_credentials(client_id: str, client_secret: str, redirect_uri: str = DEFAULT_REDIRECT_URI):
    client_id = client_id.strip()
    client_secret = client_secret.strip()
    redirect_uri = redirect_uri.strip() or DEFAULT_REDIRECT_URI

    if not ENV_PATH.exists():
        ENV_PATH.touch(mode=0o600)

    set_key(str(ENV_PATH), "SPOTIPY_CLIENT_ID", client_id)
    set_key(str(ENV_PATH), "SPOTIPY_CLIENT_SECRET", client_secret)
    set_key(str(ENV_PATH), "SPOTIPY_REDIRECT_URI", redirect_uri)

    try:
        ENV_PATH.chmod(0o600)
    except OSError:
        pass

    os.environ["SPOTIPY_CLIENT_ID"] = client_id
    os.environ["SPOTIPY_CLIENT_SECRET"] = client_secret
    os.environ["SPOTIPY_REDIRECT_URI"] = redirect_uri
