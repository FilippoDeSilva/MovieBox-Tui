use tokio::sync::mpsc;

use crate::providers::{
    fourkhdhub::FourKHdHubClient, models::RequestContext, moviebox::client::MovieBoxClient,
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

        let config = crate::tui::config::load();
        state.auto_update = config.auto_update;
        state.last_update_check = config.last_update_check;
        state.active_provider = config.active_provider;
        state.active_theme_kind = config.active_theme;
        state.bdix_enabled = config.bdix_enabled;
        state.default_player = config.default_player;

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
        let config = crate::tui::config::Config {
            auto_update: self.state.auto_update,
            last_update_check: self.state.last_update_check,
            active_provider: self.state.active_provider,
            active_theme: self.state.active_theme_kind.clone(),
            bdix_enabled: self.state.bdix_enabled,
            default_player: self.state.default_player.clone(),
        };
        crate::tui::config::save(&config);
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
