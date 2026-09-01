use crate::providers::models::{
    AudioTrackOption, CatalogItem, Episode, MediaDetails, MediaType, ProviderError, ProviderKind,
    ProviderMediaId, Release, Season, SourceMirror, SubtitleOption,
};
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

pub fn moviebox_subject_json_to_catalog_item(s: &serde_json::Value) -> Option<CatalogItem> {
    let id_str = s.get("subjectId").or_else(|| s.get("id")).and_then(|v| {
        if let Some(num) = v.as_i64() {
            Some(num.to_string())
        } else {
            v.as_str().map(|str_val| str_val.to_string())
        }
    })?;

    if id_str.is_empty() {
        return None;
    }

    let title = s
        .get("title")
        .or_else(|| s.get("name"))
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let stype = s
        .get("subjectType")
        .or_else(|| s.get("stype"))
        .and_then(|st| st.as_i64())
        .unwrap_or(1);

    let media_type = if stype == 2 {
        MediaType::Series
    } else {
        MediaType::Movie
    };

    let year = s
        .get("releaseDate")
        .or_else(|| s.get("year"))
        .or_else(|| s.get("releaseInfo"))
        .and_then(|y| y.as_str())
        .map(crate::tui::text::extract_4digit_year)
        .filter(|y| !y.is_empty());

    let poster_url = s
        .get("cover")
        .and_then(|c| c.get("url"))
        .or_else(|| s.get("coverUrl"))
        .or_else(|| s.get("poster"))
        .or_else(|| s.get("pic"))
        .and_then(|u| u.as_str())
        .map(|u| u.to_string());

    let season_count = s
        .get("season")
        .and_then(|sc| sc.as_u64())
        .map(|sc| sc as usize);

    Some(CatalogItem {
        id: ProviderMediaId {
            provider: ProviderKind::MovieBox,
            value: id_str,
        },
        title,
        media_type,
        year,
        poster_url,
        season_count,
    })
}

