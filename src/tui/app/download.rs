use super::App;
use crate::tui::{action::Action, overlay::NotificationKind, state::Screen};

impl App {
    pub(super) fn start_resilient_download(
        &mut self,
        subtitle_url: Option<String>,
        link: Option<String>,
    ) {
        if self.state.download_progress.is_some() || self.state.active_screen != Screen::Details {
            return;
        }
        let Some(link) = link else {
            if self.state.is_fetching_streams {
                self.state.is_waiting_for_download_stream = true;
                self.state.notify(
                    NotificationKind::Info,
                    "Preparing download",
                    "Waiting for stream details.",
                );
            } else {
                self.state.notify(
                    NotificationKind::Warning,
                    "Download unavailable",
                    "Select a downloadable stream first.",
                );
            }
            return;
        };

        let title = self
            .state
            .selected_details
            .as_ref()
            .and_then(|details| details.get("title"))
            .and_then(|title| title.as_str())
            .unwrap_or("MovieBox-Tui_Stream");
        let media_type = self
            .state
            .selected_details
            .as_ref()
            .map(crate::tui::state::stype)
            .unwrap_or(1);
        let season = self.state.selected_season;
        let episode = self.state.selected_episode;
        let clean_title = crate::tui::app::clean_moviebox_title(title);
        let safe_title = crate::download::safe_file_stem(&clean_title);

        let extension = self
            .state
            .selected_resources
            .as_ref()
            .and_then(|resources| resources.get("list"))
            .and_then(|list| list.as_array())
            .and_then(|list| list.get(self.state.resource_list_state.selected().unwrap_or(0)))
            .and_then(|resource| {
                resource
                    .get("fileName")
                    .or_else(|| resource.get("title"))
                    .and_then(|name| name.as_str())
            })
            .and_then(|name| std::path::Path::new(name).extension())
            .and_then(|extension| extension.to_str())
            .filter(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v"
                )
            })
            .unwrap_or("mp4")
            .to_ascii_lowercase();

        let base_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        #[cfg(target_os = "android")]
        let base_dir = if let Some(home) = dirs::home_dir() {
            let android_storage = home.join("storage/downloads");
            if android_storage.exists() {
                android_storage
            } else {
                base_dir
            }
        } else {
            base_dir
        };

        let base_dir = base_dir.join("MovieBox-TUI");
        let (target_dir, base_name) = if media_type == 2 {
            (
                base_dir
                    .join("Series")
                    .join(&safe_title)
                    .join(format!("Season {season}")),
                format!("{safe_title}_S{season:02}E{episode:02}"),
            )
        } else {
            (base_dir.join("Movies"), safe_title)
        };
        let mut destination = target_dir.join(format!("{base_name}.{extension}"));
        let mut counter = 2;
        while destination.exists() {
            destination = target_dir.join(format!("{base_name}_{counter}.{extension}"));
            counter += 1;
        }

        self.state.is_waiting_for_download_stream = false;
        self.state.download_status = Some("Preparing download...".into());
        self.state.download_progress = Some(0.0);
        self.state
            .cancel_download
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.state.notify(
            NotificationKind::Info,
            "Download started",
            "Partial data will be preserved.",
        );

        let cancel = self.state.cancel_download.clone();
        let sender = self.action_sender.clone();
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| self.client.http_client().clone());

        tokio::spawn(async move {
            if let Err(error) = tokio::fs::create_dir_all(&target_dir).await {
                sender
                    .send(Action::DownloadFailed(format!(
                        "Cannot create download directory: {error}"
                    )))
                    .ok();
                return;
            }

            if let Some(subtitle_url) = subtitle_url {
                let subtitle_path = destination.with_extension("srt");
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    client.get(subtitle_url).send(),
                )
                .await;
                match result {
                    Ok(Ok(response)) => match response.error_for_status() {
                        Ok(response) => match response.bytes().await {
                            Ok(bytes) => {
                                if let Err(error) = tokio::fs::write(subtitle_path, bytes).await {
                                    sender
                                        .send(Action::SetStatus(format!(
                                            "Error: subtitle write failed: {error}"
                                        )))
                                        .ok();
                                }
                            }
                            Err(error) => {
                                sender
                                    .send(Action::SetStatus(format!(
                                        "Error: subtitle download failed: {error}"
                                    )))
                                    .ok();
                            }
                        },
                        Err(error) => {
                            sender
                                .send(Action::SetStatus(format!(
                                    "Error: subtitle download failed: {error}"
                                )))
                                .ok();
                        }
                    },
                    Ok(Err(error)) => {
                        sender
                            .send(Action::SetStatus(format!(
                                "Error: subtitle download failed: {error}"
                            )))
                            .ok();
                    }
                    Err(_) => {
                        sender
                            .send(Action::SetStatus(
                                "Error: subtitle download timed out".to_string(),
                            ))
                            .ok();
                    }
                }
            }

            let progress_sender = sender.clone();
            let result =
                crate::download::download(&client, &link, &destination, cancel, move |progress| {
                    let total = progress.total.unwrap_or_default();
                    let percentage = if total > 0 {
                        progress.downloaded as f64 / total as f64 * 100.0
                    } else {
                        0.0
                    };
                    let speed = progress.bytes_per_second / 1024.0 / 1024.0;
                    let eta = if total > progress.downloaded && progress.bytes_per_second > 0.0 {
                        (total - progress.downloaded) as f64 / progress.bytes_per_second
                    } else {
                        0.0
                    };
                    let status = if total > 0 {
                        format!(
                            "{:.1}/{:.1} MB | {:.1} MB/s | ETA {:.0}s | {}x | attempt {}",
                            progress.downloaded as f64 / 1024.0 / 1024.0,
                            total as f64 / 1024.0 / 1024.0,
                            speed,
                            eta,
                            progress.workers,
                            progress.attempt
                        )
                    } else {
                        format!(
                            "{:.1} MB | {:.1} MB/s | {}x | attempt {}",
                            progress.downloaded as f64 / 1024.0 / 1024.0,
                            speed,
                            progress.workers,
                            progress.attempt
                        )
                    };
                    progress_sender
                        .send(Action::UpdateDownload(Some(percentage), Some(status)))
                        .ok();
                })
                .await;

            match result {
                Ok(crate::download::DownloadOutcome::Completed { .. }) => {
                    sender
                        .send(Action::DownloadCompleted(
                            destination.to_string_lossy().into_owned(),
                        ))
                        .ok();
                }
                Ok(crate::download::DownloadOutcome::Paused { .. }) => {
                    sender
                        .send(Action::DownloadPaused(
                            destination.to_string_lossy().into_owned(),
                        ))
                        .ok();
                }
                Err(error) => {
                    sender.send(Action::DownloadFailed(error.to_string())).ok();
                }
            }
        });
    }
}
