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
    overlay::NotificationKind,
    state::{AppState, InputMode, Screen},
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
            Action::Tick => {
                let mut needs_redraw = (self.state.is_loading && self.state.tick_count % 5 == 0)
                    || self.state.tick_count < 15;
                self.state.tick_count = self.state.tick_count.wrapping_add(1);
                if !self.state.notifications.is_empty() {
                    needs_redraw = true;
                    self.state.expire_notifications();
                }
                if self.state.status_timer > 0 {
                    needs_redraw = true;
                    self.state.status_timer -= 1;
                    if self.state.status_timer == 0 {
                        self.state.status_message.clear();
                    }
                }
                if needs_redraw {
                    self.state.dirty = true;
                }

                let current_query = self.state.search_query.trim().to_string();
                if current_query != self.state.last_suggest_query
                    && self.state.last_search_edit.elapsed()
                        >= std::time::Duration::from_millis(350)
                {
                    self.state.last_suggest_query = current_query.clone();
                    if !current_query.is_empty() {
                        if self.state.is_tv_mode {
                            let q = current_query.to_lowercase();
                            self.state.search_suggestions = self
                                .state
                                .tv_channels
                                .iter()
                                .filter(|c| c.name.to_lowercase().contains(&q))
                                .take(10)
                                .map(|c| c.name.clone())
                                .collect();
                        } else {
                            self.action_sender.send(Action::Suggest(current_query)).ok();
                        }
                    } else {
                        self.state.search_suggestions.clear();
                    }
                }

                if self.state.pending_episode_fetch.is_some()
                    && self.state.last_episode_nav.elapsed()
                        >= std::time::Duration::from_millis(300)
                {
                    if let Some((subject_id, se, ep)) = self.state.pending_episode_fetch.take() {
                        let mut found_cached = false;
                        if let Some(pool) = self.state.stream_pool.get(&subject_id) {
                            if let Some(cached) = pool.episode_index.get(&(se, ep)) {
                                found_cached = true;
                                let count = cached.len();
                                let mut result = serde_json::Map::new();
                                result.insert(
                                    "list".to_string(),
                                    serde_json::Value::Array(cached.clone()),
                                );
                                self.state.selected_resources =
                                    Some(serde_json::Value::Object(result));
                                self.state.is_loading = false;
                                self.state.resource_list_state.select(if count > 0 {
                                    Some(0)
                                } else {
                                    None
                                });
                                self.state.set_status(
                                    format!("Resolved {} direct stream sources (cached).", count),
                                    150,
                                );
                            }
                        }

                        if !found_cached {
                            self.action_sender
                                .send(Action::FetchEpisodeStreams {
                                    subject_id,
                                    season: se,
                                    episode: ep,
                                    force_refresh: false,
                                })
                                .ok();
                        }
                    }
                }
            }
            Action::Quit => {
                return Some(());
            }
            Action::FocusChange => {
                self.prepare_image_soft_refresh();
            }
            Action::Resize(_w, _h) => {
                self.prepare_image_refresh();
                self.state.poster_protocol = None;
                self.state.search_poster_protocols.clear();
            }
            Action::SwitchProvider(provider) => self.switch_provider(provider),
            Action::Key(key) => {
                use crossterm::event::{KeyCode, KeyModifiers};

                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if let KeyCode::Char('c') = key.code {
                        self.action_sender.send(Action::Quit).ok();
                        return Some(());
                    }
                    if let KeyCode::Char('t') = key.code {
                        self.action_sender.send(Action::ToggleTvMode).ok();
                        return None;
                    }
                    if let KeyCode::Char('p') = key.code {
                        self.cycle_provider();
                        return None;
                    }
                }

                if let KeyCode::Char('x') | KeyCode::Char('X') = key.code
                    && self.state.download_progress.is_some()
                {
                    self.action_sender.send(Action::CancelDownload).ok();
                    return None;
                }

                if let Some((version, _)) = &self.state.update_available {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            self.state.update_available = None;
                        }
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            let url = format!(
                                "https://github.com/mesamirh/MovieBox-Tui/releases/tag/v{}",
                                version
                            );
                            let _ = open::that(&url);
                            self.state.update_available = None;
                        }
                        _ => {}
                    }
                    return None;
                }

                if key.code == KeyCode::F(1) {
                    self.action_sender.send(Action::ToggleHelp).ok();
                    return None;
                }

                if self.state.show_theme_popup {
                    match key.code {
                        KeyCode::Esc => {
                            self.state.show_theme_popup = false;
                        }
                        KeyCode::Up => {
                            let max = crate::tui::theme::AVAILABLE_THEMES.len().saturating_sub(1);
                            let i = match self.state.theme_list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        max
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            self.state.theme_list_state.select(Some(i));
                            let selected_theme = crate::tui::theme::AVAILABLE_THEMES[i].to_string();
                            self.action_sender
                                .send(Action::SelectTheme(selected_theme))
                                .ok();
                        }
                        KeyCode::Down => {
                            let max = crate::tui::theme::AVAILABLE_THEMES.len().saturating_sub(1);
                            let i = match self.state.theme_list_state.selected() {
                                Some(i) => {
                                    if i >= max {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            self.state.theme_list_state.select(Some(i));
                            let selected_theme = crate::tui::theme::AVAILABLE_THEMES[i].to_string();
                            self.action_sender
                                .send(Action::SelectTheme(selected_theme))
                                .ok();
                        }
                        KeyCode::Enter => {
                            self.state.show_theme_popup = false;
                            self.persist_config();
                        }
                        _ => {}
                    }
                    return None;
                }

                match self.state.input_mode {
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => {
                            self.state.input_mode = InputMode::Normal;
                            self.state.set_status(String::new(), 150);
                        }
                        KeyCode::Enter => {
                            let query = self.state.search_query.trim().to_string();
                            if !query.is_empty() {
                                let selected_suggestion = self.state.suggest_index.is_some();
                                self.state.input_mode = InputMode::Normal;
                                self.state.search_suggestions.clear();
                                self.state.suggest_index = None;
                                self.state.search_list_state.select(None);
                                self.state.last_search_edit = std::time::Instant::now();
                                let action = if selected_suggestion {
                                    Action::SelectSuggestion { query }
                                } else {
                                    Action::Search {
                                        query,
                                        force_refresh: false,
                                    }
                                };
                                self.action_sender.send(action).ok();
                            }
                        }
                        KeyCode::Backspace => {
                            crate::tui::text::remove_last_grapheme(&mut self.state.search_query);
                            self.state.suggest_index = None;
                            self.state.last_search_edit = std::time::Instant::now();
                        }
                        KeyCode::Char(c) => {
                            self.state.search_query.push(c);
                            self.state.suggest_index = None;
                            self.state.last_search_edit = std::time::Instant::now();
                        }
                        KeyCode::Up if !self.state.search_suggestions.is_empty() => {
                            let max_idx = self.state.search_suggestions.len() - 1;
                            let next_idx = match self.state.suggest_index {
                                Some(0) | None => max_idx,
                                Some(i) => i - 1,
                            };
                            self.state.suggest_index = Some(next_idx);
                            if let Some(sug) = self.state.search_suggestions.get(next_idx) {
                                self.state.search_query = sug.clone();
                                self.state.last_suggest_query =
                                    self.state.search_query.trim().to_string();
                            }
                        }
                        KeyCode::Down if !self.state.search_suggestions.is_empty() => {
                            let max_idx = self.state.search_suggestions.len() - 1;
                            let next_idx = match self.state.suggest_index {
                                None => 0,
                                Some(i) if i == max_idx => 0,
                                Some(i) => i + 1,
                            };
                            self.state.suggest_index = Some(next_idx);
                            if let Some(sug) = self.state.search_suggestions.get(next_idx) {
                                self.state.search_query = sug.clone();
                                self.state.last_suggest_query =
                                    self.state.search_query.trim().to_string();
                            }
                        }
                        _ => {}
                    },
                    InputMode::Normal => match self.state.active_screen {
                        Screen::Startup => {}
                        Screen::Home => {
                            if self.state.tv_config_popup {
                                if self.state.tv_input_active {
                                    match key.code {
                                        KeyCode::Esc => {
                                            self.state.tv_input_active = false;
                                            self.state.tv_input_buffer.clear();
                                        }
                                        KeyCode::Enter => {
                                            let buffer =
                                                self.state.tv_input_buffer.trim().to_string();
                                            self.state.tv_input_active = false;
                                            self.state.tv_input_buffer.clear();
                                            if !buffer.is_empty() {
                                                self.action_sender
                                                    .send(Action::TvPlaylistAdd(buffer))
                                                    .ok();
                                            }
                                        }
                                        KeyCode::Backspace => {
                                            crate::tui::text::remove_last_grapheme(
                                                &mut self.state.tv_input_buffer,
                                            );
                                        }
                                        KeyCode::Char(c) if !c.is_control() => {
                                            self.state.tv_input_buffer.push(c);
                                        }
                                        _ => {}
                                    }
                                    return None;
                                }
                                match key.code {
                                    KeyCode::Esc => {
                                        self.state.tv_config_popup = false;
                                    }
                                    KeyCode::Up => {
                                        use crate::tui::state::TvManagerRow;
                                        let rows = self.state.tv_manager_rows();
                                        let total = rows.len();
                                        let mut next = if self.state.tv_manager_selected == 0 {
                                            total.saturating_sub(1)
                                        } else {
                                            self.state.tv_manager_selected - 1
                                        };
                                        while next != self.state.tv_manager_selected
                                            && matches!(
                                                rows.get(next),
                                                Some(TvManagerRow::Header(_))
                                            )
                                        {
                                            next = if next == 0 {
                                                total.saturating_sub(1)
                                            } else {
                                                next - 1
                                            };
                                        }
                                        self.state.tv_manager_selected = next;
                                    }
                                    KeyCode::Down => {
                                        use crate::tui::state::TvManagerRow;
                                        let rows = self.state.tv_manager_rows();
                                        let total = rows.len();
                                        let mut next =
                                            if self.state.tv_manager_selected + 1 >= total {
                                                0
                                            } else {
                                                self.state.tv_manager_selected + 1
                                            };
                                        while next != self.state.tv_manager_selected
                                            && matches!(
                                                rows.get(next),
                                                Some(TvManagerRow::Header(_))
                                            )
                                        {
                                            next = if next + 1 >= total { 0 } else { next + 1 };
                                        }
                                        self.state.tv_manager_selected = next;
                                    }
                                    KeyCode::Char('d') => {
                                        use crate::tui::state::TvManagerRow;
                                        if let Some(TvManagerRow::Playlist(index)) = self
                                            .state
                                            .tv_manager_rows()
                                            .get(self.state.tv_manager_selected)
                                            .copied()
                                        {
                                            self.action_sender
                                                .send(Action::TvPlaylistRemove(index))
                                                .ok();
                                        }
                                    }
                                    KeyCode::Enter => {
                                        self.tv_manager_activate();
                                    }
                                    _ => {}
                                }
                                return None;
                            }
                            match key.code {
                                KeyCode::Esc => {
                                    self.action_sender.send(Action::GoBack).ok();
                                }
                                KeyCode::Up => {
                                    self.action_sender.send(Action::MoveUp).ok();
                                }
                                KeyCode::Down => {
                                    self.action_sender.send(Action::MoveDown).ok();
                                }
                                KeyCode::Left => {
                                    self.action_sender.send(Action::MoveLeft).ok();
                                }
                                KeyCode::Right => {
                                    self.action_sender.send(Action::MoveRight).ok();
                                }
                                KeyCode::Enter => {
                                    if self.state.search_results.is_empty()
                                        && !self.state.search_query.trim().is_empty()
                                        && (self.state.search_error.is_some()
                                            || self
                                                .state
                                                .status_message
                                                .to_ascii_lowercase()
                                                .starts_with("no matches"))
                                    {
                                        self.action_sender
                                            .send(Action::Search {
                                                query: self.state.search_query.trim().to_string(),
                                                force_refresh: true,
                                            })
                                            .ok();
                                    } else {
                                        self.action_sender.send(Action::Submit).ok();
                                    }
                                }
                                KeyCode::Char('?') => {
                                    self.action_sender.send(Action::ToggleHelp).ok();
                                }
                                KeyCode::Char('q') => {
                                    self.action_sender.send(Action::Quit).ok();
                                }
                                KeyCode::Char('r') => {
                                    self.action_sender.send(Action::Refresh).ok();
                                }
                                KeyCode::Char('o') | KeyCode::Char('O')
                                    if self.state.input_mode == InputMode::Normal
                                        && self.state.is_tv_mode =>
                                {
                                    let idx_opt = self.state.search_list_state.selected();
                                    if let Some(idx) = idx_opt {
                                        if let Some(item) = self.state.search_results.get(idx) {
                                            self.action_sender
                                                .send(Action::ShowPlayerPicker(
                                                    item.id.clone(),
                                                    None,
                                                ))
                                                .ok();
                                        }
                                    }
                                }
                                KeyCode::Char(c)
                                    if (key.modifiers.is_empty()
                                        || key.modifiers == KeyModifiers::SHIFT) =>
                                {
                                    self.state.input_mode = InputMode::Editing;
                                    self.state.search_query.push(c);

                                    self.state.search_suggestions.clear();
                                    self.state.suggest_index = None;
                                    self.state.set_status(String::new(), 150);
                                    self.state.last_search_edit = std::time::Instant::now();
                                }
                                _ => {}
                            }
                        }
                        Screen::Details => match key.code {
                            KeyCode::Tab => {
                                self.action_sender.send(Action::TabPane).ok();
                            }
                            KeyCode::BackTab => {
                                self.action_sender.send(Action::BackTabPane).ok();
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                if self.state.show_season_download_confirm {
                                    self.action_sender.send(Action::ConfirmDownloadSeason).ok();
                                } else if self.state.show_episode_download_confirm {
                                    self.action_sender.send(Action::ConfirmDownloadEpisode).ok();
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                if self.state.show_season_download_confirm {
                                    self.state.show_season_download_confirm = false;
                                } else if self.state.show_episode_download_confirm {
                                    self.state.show_episode_download_confirm = false;
                                }
                            }
                            KeyCode::Esc => {
                                if self.state.show_season_download_confirm {
                                    self.state.show_season_download_confirm = false;
                                } else if self.state.show_episode_download_confirm {
                                    self.state.show_episode_download_confirm = false;
                                } else {
                                    self.action_sender.send(Action::GoBack).ok();
                                }
                            }
                            KeyCode::Char('q') => {
                                self.action_sender.send(Action::Quit).ok();
                            }
                            KeyCode::Char('o') | KeyCode::Char('O') => {
                                if !self.state.subtitle_popup && !self.state.player_picker_popup {
                                    if let crate::tui::state::DetailsPane::Streams =
                                        self.state.details_pane
                                    {
                                        self.action_sender.send(Action::PlayStream(true)).ok();
                                    }
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                if !self.state.subtitle_popup && !self.state.player_picker_popup {
                                    if let crate::tui::state::DetailsPane::Seasons =
                                        self.state.details_pane
                                    {
                                        if !self.state.available_seasons.is_empty() {
                                            self.action_sender
                                                .send(Action::PromptDownloadSeason)
                                                .ok();
                                        }
                                    } else {
                                        self.action_sender.send(Action::PromptDownloadEpisode).ok();
                                    }
                                }
                            }
                            KeyCode::Char('r') => {
                                self.action_sender.send(Action::Refresh).ok();
                            }
                            KeyCode::Char('?') => {
                                self.action_sender.send(Action::ToggleHelp).ok();
                            }
                            KeyCode::Char('b') => {
                                self.action_sender.send(Action::GoBack).ok();
                            }

                            KeyCode::Up => {
                                self.action_sender.send(Action::MoveUp).ok();
                            }
                            KeyCode::Down => {
                                self.action_sender.send(Action::MoveDown).ok();
                            }
                            KeyCode::Left => {
                                if self.state.show_season_download_confirm {
                                    self.state.season_download_confirm_yes_selected = true;
                                } else if self.state.show_episode_download_confirm {
                                    self.state.episode_download_confirm_yes_selected = true;
                                }
                            }
                            KeyCode::Right => {
                                if self.state.show_season_download_confirm {
                                    self.state.season_download_confirm_yes_selected = false;
                                } else if self.state.show_episode_download_confirm {
                                    self.state.episode_download_confirm_yes_selected = false;
                                }
                            }
                            KeyCode::Enter => {
                                let open_with = key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::SHIFT);
                                if self.state.show_season_download_confirm {
                                    if self.state.season_download_confirm_yes_selected {
                                        self.action_sender.send(Action::ConfirmDownloadSeason).ok();
                                    } else {
                                        self.state.show_season_download_confirm = false;
                                    }
                                } else if self.state.show_episode_download_confirm {
                                    if self.state.episode_download_confirm_yes_selected {
                                        self.action_sender
                                            .send(Action::ConfirmDownloadEpisode)
                                            .ok();
                                    } else {
                                        self.state.show_episode_download_confirm = false;
                                    }
                                } else if self.state.subtitle_popup
                                    || self.state.player_picker_popup
                                    || self.state.is_download_subtitle_popup
                                {
                                    self.action_sender.send(Action::Submit).ok();
                                } else {
                                    match self.state.details_pane {
                                        crate::tui::state::DetailsPane::Streams => {
                                            self.action_sender
                                                .send(Action::PlayStream(open_with))
                                                .ok();
                                        }
                                        crate::tui::state::DetailsPane::Seasons => {
                                            self.trigger_episode_fetch();
                                        }
                                        crate::tui::state::DetailsPane::Episodes => {
                                            self.trigger_episode_fetch();
                                        }
                                        crate::tui::state::DetailsPane::Languages => {
                                            let idx = self
                                                .state
                                                .language_list_state
                                                .selected()
                                                .unwrap_or(0);

                                            self.action_sender
                                                .send(Action::SelectLanguage(idx))
                                                .ok();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                    },
                }
            }

            Action::ToggleHelp => {
                if matches!(self.state.active_screen, Screen::Home | Screen::Details) {
                    self.state.show_help = !self.state.show_help;
                    if self.state.show_help {
                        self.state.tv_config_popup = false;
                        self.state.player_picker_popup = false;
                        self.state.subtitle_popup = false;
                        self.state.is_download_subtitle_popup = false;
                        self.state.show_season_download_confirm = false;
                        self.state.show_episode_download_confirm = false;
                    }
                }
            }
            Action::Refresh => match self.state.active_screen {
                Screen::Home => {
                    let query = self.state.search_query.trim().to_string();
                    if self.state.is_tv_mode {
                        if query.is_empty() {
                            self.state
                                .set_status("Reloading TV playlists...".to_string(), 150);
                            self.reload_tv_playlists();
                        } else {
                            self.action_sender
                                .send(Action::Search {
                                    query,
                                    force_refresh: true,
                                })
                                .ok();
                        }
                    } else if !query.is_empty() {
                        self.action_sender
                            .send(Action::Search {
                                query,
                                force_refresh: true,
                            })
                            .ok();
                    }
                }
                Screen::Details => {
                    if let Some(id) = self.state.active_subject_id.clone() {
                        let se = if self.state.available_seasons.is_empty() {
                            0
                        } else {
                            self.state.selected_season
                        };
                        let ep = if self.state.available_seasons.is_empty() {
                            0
                        } else {
                            self.state.selected_episode
                        };
                        let id_clone = id.clone();
                        let id_clone_2 = id.clone();
                        let provider = self.state.active_provider;
                        tokio::task::spawn_blocking(move || {
                            crate::cache::invalidate_provider_stream_cache(
                                provider, &id_clone, se, ep,
                            );
                            crate::cache::invalidate_provider_details_cache(provider, &id_clone_2);
                        });
                        self.state.selected_season = se;
                        self.state.selected_episode = ep;

                        self.action_sender
                            .send(Action::FetchDetails(id.clone(), true))
                            .ok();

                        self.action_sender
                            .send(Action::FetchEpisodeStreams {
                                subject_id: id,
                                season: se,
                                episode: ep,
                                force_refresh: true,
                            })
                            .ok();
                    }
                }
                _ => {}
            },
            Action::ClearCache => {
                tokio::task::spawn_blocking(crate::cache::clear_all_cache);
                self.state
                    .set_status("Cache cleared completely.".to_string(), 150);
            }

            Action::ToggleThemePopup => {
                self.state.show_theme_popup = !self.state.show_theme_popup;
                if self.state.show_theme_popup {
                    if let Some(idx) = crate::tui::theme::AVAILABLE_THEMES
                        .iter()
                        .position(|&t| t.eq_ignore_ascii_case(&self.state.active_theme_kind))
                    {
                        self.state.theme_list_state.select(Some(idx));
                    } else {
                        self.state.theme_list_state.select(Some(0));
                    }
                }
            }
            Action::SelectTheme(theme_name) => {
                let kind = crate::tui::theme::ThemeKind::parse(&theme_name);
                self.state.active_theme_kind = kind.as_str().to_string();
                self.theme = crate::tui::theme::Theme::from_kind(kind);
                self.state.dirty = true;
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

            Action::SetStatus(msg) => {
                self.state.is_resolving_playback = false;
                if msg.starts_with("Error:") {
                    log::error!("{msg}");
                    self.state.notify(
                        NotificationKind::Error,
                        "Operation failed",
                        msg.trim_start_matches("Error:").trim(),
                    );
                } else {
                    self.state.set_status(msg, 150);
                }
            }

            Action::CheckForUpdates => {
                let update_sender = self.action_sender.clone();
                tokio::spawn(async move {
                    let start = tokio::time::Instant::now();
                    let result = crate::tui::updater::check(env!("CARGO_PKG_VERSION")).await;

                    let elapsed = start.elapsed();
                    if elapsed.as_millis() < 1500 {
                        tokio::time::sleep(std::time::Duration::from_millis(1500) - elapsed).await;
                    }

                    match result {
                        Ok(Some((version, notes))) => {
                            update_sender
                                .send(Action::UpdateAvailable(version, notes))
                                .ok();
                        }
                        Ok(None) => {
                            update_sender
                                .send(Action::UpdateAvailable("none".into(), "".into()))
                                .ok();
                        }
                        Err(error) => {
                            update_sender
                                .send(Action::UpdateAvailable(
                                    format!("error:{}", error),
                                    "".into(),
                                ))
                                .ok();
                        }
                    }
                });
            }
            Action::UpdateAvailable(version, notes) => {
                if self.state.active_screen == Screen::Startup {
                    self.state.active_screen = Screen::Home;
                }

                if version == "none" {
                    if self.state.manual_update_check {
                        self.state.notify(
                            NotificationKind::Success,
                            "Up to date",
                            "You are using the latest version.",
                        );
                    }
                    self.state.manual_update_check = false;
                } else if version.starts_with("error:") {
                    let err = version.trim_start_matches("error:");
                    if self.state.manual_update_check {
                        self.state.notify(
                            NotificationKind::Error,
                            "Update check failed",
                            err.to_string(),
                        );
                    }
                    self.state.manual_update_check = false;
                } else {
                    self.state.manual_update_check = false;
                    self.state.update_available = Some((version, notes));
                }
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
