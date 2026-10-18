#!/usr/bin/env bash
# ==============================================================================
#  Spotify Playlist Genre Organizer - Linux / macOS Launcher
# ==============================================================================
set -e

# Change directory to the repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}========================================================${NC}"
echo -e "${BOLD}  🎵 Spotify Playlist Genre Organizer Starting...${NC}"
echo -e "${CYAN}========================================================${NC}"

# If standalone binary exists, run it directly without requiring Python/venv
if [ -f "$SCRIPT_DIR/playlist-organizer" ]; then
    if [ ! -x "$SCRIPT_DIR/playlist-organizer" ]; then
        chmod +x "$SCRIPT_DIR/playlist-organizer"
    fi
    echo -e "Starting standalone binary: ${GREEN}playlist-organizer${NC}"
    exec "$SCRIPT_DIR/playlist-organizer" "$@"
fi

# Otherwise, run from source repository with Python
# Find python3 or python (must be >= 3.9)
PYTHON_BIN=""
for candidate in python3 python; do
    if command -v "$candidate" >/dev/null 2>&1; then
        if "$candidate" -c 'import sys; exit(0 if sys.version_info >= (3, 9) else 1)' >/dev/null 2>&1; then
            PYTHON_BIN="$candidate"
            break
        fi
    fi
done

if [ -z "$PYTHON_BIN" ]; then
    echo -e "${RED}[ERROR] Python 3.9 or newer is required but was not found!${NC}"
    echo "Please install Python 3.9+ using your package manager (e.g., sudo pacman -S python, or sudo apt install python3 python3-venv)."
    exit 1
fi

echo -e "Using Python: ${GREEN}$($PYTHON_BIN --version)${NC}"

# Virtual environment management
VENV_DIR="$SCRIPT_DIR/.venv"
STAMP_FILE="$VENV_DIR/.deps_installed"

if [ ! -d "$VENV_DIR" ] || [ ! -f "$VENV_DIR/bin/activate" ]; then
    echo -e "${YELLOW}Creating virtual environment in .venv...${NC}"
    if ! "$PYTHON_BIN" -m venv "$VENV_DIR"; then
        echo -e "${RED}[ERROR] Failed to create virtual environment!${NC}"
        echo "On Debian/Ubuntu systems, install the python3-venv package:"
        echo "    sudo apt install python3-venv"
        exit 1
    fi
    rm -f "$STAMP_FILE"
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

# Install or update dependencies if requirements.txt changed or stamp is missing
if [ -f "$SCRIPT_DIR/requirements.txt" ]; then
    if [ ! -f "$STAMP_FILE" ] || [ "$SCRIPT_DIR/requirements.txt" -nt "$STAMP_FILE" ]; then
        echo -e "${YELLOW}Installing/updating dependencies from requirements.txt...${NC}"
        python -m pip install --upgrade pip
        python -m pip install -r "$SCRIPT_DIR/requirements.txt"
        touch "$STAMP_FILE"
    fi
fi

# Launch application
exec python run.py "$@"
