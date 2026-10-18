<p align="center">
  <img src="logo.svg" width="130" height="130" alt="Spotify Playlist Genre Organizer Logo">
</p>

<h1 align="center">Spotify Playlist Genre Organizer</h1>

<p align="center">
  <b>A modern, high-performance web application to automatically analyze, classify, and split your Spotify playlists into curated genre-specific playlists with a single click.</b>
</p>

<p align="center">
  <a href="https://github.com/akacantr/PlaylistOrganizer/releases/latest"><img src="https://img.shields.io/github/v/release/akacantr/PlaylistOrganizer?color=1db954&label=Release&style=flat-square" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/Python-3.9%20%7C%203.10%20%7C%203.11%20%7C%203.12%20%7C%203.13%20%7C%203.14-3776AB?style=flat-square&logo=python&logoColor=white" alt="Python Support">
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-1ED760?style=flat-square" alt="Platform Support">
  <img src="https://img.shields.io/badge/FastAPI-0.115+-009688?style=flat-square&logo=fastapi&logoColor=white" alt="FastAPI">
  <img src="https://img.shields.io/badge/Spotify%20Web%20API-OAuth2-1DB954?style=flat-square&logo=spotify&logoColor=white" alt="Spotify API">
  <img src="https://img.shields.io/badge/Docker-Ready-2496ED?style=flat-square&logo=docker&logoColor=white" alt="Docker Ready">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License: MIT"></a>
</p>

---

## 📑 Table of Contents

