use tokio::sync::mpsc;

use crate::providers::{
    fourkhdhub::FourKHdHubClient,
    models::{ProviderKind, RequestContext},
    moviebox::client::MovieBoxClient,
};
use crate::tui::{action::Action, state::AppState, theme::Theme};

mod download;
mod keyboard;
mod navigation;
mod network;
mod playback;
mod requests;
mod run;
mod system;
mod tv;

pub struct App {
    state: AppState,
    theme: Theme,
    client: MovieBoxClient,
    fourk_client: FourKHdHubClient,
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

        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("moviebox-tui").join("config.json");
            if let Ok(config_str) = std::fs::read_to_string(config_path) {
                if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_str) {
                    if let Some(auto_update) =
                        config_json.get("auto_update").and_then(|v| v.as_bool())
                    {
                        state.auto_update = auto_update;
                    }
                    if let Some(last_check) = config_json
                        .get("last_update_check")
                        .and_then(|v| v.as_u64())
                    {
                        state.last_update_check = last_check;
                    }
                    if config_json.get("active_provider").and_then(|v| v.as_str())
                        == Some(ProviderKind::FourKHdHub.cache_key())
                    {
                        state.active_provider = ProviderKind::FourKHdHub;
                    }
                    if let Some(theme_val) =
                        config_json.get("active_theme").and_then(|v| v.as_str())
                    {
                        state.active_theme_kind = theme_val.to_string();
                    }
                    if let Some(bdix) = config_json.get("bdix_enabled").and_then(|v| v.as_bool()) {
                        state.bdix_enabled = bdix;
                    }
                    if let Some(default_player) =
                        config_json.get("default_player").and_then(|v| v.as_str())
                    {
                        state.default_player = Some(default_player.to_string());
                    }
                }
            }
        }

        let mut theme = crate::tui::theme::Theme::new();
        if !state.active_theme_kind.is_empty() {
            let theme_kind = crate::tui::theme::ThemeKind::parse(&state.active_theme_kind);
            state.active_theme_kind = theme_kind.as_str().to_string();
            theme = crate::tui::theme::Theme::from_kind(theme_kind);
        } else {
            state.active_theme_kind = "Mocha".to_string();
        }

        Self {
            theme,
            state,
            client: MovieBoxClient::new(),
            fourk_client: FourKHdHubClient::new(),
            circleftp_client: crate::providers::bdix::circleftp::CircleFtpClient::new(),
            dhakaflix_client: crate::providers::bdix::dhakaflix::client::DhakaFlixClient::new(),
            action_sender,
            action_receiver,
        }
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
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("moviebox-tui");
            if std::fs::create_dir_all(&app_dir).is_err() {
                return;
            }
            let config = serde_json::json!({
                "auto_update": self.state.auto_update,
                "last_update_check": self.state.last_update_check,
                "active_provider": self.state.active_provider.cache_key(),
                "active_theme": self.state.active_theme_kind,
                "bdix_enabled": self.state.bdix_enabled,
                "default_player": self.state.default_player
            });
            let path = app_dir.join("config.json");
            let temporary = app_dir.join(format!("config.{}.tmp", std::process::id()));
            if std::fs::write(&temporary, config.to_string()).is_ok()
                && std::fs::rename(&temporary, &path).is_err()
            {
                let _ = std::fs::remove_file(&path);
                let _ = std::fs::rename(&temporary, &path);
            }
        }
    }

    fn save_tv_playlists(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("moviebox-tui");
            let _ = std::fs::create_dir_all(&app_dir);
            let path = app_dir.join("tv_config.json");
            if let Ok(json) = serde_json::to_string(&self.state.tv_playlists) {
                if let Err(error) = std::fs::write(&path, json) {
                    log::warn!("failed to save tv playlists: {error}");
                }
            }
        }
    }

    fn load_tv_playlists_from_config(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("moviebox-tui").join("tv_config.json");
            if let Ok(content) = std::fs::read_to_string(path)
                && let Ok(list) = serde_json::from_str::<Vec<String>>(&content)
                && !list.is_empty()
            {
                self.state.tv_playlists = list;
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
            if failed > 0 {
                let _ = sender.send(Action::SetStatus(format!(
                    "Error: {failed} playlist(s) failed to load."
                )));
            }
            sender.send(Action::TvChannelsLoaded(all_channels)).ok();
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
                self.state.tv_config_popup = false;
            }
            TvManagerRow::Header(_) => {}
        }
    }
}
