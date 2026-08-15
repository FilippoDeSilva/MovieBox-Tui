use tokio::sync::mpsc;

use crate::providers::{
    fourkhdhub::FourKHdHubClient, models::RequestContext, moviebox::client::MovieBoxClient,
};
use crate::tui::{action::Action, state::AppState, theme::Theme};

mod download;
mod keyboard;
mod mouse;
mod navigation;
mod network;
mod playback;
mod requests;
mod run;
mod search;
mod system;
mod tv;

pub struct App {
    state: AppState,
    theme: Theme,
    client: MovieBoxClient,
    fourk_client: Option<FourKHdHubClient>,
    circleftp_client: crate::providers::bdix::circleftp::CircleFtpClient,
    dhakaflix_client: crate::providers::bdix::dhakaflix::client::DhakaFlixClient,
    action_sender: mpsc::UnboundedSender<Action>,
    action_receiver: mpsc::UnboundedReceiver<Action>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (action_sender, action_receiver) = mpsc::unbounded_channel();
        let mut state = AppState::default();

        let config = crate::tui::config::load();
        state.auto_update = config.auto_update;
        state.last_update_check = config.last_update_check;
        state.bdix_enabled = config.bdix_enabled;
        let provider_was_sanitized = !state.bdix_enabled && config.active_provider.is_bdix();
        state.active_provider = if provider_was_sanitized {
            crate::providers::models::ProviderKind::MovieBox
        } else {
            config.active_provider
        };
        state.active_theme_kind = config.active_theme;
        state.default_player = config.default_player;
        state.download_dir = config.download_dir.map(std::path::PathBuf::from);

        let mut theme = crate::tui::theme::Theme::new();
        if let Ok(theme_env) = std::env::var("MOVIEBOX_THEME") {
            let theme_kind = crate::tui::theme::ThemeKind::parse(&theme_env);
            state.active_theme_kind = theme_kind.as_str().to_string();
            theme = crate::tui::theme::Theme::from_kind(theme_kind);
        } else if !state.active_theme_kind.is_empty() {
            let theme_kind = crate::tui::theme::ThemeKind::parse(&state.active_theme_kind);
            state.active_theme_kind = theme_kind.as_str().to_string();
            theme = crate::tui::theme::Theme::from_kind(theme_kind);
        } else {
            state.active_theme_kind = "Mocha".to_string();
        }

        let app = Self {
            theme,
            state,
            client: MovieBoxClient::new(),
            fourk_client: FourKHdHubClient::new().ok(),
            circleftp_client: crate::providers::bdix::circleftp::CircleFtpClient::new(),
            dhakaflix_client: crate::providers::bdix::dhakaflix::client::DhakaFlixClient::new(),
            action_sender,
            action_receiver,
        };
        if app.fourk_client.is_none() {
            log::warn!("4KHDHub client unavailable; provider will be disabled");
        }
        if provider_was_sanitized {
            app.persist_config();
        }
        app
    }

    fn request_context(&self) -> RequestContext {
        RequestContext {
            provider: self.state.active_provider,
            generation: self.state.provider_generation,
        }
    }

    fn context_is_current(&self, context: RequestContext) -> bool {
        context.provider == self.state.active_provider
            && context.generation == self.state.provider_generation
    }

    fn persist_config(&self) {
        let config = crate::tui::config::Config {
            auto_update: self.state.auto_update,
            last_update_check: self.state.last_update_check,
            active_provider: self.state.active_provider,
            active_theme: self.state.active_theme_kind.clone(),
            bdix_enabled: self.state.bdix_enabled,
            default_player: self.state.default_player.clone(),
            download_dir: self
                .state
                .download_dir
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
        };
        crate::tui::config::save(&config);
    }

    fn save_tv_playlists(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("moviebox-tui");
            let _ = std::fs::create_dir_all(&app_dir);
            let path = app_dir.join("tv_config.json");
            if let Ok(json) = serde_json::to_string(&self.state.tv_playlists) {
                let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
                if let Err(error) = std::fs::write(&temporary, json) {
                    log::warn!("failed to save tv playlists: {error}");
                    return;
                }
                if std::fs::rename(&temporary, &path).is_err() {
                    let _ = std::fs::remove_file(&path);
                    if let Err(error) = std::fs::rename(&temporary, &path) {
                        let _ = std::fs::remove_file(&temporary);
                        log::warn!("failed to commit tv playlists: {error}");
                    }
                }
            }
        }
    }

    fn load_tv_playlists_from_config(&mut self) {
        self.state.tv_playlists.clear();
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("moviebox-tui").join("tv_config.json");
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(list) = serde_json::from_str::<Vec<String>>(&content)
            {
                let mut seen = std::collections::HashSet::new();
                self.state.tv_playlists = list
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty() && seen.insert(item.clone()))
                    .collect();
                if self.state.tv_playlists.is_empty() {
                    let _ = std::fs::remove_file(path);
                }
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    fn reload_tv_playlists(&self) {
        let playlists = self.state.tv_playlists.clone();
        let sender = self.action_sender.clone();
        tokio::spawn(async move {
            let parser = crate::providers::m3u::M3UParser::new();
            let mut all_channels = Vec::new();
            let mut failed = 0usize;
            for source in &playlists {
                match parser.fetch_playlist(source).await {
                    Ok(channels) => all_channels.extend(channels),
                    Err(error) => {
                        failed += 1;
                        log::warn!("tv playlist failed ({source}): {error}");
                    }
                }
            }
            sender
                .send(Action::TvChannelsLoaded(all_channels, failed))
                .ok();
        });
    }

    fn tv_manager_activate(&mut self) {
        use crate::tui::state::TvManagerRow;
        let Some(row) = self
            .state
            .tv_manager_rows()
            .get(self.state.tv_manager_selected)
            .copied()
        else {
            return;
        };
        match row {
            TvManagerRow::Playlist(index) => {
                self.action_sender
                    .send(Action::TvPlaylistRemove(index))
                    .ok();
            }
            TvManagerRow::AddUrl => {
                self.action_sender.send(Action::TvInputToggle(false)).ok();
            }
            TvManagerRow::AddFile => {
                self.action_sender.send(Action::TvInputToggle(true)).ok();
            }
            TvManagerRow::Reload => {
                self.action_sender.send(Action::TvReloadPlaylists).ok();
            }
            TvManagerRow::Done => {
                self.reset_transient_overlays();
                self.state.tv_config_popup = false;
            }
            TvManagerRow::Header(_) => {}
        }
    }
}
