pub mod adapter;
pub mod aggregator;
pub mod client;
pub mod models;

pub use adapter::{
    meta_detail_to_media_details, meta_detail_to_moviebox_json, meta_to_catalog_item,
    meta_to_search_result, releases_to_moviebox_json,
};
pub use aggregator::aggregate_streams;
pub use client::AddonClient;
pub use models::{AddonManifest, InstalledAddon, MetaDetail, MetaItem, StreamItem};

use crate::providers::models::{CatalogItem, MediaDetails, ProviderError, ProviderKind};
use crate::providers::{Provider, ProviderCapabilities};

impl Provider for AddonClient {
    fn id(&self) -> ProviderKind {
        ProviderKind::Addons
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_search: true,
            supports_pagination: false,
            supports_series: true,
            supports_subtitles: true,
            supports_homepage: false,
        }
    }

    async fn search(&self, _query: &str, _page: usize) -> Result<Vec<CatalogItem>, ProviderError> {
        Ok(Vec::new())
    }

    async fn details(&self, _id: &str) -> Result<MediaDetails, ProviderError> {
        Err(ProviderError::NotFound)
    }
}
