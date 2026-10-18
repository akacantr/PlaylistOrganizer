import os
import sys
import requests
from pathlib import Path
from typing import Optional, Dict, Any, List
import spotipy
from spotipy.oauth2 import SpotifyOAuth, CacheFileHandler
from spotipy.exceptions import SpotifyException

from . import config

CACHE_PATH = config.BASE_DIR / ".spotify_token_cache"

ARTIST_GENRES_CACHE: Dict[str, Dict[str, Any]] = {}

def get_oauth_manager() -> Optional[SpotifyOAuth]:
    if not config.has_valid_credentials():
        return None

    cache_handler = CacheFileHandler(cache_path=str(CACHE_PATH))
    return SpotifyOAuth(
        client_id=config.get_client_id(),
        client_secret=config.get_client_secret(),
        redirect_uri=config.get_redirect_uri(),
        scope=" ".join(config.SPOTIFY_SCOPES),
        cache_handler=cache_handler,
        show_dialog=True
    )

def get_auth_url() -> Optional[str]:
    sp_oauth = get_oauth_manager()
    if not sp_oauth:
        return None
    return sp_oauth.get_authorize_url()

def handle_callback(code: str) -> Optional[Dict[str, Any]]:
    sp_oauth = get_oauth_manager()
    if not sp_oauth:
        return None
    token_info = sp_oauth.get_access_token(code, as_dict=True)
    return token_info

def is_authenticated() -> bool:
    sp_oauth = get_oauth_manager()
    if not sp_oauth:
        return False
    token_info = sp_oauth.validate_token(sp_oauth.cache_handler.get_cached_token())
    return token_info is not None

def logout() -> bool:
    global CURRENT_USER_CACHE
    CURRENT_USER_CACHE = None
    if CACHE_PATH.exists():
        try:
            CACHE_PATH.unlink()
            return True
        except Exception:
            return False
    return True

def get_spotify_client() -> Optional[spotipy.Spotify]:
    sp_oauth = get_oauth_manager()
    if not sp_oauth:
        return None

    token_info = sp_oauth.validate_token(sp_oauth.cache_handler.get_cached_token())
    if not token_info:
        return None

    return spotipy.Spotify(auth=token_info["access_token"])

CURRENT_USER_CACHE: Optional[Dict[str, Any]] = None

def get_current_user_profile() -> Optional[Dict[str, Any]]:
    global CURRENT_USER_CACHE
    if CURRENT_USER_CACHE:
        return CURRENT_USER_CACHE

    sp = get_spotify_client()
    if not sp:
        return None
    try:
        user = sp.current_user()
        avatar = ""
        if user.get("images") and len(user["images"]) > 0:
            avatar = user["images"][0].get("url", "")

        profile = {
            "id": user.get("id"),
            "display_name": user.get("display_name") or user.get("id"),
            "email": user.get("email", ""),
            "avatar": avatar,
            "product": user.get("product", ""),
            "country": user.get("country", ""),
            "spotify_url": user.get("external_urls", {}).get("spotify", "")
        }
        CURRENT_USER_CACHE = profile
        return profile
    except Exception as e:
        print(f"Error fetching current user profile: {e}")
        return None

def get_user_playlists() -> List[Dict[str, Any]]:
    sp = get_spotify_client()
    if not sp:
        return []

    profile = get_current_user_profile()
    my_id = profile.get("id") if profile else None

    playlists = []
    limit = 50
    offset = 0

    try:
        while True:
            response = sp.current_user_playlists(limit=limit, offset=offset)
            items = response.get("items", [])
            for pl in items:
                if not pl:
                    continue

                images = pl.get("images", [])
                image_url = images[0].get("url") if images else ""

                owner = pl.get("owner", {})
                owner_id = owner.get("id", "")
                owner_name = owner.get("display_name") or owner_id
                is_mine = bool(my_id and owner_id == my_id)

                playlists.append({
                    "id": pl.get("id"),
                    "name": pl.get("name"),
                    "description": pl.get("description", ""),
                    "image_url": image_url,
                    "total_tracks": pl.get("tracks", {}).get("total", 0),
                    "owner": owner_name,
                    "owner_id": owner_id,
                    "is_mine": is_mine,
                    "is_public": pl.get("public", False),
                    "is_collaborative": pl.get("collaborative", False),
                    "spotify_url": pl.get("external_urls", {}).get("spotify", "")
                })

            if len(items) < limit:
                break
            offset += limit

        playlists.sort(key=lambda p: (not p["is_mine"], p["name"].lower()))
        return playlists
    except Exception as e:
        print(f"Error fetching playlists: {e}")
        return []

