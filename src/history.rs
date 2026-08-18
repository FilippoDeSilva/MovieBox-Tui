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
    #[serde(default)]
    pub duration_seconds: Option<u64>,
    #[serde(default)]
    pub progress_seconds: u64,
    #[serde(default)]
    pub completed: bool,
}

impl WatchHistoryItem {
    pub fn is_in_progress(&self) -> bool {
        if self.completed {
            return false;
        }
        if self.progress_seconds < 30 {
            return false;
        }
        if let Some(dur) = self.duration_seconds {
            if dur > 0 && self.progress_seconds >= (dur as f64 * 0.90) as u64 {
                return false;
            }
        }
        true
    }

    pub fn progress_percentage(&self) -> Option<f32> {
        self.duration_seconds
            .filter(|&d| d > 0)
            .map(|d| ((self.progress_seconds as f32 / d as f32) * 100.0).clamp(0.0, 100.0))
    }

    pub fn progress_bar_parts(&self, width: usize) -> (String, String) {
        let pct = self.progress_percentage().unwrap_or(0.0) / 100.0;
        let filled = if pct > 0.0 {
            ((width as f32 * pct).round() as usize).max(1).min(width)
        } else {
            0
        };
        let empty = width.saturating_sub(filled);
        ("━".repeat(filled), "─".repeat(empty))
    }

    pub fn progress_bar(&self, width: usize) -> String {
        let (filled, empty) = self.progress_bar_parts(width);
        format!("{filled}{empty}")
    }

    pub fn formatted_progress(&self) -> String {
        let pos = crate::tui::text::format_duration(self.progress_seconds);
        if let Some(dur) = self.duration_seconds {
            if dur > 0 {
                return format!("{} / {}", pos, crate::tui::text::format_duration(dur));
            }
        }
        pos
    }

    pub fn formatted_remaining(&self) -> Option<String> {
        let dur = self.duration_seconds?;
        if dur <= self.progress_seconds {
            return None;
        }
        let remaining = dur.saturating_sub(self.progress_seconds);
        if remaining < 60 {
            Some("< 1m left".to_string())
        } else if remaining < 3600 {
            Some(format!("{}m left", remaining / 60))
        } else {
            let h = remaining / 3600;
            let m = (remaining % 3600) / 60;
            if m == 0 {
                Some(format!("{h}h left"))
            } else {
                Some(format!("{h}h {m}m left"))
            }
        }
    }

