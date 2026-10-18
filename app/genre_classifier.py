import re
from typing import List, Dict, Any, Set
from app.artist_db import CURATED_ARTISTS

MACRO_RULES = [
    ("Türkçe Arabesk & Damar", [
        r"\barabesk\b", r"\barabesque\b", r"\bdamar\b", r"\bfantezi\b", r"\btaverna\b",
        r"\balaturka\b", r"\bfas[ıi]l\b"
    ]),

    ("Türkçe Rap / Hip-Hop", [
        r"\bturkish hip hop\b", r"\bturkish rap\b", r"\bturkish trap\b", r"\bturkish drill\b",
        r"\bturkce rap\b", r"\btrap turkce\b", r"\banatolian hip hop\b", r"\btürkçe rap\b",
        r"\bturkish hip-hop\b"
    ]),

    ("Türkçe Rock / Anadolu Rock", [
        r"\banadolu rock\b", r"\bturkish rock\b", r"\bturkish alternative rock\b",
        r"\bturkish indie rock\b", r"\bturkish punk\b", r"\bturkish metal\b",
        r"\banatolian rock\b", r"\btürkçe rock\b"
    ]),

    ("Türkçe Pop", [
        r"\bturkish pop\b", r"\bturk pop\b", r"\bturkish alt pop\b", r"\bturkish singer-songwriter\b",
        r"\bturkish dance pop\b", r"\btürkçe pop\b"
    ]),

    ("Türkçe Halk & Sanat & Diğer", [
        r"\bturkish\b", r"\btürk\b", r"\banadolu\b", r"\bhalk m[üu]zi[ğg]i\b",
        r"\btürk[üu]\b", r"\bturku\b", r"\bsanat m[üu]zi[ğg]i\b", r"\bkaradeniz\b",
        r"\bözgün\b", r"\bozgun\b", r"\btürküler\b", r"\bturkish alternative\b"
    ]),

    ("Phonk / Drift & Bass", [
        r"\bphonk\b", r"\bdrift\b", r"\bmontagem\b", r"\bautomotivo\b", r"\bbrazilian funk\b",
        r"\bfunk brasileiro\b", r"\bhardstyle\b", r"\bzyzz\b", r"\bkrushclub\b", r"\bjersey club\b",
        r"\bwave\b", r"\bsped up\b", r"\bslowed\b", r"\bnightcore\b", r"\bbass house\b",
        r"\bgym phonk\b", r"\bdrift phonk\b", r"\bbrazilian phonk\b", r"\bkrush\b"
    ]),

    ("Metal", [
        r"\bmetal\b", r"\bmetalcore\b", r"\bdeathcore\b", r"\bdeath metal\b", r"\bblack metal\b",
        r"\bthrash\b", r"\bdoom metal\b", r"\bpower metal\b", r"\bheavy metal\b", r"\bnu metal\b",
        r"\bgrindcore\b", r"\bslam\b", r"\bsymphonic metal\b"
    ]),

    ("Rock", [
        r"\brock\b", r"\bgrunge\b", r"\bpunk\b", r"\bpost-punk\b", r"\bhard rock\b",
        r"\bclassic rock\b", r"\bprogressive rock\b", r"\bgarage rock\b", r"\bpsychedelic rock\b",
        r"\bemo\b", r"\bpop punk\b", r"\bglam rock\b", r"\bstoner rock\b", r"\brock & roll\b"
    ]),

    ("Hip-Hop & Rap", [
        r"\bhip hop\b", r"\bhip-hop\b", r"\bhip-hop/rap\b", r"\brap\b", r"\btrap\b", r"\bdrill\b",
        r"\bboom bap\b", r"\bgrime\b", r"\bgangsta rap\b", r"\bconscious hip hop\b", r"\beast coast\b",
        r"\bwest coast\b", r"\bsouthern hip hop\b", r"\bmelodic rap\b", r"\bpluggnb\b", r"\br&b/hip-hop\b"
    ]),

    ("Elektronik / Dance", [
        r"\bdance\b", r"\bdans\b", r"\bclub\b", r"\belectronic\b", r"\belektronik\b", r"\belectro\b",
        r"\bhouse\b", r"\btechno\b", r"\bedm\b", r"\btrance\b", r"\bdubstep\b", r"\bdrum and bass\b",
        r"\bdnb\b", r"\bambient\b", r"\bsynthwave\b", r"\bchillout\b", r"\bdisco\b", r"\bfuture bass\b",
        r"\bidm\b", r"\belectronica\b", r"\bdeep house\b", r"\btech house\b", r"\bgarage\b",
        r"\bdance-pop\b", r"\belectropop\b", r"\beurodance\b", r"\bbounce\b", r"\btechno/house\b",
        r"\bslap house\b"
    ]),

    ("R&B & Soul", [
        r"\br&b\b", r"\br&b/soul\b", r"\bsoul\b", r"\bfunk\b", r"\bneo soul\b", r"\bmotown\b",
        r"\bcontemporary r&b\b", r"\bquiet storm\b", r"\bafrobeats\b", r"\bafropop\b",
        r"\bsoul & funk\b"
    ]),

    ("Indie & Alternatif", [
        r"\balternative\b", r"\balternatif\b", r"\bindie\b", r"\bshoegaze\b", r"\bdream pop\b",
        r"\blo-fi\b", r"\blofi\b", r"\bchamber pop\b", r"\bmath rock\b", r"\bmidwest emo\b",
        r"\bart pop\b", r"\bbedroom pop\b", r"\balt-rock\b", r"\bindie rock\b", r"\bindie pop\b"
    ]),

    ("Pop", [
        r"\bpop\b", r"\bk-pop\b", r"\bj-pop\b", r"\beuropop\b", r"\bteen pop\b",
        r"\bpost-teen pop\b", r"\bmainstream\b", r"\bcommercial\b", r"\basya m[üu]zi[ğg]i\b",
        r"\byeniler\b"
    ]),

    ("Akustik & Folk", [
        r"\bfolk\b", r"\bacoustic\b", r"\bakustik\b", r"\bcountry\b", r"\bbluegrass\b",
        r"\bsinger-songwriter\b", r"\bamericana\b", r"\bindie folk\b", r"\bsinger & songwriter\b"
    ]),

    ("Jazz & Blues", [
        r"\bjazz\b", r"\bcaz\b", r"\bblues\b", r"\bbossa nova\b", r"\bbebop\b", r"\bswing\b",
        r"\bfusion\b", r"\bcool jazz\b", r"\bhard bop\b"
    ]),

    ("Klasik & Soundtrack", [
        r"\bclassical\b", r"\bklasik\b", r"\borchestral\b", r"\bsoundtrack\b", r"\bscore\b",
        r"\binstrumental\b", r"\benstr[üu]mantal\b", r"\bpiano\b", r"\bpiyano\b",
        r"\bbaroque\b", r"\bcinematic\b", r"\bvideo game music\b", r"\bfilm\b",
        r"\bfilm/oyun\b", r"\bfilm m[üu]zikleri\b", r"\boyun m[üu]zikleri\b", r"\bfilm/oyun m[üu]zi[ğg]i\b"
    ]),

    ("Latin & Reggae", [
        r"\blatin\b", r"\blatin m[üu]zik\b", r"\breggaeton\b", r"\bsalsa\b", r"\bbachata\b",
        r"\breggae\b", r"\bdub\b", r"\bdancehall\b", r"\bcumbia\b", r"\bbrezilya m[üu]zi[ğg]i\b"
    ]),
]