def get_playlist_details(playlist_id: str) -> Optional[Dict[str, Any]]:
    sp = get_spotify_client()
    if not sp:
        return None

    try:
        pl = sp.playlist(playlist_id)
        images = pl.get("images", [])
        image_url = images[0].get("url") if images else ""
        owner = pl.get("owner", {})
        owner_name = owner.get("display_name") or owner.get("id", "")

        profile = get_current_user_profile()
        my_id = profile.get("id") if profile else None
        is_mine = bool(my_id and owner.get("id") == my_id)

        return {
            "id": pl.get("id"),
            "name": pl.get("name"),
            "description": pl.get("description", ""),
            "image_url": image_url,
            "total_tracks": pl.get("tracks", {}).get("total", 0) if pl.get("tracks") else 0,
            "owner": owner_name,
            "is_mine": is_mine,
            "is_public": pl.get("public", False),
            "spotify_url": pl.get("external_urls", {}).get("spotify", "")
        }
    except Exception as e:
        print(f"Error getting playlist details: {e}")
        return None

def get_all_playlist_tracks(playlist_id: str) -> List[Dict[str, Any]]:
    sp = get_spotify_client()
    if not sp:
        return []

    tracks = []
    limit = 100
    offset = 0

    try:
        while True:
            response = sp.playlist_items(
                playlist_id,
                limit=limit,
                offset=offset,
                additional_types=("track",)
            )

            items = response.get("items", [])
            for it in items:
                if not it:
                    continue
                track_data = it.get("item") or it.get("track")
                if track_data and track_data.get("id"):
                    tracks.append({
                        "track": track_data,
                        "added_at": it.get("added_at")
                    })

            if not response.get("next") or len(items) < limit:
                break
            offset += limit

        return tracks
    except SpotifyException as e:
        if e.http_status == 403:
            raise PermissionError(
                "Spotify API kısıtlaması (Development Mode): Bu çalma listesi başka bir kullanıcıya ait olduğu için şarkıları okunamıyor. Lütfen kendi oluşturduğunuz bir çalma listesini seçin."
            )
        raise e
    except Exception as e:
        print(f"Error fetching playlist tracks: {e}")
        raise e

import concurrent.futures

import json
import threading
from app.artist_db import CURATED_ARTISTS

HTTP_SESSION = requests.Session()

DISK_CACHE_PATH = os.path.join(str(config.BASE_DIR), "data", "artist_cache.json")
DISK_CACHE_LOCK = threading.Lock()
DISK_ARTIST_CACHE: Dict[str, List[str]] = {}

try:
    os.makedirs(os.path.dirname(DISK_CACHE_PATH), exist_ok=True)
    if not os.path.exists(DISK_CACHE_PATH) and getattr(sys, "frozen", False):
        bundled_candidates = []
        if hasattr(sys, "_MEIPASS"):
            bundled_candidates.append(os.path.join(sys._MEIPASS, "data", "artist_cache.json"))
            bundled_candidates.append(os.path.join(sys._MEIPASS, "artist_cache.json"))
        exe_parent = os.path.dirname(os.path.abspath(sys.executable))
        bundled_candidates.append(os.path.join(exe_parent, "data", "artist_cache.json"))
        bundled_candidates.append(os.path.join(exe_parent, "_internal", "data", "artist_cache.json"))
        for bc in bundled_candidates:
            if os.path.exists(bc):
                try:
                    import shutil
                    shutil.copy2(bc, DISK_CACHE_PATH)
                    break
                except Exception:
                    pass
    if os.path.exists(DISK_CACHE_PATH):
        with open(DISK_CACHE_PATH, "r", encoding="utf-8") as f:
            DISK_ARTIST_CACHE = json.load(f)
except Exception as e:
    print(f"Error initializing artist disk cache: {e}")
    DISK_ARTIST_CACHE = {}

def save_to_disk_cache(artist_clean: str, genres: List[str]):
    with DISK_CACHE_LOCK:
        DISK_ARTIST_CACHE[artist_clean] = genres
        try:
            with open(DISK_CACHE_PATH, "w", encoding="utf-8") as f:
                json.dump(DISK_ARTIST_CACHE, f, ensure_ascii=False, indent=2)
        except Exception:
            pass

