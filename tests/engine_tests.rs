use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use playlist_organizer::artist_db;
use playlist_organizer::cache::ArtistCache;
use playlist_organizer::classifier::{
    analyze_playlist_tracks, infer_genres_from_track_metadata, map_raw_genres_to_macro, RawAlbum,
    RawArtistSummary, RawExternalUrls, RawTrack, RawTrackItem,
};
use playlist_organizer::config::{self, AppConfig};
use playlist_organizer::spotify::auth::SpotifyToken;
use playlist_organizer::spotify::{SpotifyAuth, SpotifyClient};

#[test]
fn test_tier1_curated_artists() {
    let ezhel = artist_db::get_curated_genres("ezhel").expect("Ezhel should be in curated DB");
    assert!(ezhel.contains(&"turkish hip hop".to_string()));

    let tarkan = artist_db::get_curated_genres("tarkan");
    assert!(tarkan.is_some());

    let non_existent = artist_db::get_curated_genres("unknown_artist_xyz");
    assert!(non_existent.is_none());
}

#[test]
fn test_tier2_artist_cache() {
    let tmp_file = std::env::temp_dir().join("test_tier2_cache.json");
    let cache = ArtistCache::new(tmp_file.clone());

    cache.insert("TestBand", vec!["alternative rock".into(), "grunge".into()]);
    let retrieved = cache.get("testband");
    assert!(retrieved.is_some());
    let list = retrieved.unwrap();
    assert_eq!(
        list,
        vec!["alternative rock".to_string(), "grunge".to_string()]
    );

    // Reload from file to ensure persistence
    let reloaded = ArtistCache::new(tmp_file.clone());
    let reloaded_list = reloaded.get("testband").expect("Should persist to disk");
    assert_eq!(
        reloaded_list,
        vec!["alternative rock".to_string(), "grunge".to_string()]
    );

    let _ = fs::remove_file(tmp_file);
}

#[test]
fn test_tier4_macro_rules_turkish_override() {
    // When specific Turkish genre is present, generic Turkish category is discarded
    let raw = vec![
        "turkish rap".to_string(),
        "turkish hip hop".to_string(),
        "turkish".to_string(),
        "anadolu".to_string(),
    ];
    let macro_genres = map_raw_genres_to_macro(&raw);
    assert!(macro_genres.contains(&"Türkçe Rap / Hip-Hop".to_string()));
    assert!(!macro_genres.contains(&"Türkçe Halk & Sanat & Diğer".to_string()));
}

#[test]
fn test_tier4_macro_rules_generic_turkish_preserved_when_no_specific() {
    let raw = vec!["halk müziği".to_string(), "anadolu".to_string()];
    let macro_genres = map_raw_genres_to_macro(&raw);
    assert_eq!(
        macro_genres,
        vec!["Türkçe Halk & Sanat & Diğer".to_string()]
    );
}

#[test]
fn test_tier4_heuristic_inference() {
    let r1 = infer_genres_from_track_metadata(
        "Sicko Mode (Freestyle)",
        &["Travis Scott".into()],
        "Astroworld",
    );
    assert_eq!(r1, vec!["Hip-Hop & Rap".to_string()]);

    let r2 = infer_genres_from_track_metadata("Tokyo Drift Phonk", &["Kordhell".into()], "");
    assert_eq!(r2, vec!["Phonk / Drift & Bass".to_string()]);

    let r3 = infer_genres_from_track_metadata("Midnight Club Extended Mix", &["Avicii".into()], "");
    assert_eq!(r3, vec!["Elektronik / Dance".to_string()]);

    let r4 = infer_genres_from_track_metadata(
        "Piano Concerto No. 5 Soundtrack OST",
        &["Beethoven".into()],
        "",
    );
    assert_eq!(r4, vec!["Klasik & Soundtrack".to_string()]);
}

#[test]
fn test_full_pipeline_analysis() {
    let raw_tracks = vec![
        RawTrackItem {
            track: Some(RawTrack {
                id: "track_1".into(),
                name: "Geceler".into(),
                uri: "spotify:track:track_1".into(),
                duration_ms: 180000,
                preview_url: Some("https://p.scdn.co/mp3-preview/test1.mp3".into()),
                popularity: 80,
                artists: vec![RawArtistSummary {
                    id: "artist_ezhel".into(),
                    name: "Ezhel".into(),
                }],
                album: Some(RawAlbum {
                    name: "Müptezhel".into(),
                    images: vec![],
                }),
                external_urls: Some(RawExternalUrls {
                    spotify: Some("https://open.spotify.com/track/track_1".into()),
                }),
            }),
            added_at: None,
        },
        RawTrackItem {
            track: Some(RawTrack {
                id: "track_2".into(),
                name: "Unknown Beat Sped Up".into(),
                uri: "spotify:track:track_2".into(),
                duration_ms: 120000,
                preview_url: None,
                popularity: 50,
                artists: vec![RawArtistSummary {
                    id: "artist_unknown".into(),
                    name: "Underground Producer".into(),
                }],
                album: None,
                external_urls: None,
            }),
            added_at: None,
        },
    ];

    let artists_map = HashMap::new();
    let analysis =
        analyze_playlist_tracks(&raw_tracks, &artists_map, artist_db::get_curated_genres);

    assert_eq!(analysis.total_tracks, 2);
    assert_eq!(analysis.unique_artists, 2);
    assert_eq!(analysis.categorized_count, 2);
    assert_eq!(analysis.uncategorized_count, 0);

    // Track 1 should be categorized as Turkish Rap from curated DB
    assert!(analysis.tracks[0]
        .macro_genres
        .contains(&"Türkçe Rap / Hip-Hop".to_string()));

    // Track 2 should be categorized as Phonk from heuristic inference rule "sped up"
    assert!(analysis.tracks[1]
        .macro_genres
        .contains(&"Phonk / Drift & Bass".to_string()));
}