- [✨ Overview](#-overview)
- [🚀 Key Features](#-key-features)
- [📦 Pre-Built Releases (Windows & Linux)](#-pre-built-releases-windows--linux)
- [🐧 Linux Port & Installation](#-linux-port--installation)
  - [Method 1: One-Click Shell Launcher (Recommended)](#method-1-one-click-shell-launcher-recommended)
  - [Method 2: Standalone Linux Executable](#method-2-standalone-linux-executable)
  - [Method 3: Desktop App Menu Integration (`.desktop`)](#method-3-desktop-app-menu-integration-desktop)
  - [Method 4: Systemd User Service (Background Daemon)](#method-4-systemd-user-service-background-daemon)
  - [Method 5: Docker & Docker Compose](#method-5-docker--docker-compose)
- [🪟 Windows Installation](#-windows-installation)
  - [Method 1: One-Click Batch Launcher](#method-1-one-click-batch-launcher)
  - [Method 2: Standalone Windows Executable](#method-2-standalone-windows-executable)
- [⚙️ Spotify Developer App Configuration](#️-spotify-developer-app-configuration)
- [🧠 4-Tier Genre Classification Engine](#-4-tier-genre-classification-engine)
- [📡 REST API Endpoints](#-rest-api-endpoints)
- [📁 Project Structure](#-project-structure)
- [🛠️ Developer Guide & Testing](#️-developer-guide--testing)
- [❓ FAQ & Troubleshooting](#-faq--troubleshooting)
- [📄 License](#-license)

---

## ✨ Overview

Have a massive playlist with hundreds of songs mixing Rock, Pop, Hip-Hop, Turkish Rap, Metal, Phonk, and Electronic? **Spotify Playlist Genre Organizer** connects directly to your Spotify account, pulls track metadata, classifies every song into comprehensive macro and micro genres using a hybrid multi-tier intelligence engine, and allows you to split them into separate new Spotify playlists instantly.

Designed with an authentic Spotify glassmorphism dark-mode UI, it features audio previews, live percentage breakdowns, instant badge filtering, and full support for both **Linux** and **Windows**.

---

## 🚀 Key Features

- **🔒 Secure Spotify OAuth2 Authentication**:
  - Direct local authorization using your own Spotify Developer App credentials.
  - Zero third-party telemetry, secrets stored locally in `.env` and `.spotify_token_cache`.
- **⚡ 4-Tier Hybrid Genre Classification Engine**:
  - **Tier 1 (Curated Knowledge Base)**: Pre-compiled database of hundreds of popular Turkish (Rap, Arabesque, Rock, Pop, Folk) and global artists with 0ms latency.
  - **Tier 2 (Persistent Disk Cache)**: Caches resolved artists in `data/artist_cache.json` permanently to disk for instant subsequent loads.
  - **Tier 3 (Deezer API Fallback)**: Automatically resolves album and genre metadata to overcome Spotify Developer Mode restrictions.
  - **Tier 4 (Heuristic Metadata Inference)**: Accurately identifies untagged underground tracks, remixes, and TikTok trends based on title cues (`phonk`, `montagem`, `slowed + reverb`, `hardstyle`, `drill`, `remix`, `acoustic`, `live`, etc.).
- **🐧 First-Class Linux & Windows Support**:
  - Dedicated `start.sh` launcher with automatic Python virtual environment bootstrap.
  - Standard Freedesktop `.desktop` application menu integration and icon installer (`install-desktop.sh`).
  - Systemd user service unit for background headless servers.
  - Native Windows one-click `start.bat` launcher.
  - Production-ready `Dockerfile` and `docker-compose.yml`.
- **🪄 One-Click Playlist Splitting & Export**:
  - **Auto-Split All Genres**: Converts every identified genre in your playlist into separate, dedicated Spotify playlists simultaneously.
  - **Export Selected Genre**: Creates a custom playlist for just the filtered genre or tracks.
  - Automatic duplicate removal and track URI deduplication.
- **📊 Interactive Visual Analytics**:
  - Real-time genre percentage breakdown bars and count badges.
  - Multi-filter search bar to quickly inspect categorized vs. uncategorized tracks.
- **🎧 Built-In Audio Preview Player**:
  - Listen to 30-second song previews directly inside the browser.
- **🌙 Authentic Spotify Dark UI**:
  - Clean frosted glass styling, responsive layout, smooth transitions, and mobile-friendly design.

---

## 📦 Pre-Built Releases (Windows & Linux)

Pre-packaged release bundles are available on the [GitHub Releases page](https://github.com/akacantr/PlaylistOrganizer/releases).

| Platform | Package | Description | Quick Run |
| :--- | :--- | :--- | :--- |
| **Linux (x86_64)** | [`PlaylistOrganizer-v1.0.0-linux-x86_64.tar.gz`](https://github.com/akacantr/PlaylistOrganizer/releases) | Standalone Linux binary distribution. No Python installation required! | `tar -xvf PlaylistOrganizer-*.tar.gz && cd PlaylistOrganizer-* && ./playlist-organizer` |
| **Windows (x64)** | [`PlaylistOrganizer-v1.0.0-windows-x64.zip`](https://github.com/akacantr/PlaylistOrganizer/releases) | Standalone Windows distribution including `PlaylistOrganizer.exe` and `start.bat`. | Extract zip and double-click `PlaylistOrganizer.exe` or `start.bat` |
| **Portable Source** | [`PlaylistOrganizer-v1.0.0-windows-portable.zip`](https://github.com/akacantr/PlaylistOrganizer/releases) | Complete portable bundle for all systems with Python 3.9+. | Extract zip, run `./start.sh` (Linux) or `start.bat` (Windows) |

Each release package includes cryptographic verification hashes (`SHA256SUMS.txt`).

---

## 🐧 Linux Port & Installation

The application has been ported to Linux with native shell launchers, desktop entry files, systemd integration, and container support.

### Method 1: One-Click Shell Launcher (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/akacantr/PlaylistOrganizer.git
   cd PlaylistOrganizer
   ```
2. Make the launcher executable and run:
   ```bash
   chmod +x start.sh
   ./start.sh
   ```
   > **Note**: `start.sh` will automatically detect your Python 3 interpreter (>= 3.9), create a localized virtual environment (`.venv`), install all requirements from `requirements.txt`, and open your default browser at `http://127.0.0.1:8888`.

3. Optional CLI Flags:
   ```bash
   ./start.sh --host 0.0.0.0 --port 9000 --no-browser
   ```

### Method 2: Standalone Linux Executable

Download the pre-compiled standalone package from [Releases](https://github.com/akacantr/PlaylistOrganizer/releases):
```bash
tar -xvf PlaylistOrganizer-v1.0.0-linux-x86_64.tar.gz
cd PlaylistOrganizer-v1.0.0-linux-x86_64
./start.sh    # or directly: ./playlist-organizer
```

### Method 3: Desktop App Menu Integration (`.desktop`)

To launch Playlist Organizer directly from your desktop application launcher (GNOME, KDE Plasma, XFCE, Cinnamon, etc.):
```bash
chmod +x install-desktop.sh
./install-desktop.sh
```
This registers the application shortcut and installs high-resolution vector icons to `~/.local/share/applications/` and `~/.local/share/icons/`.

To uninstall the desktop shortcut:
```bash
./install-desktop.sh --uninstall
```

### Method 4: Systemd User Service (Background Daemon)

If you are running the application on a headless home server or want it to start automatically upon login:
```bash
mkdir -p ~/.config/systemd/user
# Auto-configure service unit with your current working directory:
sed "s|%h/PlaylistOrganizer|$PWD|g" playlist-organizer.service > ~/.config/systemd/user/playlist-organizer.service
systemctl --user daemon-reload
systemctl --user enable --now playlist-organizer
```
To view logs:
```bash
journalctl --user -u playlist-organizer -f
```

### Method 5: Docker & Docker Compose

Deploy with Docker in seconds:
```bash
# Using Docker Compose
docker compose up -d

# Or building directly
docker build -t playlist-organizer .
docker run -d -p 8888:8888 -v $(pwd)/.env:/app/.env -v $(pwd)/data:/app/data --name playlist-organizer playlist-organizer
```
Navigate to `http://localhost:8888` in your browser.

---

## 🪟 Windows Installation

### Method 1: One-Click Batch Launcher

1. Clone or download the repository:
   ```cmd
   git clone https://github.com/akacantr/PlaylistOrganizer.git
   cd PlaylistOrganizer
   ```
2. Double-click **`start.bat`** (or execute in PowerShell/CMD):
   ```powershell
   python run.py
   ```
3. Your browser will automatically open at `http://127.0.0.1:8888`.

### Method 2: Standalone Windows Executable

Download `PlaylistOrganizer-v1.0.0-windows-x64.zip` from [Releases](https://github.com/akacantr/PlaylistOrganizer/releases), unzip, and double-click `PlaylistOrganizer.exe`.

---

## ⚙️ Spotify Developer App Configuration

Setting up your Spotify credentials takes less than 2 minutes:

1. Visit the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard) and sign in.
2. Click **Create app**:
   - **App name**: `Playlist Organizer`
   - **App description**: `Organize and split Spotify playlists by genre`
   - **Redirect URIs**: Add **both** of the following URIs:
     - `http://127.0.0.1:8888/callback`
     - `http://localhost:8888/callback`
   - **Which API/SDKs are you planning to use?**: Check **Web API**.
3. Accept the Spotify terms and click **Save**.
4. Go to the **Settings** tab:
   - Copy your **Client ID**.
   - Click **View client secret** and copy your **Client Secret**.
5. Launch Playlist Organizer:
   - Click the **Settings (gear icon)** in the top-right corner.
   - Paste your **Client ID** and **Client Secret**, then click **Save**.
   - Click **"Connect Spotify Account"** to authorize.

### Spotify OAuth Scopes Explained
The application requests only the minimum required Spotify Web API scopes:
- `user-read-private` / `user-read-email`: Display your account username and profile picture.
- `playlist-read-private` / `playlist-read-collaborative`: Retrieve and read your personal playlists.
- `playlist-modify-public` / `playlist-modify-private`: Create new genre playlists and populate them with tracks.
- `user-library-read`: Access saved user track metadata.

---

## 🧠 4-Tier Genre Classification Engine

Spotify's Web API often returns limited or empty genre tags for tracks in Developer Mode. To guarantee comprehensive classification across Turkish and international music, Playlist Organizer uses a resilient 4-tier pipeline:

```
┌────────────────────────────────────────────────────────┐
│             Spotify Track & Artist Ingestion           │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
 ┌──────────────────────────────────────────────────────┐
 │ Tier 1: Curated In-Memory Knowledge Base             │ ➔ Instant match for hundreds of Turkish &
 │ (Ezhel, Uzi, Duman, Müslüm Gürses, Travis Scott...)  │   global artists with 0ms latency.
 └─────────────────────────┬────────────────────────────┘
                           │ (if unmatched)
                           ▼
 ┌──────────────────────────────────────────────────────┐
 │ Tier 2: Persistent Disk Cache                        │ ➔ Fast lookup from previous runs
 │ (data/artist_cache.json with 700+ entries)           │   without redundant network requests.
 └─────────────────────────┬────────────────────────────┘
                           │ (if cache miss)
                           ▼
 ┌──────────────────────────────────────────────────────┐
 │ Tier 3: Deezer API Metadata Resolution Engine        │ ➔ Fetches album genre tags to overcome
 │ (Asynchronous multi-threaded artist queries)         │   Spotify Developer Mode restrictions.
 └─────────────────────────┬────────────────────────────┘
                           │ (if untagged / underground)
                           ▼
 ┌──────────────────────────────────────────────────────┐
 │ Tier 4: Heuristic Audio Metadata Inference Engine    │ ➔ Uses regex patterns to identify Phonk,
 │ (Title cues: Slowed+Reverb, Montagem, Hardstyle...)  │   TikTok trends, EDM, OSTs, and Remixes.
 └─────────────────────────┬────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│      Macro/Micro Taxonomic Grouping & Discard Filters  │
│  (Specific genres supersede generic umbrella labels)   │
└────────────────────────────────────────────────────────┘
```

### Supported Macro Genres
- **Türkçe Rap / Hip-Hop**
- **Türkçe Pop**
- **Türkçe Rock**
- **Türkçe Arabesk / Fantezi**
- **Türkçe Halk & Sanat & Diğer**
- **Hip-Hop & Rap (International)**
- **Rock & Alt-Rock**
- **Metal & Heavy**
- **Pop (International)**
- **Phonk / Drift & Bass**
- **Elektronik / Dance / Club**
- **R&B & Soul**
- **Klasik & Soundtrack**
- **Reggae / Latin / World**
- **Indie & Alternatif**

---

## 📡 REST API Endpoints

The FastAPI backend exposes fully typed REST endpoints:

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/` | Serves the web UI (`index.html`) |
| `GET` | `/api/status` | Returns credential status, preview, and Spotify auth state |
| `POST` | `/api/config` | Saves Spotify Client ID, Client Secret, and Redirect URI |
| `GET` | `/api/auth/login` | Generates Spotify OAuth authorization URL |
| `GET` | `/callback` | Handles OAuth2 callback from Spotify |
| `POST` | `/api/auth/logout` | Clears active Spotify session cache |
| `GET` | `/api/playlists` | Fetches authenticated user's Spotify playlists |
| `GET` | `/api/playlist/{id}/analyze` | Fetches tracks, resolves genres, returns breakdown |
| `POST` | `/api/playlist/create` | Creates a single custom genre playlist on Spotify |
| `POST` | `/api/playlist/split-all` | Batch creates playlists for all classified genres |

---

## 📁 Project Structure

```
PlaylistOrganizer/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Automated multi-OS CI testing (Ubuntu & Windows)
│       └── release.yml               # Automated release builder & publisher
├── app/
│   ├── artist_db.py                  # Curated artist knowledge base
│   ├── config.py                     # Environment variables & runtime path resolver
│   ├── genre_classifier.py           # Taxonomy rules & heuristic inference engine
│   ├── main.py                       # FastAPI application & REST API routes
│   ├── spotify_client.py             # Spotify OAuth2, Deezer API client & disk cache
│   ├── static/                       # CSS stylesheets, vanilla JS controller
│   └── templates/                    # Spotify Dark glassmorphism HTML UI
├── data/
│   └── artist_cache.json             # Persistent disk cache for 700+ artist genres
├── scripts/
│   └── package_release.py            # Local release packaging utility (PyInstaller)
├── tests/
│   ├── test_config.py                # Configuration and credential validation tests
│   ├── test_genre_classifier.py      # Genre taxonomy & heuristic inference tests
│   ├── test_linux_port.py            # Linux port, scripts & launcher unit tests
│   └── test_spotify_client.py        # URI deduplication and client unit tests
├── .env.example                      # Configuration template
├── .gitignore                        # Git ignore patterns
├── Dockerfile                        # Production Docker container definition
├── docker-compose.yml                # Docker Compose multi-platform configuration
├── install-desktop.sh                # Linux desktop app menu shortcut installer
├── LICENSE                           # MIT License
├── logo.svg                          # High-resolution vector brand asset
├── playlist-organizer.desktop        # Freedesktop standard desktop launcher
├── playlist-organizer.service        # Systemd user service unit template
├── README.md                         # Project documentation
├── requirements.txt                  # Production Python dependencies
├── requirements-dev.txt              # Development and test dependencies
├── run.py                            # Application entry point with CLI options
├── start.bat                         # One-click Windows batch launcher
└── start.sh                          # One-click Linux / macOS shell launcher
```

---

## 🛠️ Developer Guide & Testing

### Running the Test Suite
The repository includes a comprehensive unit test suite covering genre classification, fallback inference, URI deduplication, and Linux porting integrity:

```bash
# Run all tests
pytest -v

# Run with test coverage
pytest --cov=app tests/
```

### Building Release Packages Locally
You can build standalone binary packages locally using the build script:
```bash
python scripts/package_release.py
```
This generates:
- `dist/PlaylistOrganizer-v1.0.0-linux-x86_64.tar.gz`
- `dist/PlaylistOrganizer-v1.0.0-windows-portable.zip`
- `dist/SHA256SUMS.txt`

---

## ❓ FAQ & Troubleshooting

<details>
<summary><b>Why do I get a 403 Forbidden error on certain playlists?</b></summary>
Due to Spotify Web API policies in <i>Development Mode</i>, applications can only access playlists belonging to the developer account or explicitly invited test accounts. To organize a playlist created by another user, simply copy/duplicate it into your own Spotify library first.
</details>

<details>
<summary><b>How do I run without opening the browser automatically?</b></summary>
Pass the <code>--no-browser</code> flag or set <code>HEADLESS=1</code>:
<pre><code>./start.sh --no-browser</code></pre>
</details>

<details>
<summary><b>Can I change the default listening port (8888)?</b></summary>
Yes! Pass <code>--port &lt;number&gt;</code> to <code>start.sh</code> or <code>run.py</code>:
<pre><code>./start.sh --port 9090</code></pre>
<i>Note: Remember to also add <code>http://127.0.0.1:9090/callback</code> to your Redirect URIs in the Spotify Developer Dashboard.</i>
</details>

<details>
<summary><b>Where are my credentials and cache saved?</b></summary>
Spotify credentials are stored in <code>.env</code>. Active Spotify OAuth access tokens are stored in <code>.spotify_token_cache</code>. Resolved artist genres are cached in <code>data/artist_cache.json</code>.
</details>

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
