use crate::providers::{
    Provider, fourkhdhub::FourKHdHubClient, models::ProviderKind, moviebox::client::MovieBoxClient,
};

pub(super) async fn fetch_poster_bytes(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let response = client
        .get(url)
        .header("User-Agent", "MovieBox-Tui/1.0")
        .send()
        .await
        .ok()?;
    Some(response.bytes().await.ok()?.to_vec())
}

pub(super) async fn decode_poster(bytes: Vec<u8>) -> Option<std::sync::Arc<image::DynamicImage>> {
    tokio::task::spawn_blocking(move || image::load_from_memory(&bytes))
        .await
        .ok()?
        .ok()
        .map(std::sync::Arc::new)
}

pub(super) async fn provider_search(
    moviebox: &MovieBoxClient,
    fourk: &FourKHdHubClient,
    circleftp: &crate::providers::bdix::circleftp::CircleFtpClient,
    dhakaflix: &crate::providers::bdix::dhakaflix::client::DhakaFlixClient,
    provider: ProviderKind,
    query: &str,
    page: usize,
) -> Result<serde_json::Value, String> {
    match provider {
        ProviderKind::MovieBox => Provider::search(moviebox, query, page).await,
        ProviderKind::FourKHdHub => Provider::search(fourk, query, page).await,
        ProviderKind::BdixCircleFtp => Provider::search(circleftp, query, page).await,
        ProviderKind::BdixDhakaFlix => Provider::search(dhakaflix, query, page).await,
    }
}

pub(super) async fn provider_details(
    moviebox: &MovieBoxClient,
    fourk: &FourKHdHubClient,
    circleftp: &crate::providers::bdix::circleftp::CircleFtpClient,
    dhakaflix: &crate::providers::bdix::dhakaflix::client::DhakaFlixClient,
    provider: ProviderKind,
    subject_id: &str,
) -> Result<serde_json::Value, String> {
    match provider {
        ProviderKind::MovieBox => Provider::details(moviebox, subject_id).await,
        ProviderKind::FourKHdHub => Provider::details(fourk, subject_id).await,
        ProviderKind::BdixCircleFtp => Provider::details(circleftp, subject_id).await,
        ProviderKind::BdixDhakaFlix => Provider::details(dhakaflix, subject_id).await,
    }
}
