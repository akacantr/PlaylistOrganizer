#!/usr/bin/env bash
# ==============================================================================
#  Spotify Playlist Genre Organizer - Linux Desktop Installer
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

if [ "$1" = "--uninstall" ] || [ "$1" = "-u" ]; then
    echo "🗑️ Removing desktop entry and application icon..."
    rm -f "$APP_DIR/playlist-organizer.desktop"
    rm -f "$ICON_DIR/playlist-organizer.svg"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true
    fi
    echo "✅ Spotify Playlist Genre Organizer desktop shortcut removed."
    exit 0
fi

mkdir -p "$APP_DIR" "$ICON_DIR"

# Install application icon if present
if [ -f "$SCRIPT_DIR/logo.svg" ]; then
    cp "$SCRIPT_DIR/logo.svg" "$ICON_DIR/playlist-organizer.svg"
fi

# Ensure launchers have executable permission
if [ -f "$SCRIPT_DIR/start.sh" ]; then
    chmod +x "$SCRIPT_DIR/start.sh"
fi
if [ -f "$SCRIPT_DIR/playlist-organizer" ]; then
    chmod +x "$SCRIPT_DIR/playlist-organizer"
fi

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

# Update desktop and icon databases if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "✅ Spotify Playlist Genre Organizer desktop shortcut installed successfully!"
echo "You can now launch it from your application menu or search for 'Playlist Organizer'."
echo "To uninstall, run: ./install-desktop.sh --uninstall"
