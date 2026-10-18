use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTrack {
    pub index: usize,
    pub id: String,
    pub uri: String,
    pub name: String,
    pub artists: Vec<String>,
    pub artist_ids: Vec<String>,
    pub album: String,
    pub image_url: String,
    pub duration_ms: u64,
    pub preview_url: Option<String>,
    pub popularity: u32,
    pub spotify_url: String,
    pub raw_genres: Vec<String>,
    pub macro_genres: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreStat {
    pub name: String,
    pub count: usize,
    pub percentage: f32,
    pub track_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistAnalysis {
    pub total_tracks: usize,
    pub unique_artists: usize,
    pub macro_genres: Vec<GenreStat>,
    pub micro_genres: Vec<GenreStat>,
    pub tracks: Vec<EnrichedTrack>,
    pub categorized_count: usize,
    pub uncategorized_count: usize,
}

pub struct MacroRule {
    pub category: &'static str,
    pub patterns: &'static [&'static str],
}

pub const MACRO_RULES: &[MacroRule] = &[
    MacroRule {
        category: "Türkçe Arabesk & Damar",
        patterns: &[
            r"\barabesk\b",
            r"\barabesque\b",
            r"\bdamar\b",
            r"\bfantezi\b",
            r"\btaverna\b",
            r"\balaturka\b",
            r"\bfas[ıi]l\b",
        ],
    },
    MacroRule {
        category: "Türkçe Rap / Hip-Hop",
        patterns: &[
            r"\bturkish hip hop\b",
            r"\bturkish rap\b",
            r"\bturkish trap\b",
            r"\bturkish drill\b",
            r"\bturkce rap\b",
            r"\btrap turkce\b",
            r"\banatolian hip hop\b",
            r"\btürkçe rap\b",
            r"\bturkish hip-hop\b",
        ],
    },
    MacroRule {
        category: "Türkçe Rock / Anadolu Rock",
        patterns: &[
            r"\banadolu rock\b",
            r"\bturkish rock\b",
            r"\bturkish alternative rock\b",
            r"\bturkish indie rock\b",
            r"\bturkish punk\b",
            r"\bturkish metal\b",
            r"\banatolian rock\b",
            r"\btürkçe rock\b",
        ],
    },
    MacroRule {
        category: "Türkçe Pop",
        patterns: &[
            r"\bturkish pop\b",
            r"\bturk pop\b",
            r"\bturkish alt pop\b",
            r"\bturkish singer-songwriter\b",
            r"\bturkish dance pop\b",
            r"\btürkçe pop\b",
        ],
    },
    MacroRule {
        category: "Türkçe Halk & Sanat & Diğer",
        patterns: &[
            r"\bturkish\b",
            r"\btürk\b",
            r"\banadolu\b",
            r"\bhalk m[üu]zi[ğg]i\b",
            r"\btürk[üu]\b",
            r"\bturku\b",
            r"\bsanat m[üu]zi[ğg]i\b",
            r"\bkaradeniz\b",
            r"\bözgün\b",
            r"\bozgun\b",
            r"\btürküler\b",
            r"\bturkish alternative\b",
        ],
    },
    MacroRule {
        category: "Phonk / Drift & Bass",
        patterns: &[
            r"\bphonk\b",
            r"\bdrift\b",
            r"\bmontagem\b",
            r"\bautomotivo\b",
            r"\bbrazilian funk\b",
            r"\bfunk brasileiro\b",
            r"\bhardstyle\b",
            r"\bzyzz\b",
            r"\bkrushclub\b",
            r"\bjersey club\b",
            r"\bwave\b",
            r"\bsped up\b",
            r"\bslowed\b",
            r"\bnightcore\b",
            r"\bbass house\b",
            r"\bgym phonk\b",
            r"\bdrift phonk\b",
            r"\bbrazilian phonk\b",
            r"\bkrush\b",
        ],
    },
    MacroRule {
        category: "Metal",
        patterns: &[
            r"\bmetal\b",
            r"\bmetalcore\b",
            r"\bdeathcore\b",
            r"\bdeath metal\b",
            r"\bblack metal\b",
            r"\bthrash\b",
            r"\bdoom metal\b",
            r"\bpower metal\b",
            r"\bheavy metal\b",
            r"\bnu metal\b",
            r"\bgrindcore\b",
            r"\bslam\b",
            r"\bsymphonic metal\b",
        ],
    },
    MacroRule {
        category: "Rock",
        patterns: &[
            r"\brock\b",
            r"\bgrunge\b",
            r"\bpunk\b",
            r"\bpost-punk\b",
            r"\bhard rock\b",
            r"\bclassic rock\b",
            r"\bprogressive rock\b",
            r"\bgarage rock\b",
            r"\bpsychedelic rock\b",
            r"\bemo\b",
            r"\bpop punk\b",
            r"\bglam rock\b",
            r"\bstoner rock\b",
            r"\brock & roll\b",
        ],
    },
    MacroRule {
        category: "Hip-Hop & Rap",
        patterns: &[
            r"\bhip hop\b",
            r"\bhip-hop\b",
            r"\bhip-hop/rap\b",
            r"\brap\b",
            r"\btrap\b",
            r"\bdrill\b",
            r"\bboom bap\b",
            r"\bgrime\b",
            r"\bgangsta rap\b",
            r"\bconscious hip hop\b",
            r"\beast coast\b",
            r"\bwest coast\b",
            r"\bsouthern hip hop\b",
            r"\bmelodic rap\b",
            r"\bpluggnb\b",
            r"\br&b/hip-hop\b",
        ],
    },
    MacroRule {
        category: "Elektronik / Dance",
        patterns: &[
            r"\bdance\b",
            r"\bdans\b",
            r"\bclub\b",
            r"\belectronic\b",
            r"\belektronik\b",
            r"\belectro\b",
            r"\bhouse\b",
            r"\btechno\b",
            r"\bedm\b",
            r"\btrance\b",
            r"\bdubstep\b",
            r"\bdrum and bass\b",
            r"\bdnb\b",
            r"\bambient\b",
            r"\bsynthwave\b",
            r"\bchillout\b",
            r"\bdisco\b",
            r"\bfuture bass\b",
            r"\bidm\b",
            r"\belectronica\b",
            r"\bdeep house\b",
            r"\btech house\b",
            r"\bgarage\b",
            r"\bdance-pop\b",
            r"\belectropop\b",
            r"\beurodance\b",
            r"\bbounce\b",
            r"\btechno/house\b",
            r"\bslap house\b",
        ],
    },
    MacroRule {
        category: "R&B & Soul",
        patterns: &[
            r"\br&b\b",
            r"\br&b/soul\b",
            r"\bsoul\b",
            r"\bfunk\b",
            r"\bneo soul\b",
            r"\bmotown\b",
            r"\bcontemporary r&b\b",
            r"\bquiet storm\b",
            r"\bafrobeats\b",
            r"\bafropop\b",
            r"\bsoul & funk\b",
        ],
    },
    MacroRule {
        category: "Indie & Alternatif",
        patterns: &[
            r"\balternative\b",
            r"\balternatif\b",
            r"\bindie\b",
            r"\bshoegaze\b",
            r"\bdream pop\b",
            r"\blo-fi\b",
            r"\blofi\b",
            r"\bchamber pop\b",
            r"\bmath rock\b",
            r"\bmidwest emo\b",
            r"\bart pop\b",
            r"\bbedroom pop\b",
            r"\balt-rock\b",
            r"\bindie rock\b",
            r"\bindie pop\b",
        ],
    },
    MacroRule {
        category: "Pop",
        patterns: &[
            r"\bpop\b",
            r"\bk-pop\b",
            r"\bj-pop\b",
            r"\beuropop\b",
            r"\bteen pop\b",
            r"\bpost-teen pop\b",
            r"\bmainstream\b",
            r"\bcommercial\b",
            r"\basya m[üu]zi[ğg]i\b",
            r"\byeniler\b",
        ],
    },
    MacroRule {
        category: "Akustik & Folk",
        patterns: &[
            r"\bfolk\b",
            r"\bacoustic\b",
            r"\bakustik\b",
            r"\bcountry\b",
            r"\bbluegrass\b",
            r"\bsinger-songwriter\b",
            r"\bamericana\b",
            r"\bindie folk\b",
            r"\bsinger & songwriter\b",
        ],
    },
    MacroRule {
        category: "Jazz & Blues",
        patterns: &[
            r"\bjazz\b",
            r"\bcaz\b",
            r"\bblues\b",
            r"\bbossa nova\b",
            r"\bbebop\b",
            r"\bswing\b",
            r"\bfusion\b",
            r"\bcool jazz\b",
            r"\bhard bop\b",
        ],
    },
    MacroRule {
        category: "Klasik & Soundtrack",
        patterns: &[
            r"\bclassical\b",
            r"\bklasik\b",
            r"\borchestral\b",
            r"\bsoundtrack\b",
            r"\bscore\b",
            r"\binstrumental\b",
            r"\benstr[üu]mantal\b",
            r"\bpiano\b",
            r"\bpiyano\b",
            r"\bbaroque\b",
            r"\bcinematic\b",
            r"\bvideo game music\b",
            r"\bfilm\b",
            r"\bfilm/oyun\b",
            r"\bfilm m[üu]zikleri\b",
            r"\boyun m[üu]zikleri\b",
            r"\bfilm/oyun m[üu]zi[ğg]i\b",
        ],
    },
    MacroRule {
        category: "Latin & Reggae",
        patterns: &[
            r"\blatin\b",
            r"\blatin m[üu]zik\b",
            r"\breggaeton\b",
            r"\bsalsa\b",
            r"\bbachata\b",
            r"\breggae\b",
            r"\bdub\b",
            r"\bdancehall\b",
            r"\bcumbia\b",
            r"\bbrezilya m[üu]zi[ğg]i\b",
        ],
    },
];

struct CompiledMacroRule {
    category: &'static str,
    regexes: Vec<Regex>,
}

static COMPILED_RULES: OnceLock<Vec<CompiledMacroRule>> = OnceLock::new();

fn get_compiled_rules() -> &'static [CompiledMacroRule] {
    COMPILED_RULES.get_or_init(|| {
        MACRO_RULES
            .iter()
            .map(|rule| {
                let regexes = rule
                    .patterns
                    .iter()
                    .filter_map(|p| Regex::new(&format!("(?i){}", p)).ok())
                    .collect();
                CompiledMacroRule {
                    category: rule.category,
                    regexes,
                }
            })
            .collect()
    })
}