def fetch_artist_genre_fallback(artist_name: str) -> List[str]:
    if not artist_name:
        return []

    a_clean = artist_name.lower().strip()

    if a_clean in CURATED_ARTISTS:
        return CURATED_ARTISTS[a_clean]

    if a_clean in DISK_ARTIST_CACHE:
        return DISK_ARTIST_CACHE[a_clean]

    try:
        url = f"https://api.deezer.com/search?q={requests.utils.quote(artist_name)}&limit=1"
        res = HTTP_SESSION.get(url, timeout=2.5)
        if res.status_code == 200:
            data = res.json().get("data", [])
            if data:
                alb_id = data[0].get("album", {}).get("id")
                if alb_id:
                    alb_res = HTTP_SESSION.get(f"https://api.deezer.com/album/{alb_id}", timeout=2.5)
                    if alb_res.status_code == 200:
                        genres = [g.get("name") for g in alb_res.json().get("genres", {}).get("data", []) if g.get("name")]
                        if genres:
                            save_to_disk_cache(a_clean, genres)
                            return genres
    except Exception:
        pass

    try:
        url = f"https://api.deezer.com/search/artist?q={requests.utils.quote(artist_name)}&limit=1"
        res = HTTP_SESSION.get(url, timeout=2.5)
        if res.status_code == 200:
            data = res.json().get("data", [])
            if data:
                art_id = data[0].get("id")
                top_res = HTTP_SESSION.get(f"https://api.deezer.com/artist/{art_id}/top?limit=1", timeout=2.5)
                if top_res.status_code == 200:
                    top_tracks = top_res.json().get("data", [])
                    if top_tracks:
                        alb_id = top_tracks[0].get("album", {}).get("id")
                        if alb_id:
                            alb_res = HTTP_SESSION.get(f"https://api.deezer.com/album/{alb_id}", timeout=2.5)
                            if alb_res.status_code == 200:
                                genres = [g.get("name") for g in alb_res.json().get("genres", {}).get("data", []) if g.get("name")]
                                if genres:
                                    save_to_disk_cache(a_clean, genres)
                                    return genres
    except Exception:
        pass

    save_to_disk_cache(a_clean, [])
    return []

def batch_get_artists_genres(artist_names_map: Dict[str, str]) -> Dict[str, Dict[str, Any]]:
    result_map = {}
    missing_to_fetch = []

    for aid, aname in artist_names_map.items():
        if aid in ARTIST_GENRES_CACHE:
            result_map[aid] = ARTIST_GENRES_CACHE[aid]
        else:
            missing_to_fetch.append((aid, aname))

    if not missing_to_fetch:
        return result_map

    def _resolve(item):
        aid, aname = item
        genres = fetch_artist_genre_fallback(aname)
        return aid, aname, genres

    max_workers = min(8, len(missing_to_fetch))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
        for aid, aname, genres in executor.map(_resolve, missing_to_fetch):
            artist_obj = {
                "name": aname,
                "genres": genres,
                "popularity": 0
            }
            ARTIST_GENRES_CACHE[aid] = artist_obj
            result_map[aid] = artist_obj

    return result_map

def deduplicate_track_uris(track_uris: List[str]) -> List[str]:
    seen = set()
    unique = []
    for u in track_uris:
        if u and isinstance(u, str) and u.startswith("spotify:track:") and u not in seen:
            seen.add(u)
            unique.append(u)
    return unique

def create_user_playlist(name: str, description: str, track_uris: List[str], is_public: bool = False) -> Optional[Dict[str, Any]]:
    sp = get_spotify_client()
    if not sp:
        return None

    try:
        user = sp.current_user()
        user_id = user["id"]

        new_playlist = sp.user_playlist_create(
            user=user_id,
            name=name,
            public=is_public,
            description=description
        )

        playlist_id = new_playlist["id"]
        unique_uris = deduplicate_track_uris(track_uris)

        chunk_size = 100
        for i in range(0, len(unique_uris), chunk_size):
            chunk = unique_uris[i:i + chunk_size]
            sp.playlist_add_items(playlist_id, chunk)

        return {
            "id": playlist_id,
            "name": new_playlist.get("name"),
            "description": description,
            "spotify_url": new_playlist.get("external_urls", {}).get("spotify", ""),
            "total_tracks": len(unique_uris)
        }
    except Exception as e:
        print(f"Error creating playlist: {e}")
        return None
