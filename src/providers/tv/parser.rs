use super::models::Channel;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct M3UParser {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl Default for M3UParser {
    fn default() -> Self {
        Self::new()
    }
}

impl M3UParser {
    pub fn new() -> Self {
        let mut cache_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
        cache_dir.push(crate::config::APP_NAME);
        cache_dir.push("tv_playlists");
        std::fs::create_dir_all(&cache_dir).ok();
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { cache_dir, client }
    }

    pub async fn fetch_playlist(
        &self,
        url: &str,
    ) -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
        let trimmed = url.trim();
        let is_remote = crate::tui::text::is_http_url(trimmed);
        let content = if is_remote {
            let file_path = self.cache_dir.join(cache_filename(trimmed));
            let mut needs_download = true;

            if file_path.exists() {
                if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = SystemTime::now().duration_since(modified) {
                            if duration.as_secs() < 24 * 3600 {
                                needs_download = false;
                            }
                        }
                    }
                }
            }

            if needs_download {
                let res = self
                    .client
                    .get(trimmed)
                    .send()
                    .await?
                    .error_for_status()?
                    .text()
                    .await?;
                let _ = crate::cache::atomic_write_file_async(&file_path, res.as_bytes()).await;
                res
            } else {
                match tokio::fs::read_to_string(&file_path).await {
                    Ok(content) => content,
                    Err(_) => {
                        let _ = tokio::fs::remove_file(&file_path).await;
                        self.client
                            .get(trimmed)
                            .send()
                            .await?
                            .error_for_status()?
                            .text()
                            .await?
                    }
                }
            }
        } else {
            let path = std::path::PathBuf::from(trimmed);
            tokio::fs::read_to_string(&path)
                .await
                .map_err(|error| format!("failed to read playlist file {}: {error}", trimmed))?
        };

        let channels = self.parse_m3u(&content);
        if is_remote && channels.is_empty() {
            let file_path = self.cache_dir.join(cache_filename(trimmed));
            let _ = tokio::fs::remove_file(&file_path).await;
            let fresh = self
                .client
                .get(trimmed)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            let fresh_channels = self.parse_m3u(&fresh);
            if fresh_channels.is_empty() {
                return Ok(fresh_channels);
            }
            let _ = crate::cache::atomic_write_file_async(&file_path, fresh.as_bytes()).await;
            return Ok(fresh_channels);
        }
        Ok(channels)
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

        let extract_attr = |line: &str, attr: &str| -> String {
            if let Some(idx) = line.find(attr) {
                let start = idx + attr.len();
                if let Some(end) = line[start..].find('"') {
                    return line[start..start + end].to_string();
                }
            }
            String::new()
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("#EXTINF:") {
                current_channel.id = extract_attr(line, "tvg-id=\"");
                current_channel.logo = extract_attr(line, "tvg-logo=\"");
                current_channel.group = extract_attr(line, "group-title=\"");

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

fn cache_filename(raw: &str) -> String {
    format!("{}.m3u", crate::cache::md5_hex(raw))
}
