use super::client::FourKHdHubError;
use crate::providers::models::{
    CatalogItem, Episode, MediaDetails, MediaType, ProviderKind, ProviderMediaId, Release, Season,
    SourceMirror,
};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use std::collections::{BTreeMap, HashMap};

pub fn parse_search(base: &Url, html: &str) -> Result<Vec<CatalogItem>, FourKHdHubError> {
    let document = Html::parse_document(html);
    let card = selector("a.movie-card")?;
    let title = selector(".movie-card-title")?;
    let meta = selector(".movie-card-meta")?;
    let image = selector("img")?;
    let mut items = Vec::new();

    for node in document.select(&card) {
        let Some(href) = node.value().attr("href") else {
            continue;
        };
        let Ok(url) = base.join(href) else { continue };
        if url.host_str() != base.host_str() {
            continue;
        }
        let item_title = text_of(node.select(&title).next()).unwrap_or_default();
        if item_title.is_empty() {
            continue;
        }
        let meta_text = text_of(node.select(&meta).next()).unwrap_or_default();
        let year = first_four_digit_year(&meta_text);
        let media_type = if href.contains("-series-") {
            MediaType::Series
        } else {
            MediaType::Movie
        };
        let poster_url = node
            .select(&image)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(str::to_string);
        items.push(CatalogItem {
            id: ProviderMediaId {
                provider: ProviderKind::FourKHdHub,
                value: url.path().to_string(),
            },
            title: item_title,
            media_type,
            year,
            poster_url,
            season_count: parse_season_count(&meta_text),
        });
    }
    Ok(items)
}

pub fn parse_details(id: &str, html: &str) -> Result<MediaDetails, FourKHdHubError> {
    let document = Html::parse_document(html);
    let h1 = selector("h1")?;
    let raw_title = document
        .select(&h1)
        .find_map(|node| text_of(Some(node)))
        .filter(|text| !text.is_empty())
        .or_else(|| meta_content(&document, "meta[property=\"og:title\"]"))
        .ok_or_else(|| FourKHdHubError::Parse("title missing".into()))?;
    let title = strip_trailing_year(&raw_title);
    let media_type = if id.contains("-series-") {
        MediaType::Series
    } else {
        MediaType::Movie
    };
    let description = meta_content(&document, "meta[name=\"description\"]");
    let poster_url = meta_content(&document, "meta[property=\"og:image\"]");
    let year = find_metadata(&document, "Release:")
        .and_then(|value| first_four_digit_year(&value))
        .or_else(|| first_four_digit_year(&raw_title));
    let genres = document
        .select(&selector(".badge-outline a")?)
        .filter_map(|node| text_of(Some(node)))
        .filter(|value| {
            !matches!(
                value.as_str(),
                "Movies" | "Series" | "Hindi" | "English" | "Hollywood"
            )
        })
        .collect();
    let seasons = parse_seasons(&document)?;

    Ok(MediaDetails {
        id: ProviderMediaId {
            provider: ProviderKind::FourKHdHub,
            value: id.to_string(),
        },
        title,
        media_type,
        year,
        description,
        poster_url,
        genres,
        seasons,
    })
}