struct CompiledInferenceRule {
    category: &'static str,
    regex: Regex,
}

static INFERENCE_RULES: OnceLock<Vec<CompiledInferenceRule>> = OnceLock::new();

fn get_inference_rules() -> &'static [CompiledInferenceRule] {
    INFERENCE_RULES.get_or_init(|| {
        let patterns: &[(&'static str, &'static str)] = &[
            ("Phonk / Drift & Bass", r"(?i)\b(phonk|drift|montagem|automotivo|hardstyle|sped up|slowed|nightcore|zyzz|brazilian funk|krush)\b"),
            ("Elektronik / Dance", r"(?i)\b(remix|club mix|extended mix|vip mix|edm|techno|house|trance|dubstep|rave|synthwave|bass|slap house|radio edit)\b"),
            ("Türkçe Arabesk & Damar", r"(?i)\b(arabesk|arabesque|damar|fantezi|taverna)\b"),
            ("Türkçe Rap / Hip-Hop", r"(?i)\b(türkçe rap|turkce rap|türkçe trap|turkce trap|türkçe drill|turkce drill|türkçe hip hop|turkce hip hop|anatolian rap|anatolian hip hop)\b"),
            ("Hip-Hop & Rap", r"(?i)\b(rap|hip hop|hip-hop|trap|drill|boom bap|beat|diss|freestyle|cypher)\b"),
            ("Akustik & Folk", r"(?i)\b(akustik|acoustic|unplugged|folk|live)\b"),
            ("Klasik & Soundtrack", r"(?i)\b(soundtrack|theme|ost|score|instrumental|enstr[üu]mantal|piyano|piano|orchestra|senfoni)\b"),
            ("Rock", r"(?i)\b(rock|metal|punk|grunge)\b"),
        ];

        patterns
            .iter()
            .filter_map(|(cat, pat)| {
                Regex::new(pat).ok().map(|r| CompiledInferenceRule {
                    category: cat,
                    regex: r,
                })
            })
            .collect()
    })
}

