mod client;
mod hubcloud;
mod parser;

pub use client::{FourKHdHubClient, FourKHdHubError};
pub use parser::{details_to_moviebox_json, releases_to_moviebox_json, search_to_moviebox_json};

use crate::providers::{Provider, models::CatalogItem};

impl Provider for FourKHdHubClient {
    async fn search(&self, query: &str, _page: usize) -> Result<Vec<CatalogItem>, String> {
        self.search(query).await.map_err(|error| error.to_string())
    }
}