#[test]
fn test_uri_deduplication() {
    let input = vec![
        "spotify:track:A".to_string(),
        "spotify:track:B".to_string(),
        "spotify:track:A".to_string(),
        "".to_string(),
        "spotify:artist:ignore".to_string(),
        "spotify:track:C".to_string(),
    ];
    let result = SpotifyClient::deduplicate_track_uris(&input);
    assert_eq!(
        result,
        vec![
            "spotify:track:A".to_string(),
            "spotify:track:B".to_string(),
            "spotify:track:C".to_string(),
        ]
    );
}

#[test]
fn test_config_credentials_validation() {
    let mut cfg = AppConfig::default();
    assert!(!config::has_valid_credentials(&cfg));

    cfg.client_id = "short".into();
    cfg.client_secret = "short".into();
    assert!(!config::has_valid_credentials(&cfg));

    cfg.client_id = "mock_test_client_id_placeholder".into();
    cfg.client_secret = "mock_test_client_secret_placeholder".into();
    assert!(config::has_valid_credentials(&cfg));
}

#[test]
fn test_spotify_token_expiration() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let expired_token = SpotifyToken {
        access_token: "test".into(),
        token_type: "Bearer".into(),
        expires_in: Some(3600),
        refresh_token: Some("refresh".into()),
        scope: None,
        expires_at: now - 10,
    };
    assert!(expired_token.is_expired());

    let valid_token = SpotifyToken {
        access_token: "test".into(),
        token_type: "Bearer".into(),
        expires_in: Some(3600),
        refresh_token: Some("refresh".into()),
        scope: None,
        expires_at: now + 3600,
    };
    assert!(!valid_token.is_expired());
}

#[test]
fn test_raw_track_null_and_missing_deserialization() {
    let json_data = r#"{
        "id": null,
        "name": null,
        "uri": null,
        "duration_ms": 12345,
        "preview_url": null,
        "artists": [
            {"id": null, "name": "Test Artist"}
        ],
        "album": {
            "name": null,
            "images": [
                {"url": null}
            ]
        }
    }"#;

    let track: Result<RawTrack, _> = serde_json::from_str(json_data);
    assert!(
        track.is_ok(),
        "Should parse null fields gracefully without error: {:?}",
        track.err()
    );
    let t = track.unwrap();
    assert_eq!(t.id, "");
    assert_eq!(t.name, "");
    assert_eq!(t.uri, "");
    assert_eq!(t.duration_ms, 12345);
    assert_eq!(t.artists.len(), 1);
    assert_eq!(t.artists[0].id, "");
    assert_eq!(t.artists[0].name, "Test Artist");
    assert!(t.album.is_some());
    assert_eq!(t.album.unwrap().images[0].url, "");
}

#[test]
fn test_artist_cache_batch_insert() {
    let tmp_file =
        std::env::temp_dir().join(format!("test_cache_batch_{}.json", std::process::id()));
    let cache = ArtistCache::new(tmp_file.clone());

    let batch = vec![
        ("Artist 1".to_string(), vec!["rock".to_string()]),
        (
            "Artist 2".to_string(),
            vec!["pop".to_string(), "dance".to_string()],
        ),
        ("Artist 3".to_string(), vec!["jazz".to_string()]),
    ];

    cache.insert_batch(batch);

    assert_eq!(cache.get("artist 1"), Some(vec!["rock".to_string()]));
    assert_eq!(
        cache.get("artist 2"),
        Some(vec!["pop".to_string(), "dance".to_string()])
    );
    assert_eq!(cache.get("artist 3"), Some(vec!["jazz".to_string()]));

    // Check persistence to disk
    let reloaded = ArtistCache::new(tmp_file.clone());
    assert_eq!(
        reloaded.get("artist 2"),
        Some(vec!["pop".to_string(), "dance".to_string()])
    );

    let _ = fs::remove_file(tmp_file);
}

#[test]
fn test_auth_cancellation() {
    let auth = SpotifyAuth::new();
    let rx = auth.listen_for_callback("http://127.0.0.1:8889/callback");
    assert!(rx.is_ok());

    let rx = rx.unwrap();
    // Explicitly cancel
    auth.cancel_login();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let res = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await;
        assert!(
            res.is_ok(),
            "Cancellation should resolve quickly without waiting 120s"
        );
        let inner = res.unwrap();
        assert!(inner.is_ok());
        let final_code = inner.unwrap();
        assert!(final_code.is_err());
        assert!(final_code.unwrap_err().contains("iptal"));
    });
}

#[test]
fn test_audio_player_state_transitions() {
    let mut player = playlist_organizer::audio::AudioPlayer::new();
    player.ensure_stream();

    assert!(!player.is_playing());
    assert!(!player.is_exhausted());

    // Test volume controls and clamping
    player.set_volume(0.5);
    assert!((player.volume() - 0.5).abs() < 1e-4);
    player.set_volume(1.5); // clamps to 1.0
    assert!((player.volume() - 1.0).abs() < 1e-4);
    player.set_volume(-0.5); // clamps to 0.0
    assert!((player.volume() - 0.0).abs() < 1e-4);

    // Replay when nothing loaded should not panic
    player.replay();
    assert!(!player.is_playing());
    assert!(!player.is_exhausted());

    player.stop();
    assert!(!player.is_playing());
    assert_eq!(player.current_url(), None);
}
