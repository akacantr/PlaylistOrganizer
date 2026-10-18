use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const SEED_CACHE_JSON: &str = include_str!("../data/artist_cache.json");

#[derive(Clone)]
pub struct ArtistCache {
    cache_path: PathBuf,
    entries: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ArtistCache {
    pub fn new(cache_path: PathBuf) -> Self {
        let mut map: HashMap<String, Vec<String>> =
            serde_json::from_str(SEED_CACHE_JSON).unwrap_or_default();

        if cache_path.exists() {
            if let Ok(content) = fs::read_to_string(&cache_path) {
                if let Ok(loaded) = serde_json::from_str::<HashMap<String, Vec<String>>>(&content) {
                    for (k, v) in loaded {
                        map.insert(k, v);
                    }
                }
            }
        }

        Self {
            cache_path,
            entries: Arc::new(RwLock::new(map)),
        }
    }

    pub fn get(&self, artist_name: &str) -> Option<Vec<String>> {
        let key = artist_name.to_lowercase().trim().to_string();
        self.entries.read().ok()?.get(&key).cloned()
    }

    pub fn insert(&self, artist_name: &str, genres: Vec<String>) {
        let key = artist_name.to_lowercase().trim().to_string();
        if let Ok(mut lock) = self.entries.write() {
            lock.insert(key, genres);
        }
        self.save();
    }

    pub fn insert_batch(&self, items: Vec<(String, Vec<String>)>) {
        if items.is_empty() {
            return;
        }
        if let Ok(mut lock) = self.entries.write() {
            for (artist_name, genres) in items {
                let key = artist_name.to_lowercase().trim().to_string();
                lock.insert(key, genres);
            }
        }
        self.save();
    }

    pub fn save(&self) {
        if let Ok(lock) = self.entries.read() {
            if let Ok(json_str) = serde_json::to_string_pretty(&*lock) {
                if let Some(parent) = self.cache_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let temp_path = self
                    .cache_path
                    .with_extension(format!("tmp.{}", std::process::id()));
                if fs::write(&temp_path, json_str.as_bytes()).is_ok() {
                    let _ = fs::rename(&temp_path, &self.cache_path);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.read().map(|m| m.len()).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_cache_loaded() {
        let tmp_path = std::env::temp_dir().join("test_artist_cache.json");
        let cache = ArtistCache::new(tmp_path.clone());
        assert!(cache.len() > 100);

        // Test insertion and retrieval
        cache.insert("Test New Artist", vec!["indie rock".to_string()]);
        let genres = cache.get("test new artist");
        assert_eq!(genres, Some(vec!["indie rock".to_string()]));

        let _ = fs::remove_file(tmp_path);
    }
}
