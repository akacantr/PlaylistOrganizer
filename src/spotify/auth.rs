use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Response, Server};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::{self, AppConfig, SPOTIFY_SCOPES};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpotifyToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub expires_at: i64,
}

impl SpotifyToken {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        // Treat as expired if within 60 seconds of expiration
        self.expires_at <= now + 60
    }
}

pub struct SpotifyAuth {
    token_cache_path: PathBuf,
    current_token: Arc<Mutex<Option<SpotifyToken>>>,
    client: reqwest::Client,
    cancel_auth: Arc<AtomicBool>,
}

impl SpotifyAuth {
    pub fn new() -> Self {
        let path = config::get_token_cache_path();
        let loaded = Self::load_cached_token(&path);
        Self {
            token_cache_path: path,
            current_token: Arc::new(Mutex::new(loaded)),
            client: reqwest::Client::new(),
            cancel_auth: Arc::new(AtomicBool::new(false)),
        }
    }

    fn load_cached_token(path: &PathBuf) -> Option<SpotifyToken> {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(token) = serde_json::from_str::<SpotifyToken>(&content) {
                    return Some(token);
                }
            }
        }
        None
    }

    pub fn save_token(&self, token: &SpotifyToken) {
        if let Ok(mut lock) = self.current_token.lock() {
            *lock = Some(token.clone());
        }
        if let Ok(json_str) = serde_json::to_string_pretty(token) {
            let _ = fs::write(&self.token_cache_path, json_str);
        }
    }

    pub fn logout(&self) {
        if let Ok(mut lock) = self.current_token.lock() {
            *lock = None;
        }
        let _ = fs::remove_file(&self.token_cache_path);
    }

    pub fn is_authenticated(&self) -> bool {
        if let Ok(lock) = self.current_token.lock() {
            if let Some(t) = lock.as_ref() {
                return !t.access_token.is_empty();
            }
        }
        false
    }

    pub fn get_token(&self) -> Option<SpotifyToken> {
        self.current_token.lock().ok().and_then(|l| l.clone())
    }

    pub fn generate_auth_url(&self, app_cfg: &AppConfig) -> String {
        let scopes = SPOTIFY_SCOPES.join(" ");
        format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&show_dialog=true",
            url::form_urlencoded::byte_serialize(app_cfg.client_id.trim().as_bytes()).collect::<String>(),
            url::form_urlencoded::byte_serialize(app_cfg.redirect_uri.trim().as_bytes()).collect::<String>(),
            url::form_urlencoded::byte_serialize(scopes.as_bytes()).collect::<String>(),
        )
    }

    pub async fn get_valid_access_token(&self, app_cfg: &AppConfig) -> Option<String> {
        let token_opt = self.get_token();
        if let Some(token) = token_opt {
            if !token.is_expired() {
                return Some(token.access_token);
            }

            // Attempt to refresh
            if let Some(ref refresh_tok) = token.refresh_token {
                if let Ok(refreshed) = self.refresh_access_token(app_cfg, refresh_tok).await {
                    return Some(refreshed.access_token);
                }
            }
        }
        None
    }

    pub async fn refresh_access_token(
        &self,
        app_cfg: &AppConfig,
        refresh_token: &str,
    ) -> Result<SpotifyToken, String> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let resp = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&app_cfg.client_id, Some(&app_cfg.client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to send refresh request: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 400 || status.as_u16() == 401 {
                self.logout();
            }
            return Err(format!("Token refresh failed: {}", err_body));
        }

        let mut token_val: SpotifyToken = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse token refresh response: {}", e))?;

        if token_val.refresh_token.is_none() {
            token_val.refresh_token = Some(refresh_token.to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let expires_in = token_val.expires_in.unwrap_or(3600);
        token_val.expires_at = now + expires_in;

        self.save_token(&token_val);
        Ok(token_val)
    }

    pub async fn exchange_code_for_token(
        &self,
        app_cfg: &AppConfig,
        code: &str,
    ) -> Result<SpotifyToken, String> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", app_cfg.redirect_uri.trim()),
        ];

        let resp = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&app_cfg.client_id, Some(&app_cfg.client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to exchange authorization code: {}", e))?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", err_body));
        }

        let mut token_val: SpotifyToken = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let expires_in = token_val.expires_in.unwrap_or(3600);
        token_val.expires_at = now + expires_in;

        self.save_token(&token_val);
        Ok(token_val)
    }

    pub fn cancel_login(&self) {
        self.cancel_auth.store(true, Ordering::SeqCst);
    }

    /// Spawns a background listener for the OAuth2 callback on the given redirect URI.
    pub fn listen_for_callback(
        &self,
        redirect_uri: &str,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<String, String>>, String> {
        self.cancel_auth.store(false, Ordering::SeqCst);
        let parsed_url =
            url::Url::parse(redirect_uri).map_err(|e| format!("Geçersiz Redirect URI: {}", e))?;

        let host = parsed_url.host_str().unwrap_or("127.0.0.1");
        let port = parsed_url.port().unwrap_or(8888);
        let bind_host = if host == "localhost" {
            "127.0.0.1"
        } else {
            host
        };
        let addr = format!("{}:{}", bind_host, port);

        let server = Server::http(&addr)
            .or_else(|_| Server::http(format!("127.0.0.1:{}", port)))
            .map_err(|e| format!("Yerel port açılamadı ({}): {}. Başka bir uygulama bu portu kullanıyor olabilir.", addr, e))?;

        let cancel = self.cancel_auth.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();

        std::thread::spawn(move || {
            // Wait with a 2-minute timeout
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(120);

            let mut final_code: Result<String, String> =
                Err("Yetkilendirme zaman aşımına uğradı.".to_string());

            while start.elapsed() < timeout {
                if cancel.load(Ordering::SeqCst) || tx.is_closed() {
                    final_code = Err("Yetkilendirme iptal edildi.".to_string());
                    break;
                }

                match server.recv_timeout(std::time::Duration::from_millis(250)) {
                    Ok(Some(request)) => {
                        let url_str = request.url().to_string();
                        if url_str.contains("/callback") {
                            let dummy = format!("http://127.0.0.1{}", url_str);
                            if let Ok(u) = url::Url::parse(&dummy) {
                                let mut code_found = None;
                                let mut error_found = None;

                                for (k, v) in u.query_pairs() {
                                    if k == "code" {
                                        code_found = Some(v.to_string());
                                    } else if k == "error" {
                                        error_found = Some(v.to_string());
                                    }
                                }

                                let html = if let Some(code) = code_found {
                                    final_code = Ok(code);
                                    r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Giriş Başarılı</title>
<style>body{background:#121212;color:#fff;font-family:system-ui,-apple-system,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;}
.box{background:#181818;padding:40px;border-radius:12px;text-align:center;border:1px solid #282828;box-shadow:0 12px 32px rgba(0,0,0,0.6);}
h1{color:#1DB954;margin:0 0 12px 0;}p{color:#b3b3b3;margin:0;}</style>
</head><body><div class="box"><h1>✓ Spotify Bağlantısı Başarılı!</h1><p>Bu sekmeyi kapatıp Playlist Organizer masaüstü uygulamasına dönebilirsiniz.</p></div></body></html>"#
                                } else {
                                    let err_msg =
                                        error_found.unwrap_or_else(|| "Kod bulunamadı".to_string());
                                    final_code = Err(format!("Spotify hatası: {}", err_msg));
                                    r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Giriş Başarısız</title>
<style>body{background:#121212;color:#fff;font-family:system-ui,-apple-system,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;}
.box{background:#181818;padding:40px;border-radius:12px;text-align:center;border:1px solid #e11d48;box-shadow:0 12px 32px rgba(0,0,0,0.6);}
h1{color:#e11d48;margin:0 0 12px 0;}p{color:#b3b3b3;margin:0;}</style>
</head><body><div class="box"><h1>✕ Giriş Başarısız Oldu</h1><p>Yetkilendirme sırasında bir hata oluştu. Uygulamayı kontrol edin.</p></div></body></html>"#
                                };

                                let response = Response::from_string(html).with_header(
                                    tiny_http::Header::from_bytes(
                                        &b"Content-Type"[..],
                                        &b"text/html; charset=utf-8"[..],
                                    )
                                    .unwrap(),
                                );
                                let _ = request.respond(response);
                                break;
                            }
                        } else {
                            let response = Response::from_string("Not Found").with_status_code(404);
                            let _ = request.respond(response);
                        }
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }

            let _ = tx.send(final_code);
        });

        Ok(rx)
    }
}

impl Default for SpotifyAuth {
    fn default() -> Self {
        Self::new()
    }
}
