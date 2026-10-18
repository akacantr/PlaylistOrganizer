#!/usr/bin/env bash
# ==============================================================================
#  Spotify Playlist Genre Organizer - Linux Desktop Installer
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$APP_DIR" "$ICON_DIR"

# Install application icon
cp "$SCRIPT_DIR/logo.svg" "$ICON_DIR/playlist-organizer.svg"

# Install desktop entry with absolute paths
cat <<EOF > "$APP_DIR/playlist-organizer.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=Spotify Playlist Genre Organizer
GenericName=Playlist Organizer
Comment=Organize, filter, and split your Spotify playlists by genre
Exec="$SCRIPT_DIR/start.sh"
Icon=playlist-organizer
Terminal=false
Categories=AudioVideo;Audio;Utility;
Keywords=spotify;playlist;music;genre;organizer;
StartupNotify=true
EOF

chmod +x "$APP_DIR/playlist-organizer.desktop"

# Update desktop database if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi

echo "✅ Spotify Playlist Genre Organizer desktop shortcut installed successfully!"
echo "You can now launch it from your application menu or search for 'Playlist Organizer'."
