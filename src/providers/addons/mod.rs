pub mod adapter;
pub mod aggregator;
pub mod client;
pub mod models;

pub use adapter::{meta_detail_to_moviebox_json, meta_to_search_result, releases_to_moviebox_json};
pub use aggregator::aggregate_streams;
pub use client::AddonClient;
pub use models::{AddonManifest, InstalledAddon, MetaDetail, MetaItem, StreamItem};
