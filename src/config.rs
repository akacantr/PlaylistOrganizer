use std::fs;
use std::path::PathBuf;

pub const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";

pub const SPOTIFY_SCOPES: &[&str] = &[
    "user-read-private",
    "user-read-email",
    "user-library-read",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-public",
    "playlist-modify-private",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
        }
    }
}

pub fn get_base_dir() -> PathBuf {
    // 1. Check if running inside AppImage
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        let appimage_path = PathBuf::from(appimage);
        if let Some(parent) = appimage_path.parent() {
            if parent.join(".env").exists() {
                return parent.to_path_buf();
            }
        }
        if let Some(cfg_dir) = dirs::config_dir() {
            let app_cfg = cfg_dir.join("playlist-organizer");
            let _ = fs::create_dir_all(&app_cfg);
            return app_cfg;
        }
    }

    // 2. Check if APPDIR is set without APPIMAGE
    if std::env::var("APPDIR").is_ok() {
        if let Some(cfg_dir) = dirs::config_dir() {
            let app_cfg = cfg_dir.join("playlist-organizer");
            let _ = fs::create_dir_all(&app_cfg);
            return app_cfg;
        }
    }

    // 3. Check if executable has a .env nearby
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if exe_dir.join(".env").exists() {
                return exe_dir.to_path_buf();
            }
        }
    }

    // 4. Check current working directory
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join(".env").exists() || cwd.join("Cargo.toml").exists() {
            return cwd;
        }
    }

    // 5. Fallback to user config dir
    if let Some(cfg_dir) = dirs::config_dir() {
        let app_cfg = cfg_dir.join("playlist-organizer");
        let _ = fs::create_dir_all(&app_cfg);
        return app_cfg;
    }

    PathBuf::from(".")
}

pub fn get_env_path() -> PathBuf {
    get_base_dir().join(".env")
}

pub fn get_token_cache_path() -> PathBuf {
    get_base_dir().join(".spotify_token_cache")
}

pub fn get_artist_cache_path() -> PathBuf {
    let base = get_base_dir();
    let data_dir = base.join("data");
    let _ = fs::create_dir_all(&data_dir);
    data_dir.join("artist_cache.json")
}

pub fn load_config() -> AppConfig {
    let mut client_id = std::env::var("SPOTIPY_CLIENT_ID")
        .unwrap_or_default()
        .trim()
        .to_string();
    let mut client_secret = std::env::var("SPOTIPY_CLIENT_SECRET")
        .unwrap_or_default()
        .trim()
        .to_string();
    let mut redirect_uri = std::env::var("SPOTIPY_REDIRECT_URI")
        .unwrap_or_default()
        .trim()
        .to_string();

    let env_path = get_env_path();
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
        if let Ok(content) = fs::read_to_string(&env_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    match k {
                        "SPOTIPY_CLIENT_ID" if !v.is_empty() => client_id = v,
                        "SPOTIPY_CLIENT_SECRET" if !v.is_empty() => client_secret = v,
                        "SPOTIPY_REDIRECT_URI" if !v.is_empty() => redirect_uri = v,
                        _ => {}
                    }
                }
            }
        }
    } else {
        let _ = dotenvy::dotenv();
    }

    if redirect_uri.is_empty() {
        redirect_uri = DEFAULT_REDIRECT_URI.to_string();
    }

    AppConfig {
        client_id,
        client_secret,
        redirect_uri,
    }
}

pub fn has_valid_credentials(config: &AppConfig) -> bool {
    let cid = config.client_id.trim();
    let csec = config.client_secret.trim();
    cid.len() > 10 && csec.len() > 10
}

pub fn save_config(config: &AppConfig) -> std::io::Result<()> {
    let env_path = get_env_path();
    if let Some(parent) = env_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let client_id = config.client_id.trim();
    let client_secret = config.client_secret.trim();
    let redirect_uri = if config.redirect_uri.trim().is_empty() {
        DEFAULT_REDIRECT_URI
    } else {
        config.redirect_uri.trim()
    };

    let content = format!(
        "# Spotify Playlist Genre Organizer Configuration\nSPOTIPY_CLIENT_ID={}\nSPOTIPY_CLIENT_SECRET={}\nSPOTIPY_REDIRECT_URI={}\n",
        client_id, client_secret, redirect_uri
    );

    fs::write(&env_path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&env_path, fs::Permissions::from_mode(0o600));
    }

    std::env::set_var("SPOTIPY_CLIENT_ID", client_id);
    std::env::set_var("SPOTIPY_CLIENT_SECRET", client_secret);
    std::env::set_var("SPOTIPY_REDIRECT_URI", redirect_uri);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_redirect_uri() {
        assert_eq!(DEFAULT_REDIRECT_URI, "http://127.0.0.1:8888/callback");
    }

    #[test]
    fn test_valid_credentials() {
        let invalid = AppConfig {
            client_id: "123".into(),
            client_secret: "abc".into(),
            redirect_uri: DEFAULT_REDIRECT_URI.into(),
        };
        assert!(!has_valid_credentials(&invalid));

        let valid = AppConfig {
            client_id: "mock_test_client_id_placeholder".into(),
            client_secret: "mock_test_client_secret_placeholder".into(),
            redirect_uri: DEFAULT_REDIRECT_URI.into(),
        };
        assert!(has_valid_credentials(&valid));
    }

    #[test]
    #[cfg(unix)]
    fn test_save_config_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp_dir = std::env::temp_dir().join(format!("po_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);
        let env_file = tmp_dir.join(".env");
        let content = "TEST=1\n";
        let _ = fs::write(&env_file, content);
        let _ = fs::set_permissions(&env_file, fs::Permissions::from_mode(0o600));
        let meta = fs::metadata(&env_file).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
