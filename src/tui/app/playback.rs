use super::App;
use crate::tui::action::Action;

impl App {
    pub(super) fn launch_player(
        &mut self,
        kind: crate::tui::state::PlayerKind,
        link: String,
        subtitle: Option<String>,
        headers: Vec<(String, String)>,
    ) {
        if let Some(subject_id) = &self.state.active_subject_id {
            let season = self.state.selected_season;
            let episode = self.state.selected_episode;
            let provider = self.state.active_provider.cache_key();

            let mut title = "Unknown".to_string();
            let mut cover_url = None;
            let mut stype = 1;
            let mut release_year = "Unknown".to_string();

            if let Some(details) = &self.state.selected_details {
                if let Some(t) = details.get("title").and_then(|t| t.as_str()) {
                    title = t.to_string();
                }
                cover_url = details
                    .get("poster")
                    .or_else(|| details.get("cover"))
                    .or_else(|| details.get("pic"))
                    .and_then(|c| c.as_str().or_else(|| c.get("url").and_then(|u| u.as_str())))
                    .map(|s| s.to_string());
                stype = crate::tui::state::stype(details);
                if let Some(y) = details
                    .get("year")
                    .or_else(|| details.get("releaseYear"))
                    .and_then(|y| y.as_str())
                {
                    release_year = y.to_string();
                } else if let Some(y) = details.get("year").and_then(|y| y.as_i64()) {
                    release_year = y.to_string();
                }
            }

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            self.state
                .history
                .mark_watched(crate::history::WatchHistoryItem {
                    provider: provider.to_string(),
                    subject_id: subject_id.clone(),
                    title,
                    cover_url,
                    stype,
                    release_year,
                    season,
                    episode,
                    timestamp,
                });
            let history = self.state.history.clone();
            tokio::task::spawn_blocking(move || history.save());
            self.state.dirty = true;
        }

        let client = self.client.http_client().clone();
        let sender = self.action_sender.clone();
        let cell_size = self
            .state
            .image_picker
            .as_ref()
            .map(|picker| picker.font_size());
        let window = crossterm::terminal::size().ok().map(|(cols, rows)| {
            let (cell_width, cell_height) = cell_size
                .filter(|size| size.width > 0 && size.height > 0)
                .map(|size| (size.width as u32, size.height as u32))
                .unwrap_or((8, 16));
            (
                (cols as u32 * cell_width).clamp(320, 1920),
                (rows as u32 * cell_height).clamp(180, 1080),
            )
        });
        tokio::spawn(async move {
            let mut local_subtitle = subtitle.clone();
            let mut temporary_subtitle = None;
            if matches!(
                kind,
                crate::tui::state::PlayerKind::Vlc | crate::tui::state::PlayerKind::Iina
            ) && let Some(url) = subtitle
            {
                let mut request = client.get(&url);
                for (name, value) in &headers {
                    request = request.header(name.as_str(), value.as_str());
                }
                let mut downloaded = false;
                if let Ok(Ok(response)) =
                    tokio::time::timeout(std::time::Duration::from_secs(30), request.send()).await
                    && let Ok(response) = response.error_for_status()
                    && let Ok(bytes) = response.bytes().await
                {
                    let extension = url
                        .rsplit('.')
                        .next()
                        .map(|e| e.to_ascii_lowercase())
                        .filter(|e| matches!(e.as_str(), "srt" | "vtt" | "ass" | "ssa" | "sub"))
                        .unwrap_or_else(|| "srt".to_string());
                    let path = std::env::temp_dir().join(format!(
                        "moviebox_sub_{}_{}.{}",
                        std::process::id(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos(),
                        extension
                    ));
                    if tokio::fs::write(&path, bytes).await.is_ok() {
                        local_subtitle = Some(path.to_string_lossy().into_owned());
                        temporary_subtitle = Some(path);
                        downloaded = true;
                    }
                }
                if !downloaded {
                    log::warn!(
                        "subtitle download failed for {:?} player, playing without subtitles (url was {})",
                        kind,
                        crate::logging::sanitize_url(&url)
                    );
                    let _ = sender.send(Action::SetStatus(
                        "Subtitles unavailable; playing without subtitles.".to_string(),
                    ));
                }
            }

            let mut command = crate::tui::player::command(
                kind,
                &link,
                local_subtitle.as_deref(),
                &headers,
                window,
            );
            command.stdin(std::process::Stdio::null());
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::piped());

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }

            match command.spawn() {
                Ok(mut child) => {
                    let start_time = std::time::Instant::now();
                    let stderr_stream = child.stderr.take();

                    tokio::task::spawn_blocking(move || {
                        let mut error_output = String::new();
                        if let Some(mut stderr) = stderr_stream {
                            use std::io::Read;
                            let _ = stderr.read_to_string(&mut error_output);
                        }

                        let result = child.wait();

                        if let Ok(status) = result {
                            let clean_error = error_output.trim().to_string();
                            if !status.success()
                                && start_time.elapsed().as_secs() < 3
                                && !clean_error.is_empty()
                            {
                                sender
                                    .send(Action::PlayerCrashed(status.code(), clean_error))
                                    .ok();
                            }
                        }

                        if let Some(path) = temporary_subtitle {
                            let _ = std::fs::remove_file(path);
                        }
                    });
                }
                Err(error) => {
                    log::error!(
                        "failed to spawn player {:?} for {}: {error}",
                        kind,
                        crate::logging::sanitize_url(&link)
                    );
                    if let Some(path) = temporary_subtitle {
                        let _ = tokio::fs::remove_file(path).await;
                    }
                    sender
                        .send(Action::PlayerCrashed(
                            None,
                            format!("Failed to spawn player executable: {error}"),
                        ))
                        .ok();
                }
            }
        });
    }
}
