use crate::providers::{
    fourkhdhub::{FourKHdHubClient, details_to_moviebox_json, search_to_moviebox_json},
    models::ProviderKind,
    moviebox::client::MovieBoxClient,
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
        ProviderKind::MovieBox => moviebox
            .search(query, page)
            .await
            .map_err(|error| format!("{error:?}")),
        ProviderKind::FourKHdHub => fourk
            .search(query)
            .await
            .map(|items| search_to_moviebox_json(&items))
            .map_err(|error| error.to_string()),
        ProviderKind::BdixCircleFtp => circleftp
            .search(query)
            .await
            .map(|items| crate::providers::fourkhdhub::search_to_moviebox_json(&items))
            .map_err(|error| error.to_string()),
        ProviderKind::BdixDhakaFlix => dhakaflix
            .search(query)
            .await
            .map(|items| crate::providers::fourkhdhub::search_to_moviebox_json(&items))
            .map_err(|error| error.to_string()),
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
        ProviderKind::MovieBox => moviebox
            .get_details(subject_id)
            .await
            .map_err(|error| format!("{error:?}")),
        ProviderKind::FourKHdHub => fourk
            .details(subject_id)
            .await
            .map(|details| details_to_moviebox_json(&details))
            .map_err(|error| error.to_string()),
        ProviderKind::BdixCircleFtp => circleftp
            .details(subject_id)
            .await
            .map(|details| crate::providers::fourkhdhub::details_to_moviebox_json(&details))
            .map_err(|error| error.to_string()),
        ProviderKind::BdixDhakaFlix => dhakaflix
            .details(subject_id)
            .await
            .map(|details| crate::providers::fourkhdhub::details_to_moviebox_json(&details))
            .map_err(|error| error.to_string()),
    }
}
