pub mod client;

use crate::providers::{Provider, fourkhdhub::details_to_moviebox_json, models::CatalogItem};

impl Provider for client::DhakaFlixClient {
    async fn search(&self, query: &str, _page: usize) -> Result<Vec<CatalogItem>, String> {
        self.search(query).await.map_err(|error| error.to_string())
    }

    async fn details(&self, id: &str) -> Result<serde_json::Value, String> {
        self.details(id)
            .await
            .map(|details| details_to_moviebox_json(&details))
            .map_err(|error| error.to_string())
    }
}
