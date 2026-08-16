use crate::providers::{
    fourkhdhub::FourKHdHubClient, models::ProviderKind, moviebox::client::MovieBoxClient,
};

pub(super) use crate::service::decode_poster;

pub(super) async fn fetch_poster_bytes(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let response = client
        .get(url)
        .header("User-Agent", "MovieBox-Tui/1.0")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    Some(response.bytes().await.ok()?.to_vec())
}

pub(super) async fn provider_search(
    moviebox: &MovieBoxClient,
    fourk: Option<&FourKHdHubClient>,
    circleftp: &crate::providers::bdix::circleftp::CircleFtpClient,
    dhakaflix: &crate::providers::bdix::dhakaflix::client::DhakaFlixClient,
    provider: ProviderKind,
    query: &str,
    page: usize,
) -> Result<serde_json::Value, String> {
    let service = crate::service::MovieBoxService {
        client: moviebox.clone(),
        fourk_client: fourk.cloned(),
        circleftp_client: circleftp.clone(),
        dhakaflix_client: dhakaflix.clone(),
        http_client: moviebox.http_client().clone(),
    };
    service.search(provider, query, page).await
}

pub(super) async fn provider_details(
    moviebox: &MovieBoxClient,
    fourk: Option<&FourKHdHubClient>,
    circleftp: &crate::providers::bdix::circleftp::CircleFtpClient,
    dhakaflix: &crate::providers::bdix::dhakaflix::client::DhakaFlixClient,
    provider: ProviderKind,
    subject_id: &str,
) -> Result<serde_json::Value, String> {
    let service = crate::service::MovieBoxService {
        client: moviebox.clone(),
        fourk_client: fourk.cloned(),
        circleftp_client: circleftp.clone(),
        dhakaflix_client: dhakaflix.clone(),
        http_client: moviebox.http_client().clone(),
    };
    service.details(provider, subject_id).await
}
