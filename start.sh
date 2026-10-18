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
if [ ! -d "$VENV_DIR" ]; then
    echo -e "${YELLOW}Creating virtual environment in .venv...${NC}"
    "$PYTHON_BIN" -m venv "$VENV_DIR"
    echo -e "${YELLOW}Installing dependencies from requirements.txt...${NC}"
    "$VENV_DIR/bin/pip" install --upgrade pip
    "$VENV_DIR/bin/pip" install -r "$SCRIPT_DIR/requirements.txt"
fi

# Activate virtualenv
# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

# Launch application
exec python run.py "$@"
