import sys
from pathlib import Path
from typing import Optional, List, Dict, Any
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import HTMLResponse, RedirectResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from . import config
from . import spotify_client
from . import genre_classifier

def get_bundle_dir() -> Path:
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        return Path(sys._MEIPASS)
    return Path(__file__).resolve().parent.parent

BASE_DIR = get_bundle_dir()
STATIC_DIR = BASE_DIR / "app" / "static"
if not STATIC_DIR.exists():
    STATIC_DIR = BASE_DIR / "static"

TEMPLATES_DIR = BASE_DIR / "app" / "templates"
if not TEMPLATES_DIR.exists():
    TEMPLATES_DIR = BASE_DIR / "templates"

app = FastAPI(
    title="Spotify Playlist Genre Organizer",
    description="Spotify çalma listelerini müzik türlerine göre ayıran web arayüzü",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://127.0.0.1:8888",
        "http://localhost:8888",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.mount("/static", StaticFiles(directory=str(STATIC_DIR)), name="static")

class ConfigRequest(BaseModel):
    client_id: str
    client_secret: str
    redirect_uri: Optional[str] = None

class CreatePlaylistRequest(BaseModel):
    name: str
    description: Optional[str] = ""
    track_uris: List[str]
    is_public: Optional[bool] = False

class SplitAllRequest(BaseModel):
    playlist_name: str
    genre_mode: str = "macro"
    min_tracks: int = 1
    is_public: bool = False
    genre_data: Dict[str, List[str]]

@app.get("/", response_class=FileResponse)
async def serve_index():
    index_file = TEMPLATES_DIR / "index.html"
    if not index_file.exists():
        raise HTTPException(status_code=404, detail="Index template not found")
    return FileResponse(str(index_file))

@app.get("/api/status")
def get_status():
    has_creds = config.has_valid_credentials()
    client_id = config.get_client_id()
    preview = f"{client_id[:4]}...{client_id[-4:]}" if len(client_id) > 8 else ""

    is_auth = False
    user = None

    if has_creds:
        is_auth = spotify_client.is_authenticated()
        if is_auth:
            user = spotify_client.get_current_user_profile()
            if not user:
                is_auth = False

    return {
        "has_credentials": has_creds,
        "client_id_preview": preview,
        "redirect_uri": config.get_redirect_uri(),
        "is_authenticated": is_auth,
        "user": user
    }

@app.post("/api/config")
def save_config(req: ConfigRequest):
    if not req.client_id or not req.client_secret:
        raise HTTPException(status_code=400, detail="Client ID ve Client Secret alanları zorunludur.")

    redirect_uri = req.redirect_uri or config.DEFAULT_REDIRECT_URI
    config.save_credentials(req.client_id, req.client_secret, redirect_uri)

    return {
        "success": True,
        "message": "Spotify API ayarları başarıyla kaydedildi.",
        "redirect_uri": redirect_uri
    }

@app.get("/api/auth/login")
def login():
    if not config.has_valid_credentials():
        raise HTTPException(
            status_code=400,
            detail="Önce Spotify Client ID ve Client Secret ayarlarını kaydetmelisiniz."
        )

    auth_url = spotify_client.get_auth_url()
    if not auth_url:
        raise HTTPException(status_code=500, detail="Spotify giriş bağlantısı oluşturulamadı.")

    return {"auth_url": auth_url}

@app.get("/callback")
def auth_callback(code: Optional[str] = None, error: Optional[str] = None):
    if error:
        return RedirectResponse(url=f"/?auth_error={error}")

    if not code:
        return RedirectResponse(url="/?auth_error=Kod_bulunamadi")

    token_info = spotify_client.handle_callback(code)
    if not token_info:
        return RedirectResponse(url="/?auth_error=Yetkilendirme_basarisiz")

    return RedirectResponse(url="/?auth=success")

@app.post("/api/auth/logout")
def logout():
    spotify_client.logout()
    return {"success": True, "message": "Çıkış yapıldı."}

@app.get("/api/playlists")
def get_playlists():
    if not spotify_client.is_authenticated():
        raise HTTPException(status_code=401, detail="Spotify oturumu açılmamış.")

    playlists = spotify_client.get_user_playlists()
    return {"playlists": playlists}

@app.get("/api/playlist/{playlist_id}/analyze")
def analyze_playlist(playlist_id: str):
    if not spotify_client.is_authenticated():
        raise HTTPException(status_code=401, detail="Spotify oturumu açılmamış.")

    playlist_info = spotify_client.get_playlist_details(playlist_id)
    if not playlist_info:
        raise HTTPException(status_code=404, detail="Çalma listesi bulunamadı veya erişilemedi.")

    try:
        raw_tracks = spotify_client.get_all_playlist_tracks(playlist_id)
    except PermissionError as pe:
        raise HTTPException(status_code=403, detail=str(pe)) from pe
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Şarkılar alınırken hata oluştu: {e}") from e

    if not raw_tracks:
        return {
            "playlist": playlist_info,
            "analysis": {
                "total_tracks": 0,
                "unique_artists": 0,
                "macro_genres": [],
                "micro_genres": [],
                "tracks": [],
                "categorized_count": 0,
                "uncategorized_count": 0
            }
        }

    artist_names_map = {}
    for item in raw_tracks:
        if isinstance(item, dict):
            track = item.get("track")
            if isinstance(track, dict):
                for a in (track.get("artists") or []):
                    if isinstance(a, dict):
                        aid = a.get("id")
                        aname = a.get("name")
                        if aid and aname:
                            artist_names_map[aid] = aname

    artists_map = spotify_client.batch_get_artists_genres(artist_names_map)

    analysis_result = genre_classifier.analyze_playlist_tracks(raw_tracks, artists_map)

    return {
        "playlist": playlist_info,
        "analysis": analysis_result
    }

@app.post("/api/playlist/create")
def create_single_playlist(req: CreatePlaylistRequest):
    if not spotify_client.is_authenticated():
        raise HTTPException(status_code=401, detail="Spotify oturumu açılmamış.")

    if not req.track_uris:
        raise HTTPException(status_code=400, detail="Eklenecek şarkı bulunamadı.")

    result = spotify_client.create_user_playlist(
        name=req.name,
        description=req.description or "Spotify Playlist Genre Organizer ile oluşturuldu.",
        track_uris=req.track_uris,
        is_public=req.is_public
    )

    if not result:
        raise HTTPException(status_code=500, detail="Playlist oluşturulurken hata oluştu.")

    return {
        "success": True,
        "playlist": result
    }

@app.post("/api/playlist/split-all")
def split_all_genres(req: SplitAllRequest):
    if not spotify_client.is_authenticated():
        raise HTTPException(status_code=401, detail="Spotify oturumu açılmamış.")

    created_playlists = []
    errors = []

    for genre_name, uris in req.genre_data.items():
        if len(uris) < req.min_tracks:
            continue

        pl_name = f"{req.playlist_name} - {genre_name}"
        pl_desc = f"{req.playlist_name} listesinden '{genre_name}' türündeki şarkılar. (Playlist Organizer)"

        result = spotify_client.create_user_playlist(
            name=pl_name,
            description=pl_desc,
            track_uris=uris,
            is_public=req.is_public
        )

        if result:
            created_playlists.append({
                "genre": genre_name,
                "name": result["name"],
                "spotify_url": result["spotify_url"],
                "total_tracks": result["total_tracks"]
            })
        else:
            errors.append(genre_name)

    return {
        "success": True,
        "created_count": len(created_playlists),
        "playlists": created_playlists,
        "failed_genres": errors
    }