pub fn moviebox_search_json_to_catalog(payload: &serde_json::Value) -> Vec<CatalogItem> {
    let mut items = Vec::new();
    let subjects = payload
        .get("data")
        .and_then(|d| d.get("results"))
        .or_else(|| payload.get("results"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("subjects"))
        .and_then(|s| s.as_array());

    let subjects_slice = match subjects {
        Some(s) => s.as_slice(),
        None => {
            if let Some(list) = payload
                .get("data")
                .and_then(|d| d.get("list"))
                .or_else(|| payload.get("list"))
                .and_then(|l| l.as_array())
            {
                list.as_slice()
            } else {
                &[]
            }
        }
    };

    for s in subjects_slice {
        if let Some(item) = moviebox_subject_json_to_catalog_item(s) {
            items.push(item);
        }
    }

    items
}

pub fn moviebox_homepage_json_to_catalog(
    payload: &serde_json::Value,
) -> (
    Vec<CatalogItem>,
    std::collections::HashMap<String, crate::models::BrowseMetrics>,
) {
    let mut items = Vec::new();
    let mut metrics_map = std::collections::HashMap::new();
    let mut seen_ids = std::collections::HashSet::new();

    let groups = payload
        .get("items")
        .and_then(|i| i.as_array())
        .or_else(|| payload.as_array());

    if let Some(groups_arr) = groups {
        for group in groups_arr {
            let mut group_subjects = Vec::new();
            if let Some(banner) = group
                .get("banner")
                .and_then(|b| b.get("banners"))
                .and_then(|b| b.as_array())
            {
                for b_item in banner {
                    if let Some(subject) = b_item.get("subject") {
                        group_subjects.push(subject);
                    }
                }
            }
            if let Some(custom_data) = group
                .get("customData")
                .and_then(|c| c.get("items"))
                .and_then(|i| i.as_array())
            {
                for c_item in custom_data {
                    if let Some(subject) = c_item.get("subject") {
                        group_subjects.push(subject);
                    }
                }
            }
            if let Some(subjects) = group.get("subjects").and_then(|s| s.as_array()) {
                for s in subjects {
                    group_subjects.push(s);
                }
            }

            for (index, subject_val) in group_subjects.into_iter().enumerate() {
                if let Some(catalog_item) = moviebox_subject_json_to_catalog_item(subject_val) {
                    let id = catalog_item.id.value.clone();
                    if seen_ids.insert(id.clone()) {
                        let mut metric = crate::service::extract_browse_metrics(subject_val);
                        if metric.trending.is_none() {
                            metric.trending = Some((1000 - index.min(999)) as f64);
                        }
                        metrics_map.insert(id, metric);
                        items.push(catalog_item);
                    }
                }
            }
        }
    }

    (items, metrics_map)
}

pub fn moviebox_suggest_json_to_strings(payload: &serde_json::Value) -> Vec<String> {
    let items = moviebox_search_json_to_catalog(payload);
    items.into_iter().map(|item| item.title).collect()
}

pub fn moviebox_details_json_to_media_details(
    payload: &serde_json::Value,
) -> Result<MediaDetails, ProviderError> {
    let subject = payload
        .get("data")
        .and_then(|d| d.get("subject"))
        .or_else(|| payload.get("subject"))
        .unwrap_or(payload);

    let id_str = subject
        .get("subjectId")
        .or_else(|| subject.get("id"))
        .and_then(|v| {
            if let Some(num) = v.as_i64() {
                Some(num.to_string())
            } else {
                v.as_str().map(|str_val| str_val.to_string())
            }
        })
        .ok_or(ProviderError::NotFound)?;

    let title = subject
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let stype = subject
        .get("subjectType")
        .or_else(|| subject.get("stype"))
        .and_then(|st| st.as_i64())
        .unwrap_or(1);

    let media_type = if stype == 2 {
        MediaType::Series
    } else {
        MediaType::Movie
    };

    let year = subject
        .get("releaseDate")
        .or_else(|| subject.get("year"))
        .and_then(|y| y.as_str())
        .map(crate::tui::text::extract_4digit_year)
        .filter(|y| !y.is_empty());

    let description = subject
        .get("description")
        .or_else(|| subject.get("intro"))
        .and_then(|d| d.as_str())
        .map(|d| d.to_string());

    let tagline = subject
        .get("tagline")
        .and_then(|t| t.as_str())
        .map(|t| t.to_string());

    let imdb_rating = subject
        .get("imdbRatingValue")
        .or_else(|| subject.get("rating"))
        .and_then(|r| {
            if let Some(n) = r.as_f64() {
                Some(format!("{:.1}", n))
            } else {
                r.as_str().map(|s| s.to_string())
            }
        });

    let director = subject
        .get("director")
        .and_then(|d| d.as_str())
        .map(|d| d.to_string());

    let stars = subject
        .get("stars")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let prints = subject
        .get("prints")
        .and_then(|p| p.as_str())
        .map(|p| p.to_string());

    let audios = subject
        .get("audios")
        .and_then(|a| a.as_str())
        .map(|a| a.to_string());

    let poster_url = subject
        .get("cover")
        .and_then(|c| c.get("url"))
        .or_else(|| subject.get("coverUrl"))
        .and_then(|u| u.as_str())
        .map(|u| u.to_string());

    let duration = subject.get("duration").and_then(|d| {
        if let Some(n) = d.as_u64() {
            Some(format!("{}m", n / 60))
        } else {
            d.as_str().map(|s| s.to_string())
        }
    });

    let genres = subject
        .get("genre")
        .or_else(|| subject.get("genres"))
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut seasons = Vec::new();
    if let Some(seasons_arr) = subject
        .get("seasons")
        .and_then(|s| s.get("seasons").or(Some(s)))
        .and_then(|s| s.as_array())
    {
        for s in seasons_arr {
            let se_num = s.get("se").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let mut episodes = Vec::new();
            if let Some(eps) = s.get("episodeNumbers").and_then(|e| e.as_array()) {
                for ep in eps {
                    if let Some(ep_num) = ep.as_u64() {
                        episodes.push(Episode {
                            season: se_num,
                            number: ep_num as usize,
                            title: None,
                        });
                    }
                }
            } else if let Some(max_ep) = s.get("maxEp").and_then(|m| m.as_u64()) {
                for ep_num in 1..=max_ep as usize {
                    episodes.push(Episode {
                        season: se_num,
                        number: ep_num,
                        title: None,
                    });
                }
            }
            seasons.push(Season {
                number: se_num,
                episodes,
            });
        }
    }

    let mut dubs = Vec::new();
    if let Some(dubs_arr) = subject.get("dubs").and_then(|d| d.as_array()) {
        for d in dubs_arr {
            let subject_id = d
                .get("subjectId")
                .or_else(|| d.get("id"))
                .and_then(|v| {
                    if let Some(num) = v.as_i64() {
                        Some(num.to_string())
                    } else {
                        v.as_str().map(|str_val| str_val.to_string())
                    }
                })
                .unwrap_or_default();
            let language = d
                .get("lanName")
                .or_else(|| d.get("language"))
                .or_else(|| d.get("lang"))
                .and_then(|l| l.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let label = d
                .get("title")
                .or_else(|| d.get("name"))
                .or_else(|| d.get("lanName"))
                .and_then(|l| l.as_str())
                .unwrap_or(&language)
                .to_string();
            dubs.push(AudioTrackOption {
                subject_id,
                language,
                label,
            });
        }
    }

    Ok(MediaDetails {
        id: ProviderMediaId {
            provider: ProviderKind::MovieBox,
            value: id_str,
        },
        title,
        media_type,
        year,
        description,
        tagline,
        imdb_rating,
        director,
        stars,
        prints,
        audios,
        poster_url,
        duration,
        genres,
        seasons,
        dubs,
    })
}

pub fn moviebox_resource_item_to_release(item: &serde_json::Value) -> Release {
    if let Some(r) = item
        .get("_addon_release")
        .or_else(|| item.get("_fourk_release"))
        .and_then(|val| serde_json::from_value::<Release>(val.clone()).ok())
    {
        return r;
    }

    let filename = item
        .get("fileName")
        .or_else(|| item.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Release")
        .to_string();

    let resolution = item.get("resolution").and_then(|r| {
        if let Some(n) = r.as_u64() {
            Some(format!("{n}p"))
        } else if let Some(n) = r.as_i64() {
            Some(format!("{n}p"))
        } else {
            r.as_str().map(|s| s.to_string())
        }
    });

    let codec = item
        .get("codecName")
        .or_else(|| item.get("codec"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());

    let language = item
        .get("language")
        .or_else(|| item.get("lanName"))
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());

    let size_bytes = item.get("size").and_then(|s| {
        if let Some(n) = s.as_u64() {
            Some(n)
        } else if let Some(n) = s.as_i64() {
            Some(n as u64)
        } else if let Some(str_val) = s.as_str() {
            str_val.parse::<u64>().ok()
        } else {
            None
        }
    });

    let season = item.get("se").and_then(|v| {
        if let Some(n) = v.as_u64() {
            Some(n as usize)
        } else if let Some(n) = v.as_i64() {
            Some(n as usize)
        } else {
            v.as_str().and_then(|s| s.parse().ok())
        }
    });

    let episode = item.get("ep").and_then(|v| {
        if let Some(n) = v.as_u64() {
            Some(n as usize)
        } else if let Some(n) = v.as_i64() {
            Some(n as usize)
        } else {
            v.as_str().and_then(|s| s.parse().ok())
        }
    });

    let mut mirrors = Vec::new();
    let resource_link = item
        .get("resourceLink")
        .or_else(|| item.get("url"))
        .and_then(|l| l.as_str())
        .filter(|s| !s.is_empty());

    if let Some(link) = resource_link {
        let label = item
            .get("uploadBy")
            .or_else(|| item.get("source"))
            .and_then(|u| u.as_str())
            .unwrap_or("Direct")
            .to_string();

        mirrors.push(SourceMirror {
            label,
            resolver_url: link.to_string(),
            headers: vec![],
            direct_file: false,
        });
    }

    Release {
        provider: ProviderKind::MovieBox,
        filename,
        quality: resolution,
        codec,
        language,
        size_bytes,
        season,
        episode,
        mirrors,
    }
}

pub fn moviebox_resource_json_to_releases(payload: &serde_json::Value) -> Vec<Release> {
    let items = if let Some(list) = payload.get("list").and_then(|l| l.as_array()) {
        list.as_slice()
    } else if let Some(arr) = payload.as_array() {
        arr.as_slice()
    } else {
        &[]
    };

    items
        .iter()
        .map(moviebox_resource_item_to_release)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_moviebox_search_json_to_catalog() {
        let payload = json!({
            "data": {
                "results": [{
                    "subjects": [
                        {
                            "subjectId": "12345",
                            "title": "Inception",
                            "subjectType": 1,
                            "releaseDate": "2010-07-16",
                            "cover": { "url": "https://example.com/cover.jpg" },
                            "season": 0
                        },
                        {
                            "subjectId": "67890",
                            "title": "Breaking Bad",
                            "subjectType": 2,
                            "releaseDate": "2008",
                            "cover": { "url": "https://example.com/bb.jpg" },
                            "season": 5
                        }
                    ]
                }]
            }
        });

        let catalog = moviebox_search_json_to_catalog(&payload);
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog[0].id.value, "12345");
        assert_eq!(catalog[0].title, "Inception");
        assert_eq!(catalog[0].media_type, MediaType::Movie);
        assert_eq!(catalog[0].year.as_deref(), Some("2010"));

        assert_eq!(catalog[1].id.value, "67890");
        assert_eq!(catalog[1].title, "Breaking Bad");
        assert_eq!(catalog[1].media_type, MediaType::Series);
        assert_eq!(catalog[1].season_count, Some(5));
    }

    #[test]
    fn test_moviebox_details_json_to_media_details() {
        let payload = json!({
            "data": {
                "subject": {
                    "subjectId": "999",
                    "title": "Interstellar",
                    "subjectType": 1,
                    "releaseDate": "2014",
                    "description": "Space exploration film",
                    "imdbRatingValue": "8.7",
                    "director": "Christopher Nolan",
                    "stars": "Matthew McConaughey, Anne Hathaway",
                    "duration": 10140,
                    "genre": ["Sci-Fi", "Adventure"],
                    "cover": { "url": "https://example.com/interstellar.jpg" }
                }
            }
        });

        let details = moviebox_details_json_to_media_details(&payload).unwrap();
        assert_eq!(details.id.value, "999");
        assert_eq!(details.title, "Interstellar");
        assert_eq!(details.media_type, MediaType::Movie);
        assert_eq!(details.director.as_deref(), Some("Christopher Nolan"));
        assert_eq!(details.imdb_rating.as_deref(), Some("8.7"));
        assert_eq!(details.duration.as_deref(), Some("169m"));
        assert_eq!(details.genres, vec!["Sci-Fi", "Adventure"]);
    }

    #[test]
    fn test_moviebox_details_with_dubs_and_resource_conversion() {
        let payload = json!({
            "data": {
                "subject": {
                    "subjectId": "500",
                    "title": "Money Heist",
                    "subjectType": 2,
                    "dubs": [
                        { "subjectId": "500", "lanName": "Original", "title": "Spanish (Original)" },
                        { "subjectId": "501", "lanName": "English", "title": "English Dub" }
                    ],
                    "seasons": {
                        "seasons": [
                            { "se": 1, "maxEp": 13 }
                        ]
                    }
                }
            }
        });

        let details = moviebox_details_json_to_media_details(&payload).unwrap();
        assert!(details.is_series());
        assert!(details.has_languages());
        assert_eq!(details.dubs.len(), 2);
        assert_eq!(details.dubs[0].subject_id, "500");
        assert_eq!(details.dubs[0].language, "Original");
        assert_eq!(details.dubs[1].subject_id, "501");
        assert_eq!(details.dubs[1].language, "English");

        let resource_payload = json!({
            "list": [
                {
                    "fileName": "Money.Heist.S01E01.1080p.NF.WEB-DL.x265",
                    "resolution": 1080,
                    "codecName": "hevc",
                    "size": "850000000",
                    "se": 1,
                    "ep": 1,
                    "resourceLink": "https://stream.example.com/mh0101.mp4",
                    "uploadBy": "NF"
                }
            ]
        });

        let releases = moviebox_resource_json_to_releases(&resource_payload);
        assert_eq!(releases.len(), 1);
        assert_eq!(
            releases[0].filename,
            "Money.Heist.S01E01.1080p.NF.WEB-DL.x265"
        );
        assert_eq!(releases[0].resolution_u64(), 1080);
        assert_eq!(releases[0].codec.as_deref(), Some("hevc"));
        assert_eq!(releases[0].season, Some(1));
        assert_eq!(releases[0].episode, Some(1));
        assert_eq!(
            releases[0].direct_url(),
            Some("https://stream.example.com/mh0101.mp4")
        );
        assert_eq!(releases[0].source_label(), "NF");
    }
}
