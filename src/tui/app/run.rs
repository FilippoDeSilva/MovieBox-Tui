use super::App;
use crate::providers::models::ProviderKind;
use crate::tui::{
    action::Action,
    event::EventHandler,
    state::{InputMode, Screen},
};
use ratatui::{Frame, layout::Rect};
use std::time::Duration;

enum ForcedProtocol {
    None,
    Type(ratatui_image::picker::ProtocolType),
}

impl App {
    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> std::io::Result<()>
    where
        std::io::Error: From<<B as ratatui::backend::Backend>::Error>,
    {
        if self.state.image_picker.is_none() && self.state.image_supported {
            self.probe_terminal().await;
        }

        let mut events = EventHandler::new(Duration::from_millis(100));

        if self.state.active_provider == ProviderKind::MovieBox {
            let client = self.service.client.clone();
            tokio::spawn(async move {
                let _ = client.init().await;
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.state.auto_update && now.saturating_sub(self.state.last_update_check) > 3600 {
            self.state.manual_update_check = false;
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
        let mut last_window_title = String::new();

        loop {
            let want_beam =
                self.state.input_mode == InputMode::Editing && !self.state.basic_terminal;
            if want_beam != self.state.cursor_beam {
                use crossterm::cursor::SetCursorStyle;
                let style = if want_beam {
                    SetCursorStyle::SteadyBar
                } else {
                    SetCursorStyle::DefaultUserShape
                };
                let _ = crossterm::execute!(std::io::stdout(), style);
                self.state.cursor_beam = want_beam;
            }
            let title = self.contextual_title();
            if title != last_window_title {
                let _ = crate::tui::terminal::set_window_title(&title);
                last_window_title = title;
            }
            self.state.normalize_result_view();
            if self.state.clear_terminal_before_draw {
                if let Err(err) = terminal.clear() {
                    log::debug!("terminal clear warning: {err}");
                    let _ = terminal.backend_mut().clear();
                }
                self.state.poster_protocol = None;
                self.state.search_poster_protocols.clear();
                self.state.clear_terminal_before_draw = false;
                self.state.dirty = true;
            }
            if self.state.dirty {
                if let Err(err) = terminal.draw(|frame| self.draw(frame)) {
                    log::warn!("transient draw warning: {err}");
                }
                self.state.dirty = false;
            }

            tokio::select! {
                Some(action) = events.next() => {
                    if let Some(quit) = self.handle_action(action).await {
                        return Ok(quit);
                    }
                    while let Ok(action) = events.try_recv() {
                        if let Some(quit) = self.handle_action(action).await {
                            return Ok(quit);
                        }
                    }
                }
                Some(action) = self.action_receiver.recv() => {
                    if let Some(quit) = self.handle_action(action).await {
                        return Ok(quit);
                    }
                    while let Ok(action) = self.action_receiver.try_recv() {
                        if let Some(quit) = self.handle_action(action).await {
                            return Ok(quit);
                        }
                    }
                }
            }
        }
    }

    pub(super) async fn probe_terminal(&mut self) {
        use ratatui_image::picker::{Capability, ProtocolType};

        self.state.image_probe_attempts = self.state.image_probe_attempts.saturating_add(1);
        self.state.last_image_probe = Some(std::time::Instant::now());

        match Self::forced_protocol() {
            Some(ForcedProtocol::None) => {
                self.state.image_supported = false;
                self.state.image_picker = None;
                self.state.initial_probe_failed = false;
                return;
            }
            forced => {
                if forced.is_some() {
                    let font_size = Self::cell_size_override().unwrap_or(ratatui_image::FontSize {
                        width: 10,
                        height: 20,
                    });
                    #[allow(deprecated)]
                    let mut picker = ratatui_image::picker::Picker::from_fontsize(font_size);
                    if let Some(ForcedProtocol::Type(protocol)) = forced {
                        picker.set_protocol_type(protocol);
                    }
                    self.accept_picker(picker);
                    return;
                }
            }
        }

        let picker = self.query_picker().await;

        if self.state.theme_is_auto
            && let Some(background) = picker.as_ref().and_then(|p| {
                p.capabilities()
                    .iter()
                    .find_map(|capability| match capability {
                        Capability::Background(red, green, blue) => Some((*red, *green, *blue)),
                        _ => None,
                    })
            })
        {
            let luminance = 0.2126 * f32::from(background.0)
                + 0.7152 * f32::from(background.1)
                + 0.0722 * f32::from(background.2);
            let is_light = luminance > 128.0;
            self.theme = crate::tui::theme::Theme::detect_with_light(Some(is_light));
            self.state.active_theme_kind = if is_light { "Latte" } else { "Mocha" }.to_string();
        }

        let mut picker = match (picker, Self::cell_size_override()) {
            (Some(picker), None) => picker,
            (probed, cell_size) => {
                let font_size = cell_size.unwrap_or(ratatui_image::FontSize {
                    width: 10,
                    height: 20,
                });
                #[allow(deprecated)]
                let mut rebuilt = ratatui_image::picker::Picker::from_fontsize(font_size);
                if let Some(probed) = probed {
                    rebuilt.set_protocol_type(probed.protocol_type());
                }
                if let Some(ForcedProtocol::Type(protocol)) = Self::forced_protocol() {
                    rebuilt.set_protocol_type(protocol);
                }
                rebuilt
            }
        };

        let capabilities_empty = picker.capabilities().is_empty();
        if matches!(picker.protocol_type(), ProtocolType::Halfblocks) && !capabilities_empty {
            let caps = picker.capabilities();
            let salvaged = if caps.iter().any(|c| matches!(c, Capability::Kitty)) {
                Some(ProtocolType::Kitty)
            } else if caps.iter().any(|c| matches!(c, Capability::Sixel)) {
                Some(ProtocolType::Sixel)
            } else {
                None
            };
            if let Some(protocol) = salvaged {
                log::info!(
                    "salvaging graphics-capable terminal that answered as halfblocks: {:?}",
                    protocol
                );
                picker.set_protocol_type(protocol);
            }
        }

        if matches!(picker.protocol_type(), ProtocolType::Halfblocks) {
            self.state.image_supported = false;
            self.state.image_picker = None;
            self.state.initial_probe_failed = !capabilities_empty;
            return;
        }
        self.accept_picker(picker);
    }

    fn accept_picker(&mut self, picker: ratatui_image::picker::Picker) {
        let cell_h = picker.font_size().height;
        if cell_h > 0 {
            self.state.poster_rows = (96_u16.div_ceil(cell_h)).max(3);
        }
        self.state.image_picker = Some(picker);
        self.state.image_supported = true;
        self.state.initial_probe_failed = false;
    }

    async fn query_picker(&self) -> Option<ratatui_image::picker::Picker> {
        if !crate::tui::terminal::should_query_images() {
            return None;
        }
        tokio::task::spawn_blocking(|| {
            let options = ratatui_image::picker::cap_parser::QueryStdioOptions {
                timeout: Duration::from_millis(400),
                terminal_background_color_osc: true,
                ..Default::default()
            };
            ratatui_image::picker::Picker::from_query_stdio_with_options(options).ok()
        })
        .await
        .unwrap_or_default()
    }

    fn forced_protocol() -> Option<ForcedProtocol> {
        match std::env::var("MOVIEBOX_IMAGE_PROTOCOL")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "none" | "off" | "false" => Some(ForcedProtocol::None),
            "sixel" => Some(ForcedProtocol::Type(
                ratatui_image::picker::ProtocolType::Sixel,
            )),
            "kitty" => Some(ForcedProtocol::Type(
                ratatui_image::picker::ProtocolType::Kitty,
            )),
            "iterm2" => Some(ForcedProtocol::Type(
                ratatui_image::picker::ProtocolType::Iterm2,
            )),
            _ => None,
        }
    }

    fn cell_size_override() -> Option<ratatui_image::FontSize> {
        let raw = std::env::var("MOVIEBOX_CELL_SIZE").ok()?;
        let raw = raw.trim().to_ascii_lowercase();
        let (width, height) = raw.split_once(['x', '*'])?;
        let width: u16 = width.trim().parse().ok()?;
        let height: u16 = height.trim().parse().ok()?;
        if width == 0 || height == 0 {
            return None;
        }
        Some(ratatui_image::FontSize { width, height })
    }
    pub fn contextual_title(&self) -> String {
        match self.state.active_screen {
            Screen::Home => match self.state.mode() {
                crate::tui::state::AppMode::Streaming => "MovieBox-Tui — Streaming".to_string(),
                crate::tui::state::AppMode::Tv => "MovieBox-Tui — Live TV".to_string(),
                crate::tui::state::AppMode::Addon => "MovieBox-Tui — Addons".to_string(),
            },
            Screen::Details => {
                if let Some(details) = &self.state.selected_details {
                    if let Some(title) = details.get("title").and_then(|t| t.as_str()) {
                        if !title.is_empty() {
                            return format!("MovieBox-Tui — {title}");
                        }
                    }
                }
                "MovieBox-Tui — Details".to_string()
            }
        }
    }

    pub async fn handle_action(&mut self, action: Action) -> Option<()> {
        if self.state.last_resize_time.is_some()
            || !matches!(action, Action::Tick | Action::UpdateDownload(..))
        {
            self.state.dirty = true;
        }
        match action {
            Action::Quit => {
                return Some(());
            }

            Action::Key(key) => {
                self.handle_key(key).await;
            }

            Action::MouseClick(col, row) => {
                self.handle_mouse(col, row);
            }

            Action::ProbeTerminal => {
                self.probe_terminal().await;
                let outcome = if self.state.image_supported {
                    let protocol = self
                        .state
                        .image_picker
                        .as_ref()
                        .map(|picker| format!("{:?}", picker.protocol_type()))
                        .unwrap_or_default();
                    format!("Graphics protocol: {protocol}")
                } else {
                    "No graphics protocol detected; using text placeholders".to_string()
                };
                self.state.notify(
                    crate::tui::overlay::NotificationKind::Info,
                    "Terminal probe",
                    outcome,
                );
            }

            Action::WheelScroll { up } => {
                if self.state.show_help {
                    let delta = 3 * if up { -1i32 } else { 1 };
                    self.state.help_scroll =
                        (self.state.help_scroll as i32 + delta).max(0) as usize;
                    return None;
                }
                let steps = if self.state.active_screen == Screen::Home
                    && !self.state.favorites_focus
                    && !self.state.search_results.is_empty()
                    && self.state.input_mode != InputMode::Editing
                {
                    self.state.effective_row_height().min(8)
                } else {
                    2
                };
                let step = if up { Action::MoveUp } else { Action::MoveDown };
                for _ in 0..steps {
                    let action = step.clone();
                    if let Some(quit) = self.handle_navigation(action).await {
                        return Some(quit);
                    }
                }
            }

            Action::Tick
            | Action::FocusChange
            | Action::Resize(..)
            | Action::SwitchProvider(..)
            | Action::ToggleHelp
            | Action::Refresh
            | Action::ClearCache
            | Action::CacheCleared(..)
            | Action::ToggleThemePopup
            | Action::SelectTheme(..)
            | Action::ShowBrowseMenu
            | Action::SetStatus(..)
            | Action::CheckForUpdates
            | Action::UpdateAvailable(..)
            | Action::StartSelfUpdate
            | Action::SelfUpdateProgress(..)
            | Action::SelfUpdateComplete(..) => {
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

            Action::ToggleAddonMode
            | Action::SwitchToStreamingMode
            | Action::ShowAddonManager
            | Action::AddonAddManifest(..)
            | Action::AddonToggleEnabled(..)
            | Action::AddonRemove(..)
            | Action::AddonInputToggle(..) => {
                self.handle_addons(action).await;
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
            | Action::SelectBrowse(..)
            | Action::SelectAddonCatalog(..)
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
            | Action::MarkWatched(..)
            | Action::UpdateProgress { .. }
            | Action::ReconcileHistory
            | Action::PlayerCrashed(..)
            | Action::PlayerExited => {
                self.handle_playback(action).await;
            }

            Action::ToggleFavorite | Action::ShowFavorites | Action::OpenFavorite(..) => {
                self.handle_favorites(action).await;
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

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.draw_resize_badge(frame, area) {
            return;
        }

        if self.draw_too_small_gate(frame, area) {
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
            Screen::Home => {
                crate::tui::screens::home::draw(frame, main_area, &mut self.state, &self.theme);
            }
            Screen::Details => {
                crate::tui::screens::details::draw(frame, main_area, &mut self.state, &self.theme);
            }
        }

        if self.state.show_help {
            crate::tui::screens::help::draw(frame, main_area, &self.state, &self.theme);
        }

        self.draw_download_gauge(frame, download_area);
        self.draw_theme_picker(frame, area);
        self.draw_update_modal(frame, area);

        crate::tui::overlay::notifications(
            frame,
            area,
            &self.state.notifications,
            &self.theme,
            self.state.basic_terminal,
            self.state.download_progress.is_some(),
        );
    }

    fn draw_resize_badge(&self, frame: &mut Frame, area: Rect) -> bool {
        if let Some((_, w, h)) = self.state.last_resize_time {
            let sep = if self.state.basic_terminal { "x" } else { "×" };
            let label = format!("{} {} {}", w, sep, h);
            let label_len = label.chars().count() as u16 + 4;
            let badge_w = label_len.min(area.width);
            let badge_h = 1_u16;
            let badge_x = area.x + (area.width.saturating_sub(badge_w)) / 2;
            let badge_y = area.y + (area.height.saturating_sub(badge_h)) / 2;
            let badge_area = ratatui::layout::Rect {
                x: badge_x,
                y: badge_y,
                width: badge_w,
                height: badge_h,
            };
            let line = ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                label,
                self.theme.title,
            )]);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(line)
                    .alignment(ratatui::layout::Alignment::Center),
                badge_area,
            );
            true
        } else {
            false
        }
    }

    fn draw_too_small_gate(&self, frame: &mut Frame, area: Rect) -> bool {
        if area.width < 50 || area.height < 14 {
            use ratatui::layout::Alignment;
            use ratatui::text::Line;
            use ratatui::widgets::{Block, Borders, Paragraph};

            if area.width < 4 || area.height < 2 {
                return true;
            }

            if area.width < 25 || area.height < 5 {
                let p = Paragraph::new(format!("{}×{} (min 50×14)", area.width, area.height))
                    .style(self.theme.lavender)
                    .alignment(Alignment::Center);
                frame.render_widget(p, area);
                return true;
            }

            let msg_lines = vec![
                Line::from(format!(
                    "Terminal too small ({}×{}).",
                    area.width, area.height
                )),
                Line::from("Minimum required size: 50×14"),
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
            true
        } else {
            false
        }
    }

    fn draw_download_gauge(&self, frame: &mut Frame, download_area: Option<Rect>) {
        if let Some(prog) = self.state.download_progress {
            if let Some(dl_area) = download_area {
                use ratatui::widgets::{Block, Borders, Gauge};

                let status = self
                    .state
                    .download_status
                    .as_deref()
                    .unwrap_or("Downloading...");

                let budget = (dl_area.width as usize).saturating_sub(10);
                let status_text =
                    crate::tui::text::truncate_width(status, budget.saturating_sub(30).max(8));
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
                        status_text
                    )
                } else {
                    format!(" Download: {} [X] Cancel ", status_text)
                };

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title(title_text))
                    .gauge_style(self.theme.accent)
                    .ratio((prog / 100.0).clamp(0.0, 1.0));

                crate::tui::clear_area(frame, dl_area, &self.theme);
                frame.render_widget(gauge, dl_area);
            }
        }
    }

    fn draw_theme_picker(&mut self, frame: &mut Frame, area: Rect) {
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
    }

    fn draw_update_modal(&self, frame: &mut Frame, area: Rect) {
        if let Some((version, notes)) = &self.state.update_available {
            use ratatui::layout::Alignment;
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, Borders, Paragraph};

            let layout = crate::tui::overlay::update_modal_layout(area, notes);
            let popup_area = layout.popup_area;
            let display_count = layout.display_count;
            let has_more = layout.has_more;

            let note_lines: Vec<&str> = notes
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();

            crate::tui::overlay::clear_modal_area(frame, area, popup_area, &self.theme);

            let mut text = vec![
                Line::from(vec![Span::styled(
                    "A new version of MovieBox-Tui is available",
                    self.theme
                        .header
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![
                    Span::styled("Installed: ", self.theme.text_dim),
                    Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), self.theme.text),
                    Span::styled("   →   ", self.theme.accent),
                    Span::styled("Latest: ", self.theme.text_dim),
                    Span::styled(
                        format!("v{version}"),
                        self.theme
                            .accent
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ])
                .alignment(Alignment::Center),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Release Notes:",
                    self.theme
                        .subtext1
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )]),
            ];

