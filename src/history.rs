use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct HistoryManager {
    watched: HashSet<String>,
}

impl HistoryManager {
    pub fn new() -> Self {
        if let Some(path) = Self::history_file_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(history) = serde_json::from_str(&content) {
                        return history;
                    }
                }
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
                let _ = fs::write(path, content);
            }
        }
    }

    fn key(provider: &str, subject_id: &str, season: usize, episode: usize) -> String {
        format!("{provider}::{subject_id}::{season}::{episode}")
    }

    pub fn mark_watched(&mut self, provider: &str, subject_id: &str, season: usize, episode: usize) {
        let key = Self::key(provider, subject_id, season, episode);
        self.watched.insert(key);
        self.save();
    }

    pub fn is_watched(&self, provider: &str, subject_id: &str, season: usize, episode: usize) -> bool {
        let key = Self::key(provider, subject_id, season, episode);
        self.watched.contains(&key)
    }
}
