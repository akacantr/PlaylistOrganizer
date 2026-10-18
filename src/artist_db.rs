use std::collections::HashMap;
use std::sync::OnceLock;

const CURATED_JSON: &str = include_str!("../data/curated_artists.json");

static ARTIST_MAP: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

fn get_map() -> &'static HashMap<String, Vec<String>> {
    ARTIST_MAP.get_or_init(|| serde_json::from_str(CURATED_JSON).unwrap_or_default())
}

pub fn get_curated_genres(artist_name: &str) -> Option<&'static [String]> {
    let clean = artist_name.to_lowercase().trim().to_string();
    get_map().get(&clean).map(|v| v.as_slice())
}

#[allow(dead_code)]
pub fn total_curated_count() -> usize {
    get_map().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curated_artists_loaded() {
        assert!(total_curated_count() >= 400);
        let ezhel = get_curated_genres("Ezhel");
        assert!(ezhel.is_some());
        let genres = ezhel.unwrap();
        assert!(
            genres.contains(&"turkish rap".to_string())
                || genres.contains(&"turkish hip hop".to_string())
        );

        let muslum = get_curated_genres("Müslüm Gürses");
        assert!(muslum.is_some());
        assert!(muslum.unwrap().contains(&"arabesk".to_string()));

        let unknown = get_curated_genres("NonExistentRandomArtistXYZ");
        assert!(unknown.is_none());
    }
}
