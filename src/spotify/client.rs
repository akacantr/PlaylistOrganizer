use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::cache::ArtistCache;
use crate::classifier::RawTrackItem;
use crate::deezer;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub avatar: String,
    pub product: String,
    pub country: String,
    pub spotify_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub total_tracks: usize,
    pub owner: String,
    pub owner_id: String,
    pub is_mine: bool,
    pub is_public: bool,
    pub is_collaborative: bool,
    pub spotify_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatedPlaylistResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spotify_url: String,
    pub total_tracks: usize,
}

pub struct SpotifyClient {
    client: reqwest::Client,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn execute_with_retry<F>(&self, make_request: F) -> Result<reqwest::Response, String>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut retries = 0;
        let max_retries = 3;
        loop {
            let resp = make_request()
                .send()
                .await
                .map_err(|e| format!("İstek başarısız: {}", e))?;

            if resp.status().as_u16() == 429 && retries < max_retries {
                let retry_after = resp
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2);
                tokio::time::sleep(std::time::Duration::from_secs(retry_after.max(1))).await;
                retries += 1;
                continue;
            }

            return Ok(resp);
        }
    }

    pub async fn get_current_user_profile(
        &self,
        access_token: &str,
    ) -> Result<UserProfile, String> {
        let resp = self
            .execute_with_retry(|| {
                self.client
                    .get("https://api.spotify.com/v1/me")
                    .bearer_auth(access_token)
            })
            .await
            .map_err(|e| format!("Kullanıcı profili alınamadı: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Spotify kullanıcı bilgisi alınamadı ({}): {}",
                status, body
            ));
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| format!("Kullanıcı JSON ayrıştırılamadı: {}", e))?;

        let id = val
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let display_name = val
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let email = val
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let product = val
            .get("product")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let country = val
            .get("country")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let spotify_url = val
            .get("external_urls")
            .and_then(|u| u.get("spotify"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let avatar = val
            .get("images")
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|img| img.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(UserProfile {
            id,
            display_name,
            email,
            avatar,
            product,
            country,
            spotify_url,
        })
    }

    pub async fn get_user_playlists(
        &self,
        access_token: &str,
        my_id: &str,
    ) -> Result<Vec<PlaylistSummary>, String> {
        let mut playlists = Vec::new();
        let limit = 50;
        let mut offset = 0;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/playlists?limit={}&offset={}",
                limit, offset
            );

            let resp = self
                .execute_with_retry(|| self.client.get(&url).bearer_auth(access_token))
                .await
                .map_err(|e| format!("Çalma listeleri alınamadı: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!(
                    "Çalma listeleri yüklenemedi ({}): {}",
                    status, body
                ));
            }

            let val: Value = resp
                .json()
                .await
                .map_err(|e| format!("Çalma listesi JSON ayrıştırılamadı: {}", e))?;

            let items = match val.get("items").and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                None => break,
            };

            let count = items.len();

            for pl in items {
                let id = pl
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if id.is_empty() {
                    continue;
                }
                let name = pl
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("İsimsiz Liste")
                    .to_string();
                let description = pl
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let image_url = pl
                    .get("images")
                    .and_then(|imgs| imgs.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|img| img.get("url"))
                    .and_then(|u| u.as_str())
                    .unwrap_or_default()
                    .to_string();

                let total_tracks = pl
                    .get("tracks")
                    .and_then(|t| t.get("total"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;

                let owner_obj = pl.get("owner");
                let owner_id = owner_obj
                    .and_then(|o| o.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let owner_name = owner_obj
                    .and_then(|o| o.get("display_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&owner_id)
                    .to_string();

                let is_mine = !my_id.is_empty() && owner_id == my_id;
                let is_public = pl.get("public").and_then(|v| v.as_bool()).unwrap_or(false);
                let is_collaborative = pl
                    .get("collaborative")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let spotify_url = pl
                    .get("external_urls")
                    .and_then(|u| u.get("spotify"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                playlists.push(PlaylistSummary {
                    id,
                    name,
                    description,
                    image_url,
                    total_tracks,
                    owner: owner_name,
                    owner_id,
                    is_mine,
                    is_public,
                    is_collaborative,
                    spotify_url,
                });
            }

            if count < limit {
                break;
            }
            offset += limit;
        }

        // Sort: User's own playlists first, then alphabetical
        playlists.sort_by(|a, b| {
            b.is_mine
                .cmp(&a.is_mine)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(playlists)
    }

    #[allow(dead_code)]
    pub async fn get_playlist_details(
        &self,
        access_token: &str,
        playlist_id: &str,
        my_id: &str,
    ) -> Result<PlaylistSummary, String> {
        let url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);

        let resp = self
            .execute_with_retry(|| self.client.get(&url).bearer_auth(access_token))
            .await
            .map_err(|e| format!("Çalma listesi detayları alınamadı: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Çalma listesi detay hatası ({}): {}", status, body));
        }

        let pl: Value = resp
            .json()
            .await
            .map_err(|e| format!("Detay JSON ayrıştırılamadı: {}", e))?;

        let id = pl
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let name = pl
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let description = pl
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let image_url = pl
            .get("images")
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|img| img.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or_default()
            .to_string();

        let total_tracks = pl
            .get("tracks")
            .and_then(|t| t.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let owner_obj = pl.get("owner");
        let owner_id = owner_obj
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let owner_name = owner_obj
            .and_then(|o| o.get("display_name"))
            .and_then(|v| v.as_str())
            .unwrap_or(&owner_id)
            .to_string();

        let is_mine = !my_id.is_empty() && owner_id == my_id;
        let is_public = pl.get("public").and_then(|v| v.as_bool()).unwrap_or(false);
        let spotify_url = pl
            .get("external_urls")
            .and_then(|u| u.get("spotify"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(PlaylistSummary {
            id,
            name,
            description,
            image_url,
            total_tracks,
            owner: owner_name,
            owner_id,
            is_mine,
            is_public,
            is_collaborative: false,
            spotify_url,
        })
    }

    pub async fn get_all_playlist_tracks(
        &self,
        access_token: &str,
        playlist_id: &str,
    ) -> Result<Vec<RawTrackItem>, String> {
        let mut tracks = Vec::new();
        let limit = 100;
        let mut offset = 0;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/playlists/{}/tracks?limit={}&offset={}&additional_types=track",
                playlist_id, limit, offset
            );

            let resp = self
                .execute_with_retry(|| self.client.get(&url).bearer_auth(access_token))
                .await
                .map_err(|e| format!("Şarkılar alınamadı: {}", e))?;

            if resp.status().as_u16() == 403 {
                return Err(
                    "Spotify API kısıtlaması (Development Mode): Bu çalma listesi başka bir kullanıcıya ait olduğu için şarkıları okunamıyor. Lütfen kendi oluşturduğunuz bir çalma listesini seçin.".to_string()
                );
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Şarkı listesi yüklenemedi ({}): {}", status, body));
            }

            let val: Value = resp
                .json()
                .await
                .map_err(|e| format!("Şarkı JSON ayrıştırılamadı: {}", e))?;

            let items = match val.get("items").and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                None => break,
            };

            let count = items.len();

            for it in items {
                let track_val = it.get("item").or_else(|| it.get("track"));
                if let Some(tv) = track_val {
                    if let Ok(track) =
                        serde_json::from_value::<crate::classifier::RawTrack>(tv.clone())
                    {
                        if !track.id.is_empty() {
                            tracks.push(RawTrackItem {
                                track: Some(track),
                                added_at: it
                                    .get("added_at")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                            });
                        }
                    }
                }
            }

            let has_next = val.get("next").map(|n| !n.is_null()).unwrap_or(false);
            if !has_next || count < limit {
                break;
            }
            offset += limit;
        }

        Ok(tracks)
    }

    pub async fn batch_resolve_artists_genres(
        &self,
        artist_names_map: &HashMap<String, String>,
        cache: &ArtistCache,
        curated_fn: impl Fn(&str) -> Option<&'static [String]>,
    ) -> HashMap<String, Vec<String>> {
        let mut result_map = HashMap::new();
        let mut missing_to_fetch = Vec::new();

        for (aid, aname) in artist_names_map {
            // Check curated artists first (Tier 1)
            if let Some(curated) = curated_fn(aname) {
                result_map.insert(aid.clone(), curated.to_vec());
                continue;
            }

            // Check persistent disk cache (Tier 2)
            if let Some(cached) = cache.get(aname) {
                result_map.insert(aid.clone(), cached);
                continue;
            }

            missing_to_fetch.push((aid.clone(), aname.clone()));
        }

        if missing_to_fetch.is_empty() {
            return result_map;
        }

        // Fetch missing in parallel (Tier 3: Deezer fallback)
        let semaphore = Arc::new(tokio::sync::Semaphore::new(8));
        let client = self.client.clone();
        let mut tasks = Vec::new();

        for (aid, aname) in missing_to_fetch {
            let permit = semaphore.clone();
            let c = client.clone();
            tasks.push(tokio::spawn(async move {
                let _p = permit.acquire_owned().await;
                let genres = deezer::fetch_artist_genres(&c, &aname).await;
                (aid, aname, genres)
            }));
        }

        let mut resolved_to_cache = Vec::new();
        for task in tasks {
            if let Ok((aid, aname, genres)) = task.await {
                resolved_to_cache.push((aname, genres.clone()));
                result_map.insert(aid, genres);
            }
        }

        cache.insert_batch(resolved_to_cache);

        result_map
    }

    pub fn deduplicate_track_uris(track_uris: &[String]) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();
        for u in track_uris {
            let trimmed = u.trim();
            if trimmed.starts_with("spotify:track:") && seen.insert(trimmed.to_string()) {
                unique.push(trimmed.to_string());
            }
        }
        unique
    }

    pub async fn create_user_playlist(
        &self,
        access_token: &str,
        user_id: &str,
        name: &str,
        description: &str,
        is_public: bool,
        track_uris: &[String],
    ) -> Result<CreatedPlaylistResult, String> {
        let unique_uris = Self::deduplicate_track_uris(track_uris);
        if unique_uris.is_empty() {
            return Err("Eklenecek geçerli şarkı bulunamadı.".to_string());
        }

        let trimmed_user = user_id.trim();
        let create_url = if trimmed_user.is_empty() {
            "https://api.spotify.com/v1/me/playlists".to_string()
        } else {
            format!(
                "https://api.spotify.com/v1/users/{}/playlists",
                trimmed_user
            )
        };

        let body = serde_json::json!({
            "name": name,
            "public": is_public,
            "description": description,
        });

        let resp = self
            .execute_with_retry(|| {
                self.client
                    .post(&create_url)
                    .bearer_auth(access_token)
                    .json(&body)
            })
            .await
            .map_err(|e| format!("Çalma listesi oluşturma isteği başarısız: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Çalma listesi oluşturulamadı ({}): {}",
                status, err_body
            ));
        }

        let new_pl: Value = resp
            .json()
            .await
            .map_err(|e| format!("Oluşturulan liste yanıtı ayrıştırılamadı: {}", e))?;

        let playlist_id = new_pl
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Yeni çalma listesi kimliği (id) alınamadı.".to_string())?
            .to_string();

        let returned_name = new_pl
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string();

        let spotify_url = new_pl
            .get("external_urls")
            .and_then(|u| u.get("spotify"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        // Add tracks in chunks of 100
        let add_url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        for chunk in unique_uris.chunks(100) {
            let chunk_body = serde_json::json!({
                "uris": chunk
            });

            let add_resp = self
                .execute_with_retry(|| {
                    self.client
                        .post(&add_url)
                        .bearer_auth(access_token)
                        .json(&chunk_body)
                })
                .await
                .map_err(|e| format!("Şarkılar listeye eklenemedi: {}", e))?;

            if !add_resp.status().is_success() {
                let status = add_resp.status();
                let err_b = add_resp.text().await.unwrap_or_default();
                return Err(format!("Şarkı ekleme hatası ({}): {}", status, err_b));
            }
        }

        Ok(CreatedPlaylistResult {
            id: playlist_id,
            name: returned_name,
            description: description.to_string(),
            spotify_url,
            total_tracks: unique_uris.len(),
        })
    }
}

impl Default for SpotifyClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_track_uris_empty() {
        assert_eq!(
            SpotifyClient::deduplicate_track_uris(&[]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_deduplicate_track_uris_order_and_duplicates() {
        let uris = vec![
            "spotify:track:1".into(),
            "spotify:track:2".into(),
            "spotify:track:1".into(),
            "spotify:track:3".into(),
            "spotify:track:2".into(),
        ];
        let res = SpotifyClient::deduplicate_track_uris(&uris);
        assert_eq!(
            res,
            vec![
                "spotify:track:1".to_string(),
                "spotify:track:2".to_string(),
                "spotify:track:3".to_string(),
            ]
        );
    }

    #[test]
    fn test_deduplicate_track_uris_filters_invalid() {
        let uris = vec![
            "spotify:track:valid1".into(),
            "".into(),
            "spotify:album:invalid1".into(),
            "spotify:track:valid2".into(),
            "not_a_spotify_uri".into(),
        ];
        let res = SpotifyClient::deduplicate_track_uris(&uris);
        assert_eq!(
            res,
            vec![
                "spotify:track:valid1".to_string(),
                "spotify:track:valid2".to_string(),
            ]
        );
    }
}
