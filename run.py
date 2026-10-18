import argparse
import os
import sys
import time
import threading
import webbrowser
import uvicorn

def reconfigure_streams():
    for stream in (sys.stdout, sys.stderr):
        try:
            if hasattr(stream, "reconfigure"):
                stream.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass

reconfigure_streams()

sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

def open_browser(url: str, delay: float = 1.2):
    def _open():
        time.sleep(delay)
        print(f"[PlaylistOrganizer] Opening browser: {url}")
        try:
            webbrowser.open(url)
        except Exception as e:
            print(f"[PlaylistOrganizer] Notice: Browser could not be opened automatically: {e}")

    threading.Thread(target=_open, daemon=True).start()

def main():
    parser = argparse.ArgumentParser(description="Spotify Playlist Genre Organizer")
    parser.add_argument("--host", default=os.getenv("HOST", "127.0.0.1"), help="Host IP to bind (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=int(os.getenv("PORT", "8888")), help="Port to listen on (default: 8888)")
    parser.add_argument("--no-browser", action="store_true", help="Do not open the browser automatically")
    parser.add_argument("--reload", action="store_true", default=False, help="Enable auto-reload (development only)")
    args = parser.parse_args()

    host = args.host
    port = args.port
    url = f"http://{host}:{port}"

    print("=" * 60)
    print(" 🎵 Spotify Playlist Genre Organizer Starting...")
    print(f" 🌐 URL: {url}")
    print(f" 🖥️  Platform: {sys.platform}")
    print("=" * 60)

    if not args.no_browser and not os.getenv("HEADLESS"):
        open_browser(url)

    is_frozen = getattr(sys, "frozen", False)
    if is_frozen:
        from app.main import app
        uvicorn.run(app, host=host, port=port, log_level="info")
    else:
        reload = args.reload or bool(os.getenv("DEV_MODE"))
        uvicorn.run("app.main:app", host=host, port=port, reload=reload, log_level="info")

if __name__ == "__main__":
    main()
