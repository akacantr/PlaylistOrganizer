import os
import sys
import time
import threading
import webbrowser
import uvicorn

# Configure UTF-8 encoding for Windows console
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

# Add current directory to sys.path
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))


def open_browser(url: str, delay: float = 1.2):
    def _open():
        time.sleep(delay)
        print(f"[PlaylistOrganizer] Tarayıcı açılıyor: {url}")
        webbrowser.open(url)

    threading.Thread(target=_open, daemon=True).start()


def main():
    host = "127.0.0.1"
    port = 8888
    url = f"http://localhost:{port}"

    print("=" * 60)
    print(" 🎵 Spotify Playlist Genre Organizer Başlatılıyor...")
    print(f" 🌐 Adres: {url}")
    print("=" * 60)

    # Open browser automatically
    open_browser(url)

    # Run Uvicorn server with auto-reload
    uvicorn.run("app.main:app", host=host, port=port, reload=True, log_level="info")


if __name__ == "__main__":
    main()
