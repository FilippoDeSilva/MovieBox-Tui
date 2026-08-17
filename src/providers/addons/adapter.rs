use super::models::{MetaDetail, MetaItem, StreamItem};
use crate::models::SearchResult;
use crate::providers::models::{PlaybackSource, ProviderKind, Release, SourceMirror};
use std::collections::BTreeMap;

pub fn meta_to_search_result(item: &MetaItem) -> SearchResult {
    let is_series = item.r#type.eq_ignore_ascii_case("series")
        || item.r#type.eq_ignore_ascii_case("tv")
        || item.r#type.eq_ignore_ascii_case("anime");
    let year: String = item
        .release_info
        .as_deref()
        .or(item.year.as_deref())
        .or(item.released.as_deref())
        .map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_digit())
                .take(4)
                .collect::<String>()
        })
        .unwrap_or_default();

    let title = if !item.name.trim().is_empty() {
        item.name.clone()
    } else {
        item.title.clone().unwrap_or_else(|| "Unknown".to_string())
    };

    let cover = item.poster.clone().or_else(|| item.cover.clone());

    SearchResult {
        id: item.id.clone(),
        title,
        stype: if is_series { 2 } else { 1 },
        release_year: year,
        cover_url: cover,
        season: 0,
        episode: 0,
        provider: ProviderKind::Addons,
    }
}

