use super::App;
use crate::providers::models::ProviderKind;
use crate::tui::{action::Action, overlay::NotificationKind, state::Screen};

impl App {
    fn preferred_playback_player(
        &self,
        source: &crate::providers::models::PlaybackSource,
    ) -> Option<crate::tui::state::PlayerKind> {
        self.state
            .available_players
            .iter()
            .copied()
            .find(|kind| crate::tui::player::supports_headers(*kind, &source.headers))
    }

    fn build_watch_history_item(&self) -> Option<crate::history::WatchHistoryItem> {
        let subject_id = self.state.active_subject_id.as_ref()?;
        let provider = self.provider_for_subject(subject_id).cache_key();
        let season = self.state.selected_season;
        let episode = self.state.selected_episode;
        let mut title = "Unknown".to_string();
        let mut cover_url = None;
        let mut stype = 1;
        let mut release_year = "Unknown".to_string();

        if let Some(details) = &self.state.selected_details {
            if let Some(t) = details.get("title").and_then(|t| t.as_str()) {
                title = crate::providers::moviebox::clean_moviebox_title(t);
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

        Some(crate::history::WatchHistoryItem {
            provider: provider.to_string(),
            subject_id: subject_id.clone(),
            title,
            cover_url,
            stype,
            release_year,
            season,
            episode,
            timestamp,
        })
    }

    pub(super) fn launch_player(
        &mut self,
        kind: crate::tui::state::PlayerKind,
        link: String,
        subtitle: Option<String>,
        headers: Vec<(String, String)>,
    ) {
        let history_item = self.build_watch_history_item();

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
                    let base_dir = if let Some(home) = dirs::home_dir() {
                        let storage = home.join("storage/downloads/moviebox_subs");
                        if home.join("storage/downloads").exists() {
                            let _ = std::fs::create_dir_all(&storage);
                            storage
                        } else {
                            std::env::temp_dir().join("moviebox-tui/subs")
                        }
                    } else {
                        std::env::temp_dir().join("moviebox-tui/subs")
                    };
                    let _ = std::fs::create_dir_all(&base_dir);
                    let path = base_dir.join(format!(
                        "{}_{}.{}",
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
                    if let Some(item) = history_item {
                        sender.send(Action::MarkWatched(item)).ok();
                    }
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

impl App {
    pub(super) async fn handle_playback(&mut self, action: Action) -> Option<()> {
        match action {
            Action::PlayStream(open_with) => {
                if self.state.is_resolving_playback {
                    return None;
                }
                self.state.is_resolving_playback = true;
                if self.current_subject_provider() == ProviderKind::FourKHdHub {
                    if let Some(release) = self.get_selected_release() {
                        let Some(first_mirror) = release.mirrors.first().cloned() else {
                            self.state.is_resolving_playback = false;
                            self.state.notify(
                                NotificationKind::Error,
                                "Playback unavailable",
                                "No playable mirrors were found for this release.",
                            );
                            return None;
                        };
                        self.state.notify(
                            NotificationKind::Info,
                            "Preparing playback",
                            "Resolving the selected mirror.",
                        );
                        let default_player = self.preferred_playback_player(
                            &crate::providers::models::PlaybackSource::bare(
                                release.provider,
                                first_mirror.resolver_url.clone(),
                                None,
                            ),
                        );
                        let available_players = self.state.available_players.clone();
                        let client = if release.provider == ProviderKind::BdixCircleFtp {
                            let sender_clone = self.action_sender.clone();
                            let source = crate::providers::models::PlaybackSource::bare(
                                ProviderKind::BdixCircleFtp,
                                first_mirror.resolver_url.clone(),
                                None,
                            );
                            if open_with || default_player.is_none() {
                                sender_clone.send(Action::ShowPlaybackPicker(source)).ok();
                            } else if let Some(player) = default_player {
                                sender_clone
                                    .send(Action::LaunchPlayback(player, source))
                                    .ok();
                            }
                            return None;
                        } else {
                            match self.fourk_client.clone() {
                                Some(client) => client,
                                None => {
                                    self.state.is_resolving_playback = false;
                                    self.action_sender
                                        .send(Action::SetStatus(
                                            "Error: 4KHDHub provider is unavailable".to_string(),
                                        ))
                                        .ok();
                                    return None;
                                }
                            }
                        };
                        let sender = self.action_sender.clone();
                        tokio::spawn(async move {
                            match client.resolve_release(&release).await {
                                Ok(source) => {
                                    let default_player =
                                        available_players.iter().copied().find(|kind| {
                                            crate::tui::player::supports_headers(
                                                *kind,
                                                &source.headers,
                                            )
                                        });
                                    if open_with || default_player.is_none() {
                                        sender.send(Action::ShowPlaybackPicker(source)).ok();
                                    } else if let Some(player) = default_player {
                                        sender.send(Action::LaunchPlayback(player, source)).ok();
                                    }
                                }
                                Err(error) => {
                                    log::error!("4KHDHub resolve failed: {error}");
                                    sender
                                        .send(Action::SetStatus(format!(
                                            "Error: 4KHDHub source failed: {error}"
                                        )))
                                        .ok();
                                }
                            }
                        });
                    } else {
                        self.state.is_resolving_playback = false;
                    }
                    return None;
                }
                if self.state.active_screen == Screen::Details
                    && let Some(link) = self.get_selected_link()
                {
                    let subject_id = self
                        .state
                        .selected_details
                        .as_ref()
                        .and_then(|d| d.get("id"))
                        .and_then(crate::tui::state::subject_id)
                        .unwrap_or_default();
                    let resource_id = self.get_selected_resource_id();

                    if let Some(rid) = resource_id {
                        self.state.notify(
                            NotificationKind::Info,
                            "Preparing playback",
                            "Fetching subtitles.",
                        );
                        let client = self.client.clone();
                        let sender = self.action_sender.clone();
                        let link_clone = link.clone();
                        tokio::spawn(async move {
                            let cached = tokio::task::spawn_blocking({
                                let subject_id = subject_id.clone();
                                let rid = rid.clone();
                                move || crate::cache::get_captions_cache(&subject_id, &rid)
                            })
                            .await
                            .ok()
                            .flatten();
                            if let Some(res) = cached {
                                sender
                                    .send(Action::ShowSubtitlePopup(
                                        link_clone.clone(),
                                        res,
                                        open_with,
                                    ))
                                    .ok();
                                return;
                            }
                            let result = tokio::time::timeout(
                                std::time::Duration::from_secs(15),
                                client.get_ext_captions(&subject_id, &rid),
                            )
                            .await;
                            match result {
                                Ok(Ok(res)) => {
                                    let subject_id = subject_id.clone();
                                    let rid = rid.clone();
                                    let res_for_cache = res.clone();
                                    tokio::task::spawn_blocking(move || {
                                        crate::cache::set_captions_cache(
                                            &subject_id,
                                            &rid,
                                            &res_for_cache,
                                        );
                                    });
                                    sender
                                        .send(Action::ShowSubtitlePopup(link_clone, res, open_with))
                                        .ok();
                                }
                                _ => {
                                    if open_with {
                                        sender
                                            .send(Action::ShowPlayerPicker(link_clone, None))
                                            .ok();
                                    } else {
                                        sender.send(Action::LaunchMpv(link_clone, None)).ok();
                                    }
                                }
                            }
                        });
                    } else {
                        if open_with {
                            self.action_sender
                                .send(Action::ShowPlayerPicker(link, None))
                                .ok();
                        } else {
                            self.action_sender.send(Action::LaunchMpv(link, None)).ok();
                        }
                    }
                } else {
                    self.state.is_resolving_playback = false;
                }
            }
            Action::ShowSubtitlePopup(link, ext_captions, open_with) => {
                self.state.is_resolving_playback = false;
                let options = crate::tui::state::caption_options(&ext_captions);

                if options.len() > 1 {
                    self.state.show_help = false;
                    self.state.player_picker_popup = false;
                    self.state.is_download_subtitle_popup = false;
                    self.state.subtitle_popup = true;
                    self.state.subtitle_list = options;
                    self.state.subtitle_list_state.select(Some(0));
                    self.state.pending_play_link = Some(link);
                    self.state.pending_open_with = open_with;
                } else {
                    if open_with {
                        self.action_sender
                            .send(Action::ShowPlayerPicker(link, None))
                            .ok();
                    } else {
                        self.action_sender.send(Action::LaunchMpv(link, None)).ok();
                    }
                }
            }
            Action::ShowDownloadSubtitlePopup(ext_captions) => {
                self.state.is_resolving_playback = false;
                let options = crate::tui::state::caption_options(&ext_captions);

                if options.len() > 1 {
                    self.state.show_help = false;
                    self.state.player_picker_popup = false;
                    self.state.subtitle_popup = false;
                    self.state.is_download_subtitle_popup = true;
                    self.state.subtitle_list = options;
                    self.state.subtitle_list_state.select(Some(0));
                } else {
                    self.action_sender.send(Action::DownloadStream(None)).ok();
                }
            }
            Action::LaunchMpv(link, subtitle_url) => {
                self.state.is_resolving_playback = false;
                let player = self.state.available_players.first().cloned();
                match player {
                    None => {
                        self.state.notify(
                            NotificationKind::Error,
                            "Player unavailable",
                            "Install mpv, IINA, or VLC.",
                        );
                    }
                    Some(kind) => {
                        let player_name = match kind {
                            crate::tui::state::PlayerKind::Mpv => "MPV",
                            crate::tui::state::PlayerKind::Iina => "IINA",
                            crate::tui::state::PlayerKind::Vlc => "VLC",
                            crate::tui::state::PlayerKind::AndroidIntent => "Android Player",
                        };
                        self.state.notify(
                            NotificationKind::Info,
                            "Opening player",
                            format!("Launching {player_name}."),
                        );

                        self.action_sender
                            .send(Action::LaunchPlayer(kind, link, subtitle_url))
                            .ok();
                    }
                }
            }

            Action::ShowPlaybackPicker(source) => {
                self.state.is_resolving_playback = false;
                if self.state.available_players.is_empty() {
                    self.state.set_status(
                        "No media player found. Install mpv, IINA, VLC, or use Android Player.",
                        150,
                    );
                    return None;
                }
                self.state.show_help = false;
                self.state.tv_config_popup = false;
                self.state.player_picker_popup = true;
                self.state.player_picker_playback = Some(source);
                self.state.player_picker_link = None;
                self.state.player_picker_subtitle = None;
                self.state.player_picker_state.select(Some(0));
                self.state.subtitle_popup = false;
            }
            Action::ShowPlayerPicker(link, subtitle) => {
                self.state.is_resolving_playback = false;
                if self.state.available_players.is_empty() {
                    self.state.notify(
                        NotificationKind::Error,
                        "Player unavailable",
                        "Install mpv, IINA, VLC, or use Android Player.",
                    );
                    return None;
                }
                self.state.show_help = false;
                self.state.tv_config_popup = false;
                self.state.player_picker_popup = true;
                self.state.player_picker_playback = None;
                self.state.player_picker_link = Some(link);
                self.state.player_picker_subtitle = subtitle;
                self.state.player_picker_state.select(Some(0));
                self.state.subtitle_popup = false;
            }
            Action::LaunchPlayer(kind, link, sub) => {
                self.state.is_resolving_playback = false;
                self.state.player_picker_popup = false;
                self.launch_player(kind, link, sub, Vec::new());
            }
            Action::LaunchPlayback(kind, source) => {
                self.state.is_resolving_playback = false;
                self.state.player_picker_popup = false;
                if !crate::tui::player::supports_headers(kind, &source.headers) {
                    self.state.set_status(
                        format!(
                            "This source needs headers {} cannot provide; use mpv or IINA.",
                            kind.label()
                        ),
                        180,
                    );
                    return None;
                }
                self.launch_player(kind, source.url, source.subtitle, source.headers);
            }
            Action::MarkWatched(item) => {
                self.state.history.mark_watched(item);
                let history = self.state.history.clone();
                tokio::task::spawn_blocking(move || history.save());
            }
            Action::PlayerCrashed(code, error_msg) => {
                let code_str = code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".into());
                log::error!("player crashed (code {code_str}): {error_msg}");

                let display_err = if error_msg.is_empty() {
                    "No error output provided by player.".to_string()
                } else {
                    error_msg.lines().last().unwrap_or(&error_msg).to_string()
                };

                self.state.set_status(
                    format!("Player crashed (code {code_str}): {display_err}"),
                    300,
                );

                self.state.notify(
                    NotificationKind::Error,
                    "Player Error",
                    format!("Crash code: {code_str}\n{display_err}"),
                );
            }
            _ => return None,
        }
        None
    }
}