pub fn parse_releases(
    html: &str,
    season: usize,
    episode: usize,
) -> Result<Vec<Release>, FourKHdHubError> {
    let document = Html::parse_document(html);
    let item_selector = if season > 0 {
        selector("#episodes .episode-download-item")?
    } else {
        selector(".download-item")?
    };
    let filename_selector = if season > 0 {
        selector(".episode-file-title")?
    } else {
        selector(".file-title")?
    };
    let link_selector = selector("a[href]")?;
    let size_selector = selector(".badge-size, .badge")?;
    let mut grouped: HashMap<String, Release> = HashMap::new();

    for item in document.select(&item_selector) {
        let filename = text_of(item.select(&filename_selector).next()).unwrap_or_default();
        if filename.is_empty() || is_archive(&filename) {
            continue;
        }
        let parsed_episode = parse_season_episode(&filename);
        if season > 0 && parsed_episode != Some((season, episode)) {
            continue;
        }
        let mirrors = item
            .select(&link_selector)
            .filter_map(|link| {
                let href = link.value().attr("href")?;
                if !href.starts_with("https://") || href.contains("logout") {
                    return None;
                }
                let label = text_of(Some(link)).unwrap_or_else(|| "Source".into());
                Some(SourceMirror {
                    label,
                    resolver_url: href.to_string(),
                    headers: Vec::new(),
                    direct_file: !href.contains("hubcloud.") && !href.contains("hubdrive."),
                })
            })
            .collect::<Vec<_>>();
        if mirrors.is_empty() {
            continue;
        }
        let size_text = item
            .select(&size_selector)
            .filter_map(|node| text_of(Some(node)))
            .find(|text| parse_size_bytes(text).is_some());
        let key = normalize_filename(&filename);
        let release = grouped.entry(key).or_insert_with(|| Release {
            provider: ProviderKind::FourKHdHub,
            quality: detect_quality(&filename),
            codec: detect_codec(&filename),
            language: detect_language(&filename),
            size_bytes: size_text.as_deref().and_then(parse_size_bytes),
            season: parsed_episode.map(|value| value.0),
            episode: parsed_episode.map(|value| value.1),
            filename: filename.clone(),
            mirrors: Vec::new(),
        });
        for mirror in mirrors {
            if !release
                .mirrors
                .iter()
                .any(|existing| existing.resolver_url == mirror.resolver_url)
            {
                release.mirrors.push(mirror);
            }
        }
    }
    let mut releases = grouped.into_values().collect::<Vec<_>>();
    releases.sort_by(|left, right| right.quality.cmp(&left.quality));
    Ok(releases)
}

fn parse_seasons(document: &Html) -> Result<Vec<Season>, FourKHdHubError> {
    let item = selector("#episodes .episode-download-item")?;
    let title = selector(".episode-file-title")?;
    let mut seasons: BTreeMap<usize, BTreeMap<usize, Episode>> = BTreeMap::new();
    for node in document.select(&item) {
        let filename = text_of(node.select(&title).next()).unwrap_or_default();
        let Some((season, episode)) = parse_season_episode(&filename) else {
            continue;
        };
        seasons
            .entry(season)
            .or_default()
            .entry(episode)
            .or_insert(Episode {
                season,
                number: episode,
                title: None,
            });
    }
    Ok(seasons
        .into_iter()
        .map(|(number, episodes)| Season {
            number,
            episodes: episodes.into_values().collect(),
        })
        .collect())
}