pub fn map_raw_genres_to_macro(raw_genres: &[String]) -> Vec<String> {
    if raw_genres.is_empty() {
        return Vec::new();
    }

    let mut matched_categories: HashSet<&'static str> = HashSet::new();
    let compiled = get_compiled_rules();

    for genre_str in raw_genres {
        let clean = genre_str.to_lowercase().trim().to_string();
        for rule in compiled {
            if rule.regexes.iter().any(|r| r.is_match(&clean)) {
                matched_categories.insert(rule.category);
            }
        }
    }

    let turkish_specific = [
        "Türkçe Arabesk & Damar",
        "Türkçe Rap / Hip-Hop",
        "Türkçe Rock / Anadolu Rock",
        "Türkçe Pop",
    ];

    let has_turkish_specific = turkish_specific
        .iter()
        .any(|cat| matched_categories.contains(cat));
    if has_turkish_specific {
        matched_categories.remove("Türkçe Halk & Sanat & Diğer");
    }

    if matched_categories.is_empty() {
        return Vec::new();
    }

    // Preserve original macro rule order
    let mut result = Vec::new();
    for rule in MACRO_RULES {
        if matched_categories.contains(rule.category) {
            result.push(rule.category.to_string());
        }
    }
    result
}

pub fn infer_genres_from_track_metadata(
    track_name: &str,
    artist_names: &[String],
    album_name: &str,
) -> Vec<String> {
    let combined =
        format!("{} {} {}", track_name, artist_names.join(" "), album_name).to_lowercase();
    let rules = get_inference_rules();

    for rule in rules {
        if rule.regex.is_match(&combined) {
            return vec![rule.category.to_string()];
        }
    }

    Vec::new()
}