    pub fn formatted_relative_time(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if self.timestamp == 0 || now < self.timestamp {
            return "recently".to_string();
        }
        let delta = now - self.timestamp;
        if delta < 60 {
            "just now".to_string()
        } else if delta < 3600 {
            format!("{}m ago", delta / 60)
        } else if delta < 86400 {
            format!("{}h ago", delta / 3600)
        } else if delta < 172800 {
            "Yesterday".to_string()
        } else if delta < 604800 {
            format!("{}d ago", delta / 86400)
        } else {
            format!("{}w ago", delta / 604800)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPlaybackState {
    pub provider: String,
    pub subject_id: String,
    pub season: usize,
    pub episode: usize,
    pub progress_seconds: u64,
    pub duration_seconds: Option<u64>,
    pub completed: bool,
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
        let mut history = if let Some(path) = Self::history_file_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut hist) = serde_json::from_str::<Self>(&content) {
                        hist.hydrate_watched_index();
                        hist
                    } else {
                        let _ = fs::remove_file(&path);
                        Self::default()
                    }
                } else {
                    Self::default()
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        history.reconcile_pending_playback_states();
        history
    }

    fn history_file_path() -> Option<PathBuf> {
        crate::config::data_dir().map(|dir| dir.join("history.json"))
    }

    pub fn playback_state_dir() -> Option<PathBuf> {
        crate::config::playback_state_dir()
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
        let canon_provider = crate::providers::models::ProviderKind::parse(provider)
            .map(|p| p.cache_key())
            .unwrap_or(provider);
        format!("{canon_provider}::{subject_id}::{season}::{episode}")
    }

    pub fn is_same_show(a: &WatchHistoryItem, b: &WatchHistoryItem) -> bool {
        let prov_a = crate::providers::models::ProviderKind::parse(&a.provider);
        let prov_b = crate::providers::models::ProviderKind::parse(&b.provider);
        let same_provider = match (prov_a, prov_b) {
            (Some(pa), Some(pb)) => pa == pb,
            _ => a.provider.trim().eq_ignore_ascii_case(b.provider.trim()),
        };

        if same_provider && !a.subject_id.is_empty() && a.subject_id == b.subject_id {
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
                .filter(|item| item.completed)
                .map(|item| Self::key(&item.provider, &item.subject_id, item.season, item.episode))
                .collect();
        } else {
            for item in &mut self.recent {
                let key = Self::key(&item.provider, &item.subject_id, item.season, item.episode);
                if self.watched.contains(&key) {
                    item.completed = true;
                } else if item.completed {
                    self.watched.insert(key);
                }
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

    pub fn get_item(
        &self,
        provider: &str,
        subject_id: &str,
        season: usize,
        episode: usize,
        title: Option<&str>,
    ) -> Option<&WatchHistoryItem> {
        self.recent.iter().find(|i| {
            let same_provider = crate::providers::models::ProviderKind::parse(&i.provider)
                .zip(crate::providers::models::ProviderKind::parse(provider))
                .map_or_else(
                    || i.provider.trim().eq_ignore_ascii_case(provider.trim()),
                    |(p1, p2)| p1 == p2,
                );

            if same_provider && i.subject_id == subject_id {
                if i.stype == 1 {
                    return true;
                }
                if i.season == season && i.episode == episode {
                    return true;
                }
            }

            if let Some(t) = title {
                let clean_i = crate::providers::moviebox::clean_moviebox_title(&i.title);
                let clean_t = crate::providers::moviebox::clean_moviebox_title(t);
                if !clean_i.is_empty() && clean_i.eq_ignore_ascii_case(&clean_t) {
                    if i.stype == 1 {
                        return true;
                    }
                    if i.season == season && i.episode == episode {
                        return true;
                    }
                }
            }

            false
        })
    }

    pub fn mark_watched(&mut self, mut item: WatchHistoryItem) {
        item.completed = true;
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

    pub fn update_progress(
        &mut self,
        mut item: WatchHistoryItem,
        progress: u64,
        duration: Option<u64>,
        completed: bool,
    ) {
        item.progress_seconds = progress;
        item.duration_seconds = duration;
        item.completed = completed;
        item.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let key = Self::key(&item.provider, &item.subject_id, item.season, item.episode);
        if completed {
            self.watched.insert(key);
        } else {
            self.watched.remove(&key);
        }

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
        self.save();
    }

    pub fn remove(&mut self, provider: &str, subject_id: &str, season: usize, episode: usize) {
        let key = Self::key(provider, subject_id, season, episode);
        self.watched.remove(&key);
        self.recent.retain(|i| {
            let same_provider = crate::providers::models::ProviderKind::parse(&i.provider)
                .zip(crate::providers::models::ProviderKind::parse(provider))
                .map_or_else(
                    || i.provider.trim().eq_ignore_ascii_case(provider.trim()),
                    |(p1, p2)| p1 == p2,
                );
            !(same_provider
                && i.subject_id == subject_id
                && i.season == season
                && i.episode == episode)
        });
        self.save();
    }

    pub fn clear(&mut self) {
        self.watched.clear();
        self.recent.clear();
        self.save();
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
        if self.watched.contains(&key) {
            return true;
        }
        if let Some(item) = self.get_item(provider, subject_id, season, episode, None) {
            return item.completed;
        }
        false
    }

    pub fn reconcile_pending_playback_states(&mut self) {
        let Some(dir) = Self::playback_state_dir() else {
            return;
        };
        let Ok(entries) = fs::read_dir(&dir) else {
            return;
        };

        let mut modified = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(state) = serde_json::from_str::<PendingPlaybackState>(&content) {
                        if let Some(existing) = self.recent.iter_mut().find(|i| {
                            let same_provider =
                                crate::providers::models::ProviderKind::parse(&i.provider)
                                    .zip(crate::providers::models::ProviderKind::parse(
                                        &state.provider,
                                    ))
                                    .map_or_else(
                                        || {
                                            i.provider
                                                .trim()
                                                .eq_ignore_ascii_case(state.provider.trim())
                                        },
                                        |(p1, p2)| p1 == p2,
                                    );
                            if !same_provider {
                                return false;
                            }
                            if i.subject_id == state.subject_id {
                                if i.stype == 1 {
                                    return true;
                                }
                                return i.season == state.season && i.episode == state.episode;
                            }
                            false
                        }) {
                            existing.progress_seconds = state.progress_seconds;
                            existing.duration_seconds = state.duration_seconds;
                            existing.completed = state.completed;
                            existing.timestamp = state.timestamp;
                            let key = Self::key(
                                &existing.provider,
                                &existing.subject_id,
                                existing.season,
                                existing.episode,
                            );
                            if existing.completed {
                                self.watched.insert(key);
                            } else {
                                self.watched.remove(&key);
                            }
                            modified = true;
                        }
                    }
                }
                let _ = fs::remove_file(&path);
            }
        }

        if modified {
            self.save();
        }
    }
}