pub fn search_to_moviebox_json(items: &[CatalogItem]) -> serde_json::Value {
    let subjects = items
        .iter()
        .map(|item| {
            serde_json::json!({
                "subjectId": item.id.value,
                "title": item.title,
                "subjectType": if item.media_type == MediaType::Series { 2 } else { 1 },
                "releaseDate": item.year,
                "cover": { "url": item.poster_url },
                "season": item.season_count.unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "results": [{ "subjects": subjects }] })
}

pub fn details_to_moviebox_json(details: &MediaDetails) -> serde_json::Value {
    let seasons = details
        .seasons
        .iter()
        .map(|season| {
            serde_json::json!({
                "se": season.number,
                "maxEp": season.episodes.len(),
                "episodeNumbers": season.episodes.iter().map(|episode| episode.number).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "id": details.id.value,
        "subjectId": details.id.value,
        "title": details.title,
        "subjectType": if details.media_type == MediaType::Series { 2 } else { 1 },
        "releaseDate": details.year,
        "description": details.description,
        "cover": { "url": details.poster_url },
        "genre": details.genres,
        "seasons": { "seasons": seasons }
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
                .and_then(|quality| quality.trim_end_matches('p').parse::<u64>().ok())
                .unwrap_or_default();
            serde_json::json!({
                "resourceId": format!("fourk-{}", index),
                "resourceLink": release.mirrors.first().map(|mirror| mirror.resolver_url.clone()),
                "title": release.filename,
                "fileName": release.filename,
                "size": release.size_bytes.map(|size| size.to_string()),
                "resolution": resolution,
                "codecName": release.codec,
                "uploadBy": format!("{} sources", release.mirrors.len()),
                "se": release.season.unwrap_or_default(),
                "ep": release.episode.unwrap_or_default(),
                "_fourk_release": release
            })
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(list)
}

fn selector(value: &str) -> Result<Selector, FourKHdHubError> {
    Selector::parse(value).map_err(|_| FourKHdHubError::Parse(format!("invalid selector: {value}")))
}

fn text_of(node: Option<ElementRef<'_>>) -> Option<String> {
    node.map(|node| node.text().collect::<Vec<_>>().join(" "))
        .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|text| !text.is_empty())
}

fn meta_content(document: &Html, query: &str) -> Option<String> {
    let selector = Selector::parse(query).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|node| node.value().attr("content"))
        .map(str::to_string)
}

fn find_metadata(document: &Html, label: &str) -> Option<String> {
    let item = Selector::parse(".metadata-item").ok()?;
    let label_selector = Selector::parse(".metadata-label").ok()?;
    let value_selector = Selector::parse(".metadata-value").ok()?;
    document.select(&item).find_map(|node| {
        let current = text_of(node.select(&label_selector).next())?;
        (current == label).then(|| text_of(node.select(&value_selector).next()))?
    })
}

fn first_four_digit_year(value: &str) -> Option<String> {
    value
        .as_bytes()
        .windows(4)
        .find(|window| window.iter().all(u8::is_ascii_digit) && matches!(window[0], b'1' | b'2'))
        .and_then(|window| std::str::from_utf8(window).ok())
        .map(str::to_string)
}

fn strip_trailing_year(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 6 {
        let suffix = &trimmed[trimmed.len() - 6..];
        if suffix.starts_with('(')
            && suffix.ends_with(')')
            && suffix[1..5].bytes().all(|byte| byte.is_ascii_digit())
        {
            return trimmed[..trimmed.len() - 6].trim_end().to_string();
        }
    }
    trimmed.to_string()
}

fn parse_season_count(value: &str) -> Option<usize> {
    let marker = value.find('S')?;
    let suffix = &value[marker..];
    suffix
        .split(['-', ' ', '•'])
        .filter_map(|part| part.trim_start_matches('S').parse::<usize>().ok())
        .max()
}

fn parse_season_episode(value: &str) -> Option<(usize, usize)> {
    let upper = value.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    for index in 0..bytes.len().saturating_sub(4) {
        if bytes[index] != b'S' {
            continue;
        }
        let Some(season_end) = bytes[index + 1..]
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .map(|offset| index + 1 + offset)
        else {
            continue;
        };
        if season_end == index + 1 || bytes.get(season_end) != Some(&b'E') {
            continue;
        }
        let episode_end = bytes[season_end + 1..]
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .map(|offset| season_end + 1 + offset)
            .unwrap_or(bytes.len());
        if episode_end == season_end + 1 {
            continue;
        }
        if let (Ok(season), Ok(episode)) = (
            upper[index + 1..season_end].parse(),
            upper[season_end + 1..episode_end].parse(),
        ) {
            return Some((season, episode));
        }
    }
    None
}

fn parse_size_bytes(value: &str) -> Option<u64> {
    let normalized = value.replace(' ', "").to_ascii_uppercase();
    for (suffix, multiplier) in [
        ("GB", 1024_u64.pow(3)),
        ("MB", 1024_u64.pow(2)),
        ("KB", 1024_u64),
    ] {
        if let Some(number) = normalized.strip_suffix(suffix)
            && let Ok(number) = number.parse::<f64>()
        {
            return Some((number * multiplier as f64) as u64);
        }
    }
    None
}

fn detect_quality(value: &str) -> Option<String> {
    ["2160p", "1080p", "720p", "480p"]
        .into_iter()
        .find(|quality| {
            value
                .to_ascii_lowercase()
                .contains(&quality.to_ascii_lowercase())
        })
        .map(str::to_string)
}

fn detect_codec(value: &str) -> Option<String> {
    ["HEVC", "x265", "x264", "AV1", "REMUX"]
        .into_iter()
        .find(|codec| {
            value
                .to_ascii_lowercase()
                .contains(&codec.to_ascii_lowercase())
        })
        .map(str::to_string)
}

fn detect_language(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    match (lower.contains("hindi"), lower.contains("english")) {
        (true, true) => Some("Hindi, English".into()),
        (true, false) => Some("Hindi".into()),
        (false, true) => Some("English".into()),
        _ => None,
    }
}

fn is_archive(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.contains("complete season") || lower.contains("season pack")
}

fn normalize_filename(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_episode_identity() {
        assert_eq!(
            parse_season_episode("Breaking.Bad.S01E07.1080p.mkv"),
            Some((1, 7))
        );
        assert_eq!(parse_season_episode("movie.mkv"), None);
    }

    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size_bytes("1.5 GB"), Some(1_610_612_736));
        assert_eq!(parse_size_bytes("480 MB"), Some(503_316_480));
    }

    #[test]
    fn strips_only_a_trailing_year_from_titles() {
        assert_eq!(strip_trailing_year("Oppenheimer (2023)"), "Oppenheimer");
        assert_eq!(strip_trailing_year("The 100"), "The 100");
    }

    #[test]
    fn archive_detection_excludes_packs() {
        assert!(is_archive("Show S01 Complete Season.zip"));
        assert!(!is_archive("Show S01E01.mkv"));
    }

    #[test]
    fn fixture_groups_mirrors_and_rejects_adjacent_episodes() {
        let html = r#"
          <div id="episodes">
            <div class="episode-download-item">
              <div class="episode-file-title">Show.S01E01.1080p.HEVC.English.mkv</div>
              <span class="badge-size">1.5 GB</span>
              <a href="https://hubcloud.ist/drive/one">HubCloud</a>
              <a href="https://hubdrive.tips/file/one">HubDrive</a>
            </div>
            <div class="episode-download-item">
              <div class="episode-file-title">Show.S01E02.1080p.HEVC.English.mkv</div>
              <a href="https://hubcloud.ist/drive/two">HubCloud</a>
            </div>
            <div class="episode-download-item">
              <div class="episode-file-title">Show.S01.Complete.Season.zip</div>
              <a href="https://hubcloud.ist/drive/pack">HubCloud</a>
            </div>
          </div>
        "#;
        let releases = parse_releases(html, 1, 1).expect("fixture parses");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].mirrors.len(), 2);
        assert_eq!(releases[0].episode, Some(1));
    }

    #[test]
    fn compatibility_release_payload_is_an_array() {
        let payload = releases_to_moviebox_json(&[Release {
            provider: ProviderKind::FourKHdHub,
            filename: "Movie.2160p.HEVC.mkv".into(),
            quality: Some("2160p".into()),
            codec: Some("HEVC".into()),
            language: None,
            size_bytes: Some(1024),
            season: None,
            episode: None,
            mirrors: vec![SourceMirror {
                label: "HubCloud".into(),
                resolver_url: "https://hubcloud.ist/drive/test".into(),
                headers: vec![],
                direct_file: false,
            }],
        }]);
        assert!(payload.is_array());
        assert_eq!(payload[0]["resolution"], 2160);
        assert_eq!(payload[0]["se"], 0);
        assert_eq!(payload[0]["ep"], 0);
    }

    #[test]
    fn episode_payload_preserves_stream_pool_identity() {
        let payload = releases_to_moviebox_json(&[Release {
            provider: ProviderKind::FourKHdHub,
            filename: "Off.Campus.S01E01.2160p.mkv".into(),
            quality: Some("2160p".into()),
            codec: Some("HEVC".into()),
            language: Some("English".into()),
            size_bytes: Some(1024),
            season: Some(1),
            episode: Some(1),
            mirrors: vec![SourceMirror {
                label: "HubCloud".into(),
                resolver_url: "https://hubcloud.ist/drive/off-campus".into(),
                headers: vec![],
                direct_file: false,
            }],
        }]);
        assert_eq!(payload[0]["se"], 1);
        assert_eq!(payload[0]["ep"], 1);
    }
}