pub fn analyze_playlist_tracks(
    raw_tracks: &[RawTrackItem],
    artists_map: &HashMap<String, Vec<String>>,
    curated_lookup: impl Fn(&str) -> Option<&'static [String]>,
) -> PlaylistAnalysis {
    let mut enriched_tracks = Vec::new();
    let mut macro_genre_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut micro_genre_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut unique_artists_set: HashSet<String> = HashSet::new();

    for (idx, item) in raw_tracks.iter().enumerate() {
        let track = match &item.track {
            Some(t) => t,
            None => continue,
        };

        if track.id.is_empty() {
            continue;
        }

        let track_id = track.id.clone();
        let track_name = if track.name.is_empty() {
            "İsimsiz Şarkı".to_string()
        } else {
            track.name.clone()
        };
        let track_uri = track.uri.clone();
        let duration_ms = track.duration_ms;
        let preview_url = track.preview_url.clone();
        let popularity = track.popularity;

        let artist_names: Vec<String> = track
            .artists
            .iter()
            .filter(|a| !a.name.is_empty())
            .map(|a| a.name.clone())
            .collect();
        let artist_ids: Vec<String> = track
            .artists
            .iter()
            .filter(|a| !a.id.is_empty())
            .map(|a| a.id.clone())
            .collect();

        for aid in &artist_ids {
            unique_artists_set.insert(aid.clone());
        }

        let album_name = track
            .album
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        let image_url = track
            .album
            .as_ref()
            .and_then(|a| a.images.first().map(|img| img.url.clone()))
            .unwrap_or_default();
        let spotify_url = track
            .external_urls
            .as_ref()
            .and_then(|urls| urls.spotify.clone())
            .unwrap_or_default();

        let mut track_raw_genres: HashSet<String> = HashSet::new();

        // 1. From artist map (Deezer/Spotify cache)
        for aid in &artist_ids {
            if let Some(genres) = artists_map.get(aid) {
                for g in genres {
                    track_raw_genres.insert(g.trim().to_string());
                }
            }
        }

        // 2. From curated artists
        for a_name in &artist_names {
            if let Some(genres) = curated_lookup(a_name) {
                for g in genres {
                    track_raw_genres.insert(g.trim().to_string());
                }
            }
        }

        let mut raw_genres_list: Vec<String> = track_raw_genres.into_iter().collect();
        raw_genres_list.sort();

        let mut macro_genres = map_raw_genres_to_macro(&raw_genres_list);

        if macro_genres.is_empty() {
            let inferred =
                infer_genres_from_track_metadata(&track_name, &artist_names, &album_name);
            if !inferred.is_empty() {
                macro_genres = inferred.clone();
                raw_genres_list = inferred.into_iter().map(|g| g.to_lowercase()).collect();
            } else {
                macro_genres = vec!["Diğer / Belirsiz".to_string()];
            }
        }

        let display_raw_genres = if raw_genres_list.is_empty() {
            vec!["Etiketsiz".to_string()]
        } else {
            raw_genres_list.clone()
        };

        for mg in &macro_genres {
            macro_genre_counts
                .entry(mg.clone())
                .or_default()
                .push(track_id.clone());
        }

        if !raw_genres_list.is_empty() {
            for rg in &raw_genres_list {
                let rg_clean = rg.to_lowercase().trim().to_string();
                micro_genre_counts
                    .entry(rg_clean)
                    .or_default()
                    .push(track_id.clone());
            }
        } else {
            micro_genre_counts
                .entry("etiketsiz".to_string())
                .or_default()
                .push(track_id.clone());
        }

        enriched_tracks.push(EnrichedTrack {
            index: idx + 1,
            id: track_id,
            uri: track_uri,
            name: track_name,
            artists: artist_names,
            artist_ids,
            album: album_name,
            image_url,
            duration_ms,
            preview_url,
            popularity,
            spotify_url,
            raw_genres: display_raw_genres,
            macro_genres,
        });
    }

    let total_valid = enriched_tracks.len();

    let mut macro_stats: Vec<GenreStat> = macro_genre_counts
        .into_iter()
        .map(|(name, t_ids)| {
            let count = t_ids.len();
            let pct = if total_valid > 0 {
                ((count as f32 / total_valid as f32) * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };
            GenreStat {
                name,
                count,
                percentage: pct,
                track_ids: t_ids,
            }
        })
        .collect();
    macro_stats.sort_by_key(|a| std::cmp::Reverse(a.count));

    let mut micro_stats: Vec<GenreStat> = micro_genre_counts
        .into_iter()
        .map(|(name, t_ids)| {
            let count = t_ids.len();
            let pct = if total_valid > 0 {
                ((count as f32 / total_valid as f32) * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };
            GenreStat {
                name,
                count,
                percentage: pct,
                track_ids: t_ids,
            }
        })
        .collect();
    micro_stats.sort_by_key(|a| std::cmp::Reverse(a.count));

    let uncategorized_count = macro_stats
        .iter()
        .find(|s| s.name == "Diğer / Belirsiz")
        .map(|s| s.count)
        .unwrap_or(0);

    PlaylistAnalysis {
        total_tracks: total_valid,
        unique_artists: unique_artists_set.len(),
        macro_genres: macro_stats,
        micro_genres: micro_stats,
        tracks: enriched_tracks,
        categorized_count: total_valid.saturating_sub(uncategorized_count),
        uncategorized_count,
    }
}

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

// Models representing raw Spotify API responses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTrackItem {
    pub track: Option<RawTrack>,
    pub added_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTrack {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub uri: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub duration_ms: u64,
    pub preview_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub popularity: u32,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub artists: Vec<RawArtistSummary>,
    pub album: Option<RawAlbum>,
    pub external_urls: Option<RawExternalUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawArtistSummary {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawAlbum {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub images: Vec<RawImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawImage {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawExternalUrls {
    pub spotify: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_raw_genres() {
        assert_eq!(map_raw_genres_to_macro(&[]), Vec::<String>::new());
    }

    #[test]
    fn test_specific_turkish_pop_discards_generic_turkish() {
        let raw = vec!["turkish pop".to_string(), "turkish".to_string()];
        let res = map_raw_genres_to_macro(&raw);
        assert!(res.contains(&"Türkçe Pop".to_string()));
        assert!(!res.contains(&"Türkçe Halk & Sanat & Diğer".to_string()));
    }

    #[test]
    fn test_specific_turkish_rap_discards_generic_turkish() {
        let raw = vec![
            "turkish hip hop".to_string(),
            "turkish rap".to_string(),
            "turkish".to_string(),
        ];
        let res = map_raw_genres_to_macro(&raw);
        assert!(res.contains(&"Türkçe Rap / Hip-Hop".to_string()));
        assert!(!res.contains(&"Türkçe Halk & Sanat & Diğer".to_string()));
    }

    #[test]
    fn test_generic_turkish_kept_when_no_specific_match() {
        let raw = vec!["halk müziği".to_string(), "anadolu".to_string()];
        let res = map_raw_genres_to_macro(&raw);
        assert_eq!(res, vec!["Türkçe Halk & Sanat & Diğer".to_string()]);
    }

    #[test]
    fn test_global_metal_and_rock() {
        let raw = vec!["death metal".to_string(), "hard rock".to_string()];
        let res = map_raw_genres_to_macro(&raw);
        assert!(res.contains(&"Metal".to_string()));
        assert!(res.contains(&"Rock".to_string()));
    }

    #[test]
    fn test_phonk_and_electronic() {
        let raw = vec!["drift phonk".to_string(), "synthwave".to_string()];
        let res = map_raw_genres_to_macro(&raw);
        assert!(res.contains(&"Phonk / Drift & Bass".to_string()));
        assert!(res.contains(&"Elektronik / Dance".to_string()));
    }

    #[test]
    fn test_international_rap_not_classified_as_turkish() {
        let r1 = infer_genres_from_track_metadata(
            "SICKO MODE (Freestyle)",
            &["Travis Scott".into()],
            "",
        );
        assert_eq!(r1, vec!["Hip-Hop & Rap".to_string()]);

        let r2 = infer_genres_from_track_metadata("Drill Beat", &["Central Cee".into()], "");
        assert_eq!(r2, vec!["Hip-Hop & Rap".to_string()]);

        let r3 = infer_genres_from_track_metadata("Trap God", &["Gucci Mane".into()], "");
        assert_eq!(r3, vec!["Hip-Hop & Rap".to_string()]);

        let r4 = infer_genres_from_track_metadata("Rap God", &["Eminem".into()], "");
        assert_eq!(r4, vec!["Hip-Hop & Rap".to_string()]);
    }

    #[test]
    fn test_explicit_turkish_rap_cues() {
        let r1 = infer_genres_from_track_metadata(
            "Türkçe Rap Cypher",
            &["Çeşitli Sanatçılar".into()],
            "",
        );
        assert_eq!(r1, vec!["Türkçe Rap / Hip-Hop".to_string()]);

        let r2 = infer_genres_from_track_metadata("Turkce Trap Beat", &["Sanatçı".into()], "");
        assert_eq!(r2, vec!["Türkçe Rap / Hip-Hop".to_string()]);
    }

    #[test]
    fn test_phonk_inference() {
        let r = infer_genres_from_track_metadata(
            "Automotivo Montagem Slowed",
            &["DJ Phonk".into()],
            "",
        );
        assert_eq!(r, vec!["Phonk / Drift & Bass".to_string()]);
    }

    #[test]
    fn test_electronic_inference() {
        let r = infer_genres_from_track_metadata("Midnight City (Club Mix)", &["M83".into()], "");
        assert_eq!(r, vec!["Elektronik / Dance".to_string()]);
    }

    #[test]
    fn test_classical_soundtrack_inference() {
        let r = infer_genres_from_track_metadata(
            "Main Theme OST Piano Instrumental",
            &["Composer".into()],
            "",
        );
        assert_eq!(r, vec!["Klasik & Soundtrack".to_string()]);
    }

    #[test]
    fn test_unmatched_track() {
        let r = infer_genres_from_track_metadata("Random Song Title", &["Some Band".into()], "");
        assert!(r.is_empty());
    }
}
