pub mod client;
pub mod parser;

pub use client::{CircleFtpClient, CircleFtpError};

use crate::providers::{
    Provider, ReleaseProvider,
    fourkhdhub::{details_to_moviebox_json, search_to_moviebox_json},
    models::Release,
};

impl Provider for client::CircleFtpClient {
    async fn search(&self, query: &str, _page: usize) -> Result<serde_json::Value, String> {
        self.search(query)
            .await
            .map(|items| search_to_moviebox_json(&items))
            .map_err(|error| error.to_string())
    }

    async fn details(&self, id: &str) -> Result<serde_json::Value, String> {
        self.details(id)
            .await
            .map(|details| details_to_moviebox_json(&details))
            .map_err(|error| error.to_string())
    }
}

impl ReleaseProvider for client::CircleFtpClient {
    async fn episode_streams(
        &self,
        id: &str,
        season: usize,
        episode: usize,
    ) -> Result<Vec<Release>, String> {
        self.releases(id, Some(season), Some(episode))
            .await
            .map_err(|error| error.to_string())
    }
}
