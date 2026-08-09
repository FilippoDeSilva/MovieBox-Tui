use ratatui::Frame;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::providers::{
    fourkhdhub::FourKHdHubClient,
    models::{ProviderKind, RequestContext},
    moviebox::client::MovieBoxClient,
};
use crate::tui::{
    action::Action,
    event::EventHandler,
    state::{AppState, Screen},
    theme::Theme,
};

mod download;
mod keyboard;
mod navigation;
mod network;
mod playback;
mod requests;
mod run;
mod system;
mod tv;

pub fn clean_moviebox_title(raw_title: &str) -> String {
    let mut end = raw_title.len();

    if let Some(start) = raw_title[..end].find(" [") {
        end = start;
    }
    if let Some(start) = raw_title[..end].find(" (") {
        let inside = &raw_title[start..end].to_lowercase();
        if inside.contains("dub") || inside.contains("hindi") {
            end = start;
        }
    }

    if let Some(s_idx) = raw_title[..end].rfind(" S") {
        let suffix = &raw_title[s_idx + 2..end];
        let is_season = suffix
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == 'S');
        if is_season && suffix.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            end = s_idx;
        }
    }
    raw_title[..end].trim_end().to_string()
}

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
            let parser = crate::providers::iptv_org::m3u::M3UParser::new();
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

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> std::io::Result<()>
    where
        std::io::Error: From<<B as ratatui::backend::Backend>::Error>,
    {
        if self.state.image_picker.is_none() && self.state.image_supported {
            match ratatui_image::picker::Picker::from_query_stdio() {
                Ok(picker) => {
                    let cell_h = picker.font_size().height;
                    if cell_h > 0 {
                        self.state.poster_rows = (96_u16.div_ceil(cell_h)).max(3);
                    }
                    self.state.image_picker = Some(picker);
                }
                Err(_) => {
                    self.state.image_supported = false;
                }
            }
        }

        let mut events = EventHandler::new(Duration::from_millis(100));

        if self.state.active_provider == ProviderKind::MovieBox {
            let client = self.client.clone();
            tokio::spawn(async move {
                let _ = client.init().await;
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.state.auto_update && now.saturating_sub(self.state.last_update_check) > 3600 {
            self.state.last_update_check = now;
            self.state.manual_update_check = false;
            self.persist_config();
            self.action_sender.send(Action::CheckForUpdates).ok();
        }
        self.state.active_screen = Screen::Home;

        self.state.available_players = crate::tui::player::detect();
        let preferred = std::env::var("MOVIEBOX_PLAYER")
            .ok()
            .and_then(|value| crate::tui::state::PlayerKind::parse(&value))
            .or_else(|| {
                self.state
                    .default_player
                    .as_deref()
                    .and_then(crate::tui::state::PlayerKind::parse)
            });
        if let Some(preferred) = preferred
            && let Some(index) = self
                .state
                .available_players
                .iter()
                .position(|&k| k == preferred)
        {
            let kind = self.state.available_players.remove(index);
            self.state.available_players.insert(0, kind);
        }

        loop {
            if self.state.clear_terminal_before_draw {
                terminal.current_buffer_mut().reset();
                terminal.backend_mut().clear()?;
                terminal.swap_buffers();
                self.state.clear_terminal_before_draw = false;
                self.state.dirty = true;
            }
            if self.state.dirty {
                terminal.draw(|frame| self.draw(frame))?;
                self.state.dirty = false;
            }

            tokio::select! {
                Some(action) = events.next() => {
                    if let Some(quit) = self.handle_action(action).await {
                        return Ok(quit);
                    }
                }
                Some(action) = self.action_receiver.recv() => {
                    if let Some(quit) = self.handle_action(action).await {
                        return Ok(quit);
                    }
                }
            }
        }
    }

    async fn handle_action(&mut self, action: Action) -> Option<()> {
        if !matches!(action, Action::Tick | Action::UpdateDownload(..)) {
            self.state.dirty = true;
        }
        match action {
            Action::Key(key) => {
                self.handle_key(key).await;
            }

            Action::Tick
            | Action::Quit
            | Action::FocusChange
            | Action::Resize(..)
            | Action::SwitchProvider(..)
            | Action::ToggleHelp
            | Action::Refresh
            | Action::ClearCache
            | Action::ToggleThemePopup
            | Action::SelectTheme(..)
            | Action::SetStatus(..)
            | Action::CheckForUpdates
            | Action::UpdateAvailable(..) => {
                self.handle_system(action).await;
            }

            Action::ToggleTvMode
            | Action::ShowTvConfig
            | Action::TvPlaylistAdd(..)
            | Action::TvPlaylistRemove(..)
            | Action::TvReloadPlaylists
            | Action::TvInputToggle(..)
            | Action::TvChannelsLoaded(..) => {
                self.handle_tv(action).await;
            }

            Action::MoveUp
            | Action::MoveDown
            | Action::MoveLeft
            | Action::MoveRight
            | Action::Submit
            | Action::GoBack
            | Action::TabPane
            | Action::BackTabPane
            | Action::SelectLanguage(..) => {
                self.handle_navigation(action).await;
            }

            Action::Suggest(..)
            | Action::SuggestSuccess(..)
            | Action::SelectSuggestion { .. }
            | Action::Search { .. }
            | Action::FetchHomepage { .. }
            | Action::SearchSuccess { .. }
            | Action::SearchFailure(..)
            | Action::HomepageSuccess { .. }
            | Action::HomepageFailure(..)
            | Action::FetchDetails(..)
            | Action::DetailsSuccess(..)
            | Action::DetailsFailure(..)
            | Action::FetchPreview(..)
            | Action::PreviewSuccess(..)
            | Action::PreviewFailure(..)
            | Action::FetchEpisodeStreams { .. }
            | Action::EpisodeStreamsReady(..)
            | Action::EpisodeStreamsFailed(..)
            | Action::InitStreamPool(..)
            | Action::StreamPoolInitialized(..)
            | Action::PosterSuccess(..)
            | Action::SearchPosterLoaded(..) => {
                self.handle_requests(action).await;
            }

            Action::PlayStream(..)
            | Action::ShowSubtitlePopup(..)
            | Action::ShowDownloadSubtitlePopup(..)
            | Action::ShowPlaybackPicker(..)
            | Action::ShowPlayerPicker(..)
            | Action::LaunchMpv(..)
            | Action::LaunchPlayback(..)
            | Action::LaunchPlayer(..)
            | Action::PlayerCrashed(..) => {
                self.handle_playback(action).await;
            }

            Action::DownloadStream(..)
            | Action::StartDownload(..)
            | Action::PromptDownloadEpisode
            | Action::ConfirmDownloadEpisode
            | Action::PromptDownloadSeason
            | Action::ConfirmDownloadSeason
            | Action::ProcessDownloadQueue
            | Action::UpdateDownload(..)
            | Action::DownloadCompleted(..)
            | Action::DownloadFailed(..)
            | Action::DownloadPaused(..)
            | Action::ClearDownload
            | Action::CancelDownload => {
                self.handle_download(action).await;
            }
        }
        None
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if area.width < 85 || area.height < 24 {
            use ratatui::layout::Alignment;
            use ratatui::text::Line;
            use ratatui::widgets::{Block, Borders, Paragraph};

            let msg_lines = vec![
                Line::from(format!(
                    "Terminal too small ({}x{}).",
                    area.width, area.height
                )),
                Line::from("Minimum required size: 85x24"),
                Line::from("Please enlarge your terminal window."),
            ];

            let padding_top = area.height.saturating_sub(2).saturating_sub(3) / 2;
            let mut msg = Vec::new();
            for _ in 0..padding_top {
                msg.push(Line::from(""));
            }
            msg.extend(msg_lines);

            let p = Paragraph::new(msg)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.theme.border),
                )
                .alignment(Alignment::Center);

            frame.render_widget(p, area);
            return;
        }

        let mut main_area = frame.area();
        let mut download_area = None;

        if self.state.download_progress.is_some() {
            use ratatui::layout::{Constraint, Direction, Layout};
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(main_area);

            main_area = chunks[0];
            download_area = Some(chunks[1]);
        }

        match self.state.active_screen {
            Screen::Startup => {
                super::screens::startup::draw(frame, main_area, &mut self.state, &self.theme);
            }
            Screen::Home => {
                super::screens::home::draw(frame, main_area, &mut self.state, &self.theme);
            }
            Screen::Details => {
                super::screens::details::draw(frame, main_area, &mut self.state, &self.theme);
            }
        }

        if self.state.show_help {
            super::screens::help::draw(frame, main_area, &self.state, &self.theme);
        }
        if let Some(prog) = self.state.download_progress {
            if let Some(dl_area) = download_area {
                use ratatui::widgets::{Block, Borders, Gauge};

                let status = self
                    .state
                    .download_status
                    .as_deref()
                    .unwrap_or("Downloading...");

                let title_text = if self.state.download_queue_total > 0 {
                    let total = self.state.download_queue_total;
                    let remaining = self.state.download_queue.len();
                    let current = total - remaining;
                    format!(
                        " Download: S{:02}E{:02} ({}/{}) | {} [X] Cancel ",
                        self.state.selected_season,
                        self.state.selected_episode,
                        current,
                        total,
                        status
                    )
                } else {
                    format!(" Download: {} [X] Cancel ", status)
                };

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title(title_text))
                    .gauge_style(self.theme.accent)
                    .ratio((prog / 100.0).clamp(0.0, 1.0));

                crate::tui::clear_area(frame, dl_area, &self.theme);
                frame.render_widget(gauge, dl_area);
            }
        }

        if self.state.show_theme_popup {
            let items: Vec<String> = crate::tui::theme::AVAILABLE_THEMES
                .iter()
                .map(|s| s.to_string())
                .collect();
            crate::tui::overlay::picker(
                frame,
                area,
                &items,
                &mut self.state.theme_list_state,
                crate::tui::overlay::PickerSpec {
                    title: "Select Theme",
                    confirm_label: "Apply",
                    minimum_width: 32,
                },
                &self.theme,
                self.state.basic_terminal,
            );
        }

        if let Some((version, notes)) = &self.state.update_available {
            use ratatui::layout::{Alignment, Constraint, Direction, Layout};
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, Borders, Clear, Paragraph};

            let popup_width = 65;
            let popup_height = 20;

            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(popup_width),
                    Constraint::Min(0),
                ])
                .split(area);

            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(popup_height),
                    Constraint::Min(0),
                ])
                .split(h_chunks[1]);

            let popup_area = v_chunks[1];
            frame.render_widget(Clear, popup_area);

            let mut text = vec![
                Line::from(Span::styled("Update Available!", self.theme.accent))
                    .alignment(Alignment::Center),
                Line::from(""),
                Line::from("A new version of MovieBox-Tui is available."),
                Line::from(""),
                Line::from(format!("Current: v{}", env!("CARGO_PKG_VERSION"))),
                Line::from(format!("Latest:  v{}", version)),
                Line::from(""),
                Line::from(Span::styled("Release Notes:", self.theme.highlight)),
            ];

            let note_lines = notes
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(3)
                .collect::<Vec<_>>();
            for line in note_lines {
                let mut truncated = line.to_string();
                if truncated.len() > 55 {
                    truncated.truncate(55);
                    truncated.push_str("...");
                }
                text.push(Line::from(truncated));
            }
            if notes.lines().count() > 3 {
                text.push(Line::from(Span::styled(
                    "... (read more on GitHub)",
                    self.theme.text_dim,
                )));
            }

            text.push(Line::from(""));
            text.push(
                Line::from(Span::styled(
                    "[Enter] Close popup   [o] Open in Browser",
                    self.theme.accent,
                ))
                .alignment(Alignment::Center),
            );

            let popup = Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border),
            );

            frame.render_widget(popup, popup_area);
        }

        crate::tui::overlay::notifications(
            frame,
            area,
            &self.state.notifications,
            &self.theme,
            self.state.basic_terminal,
        );
    }
}
