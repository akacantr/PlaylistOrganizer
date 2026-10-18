use serde_json::Value;
use std::time::Duration;

pub async fn fetch_artist_genres(client: &reqwest::Client, artist_name: &str) -> Vec<String> {
    if artist_name.trim().is_empty() {
        return Vec::new();
    }

    let timeout = Duration::from_millis(2500);

    let encoded_name =
        url::form_urlencoded::byte_serialize(artist_name.as_bytes()).collect::<String>();
    let search_url = format!("https://api.deezer.com/search?q={}&limit=1", encoded_name);

    if let Ok(resp) = client.get(&search_url).timeout(timeout).send().await {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<Value>().await {
                if let Some(items) = v.get("data").and_then(|d| d.as_array()) {
                    if let Some(first) = items.first() {
                        if let Some(album_id) = first
                            .get("album")
                            .and_then(|a| a.get("id"))
                            .and_then(|id| id.as_i64())
                        {
                            let alb_url = format!("https://api.deezer.com/album/{}", album_id);
                            if let Ok(alb_resp) = client.get(&alb_url).timeout(timeout).send().await
                            {
                                if alb_resp.status().is_success() {
                                    if let Ok(alb_v) = alb_resp.json::<Value>().await {
                                        let genres = extract_genres_from_album(&alb_v);
                                        if !genres.is_empty() {
                                            return genres;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Strategy 2: Search artist -> top tracks -> album -> genres
    let artist_search_url = format!(
        "https://api.deezer.com/search/artist?q={}&limit=1",
        encoded_name
    );

    if let Ok(resp) = client.get(&artist_search_url).timeout(timeout).send().await {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<Value>().await {
                if let Some(items) = v.get("data").and_then(|d| d.as_array()) {
                    if let Some(first) = items.first() {
                        if let Some(art_id) = first.get("id").and_then(|id| id.as_i64()) {
                            let top_url =
                                format!("https://api.deezer.com/artist/{}/top?limit=1", art_id);
                            if let Ok(top_resp) = client.get(&top_url).timeout(timeout).send().await
                            {
                                if top_resp.status().is_success() {
                                    if let Ok(top_v) = top_resp.json::<Value>().await {
                                        if let Some(top_data) =
                                            top_v.get("data").and_then(|d| d.as_array())
                                        {
                                            if let Some(top_item) = top_data.first() {
                                                if let Some(alb_id) = top_item
                                                    .get("album")
                                                    .and_then(|a| a.get("id"))
                                                    .and_then(|id| id.as_i64())
                                                {
                                                    let alb_url = format!(
                                                        "https://api.deezer.com/album/{}",
                                                        alb_id
                                                    );
                                                    if let Ok(alb_resp) = client
                                                        .get(&alb_url)
                                                        .timeout(timeout)
                                                        .send()
                                                        .await
                                                    {
                                                        if alb_resp.status().is_success() {
                                                            if let Ok(alb_v) =
                                                                alb_resp.json::<Value>().await
                                                            {
                                                                let genres =
                                                                    extract_genres_from_album(
                                                                        &alb_v,
                                                                    );
                                                                if !genres.is_empty() {
                                                                    return genres;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

fn extract_genres_from_album(album_json: &Value) -> Vec<String> {
    let mut genres = Vec::new();
    if let Some(genres_arr) = album_json
        .get("genres")
        .and_then(|g| g.get("data"))
        .and_then(|d| d.as_array())
    {
        for g_item in genres_arr {
            if let Some(name) = g_item.get("name").and_then(|n| n.as_str()) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    genres.push(trimmed.to_string());
                }
            }
        }
    }
    genres
}
