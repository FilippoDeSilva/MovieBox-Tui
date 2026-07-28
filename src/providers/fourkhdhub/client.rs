use super::{hubcloud, parser};
use crate::providers::models::{CatalogItem, MediaDetails, PlaybackSource, ProviderKind, Release};
use reqwest::Url;

const DEFAULT_BASE_URL: &str = "https://4khdhub.one/";

#[derive(thiserror::Error, Debug)]
pub enum FourKHdHubError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("invalid provider URL: {0}")]
    InvalidUrl(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("no playable mirror resolved")]
    NoPlayableMirror,
}

#[derive(Clone)]
pub struct FourKHdHubClient {
    client: reqwest::Client,
    base_url: Url,
}

impl Default for FourKHdHubClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FourKHdHubClient {
    pub fn new() -> Self {
        let base = std::env::var("MOVIEBOX_FOURKHDHUB_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self::with_base_url(&base).unwrap_or_else(|_| Self {
            client: build_client(),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("valid default 4KHDHub URL"),
        })
    }

    pub fn with_base_url(base: &str) -> Result<Self, FourKHdHubError> {
        let base_url =
            Url::parse(base).map_err(|_| FourKHdHubError::InvalidUrl(base.to_string()))?;
        if base_url.scheme() != "https" {
            return Err(FourKHdHubError::InvalidUrl(base.to_string()));
        }
        Ok(Self {
            client: build_client(),
            base_url,
        })
    }

    pub async fn health_check(&self) -> Result<(), FourKHdHubError> {
        let response = self.client.get(self.base_url.clone()).send().await?;
        if !response.status().is_success() {
            return Err(FourKHdHubError::Parse(format!(
                "health check returned {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<CatalogItem>, FourKHdHubError> {
        let mut url = self.base_url.clone();
        url.query_pairs_mut().append_pair("s", query);
        let html = self.fetch_text(url).await?;
        parser::parse_search(&self.base_url, &html)
    }

    pub async fn details(&self, id: &str) -> Result<MediaDetails, FourKHdHubError> {
        let url = self.provider_url(id)?;
        let html = self.fetch_text(url).await?;
        parser::parse_details(id, &html)
    }

    pub async fn releases(
        &self,
        id: &str,
        season: usize,
        episode: usize,
    ) -> Result<Vec<Release>, FourKHdHubError> {
        let url = self.provider_url(id)?;
        let html = self.fetch_text(url).await?;
        parser::parse_releases(&html, season, episode)
    }

    pub async fn resolve_release(
        &self,
        release: &Release,
    ) -> Result<PlaybackSource, FourKHdHubError> {
        if release.provider != ProviderKind::FourKHdHub {
            return Err(FourKHdHubError::Parse(
                "release belongs to another provider".into(),
            ));
        }
        for mirror in &release.mirrors {
            let candidates = if mirror.resolver_url.contains("hubcloud.") {
                hubcloud::resolve(&self.client, &mirror.resolver_url).await
            } else if mirror.resolver_url.contains("hubdrive.") {
                hubcloud::resolve_hubdrive(&self.client, &mirror.resolver_url).await
            } else {
                hubcloud::validate_playback_url(&mirror.resolver_url)
                    .map(|url| vec![(url, mirror.label.clone(), mirror.headers.clone())])
            };
            if let Ok(candidates) = candidates {
                for (url, label, headers) in candidates {
                    if let Ok(playable_url) = self.preflight(&url, &headers).await {
                        return Ok(PlaybackSource {
                            provider: ProviderKind::FourKHdHub,
                            url: playable_url,
                            headers,
                            subtitle: None,
                            source_label: label,
                        });
                    }
                }
            }
        }
        Err(FourKHdHubError::NoPlayableMirror)
    }

    async fn preflight(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<String, FourKHdHubError> {
        hubcloud::validate_playback_url(url)?;
        let mut request = self
            .client
            .get(url)
            .header(reqwest::header::RANGE, "bytes=0-0");
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = request.send().await?.error_for_status()?;
        let mut final_url = response.url().clone();
        hubcloud::validate_playback_url(final_url.as_str())?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if content_type.contains("text/html")
            || content_type.contains("application/zip")
            || content_type.contains("text/plain")
        {
            let wrapped = final_url
                .query_pairs()
                .find(|(name, _)| name == "link")
                .map(|(_, value)| value.into_owned())
                .filter(|value| value.starts_with("https://"))
                .ok_or_else(|| {
                    FourKHdHubError::Parse(format!("invalid media content type: {content_type}"))
                })?;
            hubcloud::validate_playback_url(&wrapped)?;
            let mut wrapped_request = self
                .client
                .get(&wrapped)
                .header(reqwest::header::RANGE, "bytes=0-0");
            for (name, value) in headers {
                wrapped_request = wrapped_request.header(name, value);
            }
            let wrapped_response = wrapped_request.send().await?.error_for_status()?;
            final_url = wrapped_response.url().clone();
            hubcloud::validate_playback_url(final_url.as_str())?;
            let wrapped_type = wrapped_response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if wrapped_type.contains("text/html")
                || wrapped_type.contains("application/zip")
                || wrapped_type.contains("text/plain")
            {
                return Err(FourKHdHubError::Parse(format!(
                    "invalid wrapped media content type: {wrapped_type}"
                )));
            }
        }
        Ok(final_url.to_string())
    }

    async fn fetch_text(&self, url: Url) -> Result<String, FourKHdHubError> {
        let response = self.client.get(url).send().await?.error_for_status()?;
        Ok(response.text().await?)
    }

    fn provider_url(&self, id: &str) -> Result<Url, FourKHdHubError> {
        let url = self
            .base_url
            .join(id.trim_start_matches('/'))
            .map_err(|_| FourKHdHubError::InvalidUrl(id.to_string()))?;
        if url.host_str() != self.base_url.host_str() {
            return Err(FourKHdHubError::InvalidUrl(id.to_string()));
        }
        Ok(url)
    }
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .connect_timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 MovieBox-TUI/0.1")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap_or_default()
}

#[cfg(test)]
mod live_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires live 4KHDHub network access"]
    async fn movie_and_series_catalog_gate() {
        let client = FourKHdHubClient::new();
        client.health_check().await.expect("provider health");

        for title in ["Oppenheimer", "Inception", "Interstellar"] {
            let movies = client.search(title).await.expect("movie search");
            let movie = movies
                .iter()
                .find(|item| item.title == title)
                .expect("exact movie");
            let details = client
                .details(&movie.id.value)
                .await
                .expect("movie details");
            assert_eq!(details.title, title);
            assert!(
                !client
                    .releases(&movie.id.value, 0, 0)
                    .await
                    .expect("movie releases")
                    .is_empty()
            );
        }

        for title in ["Breaking Bad", "Chernobyl", "Stranger Things", "Off Campus"] {
            let series = client.search(title).await.expect("series search");
            let show = series
                .iter()
                .find(|item| item.title == title)
                .expect("exact series");
            let show_details = client
                .details(&show.id.value)
                .await
                .expect("series details");
            assert!(show_details.seasons.iter().any(|season| {
                season.number == 1 && season.episodes.iter().any(|episode| episode.number == 1)
            }));
            let releases = client
                .releases(&show.id.value, 1, 1)
                .await
                .expect("episode releases");
            assert!(!releases.is_empty());
            assert!(
                releases
                    .iter()
                    .all(|release| release.season == Some(1) && release.episode == Some(1))
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live resolver and media hosts"]
    async fn hubcloud_resolution_gate() {
        let client = FourKHdHubClient::new();
        let movies = client.search("Oppenheimer").await.expect("movie search");
        let movie = movies
            .iter()
            .find(|item| item.title == "Oppenheimer")
            .expect("exact movie");
        let releases = client
            .releases(&movie.id.value, 0, 0)
            .await
            .expect("movie releases");
        let source = client
            .resolve_release(releases.first().expect("release"))
            .await
            .expect("playback source");
        assert_eq!(source.provider, ProviderKind::FourKHdHub);
        assert!(source.url.starts_with("https://"));
    }

    #[tokio::test]
    #[ignore = "requires live Off Campus resolver and PixelDrain"]
    async fn off_campus_episode_resolution_gate() {
        let client = FourKHdHubClient::new();
        let series = client.search("Off Campus").await.expect("series search");
        let show = series
            .iter()
            .find(|item| item.title == "Off Campus")
            .expect("exact series");
        let releases = client
            .releases(&show.id.value, 1, 1)
            .await
            .expect("episode releases");
        let source = client
            .resolve_release(releases.first().expect("episode release"))
            .await
            .expect("episode playback source");
        assert_eq!(source.provider, ProviderKind::FourKHdHub);
        assert!(source.url.contains("pixeldrain.dev/api/file/"));
    }
}