COMPILED_RULES = [
    (category, [re.compile(pat, re.IGNORECASE) for pat in patterns])
    for category, patterns in MACRO_RULES
]

INFERENCE_PATTERNS = [
    ("Phonk / Drift & Bass", re.compile(r'\b(phonk|drift|montagem|automotivo|hardstyle|sped up|slowed|nightcore|zyzz|brazilian funk|krush)\b', re.IGNORECASE)),
    ("Elektronik / Dance", re.compile(r'\b(remix|club mix|extended mix|vip mix|edm|techno|house|trance|dubstep|rave|synthwave|bass|slap house|radio edit)\b', re.IGNORECASE)),
    ("Türkçe Arabesk & Damar", re.compile(r'\b(arabesk|arabesque|damar|fantezi|taverna)\b', re.IGNORECASE)),
    ("Türkçe Rap / Hip-Hop", re.compile(r'\b(türkçe rap|turkce rap|türkçe trap|turkce trap|türkçe drill|turkce drill|türkçe hip hop|turkce hip hop|anatolian rap|anatolian hip hop)\b', re.IGNORECASE)),
    ("Hip-Hop & Rap", re.compile(r'\b(rap|hip hop|hip-hop|trap|drill|boom bap|beat|diss|freestyle|cypher)\b', re.IGNORECASE)),
    ("Akustik & Folk", re.compile(r'\b(akustik|acoustic|unplugged|folk|live)\b', re.IGNORECASE)),
    ("Klasik & Soundtrack", re.compile(r'\b(soundtrack|theme|ost|score|instrumental|enstr[üu]mantal|piyano|piano|orchestra|senfoni)\b', re.IGNORECASE)),
    ("Rock", re.compile(r'\b(rock|metal|punk|grunge)\b', re.IGNORECASE)),
]

def map_raw_genres_to_macro(raw_genres: List[str]) -> List[str]:
    if not raw_genres:
        return []

    matched_categories: Set[str] = set()

    for genre_str in raw_genres:
        clean_genre = genre_str.lower().strip()
        for category, patterns in COMPILED_RULES:
            if any(p.search(clean_genre) for p in patterns):
                matched_categories.add(category)

    turkish_specific = {
        "Türkçe Arabesk & Damar",
        "Türkçe Rap / Hip-Hop",
        "Türkçe Rock / Anadolu Rock",
        "Türkçe Pop"
    }
    if matched_categories.intersection(turkish_specific) and "Türkçe Halk & Sanat & Diğer" in matched_categories:
        matched_categories.discard("Türkçe Halk & Sanat & Diğer")

    if not matched_categories:
        return []

    order_map = {cat: i for i, (cat, _) in enumerate(MACRO_RULES)}
    return sorted(list(matched_categories), key=lambda x: order_map.get(x, 100))

