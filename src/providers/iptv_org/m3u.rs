use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub logo: String,
    pub group: String,
    pub stream_url: String,
}

pub struct M3UParser {
    cache_dir: PathBuf,
}

impl M3UParser {
    pub fn new() -> Self {
        let mut cache_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        cache_dir.push(".moviebox");
        cache_dir.push("tv_playlists");
        fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    pub async fn fetch_playlist(&self, url: &str, filename: &str) -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
        let file_path = self.cache_dir.join(filename);
        let mut needs_download = true;

        if file_path.exists() {
            if let Ok(metadata) = fs::metadata(&file_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = SystemTime::now().duration_since(modified) {
                        if duration.as_secs() < 24 * 3600 {
                            needs_download = false;
                        }
                    }
                }
            }
        }

        let content = if needs_download {
            let client = reqwest::Client::new();
            let res = client.get(url).send().await?.text().await?;
            fs::write(&file_path, &res).ok();
            res
        } else {
            fs::read_to_string(&file_path)?
        };

        Ok(self.parse_m3u(&content))
    }

    fn parse_m3u(&self, content: &str) -> Vec<Channel> {
        let mut channels = Vec::new();
        let mut current_channel = Channel {
            id: String::new(),
            name: String::new(),
            logo: String::new(),
            group: String::new(),
            stream_url: String::new(),
        };

        let tvg_id_re = Regex::new(r#"tvg-id="([^"]*)""#).unwrap();
        let tvg_logo_re = Regex::new(r#"tvg-logo="([^"]*)""#).unwrap();
        let group_title_re = Regex::new(r#"group-title="([^"]*)""#).unwrap();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("#EXTINF:") {
                if let Some(caps) = tvg_id_re.captures(line) {
                    current_channel.id = caps.get(1).map_or("", |m| m.as_str()).to_string();
                }
                if let Some(caps) = tvg_logo_re.captures(line) {
                    current_channel.logo = caps.get(1).map_or("", |m| m.as_str()).to_string();
                }
                if let Some(caps) = group_title_re.captures(line) {
                    current_channel.group = caps.get(1).map_or("", |m| m.as_str()).to_string();
                }
                
                if let Some(idx) = line.rfind(',') {
                    current_channel.name = line[idx + 1..].trim().to_string();
                }
            } else if !line.starts_with('#') {
                current_channel.stream_url = line.to_string();
                if current_channel.id.is_empty() {
                    current_channel.id = current_channel.name.clone();
                }
                channels.push(current_channel.clone());

                current_channel = Channel {
                    id: String::new(),
                    name: String::new(),
                    logo: String::new(),
                    group: String::new(),
                    stream_url: String::new(),
                };
            }
        }

        channels
    }
}