pub fn metas_to_moviebox_search_json(metas: Vec<MetaItem>) -> serde_json::Value {
    let mut seen_ids = std::collections::HashSet::new();
    let subjects = metas
        .into_iter()
        .filter(|item| seen_ids.insert(item.id.clone()))
        .map(|item| {
            let is_series = item.r#type.eq_ignore_ascii_case("series")
                || item.r#type.eq_ignore_ascii_case("tv")
                || item.r#type.eq_ignore_ascii_case("anime");
            let year: String = item
                .release_info
                .as_deref()
                .or(item.year.as_deref())
                .or(item.released.as_deref())
                .map(|s| {
                    s.chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(4)
                        .collect::<String>()
                })
                .unwrap_or_default();

            let title = if !item.name.trim().is_empty() {
                item.name.clone()
            } else {
                item.title.clone().unwrap_or_else(|| "Unknown".to_string())
            };

            let poster_url = item
                .poster
                .clone()
                .or_else(|| item.cover.clone())
                .unwrap_or_default();

            let desc = item.description.clone();
            let rating = item.imdb_rating.clone().or_else(|| item.rating.clone());
            let genres = item.genres.clone();

            serde_json::json!({
                "subjectId": item.id,
                "title": title,
                "subjectType": if is_series { 2 } else { 1 },
                "releaseDate": year,
                "description": desc,
                "intro": desc,
                "imdbRatingValue": rating,
                "genre": genres,
                "cover": {
                    "url": poster_url
                }
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "results": [{
            "subjects": subjects
        }]
    })
}

pub fn meta_detail_to_moviebox_json(detail: &MetaDetail) -> serde_json::Value {
    let is_series = detail.r#type.eq_ignore_ascii_case("series")
        || detail.r#type.eq_ignore_ascii_case("tv")
        || detail.r#type.eq_ignore_ascii_case("anime")
        || !detail.videos.is_empty();

    let mut season_map: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for video in &detail.videos {
        let s = video.season.unwrap_or(1);
        let e = video.episode.unwrap_or(1);
        let eps = season_map.entry(s).or_default();
        if !eps.contains(&e) {
            eps.push(e);
        }
    }

    for eps in season_map.values_mut() {
        eps.sort_unstable();
    }

    let seasons_json = season_map
        .into_iter()
        .map(|(season_num, eps)| {
            serde_json::json!({
                "se": season_num,
                "maxEp": eps.len(),
                "episodeNumbers": eps
            })
        })
        .collect::<Vec<_>>();

    let year_raw = detail
        .release_info
        .as_deref()
        .or(detail.year.as_deref())
        .or(detail.released.as_deref())
        .unwrap_or_default();
    let year: String = year_raw
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(4)
        .collect();

    let mut genres = detail.genres.clone();
    if genres.is_empty() {
        genres = detail.genre.clone();
    }

    let poster_url = detail
        .poster
        .clone()
        .or_else(|| detail.cover.clone())
        .or_else(|| detail.background.clone());

    let title = if !detail.name.trim().is_empty() {
        detail.name.clone()
    } else {
        detail
            .title
            .clone()
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let description = detail
        .description
        .clone()
        .or_else(|| detail.overview.clone())
        .or_else(|| detail.synopsis.clone());

    let rating = detail.imdb_rating.clone().or_else(|| detail.rating.clone());

    serde_json::json!({
        "id": detail.id,
        "subjectId": detail.id,
        "title": title,
        "subjectType": if is_series { 2 } else { 1 },
        "releaseDate": if year.is_empty() { year_raw.to_string() } else { year },
        "description": description.clone(),
        "intro": description.clone(),
        "synopsis": description,
        "imdbRatingValue": rating,
        "director": if detail.director.is_empty() { None } else { Some(detail.director.join(", ")) },
        "stars": if detail.cast.is_empty() { None } else { Some(detail.cast.join(", ")) },
        "duration": detail.runtime,
        "cover": { "url": poster_url },
        "genre": genres,
        "seasons": { "seasons": seasons_json }
    })
}

pub fn parse_quality(text: &str) -> Option<String> {
    let upper = text.to_ascii_uppercase();
    if upper.contains("2160P") || upper.contains("4K") || upper.contains("UHD") {
        Some("2160p".to_string())
    } else if upper.contains("1080P") || upper.contains("FHD") || upper.contains("FULL HD") {
        Some("1080p".to_string())
    } else if upper.contains("720P") || upper.contains("HD") {
        Some("720p".to_string())
    } else if upper.contains("480P") || upper.contains("SD") {
        Some("480p".to_string())
    } else {
        None
    }
}

pub fn parse_codec(text: &str) -> Option<String> {
    let upper = text.to_ascii_uppercase();
    if upper.contains("HEVC")
        || upper.contains("X265")
        || upper.contains("H.265")
        || upper.contains("H265")
    {
        Some("HEVC/x265".to_string())
    } else if upper.contains("X264")
        || upper.contains("H.264")
        || upper.contains("H264")
        || upper.contains("AVC")
    {
        Some("AVC/x264".to_string())
    } else if upper.contains("AV1") {
        Some("AV1".to_string())
    } else {
        None
    }
}

pub fn parse_size_bytes_from_text(text: &str) -> Option<u64> {
    let lower = text.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    for i in 0..parts.len() {
        let clean = parts[i].trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if let Ok(num) = clean.parse::<f64>() {
            if i + 1 < parts.len() {
                let unit = parts[i + 1]
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .to_ascii_uppercase();
                if unit == "GB" || unit == "GIB" || unit == "G" {
                    return Some((num * 1_073_741_824.0) as u64);
                } else if unit == "MB" || unit == "MIB" || unit == "M" {
                    return Some((num * 1_048_576.0) as u64);
                }
            }
        }
        if clean.ends_with("gb") || clean.ends_with("gib") {
            let num_str = clean.trim_end_matches("gb").trim_end_matches("gib");
            if let Ok(num) = num_str.parse::<f64>() {
                return Some((num * 1_073_741_824.0) as u64);
            }
        } else if clean.ends_with("mb") || clean.ends_with("mib") {
            let num_str = clean.trim_end_matches("mb").trim_end_matches("mib");
            if let Ok(num) = num_str.parse::<f64>() {
                return Some((num * 1_048_576.0) as u64);
            }
        }
    }
    None
}

pub fn parse_audio_tracks(text: &str) -> Option<String> {
    let upper = text.to_ascii_uppercase();
    let mut langs = Vec::new();
    let candidates = [
        ("HINDI", "Hindi"),
        ("ENGLISH", "English"),
        ("ENG", "English"),
        ("HIN", "Hindi"),
        ("TAMIL", "Tamil"),
        ("TELUGU", "Telugu"),
        ("BENGALI", "Bengali"),
        ("BEN", "Bengali"),
        ("MALAYALAM", "Malayalam"),
        ("KANNADA", "Kannada"),
        ("MARATHI", "Marathi"),
        ("PUNJABI", "Punjabi"),
        ("GUJARATI", "Gujarati"),
        ("URDU", "Urdu"),
        ("SPANISH", "Spanish"),
        ("FRENCH", "French"),
        ("GERMAN", "German"),
        ("ITALIAN", "Italian"),
        ("JAPANESE", "Japanese"),
        ("JAP", "Japanese"),
        ("KOREAN", "Korean"),
        ("KOR", "Korean"),
        ("RUSSIAN", "Russian"),
        ("CHINESE", "Chinese"),
        ("DUAL", "Dual Audio"),
        ("MULTI", "Multi Audio"),
    ];

    for (needle, label) in candidates {
        if upper.contains(needle) && !langs.contains(&label) {
            langs.push(label);
        }
    }

    if langs.is_empty() {
        None
    } else {
        Some(langs.join(" + "))
    }
}

fn extract_domain_label(url: &str) -> Option<String> {
    let url_clean = url.trim();
    let without_proto = if let Some(stripped) = url_clean.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = url_clean.strip_prefix("http://") {
        stripped
    } else {
        url_clean
    };

    let host = without_proto.split(['/', ':', '?', '#']).next()?.trim();
    if host.is_empty() || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let main_part = if parts.len() >= 2 {
        if (parts[parts.len() - 2] == "co" || parts[parts.len() - 2] == "com") && parts.len() >= 3 {
            parts[parts.len() - 3]
        } else {
            parts[parts.len() - 2]
        }
    } else {
        parts[0]
    };

    if main_part.is_empty() || main_part == "www" || main_part == "api" || main_part == "cdn" {
        return None;
    }

    let capitalized = main_part
        .split(['-', '_'])
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    Some(capitalized)
}

pub fn detect_stream_host(
    addon_name: &str,
    stream_name: &str,
    _title: &str,
    _description: &str,
    url: &str,
) -> String {
    let mut detected_labels = Vec::new();

    for line in stream_name.lines() {
        let trimmed = line.trim().trim_matches(['[', ']', '(', ')', ' ']);
        if !trimmed.is_empty()
            && !trimmed.eq_ignore_ascii_case(addon_name)
            && !detected_labels.contains(&trimmed.to_string())
        {
            detected_labels.push(trimmed.to_string());
        }
    }

    if let Some(domain_label) = extract_domain_label(url) {
        if !domain_label.eq_ignore_ascii_case(addon_name)
            && !detected_labels
                .iter()
                .any(|l| l.eq_ignore_ascii_case(&domain_label))
        {
            detected_labels.push(domain_label);
        }
    }

    if let Some(first_tag) = detected_labels.first() {
        format!("{addon_name} · {first_tag}")
    } else {
        addon_name.to_string()
    }
}

pub fn stream_item_to_release(
    addon_name: &str,
    stream: &StreamItem,
    season: usize,
    episode: usize,
) -> Option<Release> {
    let url = stream.url.as_ref()?.trim();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }

    let stream_name_str = stream.name.as_deref().unwrap_or_default();
    let title_str = stream.title.as_deref().unwrap_or_default();
    let desc_str = stream.description.as_deref().unwrap_or_default();

    let combined_text = format!("{stream_name_str} {title_str} {desc_str}");

    let quality = parse_quality(&combined_text);
    let codec = parse_codec(&combined_text);
    let language = parse_audio_tracks(&combined_text);

    let size_bytes = stream
        .behavior_hints
        .as_ref()
        .and_then(|h| h.video_size)
        .or_else(|| parse_size_bytes_from_text(&combined_text));

    let raw_filename = stream
        .title
        .as_ref()
        .or(stream.description.as_ref())
        .or(stream.name.as_ref())
        .map(|s| s.lines().next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{addon_name} Stream"));

    let filename = crate::tui::text::clean_stream_text(&raw_filename);
    let raw_source_label =
        detect_stream_host(addon_name, stream_name_str, title_str, desc_str, url);
    let source_label = crate::tui::text::clean_stream_text(&raw_source_label);
    let language = language.map(|l| crate::tui::text::clean_stream_text(&l));

    let mut headers = Vec::new();
    if let Some(hints) = &stream.behavior_hints
        && let Some(hdr_map) = &hints.headers
    {
        for (k, v) in hdr_map {
            headers.push((k.clone(), v.clone()));
        }
    }

    Some(Release {
        provider: ProviderKind::Addons,
        filename,
        quality,
        codec,
        language,
        size_bytes,
        season: if season > 0 { Some(season) } else { None },
        episode: if episode > 0 { Some(episode) } else { None },
        mirrors: vec![SourceMirror {
            label: source_label,
            resolver_url: url.to_string(),
            headers,
            direct_file: true,
        }],
    })
}

pub fn releases_to_moviebox_json(releases: &[Release]) -> serde_json::Value {
    let list = releases
        .iter()
        .enumerate()
        .map(|(index, release)| {
            let resolution = release
                .quality
                .as_deref()
                .and_then(|q| q.trim_end_matches('p').parse::<u64>().ok())
                .unwrap_or(1080);
            let first_mirror = release.mirrors.first();
            let source_label = first_mirror
                .map(|m| m.label.clone())
                .unwrap_or_else(|| "Addon".to_string());
            let resource_link = first_mirror.map(|m| m.resolver_url.clone());

            serde_json::json!({
                "resourceId": format!("addon-{index}"),
                "resourceLink": resource_link,
                "title": release.filename,
                "fileName": release.filename,
                "size": release.size_bytes.map(|s| s.to_string()),
                "resolution": resolution,
                "codecName": release.codec,
                "language": release.language,
                "sourceCount": release.mirrors.len(),
                "uploadBy": source_label,
                "se": release.season.unwrap_or_default(),
                "ep": release.episode.unwrap_or_default(),
                "_addon_release": release
            })
        })
        .collect::<Vec<_>>();

    serde_json::Value::Array(list)
}

pub fn release_to_playback_source(release: &Release) -> Option<PlaybackSource> {
    let mirror = release.mirrors.first()?;
    Some(PlaybackSource {
        provider: ProviderKind::Addons,
        url: mirror.resolver_url.clone(),
        headers: mirror.headers.clone(),
        subtitle: None,
        source_label: mirror.label.clone(),
    })
}