def infer_genres_from_track_metadata(track_name: str, artist_names: List[str], album_name: str = "") -> List[str]:
    combined = f"{track_name} {' '.join(artist_names)} {album_name}".lower()

    inferred = []
    for cat, regex in INFERENCE_PATTERNS:
        if regex.search(combined):
            inferred.append(cat)
            break

    return inferred

def analyze_playlist_tracks(raw_tracks: List[Dict[str, Any]], artists_map: Dict[str, Dict[str, Any]]) -> Dict[str, Any]:
    enriched_tracks = []
    macro_genre_counts: Dict[str, List[str]] = {}
    micro_genre_counts: Dict[str, List[str]] = {}
    unique_artists_set = set()

    for idx, item in enumerate(raw_tracks):
        track = item.get("track")
        if not track or not track.get("id"):
            continue

        track_id = track.get("id")
        track_name = track.get("name") or "İsimsiz Şarkı"
        track_uri = track.get("uri")
        duration_ms = track.get("duration_ms", 0)
        preview_url = track.get("preview_url")
        popularity = track.get("popularity", 0)

        track_artists = track.get("artists", [])
        artist_names = [a.get("name") for a in track_artists if a.get("name")]
        artist_ids = [a.get("id") for a in track_artists if a.get("id")]
        unique_artists_set.update(artist_ids)

        album = track.get("album") or {}
        album_name = album.get("name") or ""
        album_images = album.get("images") or []
        image_url = album_images[0].get("url") if album_images else ""
        spotify_url = (track.get("external_urls") or {}).get("spotify") or ""

        track_raw_genres = set()
        for a_id in artist_ids:
            if a_id in artists_map:
                artist_data = artists_map[a_id]
                for g in artist_data.get("genres", []):
                    track_raw_genres.add(g.strip())

        for a_name in artist_names:
            if a_name:
                a_clean = a_name.lower().strip()
                if a_clean in CURATED_ARTISTS:
                    for g in CURATED_ARTISTS[a_clean]:
                        track_raw_genres.add(g.strip())

        raw_genres_list = sorted(list(track_raw_genres))
        macro_genres = map_raw_genres_to_macro(raw_genres_list)

        if not macro_genres:
            inferred = infer_genres_from_track_metadata(track_name, artist_names, album_name)
            if inferred:
                macro_genres = inferred
                raw_genres_list = [g.lower() for g in inferred]
            else:
                macro_genres = ["Diğer / Belirsiz"]

        enriched_track = {
            "index": idx + 1,
            "id": track_id,
            "uri": track_uri,
            "name": track_name,
            "artists": artist_names,
            "artist_ids": artist_ids,
            "album": album_name,
            "image_url": image_url,
            "duration_ms": duration_ms,
            "preview_url": preview_url,
            "popularity": popularity,
            "spotify_url": spotify_url,
            "raw_genres": raw_genres_list if raw_genres_list else ["Etiketsiz"],
            "macro_genres": macro_genres,
        }
        enriched_tracks.append(enriched_track)

        for mg in macro_genres:
            if mg not in macro_genre_counts:
                macro_genre_counts[mg] = []
            macro_genre_counts[mg].append(track_id)

        if raw_genres_list:
            for rg in raw_genres_list:
                rg_clean = rg.lower().strip()
                if rg_clean not in micro_genre_counts:
                    micro_genre_counts[rg_clean] = []
                micro_genre_counts[rg_clean].append(track_id)
        else:
            if "etiketsiz" not in micro_genre_counts:
                micro_genre_counts["etiketsiz"] = []
            micro_genre_counts["etiketsiz"].append(track_id)

    total_valid_tracks = len(enriched_tracks)

    macro_stats = []
    for mg, t_ids in sorted(macro_genre_counts.items(), key=lambda x: len(x[1]), reverse=True):
        count = len(t_ids)
        pct = round((count / total_valid_tracks * 100), 1) if total_valid_tracks > 0 else 0
        macro_stats.append({
            "name": mg,
            "count": count,
            "percentage": pct,
            "track_ids": t_ids
        })

    micro_stats = []
    for rg, t_ids in sorted(micro_genre_counts.items(), key=lambda x: len(x[1]), reverse=True):
        count = len(t_ids)
        pct = round((count / total_valid_tracks * 100), 1) if total_valid_tracks > 0 else 0
        micro_stats.append({
            "name": rg,
            "count": count,
            "percentage": pct,
            "track_ids": t_ids
        })

    uncategorized_count = len(macro_genre_counts.get("Diğer / Belirsiz", []))

    return {
        "total_tracks": total_valid_tracks,
        "unique_artists": len(unique_artists_set),
        "macro_genres": macro_stats,
        "micro_genres": micro_stats,
        "tracks": enriched_tracks,
        "categorized_count": total_valid_tracks - uncategorized_count,
        "uncategorized_count": uncategorized_count,
    }
