use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const CACHE_EXPIRY_SECS: u64 = 24 * 60 * 60; // 24 hours

pub fn get_cache_dir(subdir: &str) -> PathBuf {
    let mut path = dirs::cache_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("moviebox-tui");
    path.push(subdir);
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn get_cache_path(subject_id: &str, season: usize, episode: usize) -> PathBuf {
    let mut path = get_cache_dir("streams");
    path.push(format!("{}_{}_{}.json", subject_id, season, episode));
    path
}

pub fn get_stream_cache(
    subject_id: &str,
    season: usize,
    episode: usize,
) -> Option<serde_json::Value> {
    let path = get_cache_path(subject_id, season, episode);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = SystemTime::now().duration_since(modified) {
                    if duration.as_secs() > CACHE_EXPIRY_SECS {
                        let _ = fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Some(val);
            }
        }
    }
    None
}

pub fn get_details_path(subject_id: &str) -> PathBuf {
    let mut path = get_cache_dir("details");
    path.push(format!("details_{}.json", subject_id));
    path
}

pub fn get_details_cache(subject_id: &str) -> Option<serde_json::Value> {
    let path = get_details_path(subject_id);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = SystemTime::now().duration_since(modified) {
                    if duration.as_secs() > CACHE_EXPIRY_SECS {
                        let _ = fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Some(val);
            }
        }
    }
    None
}

pub fn get_search_path(query: &str) -> PathBuf {
    let mut path = get_cache_dir("search");
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(query.as_bytes());
    let safe_query: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    path.push(format!("search_{}.json", safe_query));
    path
}

pub fn get_search_cache(query: &str) -> Option<serde_json::Value> {
    let path = get_search_path(query);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = SystemTime::now().duration_since(modified) {
                    if duration.as_secs() > CACHE_EXPIRY_SECS {
                        let _ = fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Some(val);
            }
        }
    }
    None
}

pub fn set_search_cache(query: &str, data: &serde_json::Value) {
    let path = get_search_path(query);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn set_details_cache(subject_id: &str, data: &serde_json::Value) {
    let path = get_details_path(subject_id);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn set_stream_cache(subject_id: &str, season: usize, episode: usize, data: &serde_json::Value) {
    let path = get_cache_path(subject_id, season, episode);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn invalidate_stream_cache(subject_id: &str, season: usize, episode: usize) {
    let path = get_cache_path(subject_id, season, episode);
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
}

pub fn clear_all_cache() {
    let mut path = dirs::cache_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("moviebox-tui");
    if path.exists() {
        let _ = fs::remove_dir_all(&path);
    }
}

fn get_homepage_path(tab_id: &str, page: usize) -> PathBuf {
    let mut path = get_cache_dir("homepage");
    path.push(format!("{}_{}.json", tab_id, page));
    path
}

pub fn get_homepage_cache(tab_id: &str, page: usize) -> Option<serde_json::Value> {
    let path = get_homepage_path(tab_id, page);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 3600 {
                        let _ = fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Some(val);
            }
        }
    }
    None
}

pub fn set_homepage_cache(tab_id: &str, page: usize, data: &serde_json::Value) {
    let path = get_homepage_path(tab_id, page);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}