            let line_width = popup_area.width.saturating_sub(6) as usize;
            for line in note_lines.iter().take(display_count) {
                let trimmed = line.trim();
                let mut spans = vec![Span::raw("  ")];

                if trimmed.starts_with("### ")
                    || trimmed.starts_with("## ")
                    || trimmed.starts_with("# ")
                {
                    let text_start = trimmed.find(' ').unwrap_or(0);
                    spans.push(Span::styled(
                        if self.state.basic_terminal {
                            "> "
                        } else {
                            "▌ "
                        },
                        self.theme.accent,
                    ));
                    spans.push(Span::styled(
                        crate::tui::text::truncate_width(
                            &trimmed[text_start..],
                            line_width.saturating_sub(4),
                        ),
                        self.theme.highlight,
                    ));
                } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    let text_start = trimmed.find(' ').unwrap_or(0) + 1;
                    spans.push(Span::styled(
                        if self.state.basic_terminal {
                            "- "
                        } else {
                            "• "
                        },
                        self.theme.accent,
                    ));
                    spans.push(Span::raw(crate::tui::text::truncate_width(
                        trimmed[text_start..].trim_start(),
                        line_width.saturating_sub(4),
                    )));
                } else {
                    spans.push(Span::raw(crate::tui::text::truncate_width(
                        trimmed,
                        line_width.saturating_sub(2),
                    )));
                }
                text.push(Line::from(spans));
            }

            if has_more {
                text.push(
                    Line::from(Span::styled(
                        "... (read more on GitHub release page)",
                        self.theme.text_dim,
                    ))
                    .alignment(Alignment::Center),
                );
            }

            text.push(Line::from(""));
            text.push(
                Line::from(vec![
                    Span::styled("[u]", self.theme.shortcut),
                    Span::styled(" Update Now    ", self.theme.text),
                    Span::styled("[o]", self.theme.shortcut),
                    Span::styled(" Open Release Page    ", self.theme.text),
                    Span::styled("[Esc]", self.theme.shortcut),
                    Span::styled(" Dismiss", self.theme.text),
                ])
                .alignment(Alignment::Center),
            );

            let block = Block::default()
                .title(" Update Available ")
                .title_alignment(Alignment::Center)
                .title_style(self.theme.title)
                .borders(Borders::ALL)
                .border_type(crate::tui::overlay::border_type(self.state.basic_terminal))
                .border_style(self.theme.border_focus);

            let popup = Paragraph::new(text).block(block);
            frame.render_widget(popup, popup_area);
        }
    }
}
