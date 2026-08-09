pub mod client;

use crate::providers::{Provider, models::CatalogItem};

impl Provider for client::DhakaFlixClient {
    async fn search(&self, query: &str, _page: usize) -> Result<Vec<CatalogItem>, String> {
        self.search(query).await.map_err(|error| error.to_string())
    }
}
