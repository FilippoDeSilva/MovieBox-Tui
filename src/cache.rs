use crate::providers::models::ProviderKind;
use std::fs;
use std::path::PathBuf;

const CACHE_EXPIRY_SECS: u64 = 24 * 60 * 60;

fn read_json_cache(path: &PathBuf, expiry_secs: u64) -> Option<serde_json::Value> {
    if path.exists() {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > expiry_secs {
                        let _ = fs::remove_file(path);
                        return None;
                    }
                }
            }
        }
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return Some(val);
            }
        }
    }
    None
}

pub fn get_cache_dir(subdir: &str) -> PathBuf {
    get_provider_cache_dir(ProviderKind::MovieBox, subdir)
}

pub fn get_provider_cache_dir(provider: ProviderKind, subdir: &str) -> PathBuf {
    let mut path = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    path.push("moviebox-tui");
    path.push(provider.cache_key());
    path.push(subdir);
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn get_cache_path(subject_id: &str, season: usize, episode: usize) -> PathBuf {
    get_provider_stream_path(ProviderKind::MovieBox, subject_id, season, episode)
}

pub fn get_provider_stream_path(
    provider: ProviderKind,
    subject_id: &str,
    season: usize,
    episode: usize,
) -> PathBuf {
    let mut path = get_provider_cache_dir(provider, "streams");
    let schema = if provider == ProviderKind::FourKHdHub {
        "v3_"
    } else {
        ""
    };
    path.push(format!("{schema}{subject_id}_{season}_{episode}.json"));
    path
}

pub fn get_stream_cache(
    subject_id: &str,
    season: usize,
    episode: usize,
) -> Option<serde_json::Value> {
    read_json_cache(
        &get_cache_path(subject_id, season, episode),
        CACHE_EXPIRY_SECS,
    )
}

pub fn get_provider_stream_cache(
    provider: ProviderKind,
    subject_id: &str,
    season: usize,
    episode: usize,
) -> Option<serde_json::Value> {
    read_json_cache(
        &get_provider_stream_path(provider, subject_id, season, episode),
        CACHE_EXPIRY_SECS,
    )
}

pub fn get_details_path(subject_id: &str) -> PathBuf {
    get_provider_details_path(ProviderKind::MovieBox, subject_id)
}

pub fn get_provider_details_path(provider: ProviderKind, subject_id: &str) -> PathBuf {
    let mut path = get_provider_cache_dir(provider, "details");
    let schema = if provider == ProviderKind::FourKHdHub {
        "v2_"
    } else {
        ""
    };
    path.push(format!("details_{schema}{subject_id}.json"));
    path
}

pub fn get_details_cache(subject_id: &str) -> Option<serde_json::Value> {
    read_json_cache(&get_details_path(subject_id), CACHE_EXPIRY_SECS)
}

pub fn get_provider_details_cache(
    provider: ProviderKind,
    subject_id: &str,
) -> Option<serde_json::Value> {
    read_json_cache(
        &get_provider_details_path(provider, subject_id),
        CACHE_EXPIRY_SECS,
    )
}

pub fn get_search_path(query: &str) -> PathBuf {
    get_provider_search_path(ProviderKind::MovieBox, query)
}

pub fn get_provider_search_path(provider: ProviderKind, query: &str) -> PathBuf {
    let mut path = get_provider_cache_dir(provider, "search");
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(query.as_bytes());
    let result = hasher.finalize();
    let mut safe_query = String::with_capacity(32);
    for b in result {
        use std::fmt::Write;
        let _ = write!(&mut safe_query, "{:02x}", b);
    }
    path.push(format!("search_{}.json", safe_query));
    path
}

pub fn get_search_cache(query: &str) -> Option<serde_json::Value> {
    read_json_cache(&get_search_path(query), CACHE_EXPIRY_SECS)
}

pub fn get_provider_search_cache(provider: ProviderKind, query: &str) -> Option<serde_json::Value> {
    let path = get_provider_search_path(provider, query);
    let value = read_json_cache(&path, CACHE_EXPIRY_SECS)?;
    if search_payload_has_results(&value) {
        Some(value)
    } else {
        let _ = fs::remove_file(path);
        None
    }
}

pub fn set_search_cache(query: &str, data: &serde_json::Value) {
    let path = get_search_path(query);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn set_provider_search_cache(provider: ProviderKind, query: &str, data: &serde_json::Value) {
    let path = get_provider_search_path(provider, query);
    if !search_payload_has_results(data) {
        let _ = fs::remove_file(path);
        return;
    }
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(path, content);
    }
}

fn search_payload_has_results(data: &serde_json::Value) -> bool {
    data.get("results")
        .and_then(|results| results.as_array())
        .and_then(|results| results.first())
        .and_then(|result| result.get("subjects"))
        .and_then(|subjects| subjects.as_array())
        .is_some_and(|subjects| !subjects.is_empty())
}

pub fn set_details_cache(subject_id: &str, data: &serde_json::Value) {
    let path = get_details_path(subject_id);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn set_provider_details_cache(
    provider: ProviderKind,
    subject_id: &str,
    data: &serde_json::Value,
) {
    let path = get_provider_details_path(provider, subject_id);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(path, content);
    }
}

pub fn set_stream_cache(subject_id: &str, season: usize, episode: usize, data: &serde_json::Value) {
    let path = get_cache_path(subject_id, season, episode);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}

pub fn set_provider_stream_cache(
    provider: ProviderKind,
    subject_id: &str,
    season: usize,
    episode: usize,
    data: &serde_json::Value,
) {
    let path = get_provider_stream_path(provider, subject_id, season, episode);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(path, content);
    }
}

pub fn invalidate_stream_cache(subject_id: &str, season: usize, episode: usize) {
    let path = get_cache_path(subject_id, season, episode);
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
}

pub fn clear_all_cache() {
    let mut path = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    path.push("moviebox-tui");
    if path.exists() {
        let _ = fs::remove_dir_all(&path);
    }
}

fn get_homepage_path(tab_id: &str, page: usize) -> PathBuf {
    let mut path = get_provider_cache_dir(ProviderKind::MovieBox, "homepage");
    path.push(format!("{}_{}.json", tab_id, page));
    path
}

pub fn get_homepage_cache(tab_id: &str, page: usize) -> Option<serde_json::Value> {
    read_json_cache(&get_homepage_path(tab_id, page), 3600)
}

pub fn set_homepage_cache(tab_id: &str, page: usize, data: &serde_json::Value) {
    let path = get_homepage_path(tab_id, page);
    if let Ok(content) = serde_json::to_string(data) {
        let _ = fs::write(&path, content);
    }
}
