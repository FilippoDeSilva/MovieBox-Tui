use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchHistoryItem {
    pub provider: String,
    pub subject_id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub stype: i64,
    pub release_year: String,
    pub season: usize,
    pub episode: usize,
    pub timestamp: u64,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct HistoryManager {
    #[serde(default)]
    watched: HashSet<String>,
    #[serde(default)]
    pub recent: Vec<WatchHistoryItem>,
}

impl HistoryManager {
    pub fn new() -> Self {
        if let Some(path) = Self::history_file_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut history) = serde_json::from_str::<Self>(&content) {
                        history.hydrate_watched_index();
                        return history;
                    }
                }
                let _ = fs::remove_file(path);
            }
        }
        Self::default()
    }

    fn history_file_path() -> Option<PathBuf> {
        let mut path = dirs::data_dir()?;
        path.push("moviebox-tui");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("history.json");
        Some(path)
    }

    pub fn save(&self) {
        if let Some(path) = Self::history_file_path() {
            if let Ok(content) = serde_json::to_string(self) {
                if let Err(error) = crate::cache::atomic_write_file(&path, content.as_bytes()) {
                    log::warn!("failed to save watch history: {error}");
                }
            }
        }
    }

    fn key(provider: &str, subject_id: &str, season: usize, episode: usize) -> String {
        format!("{provider}::{subject_id}::{season}::{episode}")
    }

    pub fn is_same_show(a: &WatchHistoryItem, b: &WatchHistoryItem) -> bool {
        let prov_a = crate::providers::models::ProviderKind::parse(&a.provider);
        let prov_b = crate::providers::models::ProviderKind::parse(&b.provider);
        let same_provider = match (prov_a, prov_b) {
            (Some(pa), Some(pb)) => pa == pb,
            _ => a.provider.trim().eq_ignore_ascii_case(b.provider.trim()),
        };
        if !same_provider {
            return false;
        }
        if !a.subject_id.is_empty() && a.subject_id == b.subject_id {
            return true;
        }
        let clean_a = crate::providers::moviebox::clean_moviebox_title(&a.title);
        let clean_b = crate::providers::moviebox::clean_moviebox_title(&b.title);
        if !clean_a.is_empty() && clean_a.eq_ignore_ascii_case(&clean_b) {
            return true;
        }
        false
    }

    fn hydrate_watched_index(&mut self) {
        if self.watched.is_empty() {
            self.watched = self
                .recent
                .iter()
                .map(|item| Self::key(&item.provider, &item.subject_id, item.season, item.episode))
                .collect();
        } else {
            for item in &self.recent {
                self.watched.insert(Self::key(
                    &item.provider,
                    &item.subject_id,
                    item.season,
                    item.episode,
                ));
            }
        }
        self.consolidate_recent();
    }

    fn consolidate_recent(&mut self) {
        let original_len = self.recent.len();
        let mut consolidated: Vec<WatchHistoryItem> = Vec::new();

        let mut sorted = self.recent.clone();
        sorted.sort_by_key(|item| item.timestamp);

        for item in sorted {
            if let Some(existing) = consolidated
                .iter_mut()
                .find(|e| Self::is_same_show(e, &item))
            {
                if item.timestamp >= existing.timestamp {
                    let cover = item
                        .cover_url
                        .clone()
                        .or_else(|| existing.cover_url.clone());
                    *existing = item;
                    existing.cover_url = cover;
                } else if existing.cover_url.is_none() && item.cover_url.is_some() {
                    existing.cover_url = item.cover_url.clone();
                }
            } else {
                consolidated.push(item);
            }
        }

        self.recent = consolidated;
        if self.recent.len() != original_len {
            self.save();
        }
    }

    pub fn mark_watched(&mut self, mut item: WatchHistoryItem) {
        let key = Self::key(&item.provider, &item.subject_id, item.season, item.episode);
        self.watched.insert(key);

        if item.cover_url.is_none() {
            if let Some(existing) = self
                .recent
                .iter()
                .find(|i| Self::is_same_show(i, &item))
                .and_then(|i| i.cover_url.clone())
            {
                item.cover_url = Some(existing);
            }
        }

        self.recent.retain(|i| !Self::is_same_show(i, &item));
        self.recent.push(item);

        if self.recent.len() > 100 {
            let excess = self.recent.len() - 100;
            self.recent.drain(0..excess);
        }
    }

    pub fn update_cover_url(&mut self, subject_id: &str, cover_url: &str) {
        let mut modified = false;
        for item in &mut self.recent {
            if item.subject_id == subject_id && item.cover_url.is_none() {
                item.cover_url = Some(cover_url.to_string());
                modified = true;
            }
        }
        if modified {
            self.save();
        }
    }

    pub fn is_watched(
        &self,
        provider: &str,
        subject_id: &str,
        season: usize,
        episode: usize,
    ) -> bool {
        let key = Self::key(provider, subject_id, season, episode);
        self.watched.contains(&key)
    }
}
