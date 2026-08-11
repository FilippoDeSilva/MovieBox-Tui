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
                let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
                if let Err(error) = fs::write(&temporary, content) {
                    log::warn!("failed to save watch history: {error}");
                    return;
                }
                if fs::rename(&temporary, &path).is_err() {
                    let _ = fs::remove_file(&path);
                    if let Err(error) = fs::rename(&temporary, &path) {
                        let _ = fs::remove_file(&temporary);
                        log::warn!("failed to commit watch history: {error}");
                    }
                }
            }
        }
    }

    fn key(provider: &str, subject_id: &str, season: usize, episode: usize) -> String {
        format!("{provider}::{subject_id}::{season}::{episode}")
    }

    fn hydrate_watched_index(&mut self) {
        if self.watched.is_empty() {
            self.watched = self
                .recent
                .iter()
                .map(|item| Self::key(&item.provider, &item.subject_id, item.season, item.episode))
                .collect();
            return;
        }

        for item in &self.recent {
            self.watched.insert(Self::key(
                &item.provider,
                &item.subject_id,
                item.season,
                item.episode,
            ));
        }
    }

    pub fn mark_watched(&mut self, item: WatchHistoryItem) {
        let key = Self::key(&item.provider, &item.subject_id, item.season, item.episode);
        self.watched.insert(key);

        self.recent.retain(|i| {
            !(i.provider == item.provider
                && i.subject_id == item.subject_id
                && i.season == item.season
                && i.episode == item.episode)
        });

        self.recent.push(item);

        if self.recent.len() > 100 {
            let excess = self.recent.len() - 100;
            self.recent.drain(0..excess);
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
