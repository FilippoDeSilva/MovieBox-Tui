use crate::providers::models::{
    CatalogItem, Episode, MediaDetails, MediaType, ProviderError, ProviderKind, ProviderMediaId,
    Season, SubtitleOption,
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
        let id_str = s
            .get("subjectId")
            .or_else(|| s.get("id"))
            .and_then(|v| {
                if let Some(num) = v.as_i64() {
                    Some(num.to_string())
                } else {
                    v.as_str().map(|str_val| str_val.to_string())
                }
            })
            .unwrap_or_default();

        if id_str.is_empty() {
            continue;
        }

        let title = s
            .get("title")
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
            .and_then(|y| y.as_str())
            .map(crate::tui::text::extract_4digit_year)
            .filter(|y| !y.is_empty());

        let poster_url = s
            .get("cover")
            .and_then(|c| c.get("url"))
            .or_else(|| s.get("coverUrl"))
            .and_then(|u| u.as_str())
            .map(|u| u.to_string());

        let season_count = s
            .get("season")
            .and_then(|sc| sc.as_u64())
            .map(|sc| sc as usize);

        items.push(CatalogItem {
            id: ProviderMediaId {
                provider: ProviderKind::MovieBox,
                value: id_str,
            },
            title,
            media_type,
            year,
            poster_url,
            season_count,
        });
    }

    items
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
    })
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
}
