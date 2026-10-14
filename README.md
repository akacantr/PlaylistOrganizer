<p align="center">
  <img src="logo.svg" width="140" height="140" alt="Spotify Playlist Genre Organizer Logo">
</p>

<h1 align="center">Spotify Playlist Genre Organizer</h1>

<p align="center">
  A modern full-stack web application that analyzes your Spotify playlists, classifies tracks by genre (<b>Rock, Pop, Hip-Hop, Turkish Rap, Turkish Arabesque, Phonk / Drift & Bass, Metal, Electronic / Dance, R&B, etc.</b>), and splits them into dedicated new Spotify playlists with a single click.
</p>

---

## 🚀 Key Features

- **🔒 Secure Spotify OAuth2 Authentication**: Direct authorization via your own Spotify Developer App credentials.
- **⚡ Multi-Tier Intelligent Genre Classification Engine**:
  - **Curated Knowledge Base**: Identifies hundreds of popular Turkish (Rap, Arabesque, Rock, Pop, Folk) and global artists with 0ms latency.
  - **Deezer API Integration**: Seamlessly resolves real-time artist album and genre metadata to overcome Spotify Developer Mode restrictions.
  - **Persistent Disk Cache (`artist_cache.json`)**: Caches resolved artists permanently to disk, enabling instant loading on subsequent runs.
  - **Heuristic Metadata Inference**: Automatically detects untagged underground and TikTok tracks based on title cues (`phonk`, `montagem`, `automotivo`, `slowed + reverb`, `hardstyle`, `remix`, etc.).
- **📊 Visual Genre Distribution**: Interactive percentage breakdown and distribution bars.
- **🎯 Flexible Filtering & Search**: Click any genre badge to filter songs instantly, or search by track name and artist.
- **🎧 Audio Preview Player**: Listen to 30-second audio previews directly in the browser.
- **🪄 One-Click Playlist Splitting & Exporting**:
  - **Auto-Split All Genres**: Converts all identified genres into separate new Spotify playlists simultaneously.
  - **Export Selected Genre**: Creates a custom playlist for just the selected genre or filtered tracks.
- **🌙 Spotify Dark Mode UI**: Clean glassmorphism styling, smooth transitions, and responsive mobile-ready layout.

---

## 🛠️ Installation & Setup

### 1. Prerequisites
- Python 3.9 or higher installed.

### 2. Clone the Repository & Install Dependencies
```bash
git clone https://github.com/YOUR_USERNAME/PlaylistOrganizer.git
cd PlaylistOrganizer
pip install -r requirements.txt
```

### 3. Spotify Developer App Configuration (2 Minutes)
1. Navigate to the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard) and log in.
2. Click **"Create app"**:
   - **App name**: `Playlist Organizer`
   - **App description**: `Organize and split Spotify playlists by genre`
   - **Redirect URIs**: Add **both** of the following URIs and click **Add**:
     - `http://127.0.0.1:8888/callback`
     - `http://localhost:8888/callback`
   - **Which API/SDKs are you planning to use?**: Select `Web API`.
3. Accept the Terms of Service and click **Save**.
4. Go to the **Settings** tab.
5. Copy your **Client ID** and click **View client secret** to copy your **Client Secret**.

### 4. Run the Application
- On Windows: Double-click **`start.bat`** or run in terminal:
```powershell
python run.py
```
- Your browser will automatically open at `http://127.0.0.1:8888`.

### 5. Usage
1. Click the **Settings** (gear icon) in the top-right corner.
2. Enter your **Client ID** and **Client Secret**, then click **Save**.
3. Click **"Connect Spotify Account"** to authenticate.
4. Select any playlist and start organizing your tracks by genre!

---

## 📁 Project Structure

```
PlaylistOrganizer/
├── app/
│   ├── artist_db.py          # Pre-compiled artist and genre knowledge base
│   ├── config.py             # Environment variables and configuration manager
│   ├── genre_classifier.py   # Macro/micro taxonomy and heuristic inference engine
│   ├── main.py               # FastAPI backend routes and API endpoints
│   ├── spotify_client.py     # Spotify OAuth, Deezer API client, and caching
│   ├── static/               # CSS styles and Vanilla JS application controller
│   └── templates/            # Responsive Spotify Dark theme HTML UI
├── data/
│   └── artist_cache.json     # Persistent cache for 700+ artist genres
├── .env.example              # Configuration template
├── .gitignore                # Protection for secrets and session tokens
├── README.md                 # Project documentation
├── requirements.txt          # Python dependencies
├── run.py                    # Server launcher
└── start.bat                 # One-click Windows batch launcher
```

---

## 📄 License
This project is licensed under the [MIT License](LICENSE).
