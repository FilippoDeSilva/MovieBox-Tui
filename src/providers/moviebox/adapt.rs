use crate::providers::models::{
    CatalogItem, MediaDetails, MediaType, ProviderKind, ProviderMediaId, StreamItem, SubtitleOption,
};

fn provider_id(id: &str) -> ProviderMediaId {
    ProviderMediaId {
        provider: ProviderKind::MovieBox,
        value: id.to_string(),
    }
}

fn media_type(value: &serde_json::Value) -> MediaType {
    if value
        .get("subjectType")
        .or_else(|| value.get("stype"))
        .and_then(|s| s.as_i64())
        == Some(2)
    {
        MediaType::Series
    } else {
        MediaType::Movie
    }
}

fn cover_url(value: &serde_json::Value) -> Option<String> {
    value
        .get("poster")
        .or_else(|| value.get("cover"))
        .or_else(|| value.get("pic"))
        .and_then(|c| c.as_str().or_else(|| c.get("url").and_then(|u| u.as_str())))
        .map(|s| s.to_string())
}

fn year(value: &serde_json::Value) -> Option<String> {
    value
        .get("year")
        .or_else(|| value.get("releaseDate"))
        .or_else(|| value.get("releaseYear"))
        .and_then(|y| {
            y.as_str()
                .map(|s| s.to_string())
                .or_else(|| y.as_i64().map(|n| n.to_string()))
        })
}

pub fn search_json_to_catalog(payload: &serde_json::Value) -> Vec<CatalogItem> {
    let Some(subjects) = payload
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("subjects"))
        .and_then(|s| s.as_array())
    else {
        return Vec::new();
    };

    subjects
        .iter()
        .filter_map(|item| {
            let id = item.get("subjectId").and_then(|i| i.as_str())?;
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            Some(CatalogItem {
                id: provider_id(id),
                title: title.to_string(),
                media_type: media_type(item),
                year: year(item),
                poster_url: cover_url(item),
                season_count: item
                    .get("season")
                    .or_else(|| item.get("seasonCount"))
                    .and_then(|s| s.as_u64())
                    .map(|n| n as usize),
            })
        })
        .collect()
}

pub fn details_json_to_media(details: &serde_json::Value) -> MediaDetails {
    let id = details
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();
    MediaDetails {
        id: provider_id(&id),
        title: details
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        media_type: media_type(details),
        year: year(details),
        description: details
            .get("synopsis")
            .or_else(|| details.get("description"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        tagline: details
            .get("intro")
            .or_else(|| details.get("tagline"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        imdb_rating: details
            .get("imdbRatingValue")
            .or_else(|| details.get("imdbRating"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        director: details
            .get("director")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        stars: details
            .get("stars")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        prints: details
            .get("prints")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        audios: details
            .get("audios")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        poster_url: cover_url(details),
        duration: details
            .get("duration")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        genres: match details.get("genre") {
            Some(serde_json::Value::Array(items)) => items
                .iter()
                .filter_map(|g| g.as_str())
                .map(|s| s.to_string())
                .collect(),
            Some(value) => value
                .as_str()
                .map(|s| {
                    s.split(['/', ','])
                        .map(str::trim)
                        .filter(|g| !g.is_empty())
                        .map(|g| g.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        },
        seasons: Vec::new(),
    }
}

pub fn stream_json_to_items(payload: &serde_json::Value) -> Vec<StreamItem> {
    let items = match payload {
        serde_json::Value::Array(items) => items,
        value => match value.get("list") {
            Some(serde_json::Value::Array(items)) => items,
            _ => return Vec::new(),
        },
    };

    items
        .iter()
        .filter_map(|item| {
            let link = item.get("resourceLink").and_then(|l| l.as_str())?;
            if link.is_empty() {
                return None;
            }
            Some(StreamItem {
                resource_id: item
                    .get("resourceId")
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string(),
                link: link.to_string(),
                resolution: item
                    .get("resolution")
                    .and_then(|r| r.as_u64())
                    .map(|n| n as u32),
                season: item.get("se").and_then(|s| s.as_u64()).map(|n| n as usize),
                episode: item.get("ep").and_then(|s| s.as_u64()).map(|n| n as usize),
            })
        })
        .collect()
}

pub fn captions_json_to_options(payload: &serde_json::Value) -> Vec<SubtitleOption> {
    let Some(captions) = payload.get("extCaptions").and_then(|c| c.as_array()) else {
        return Vec::new();
    };
    captions
        .iter()
        .filter_map(|cap| {
            let url = cap.get("url").and_then(|u| u.as_str())?;
            if url.is_empty() {
                return None;
            }
            let name = cap
                .get("lanName")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string();
            Some(SubtitleOption {
                name,
                url: url.to_string(),
            })
        })
        .collect()
}
