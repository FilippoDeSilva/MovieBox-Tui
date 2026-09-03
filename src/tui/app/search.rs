use super::{App, network};
use crate::models::CatalogItem;
use crate::providers::models::{ProviderKind, RequestContext};
use crate::tui::{
    action::Action,
    overlay::NotificationKind,
    state::{InputMode, Screen, SearchResult},
};

fn compare_browse_values(
    left: Option<f64>,
    right: Option<f64>,
    descending: bool,
) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => {
            let order = left
                .partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal);
            if descending { order.reverse() } else { order }
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

impl App {
    pub(super) fn apply_tv_search_results(&mut self, query: &str, lower_query: &str) {
        self.state.search_results = self
            .state
            .tv_channels
            .iter()
            .filter(|channel| {
                lower_query == "/list"
                    || channel.name.to_lowercase().contains(lower_query)
                    || channel.group.to_lowercase().contains(lower_query)
            })
            .map(|channel| SearchResult {
                id: channel.stream_url.clone(),
                title: channel.name.clone(),
                stype: 3,
                release_year: channel.group.clone(),
                cover_url: Some(channel.logo.clone()),
                season: 1,
                episode: 1,
                provider: ProviderKind::MovieBox,
            })
            .collect();
        self.state.is_loading = false;
        self.state
            .search_list_state
            .select(if self.state.search_results.is_empty() {
                None
            } else {
                Some(0)
            });
        if !self.state.search_results.is_empty() {
            self.prefetch_visible_posters();
        }
        self.state
            .set_status_default(if self.state.search_results.is_empty() {
                format!("No matches for '{}'.", query)
            } else {
                format!("Found {} channels.", self.state.search_results.len())
            });
    }

    pub(super) fn handle_search_command(&mut self, query: &str, lower_query: &str) -> Option<bool> {
        let trimmed = query.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        if trimmed == "/" {
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
            return Some(true);
        }

        let parsed = match crate::tui::commands::SlashCommand::parse(trimmed) {
            Some(p) => p,
            None => {
                let cmd_name = trimmed.split_whitespace().next().unwrap_or(trimmed);
                self.state.search_query.clear();
                self.state.input_mode = InputMode::Normal;
                self.state.notify(
                    NotificationKind::Warning,
                    "Unknown Command",
                    format!("Command '{cmd_name}' is not recognized. Type '/' to view available commands."),
                );
                return Some(true);
            }
        };

        let current_mode = self.state.mode();
        let ctrl_s = crate::tui::text::CTRL_S_STR;
        let ctrl_t = crate::tui::text::CTRL_T_STR;
        let ctrl_a = crate::tui::text::CTRL_A_STR;

        let will_handle = !matches!(
            &parsed,
            crate::tui::commands::ParsedCommand::History if current_mode != crate::tui::state::AppMode::Tv
        ) && !matches!(
            &parsed,
            crate::tui::commands::ParsedCommand::Favorites if current_mode != crate::tui::state::AppMode::Tv
        ) && !matches!(
            &parsed,
            crate::tui::commands::ParsedCommand::List if current_mode == crate::tui::state::AppMode::Tv
        );

        if will_handle {
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
        }

        match parsed {
            crate::tui::commands::ParsedCommand::Exit => {
                self.action_sender.send(Action::Quit).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Settings => {
                self.action_sender.send(Action::ToggleSettingsPopup).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::ClearCache => {
                self.action_sender.send(Action::ClearCache).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Clear => {
                self.state.clear_search_state();
                self.state.set_status_default("Search cleared.");
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Help => {
                self.action_sender.send(Action::ToggleHelp).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Github => {
                let _ = open::that("https://github.com/mesamirh/MovieBox-Tui");
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Probe => {
                self.action_sender.send(Action::ProbeTerminal).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Update => {
                if !self.state.is_checking_updates {
                    self.state.update_available = None;
                    self.state.manual_update_check = true;
                    self.state.set_status_long("Checking GitHub for updates...");
                    self.action_sender.send(Action::CheckForUpdates).ok();
                } else {
                    self.state.set_status_long("Checking GitHub for updates...");
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::ToggleUpdate => {
                self.state.auto_update = !self.state.auto_update;
                self.persist_config();
                self.state.notify(
                    NotificationKind::Info,
                    "Auto Update Check",
                    if self.state.auto_update {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                );
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Theme => {
                self.action_sender.send(Action::ToggleThemePopup).ok();
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Browse => {
                if current_mode == crate::tui::state::AppMode::Tv {
                    self.state.notify(
                        NotificationKind::Info,
                        "TV Mode",
                        format!("Command /browse is available in Streaming Mode ({ctrl_s}) or Addon Mode ({ctrl_a})."),
                    );
                } else {
                    self.action_sender.send(Action::ShowBrowseMenu).ok();
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::History => {
                if current_mode == crate::tui::state::AppMode::Tv {
                    self.state.notify(
                        NotificationKind::Info,
                        "TV Mode",
                        format!("Command /history is available in Streaming Mode ({ctrl_s}) or Addon Mode ({ctrl_a})."),
                    );
                    Some(true)
                } else {
                    None
                }
            }
            crate::tui::commands::ParsedCommand::Favorites => {
                if current_mode == crate::tui::state::AppMode::Tv {
                    self.state.notify(
                        NotificationKind::Info,
                        "TV Mode",
                        format!("Command /favorites is available in Streaming Mode ({ctrl_s}) or Addon Mode ({ctrl_a})."),
                    );
                    Some(true)
                } else {
                    None
                }
            }
            crate::tui::commands::ParsedCommand::List => {
                if current_mode == crate::tui::state::AppMode::Tv {
                    self.apply_tv_search_results(query, lower_query);
                } else {
                    self.state.notify(
                        NotificationKind::Info,
                        "TV Mode",
                        format!(
                            "Command /list is only available in TV Mode. Switch with {ctrl_t}."
                        ),
                    );
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::Config => {
                let current_mode = self.state.mode();
                if current_mode == crate::tui::state::AppMode::Tv {
                    self.action_sender.send(Action::ShowTvConfig).ok();
                } else if current_mode == crate::tui::state::AppMode::Addon {
                    self.action_sender.send(Action::ShowAddonManager).ok();
                } else {
                    self.action_sender.send(Action::ToggleSettingsPopup).ok();
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::DownloadDir(raw_arg) => {
                if raw_arg.is_empty() {
                    let current = crate::logging::sanitize_path(self.resolve_download_base_dir());
                    self.state
                        .notify(NotificationKind::Info, "Download Directory", current);
                    return Some(true);
                }

                if raw_arg.eq_ignore_ascii_case("reset") || raw_arg.eq_ignore_ascii_case("default")
                {
                    let default_path =
                        crate::logging::sanitize_path(self.resolve_download_base_dir());
                    if self.state.download_dir.is_none() {
                        self.state.notify(
                            NotificationKind::Info,
                            "Download Directory",
                            format!("Already using system default ({default_path})"),
                        );
                    } else {
                        self.state.download_dir = None;
                        self.persist_config();
                        self.state.notify(
                            NotificationKind::Success,
                            "Download Directory",
                            format!("Reset to default ({default_path})"),
                        );
                    }
                    return Some(true);
                }

                let clean_arg = raw_arg.trim_matches(|c| c == '\'' || c == '"').trim();
                if clean_arg == "<path>"
                    || clean_arg == "path"
                    || clean_arg == "<dir>"
                    || clean_arg == "dir"
                {
                    self.state.notify(
                        NotificationKind::Info,
                        "Download Directory",
                        "Usage: /download-dir <folder_path>\nExample: /download-dir ~/Movies",
                    );
                    return Some(true);
                }
                let expanded_path = crate::tui::state::AppState::expand_download_path(clean_arg)
                    .unwrap_or_else(|| std::path::PathBuf::from(clean_arg));

                match std::fs::create_dir_all(&expanded_path) {
                    Ok(_) => {
                        let test_file =
                            expanded_path.join(format!(".mb_probe_{}", std::process::id()));
                        match std::fs::write(&test_file, b"ok") {
                            Ok(_) => {
                                let _ = std::fs::remove_file(&test_file);
                                let canonical =
                                    std::fs::canonicalize(&expanded_path).unwrap_or(expanded_path);
                                let clean_path = {
                                    let s = canonical.to_string_lossy();
                                    if let Some(stripped) = s.strip_prefix(r"\\?\") {
                                        std::path::PathBuf::from(stripped)
                                    } else {
                                        canonical
                                    }
                                };
                                self.state.download_dir = Some(clean_path.clone());
                                self.persist_config();
                                let effective =
                                    crate::logging::sanitize_path(self.resolve_download_base_dir());
                                self.state.notify(
                                    NotificationKind::Success,
                                    "Download Directory",
                                    format!("Saved: {effective}"),
                                );
                            }
                            Err(err) => {
                                self.state.notify(
                                    NotificationKind::Error,
                                    "Permission Denied",
                                    format!(
                                        "Cannot write to '{}': {}",
                                        expanded_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        self.state.notify(
                            NotificationKind::Error,
                            "Invalid Directory",
                            format!("Cannot create '{}': {}", expanded_path.display(), err),
                        );
                    }
                }

                Some(true)
            }
            crate::tui::commands::ParsedCommand::ToggleBdix => {
                let enable_req = !self.state.bdix_enabled;
                if current_mode != crate::tui::state::AppMode::Streaming {
                    self.state.notify(
                        NotificationKind::Info,
                        "BDIX Sources",
                        format!("BDIX FTP sources are only available in Streaming Mode. Switch with {ctrl_s}."),
                    );
                    return Some(true);
                }

                self.state.bdix_enabled = enable_req;
                self.persist_config();
                self.state.notify(
                    NotificationKind::Info,
                    "BDIX Providers",
                    if self.state.bdix_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                );
                if !self.state.bdix_enabled && self.state.active_provider.is_bdix() {
                    let new_provider = ProviderKind::ENABLED
                        .iter()
                        .copied()
                        .find(|provider| !provider.is_bdix())
                        .unwrap_or(ProviderKind::MovieBox);
                    self.action_sender
                        .send(Action::SwitchProvider(new_provider))
                        .ok();
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::ToggleStreaming => {
                let enable_req = !self.state.streaming_enabled;
                if !enable_req && !self.state.tv_enabled && !self.state.addons_enabled {
                    self.state.notify(
                        NotificationKind::Warning,
                        "Streaming Mode",
                        "Cannot disable: at least one mode must remain active.",
                    );
                    return Some(true);
                }

                self.state.streaming_enabled = enable_req;
                self.persist_config();
                self.state.notify(
                    NotificationKind::Info,
                    "Streaming Mode",
                    if self.state.streaming_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                );
                if !self.state.streaming_enabled
                    && !self.state.is_tv_mode
                    && !self.state.is_addon_mode
                {
                    if self.state.tv_enabled {
                        self.action_sender.send(Action::ToggleTvMode).ok();
                    } else if self.state.addons_enabled {
                        self.action_sender.send(Action::ToggleAddonMode).ok();
                    }
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::ToggleTv => {
                let enable_req = !self.state.tv_enabled;
                if !enable_req && !self.state.streaming_enabled && !self.state.addons_enabled {
                    self.state.notify(
                        NotificationKind::Warning,
                        "TV Mode",
                        "Cannot disable: at least one mode must remain active.",
                    );
                    return Some(true);
                }

                self.state.tv_enabled = enable_req;
                self.persist_config();
                self.state.notify(
                    NotificationKind::Info,
                    "TV Mode",
                    if self.state.tv_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                );
                if !self.state.tv_enabled && self.state.is_tv_mode {
                    if self.state.streaming_enabled {
                        self.action_sender.send(Action::SwitchToStreamingMode).ok();
                    } else if self.state.addons_enabled {
                        self.action_sender.send(Action::ToggleAddonMode).ok();
                    }
                }
                Some(true)
            }
            crate::tui::commands::ParsedCommand::ToggleAddons => {
                let enable_req = !self.state.addons_enabled;
                if !enable_req && !self.state.streaming_enabled && !self.state.tv_enabled {
                    self.state.notify(
                        NotificationKind::Warning,
                        "Addon Mode",
                        "Cannot disable: at least one mode must remain active.",
                    );
                    return Some(true);
                }

                self.state.addons_enabled = enable_req;
                self.persist_config();
                self.state.notify(
                    NotificationKind::Info,
                    "Addon Mode",
                    if self.state.addons_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                );
                if !self.state.addons_enabled && self.state.is_addon_mode {
                    if self.state.streaming_enabled {
                        self.action_sender.send(Action::SwitchToStreamingMode).ok();
                    } else if self.state.tv_enabled {
                        self.action_sender.send(Action::ToggleTvMode).ok();
                    }
                }
                Some(true)
            }
        }
    }

    pub(super) fn prepare_search_request(&mut self, query: &str) -> RequestContext {
        self.state.active_search_request = self.state.active_search_request.wrapping_add(1);
        self.state.active_preview_request = self.state.active_preview_request.wrapping_add(1);
        self.state.is_homepage_mode = false;
        self.state.active_browse_preset = None;
        self.state.active_addon_catalog = None;
        self.state.browse_metrics.clear();
        self.state.current_page = 1;
        self.state.active_screen = Screen::Home;
        self.state.active_subject_id = None;
        self.state.selected_details = None;
        self.state.selected_resources.clear();
        self.state.is_loading = true;
        self.state.has_search_settled = false;
        self.state.search_error = None;
        self.state.search_list_state.select(Some(0));
        self.state.search_suggestions.clear();
        self.state.suggest_index = None;
        self.state.search_preview = None;
        self.state.preview_loading = false;
        self.state.poster_image = None;
        self.state.poster_protocol = None;
        self.state
            .set_status_default(format!("Searching for '{}'...", query));
        self.request_context()
    }

    pub(super) fn prepare_addon_catalog_request(
        &mut self,
        target: &crate::providers::addons::models::AddonCatalogTarget,
    ) -> RequestContext {
        self.state.active_search_request = self.state.active_search_request.wrapping_add(1);
        self.state.active_preview_request = self.state.active_preview_request.wrapping_add(1);
        self.state.is_homepage_mode = false;
        self.state.active_browse_preset = None;
        self.state.active_addon_catalog = Some(target.clone());
        self.state.browse_metrics.clear();
        self.state.current_page = 1;
        self.state.active_screen = Screen::Home;
        self.state.active_subject_id = None;
        self.state.selected_details = None;
        self.state.selected_resources.clear();
        self.state.is_loading = true;
        self.state.has_search_settled = false;
        self.state.search_error = None;
        self.state.search_list_state.select(Some(0));
        self.state.search_suggestions.clear();
        self.state.suggest_index = None;
        self.state.search_preview = None;
        self.state.preview_loading = false;
        self.state.poster_image = None;
        self.state.poster_protocol = None;
        self.state.failed_posters.clear();
        self.state.in_flight_posters.clear();
        self.state.search_results.clear();
        self.state.search_query.clear();
        self.request_context()
    }

    pub(super) fn run_search_request(
        &mut self,
        query: String,
        force_refresh: bool,
        context: RequestContext,
    ) {
        self.request_tasks.cancel_search();
        let request_id = self.state.active_search_request;
        let page = 1;
        let sender = self.action_sender.clone();
        let service = self.service.clone();
        self.request_tasks.search = Some(tokio::spawn(async move {
            if !force_refresh {
                let q = query.clone();
                let provider = context.provider;
                if let Ok(Some(cached)) = tokio::task::spawn_blocking(move || {
                    crate::cache::get_provider_search_cache_typed(provider, &q, page)
                })
                .await
                {
                    sender
                        .send(Action::SearchSuccess {
                            context,
                            request_id,
                            query: query.clone(),
                            page,
                            items: cached,
                        })
                        .ok();
                    return;
                }
            }

            let result = service.search_typed(context.provider, &query, page).await;
            match result {
                Ok(items) => {
                    let q = query.clone();
                    let provider = context.provider;
                    let cached = items.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::cache::set_provider_search_cache_typed(provider, &q, page, &cached);
                    });
                    sender
                        .send(Action::SearchSuccess {
                            context,
                            request_id,
                            query,
                            page,
                            items,
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(Action::SearchFailure(
                            context,
                            request_id,
                            page,
                            error.user_message(context.provider),
                        ))
                        .ok();
                }
            }
        }));
    }

    pub(super) fn prepare_homepage_request(&mut self, tab_id: &str, page: usize) {
        self.state.active_homepage_request = self.state.active_homepage_request.wrapping_add(1);
        self.state.is_homepage_mode = true;
        self.state.current_tab_id = tab_id.to_string();
        self.state.current_page = page;
        self.state.active_screen = Screen::Home;
        self.state.is_loading = true;
        self.state.has_search_settled = false;
        self.state.search_error = None;
        if page == 1 {
            self.state.active_preview_request = self.state.active_preview_request.wrapping_add(1);
            self.state.active_subject_id = None;
            self.state.selected_details = None;
            self.state.selected_resources.clear();
            self.state.search_results.clear();
            self.state.browse_metrics.clear();
            self.state.search_list_state.select(Some(0));
            self.state.search_suggestions.clear();
            self.state.suggest_index = None;
            self.state.search_preview = None;
            self.state.preview_loading = false;
            self.state.poster_image = None;
            self.state.poster_protocol = None;
            self.state.set_status_default("Loading discover tab...");
        }
    }

    pub(super) fn prepare_details_request(&mut self, id: &str) -> RequestContext {
        self.state.active_subject_id = Some(id.to_string());
        self.state.poster_protocol = None;
        self.state.is_loading = true;
        self.state.details_error = None;
        self.state.active_details_request = self.state.active_details_request.wrapping_add(1);
        self.state
            .fetch_cancel
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.state.set_status_default("Fetching details...");
        self.state.stream_pool.clear();

        let provider = self.provider_for_subject(id);
        let mut context = self.request_context();
        context.provider = provider;
        context
    }

    pub(super) fn run_details_request(
        &mut self,
        id: String,
        force_refresh: bool,
        context: RequestContext,
    ) {
        self.request_tasks.cancel_details();
        let request_id = self.state.active_details_request;
        if !force_refresh {
            if let Some(cached) = self.state.preview_cache.get(&id).cloned() {
                self.action_sender
                    .send(Action::DetailsSuccess(context, request_id, id, cached))
                    .ok();
                return;
            }
        }
        let service = self.service.clone();
        let sender = self.action_sender.clone();
        self.request_tasks.details = Some(tokio::spawn(async move {
            if !force_refresh {
                let id_for_cache = id.clone();
                if let Ok(Some(cached)) = tokio::task::spawn_blocking(move || {
                    crate::cache::get_provider_details_cache_typed(context.provider, &id_for_cache)
                })
                .await
                {
                    sender
                        .send(Action::DetailsSuccess(
                            context,
                            request_id,
                            id.clone(),
                            cached,
                        ))
                        .ok();
                    return;
                }
            }

            let result = service.details_typed(context.provider, &id).await;
            match result {
                Ok(details) => {
                    let id_for_cache = id.clone();
                    let details_for_cache = details.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        crate::cache::set_provider_details_cache_typed(
                            context.provider,
                            &id_for_cache,
                            &details_for_cache,
                        )
                    })
                    .await;
                    sender
                        .send(Action::DetailsSuccess(context, request_id, id, details))
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(Action::DetailsFailure(
                            context,
                            request_id,
                            error.user_message(context.provider),
                        ))
                        .ok();
                }
            }
        }));
    }

    pub(super) fn run_homepage_request(&mut self, tab_id: String, page: usize) {
        self.request_tasks.cancel_homepage();
        let request_id = self.state.active_homepage_request;
        if let Some((items, metrics)) = self
            .state
            .homepage_cache
            .get(&(tab_id.clone(), page))
            .cloned()
        {
            self.action_sender
                .send(Action::HomepageSuccess {
                    request_id,
                    tab_id,
                    page,
                    items,
                    metrics,
                })
                .ok();
            return;
        }
        let service = self.service.clone();
        let sender = self.action_sender.clone();
        self.request_tasks.homepage = Some(tokio::spawn(async move {
            let t_clone = tab_id.clone();
            if let Ok(Some((items, metrics))) = tokio::task::spawn_blocking(move || {
                crate::cache::get_homepage_cache_typed(&t_clone, page)
            })
            .await
            {
                sender
                    .send(Action::HomepageSuccess {
                        request_id,
                        tab_id: tab_id.clone(),
                        page,
                        items,
                        metrics,
                    })
                    .ok();
                return;
            }

            match service.homepage(&tab_id, page).await {
                Ok((items, metrics)) => {
                    let items_clone = items.clone();
                    let metrics_clone = metrics.clone();
                    let t_clone = tab_id.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::cache::set_homepage_cache_typed(
                            &t_clone,
                            page,
                            &(items_clone, metrics_clone),
                        );
                    });
                    sender
                        .send(Action::HomepageSuccess {
                            request_id,
                            tab_id,
                            page,
                            items,
                            metrics,
                        })
                        .ok();
                }
                Err(error) => {
                    sender.send(Action::HomepageFailure(request_id, error)).ok();
                }
            }
        }));
    }

    pub(super) fn append_homepage_items(
        &mut self,
        items: Vec<CatalogItem>,
        metrics_map: std::collections::HashMap<String, crate::models::BrowseMetrics>,
    ) -> usize {
        let mut count = 0;
        for item in items {
            let id = item.id.value.clone();
            let clean_title = crate::providers::moviebox::clean_moviebox_title(&item.title);
            let stype = if item.media_type == crate::models::MediaType::Series {
                2
            } else {
                1
            };
            let release_year = item.year.clone().unwrap_or_default();
            let cover_url = item.poster_url.clone();
            let season = item.season_count.unwrap_or(0);
            let metrics = metrics_map.get(&id).copied().unwrap_or_default();

            if let Some(existing) = self.state.search_results.iter_mut().find(|r| r.id == id) {
                let stored_metrics = self.state.browse_metrics.entry(id.clone()).or_default();
                stored_metrics.trending = stored_metrics.trending.or(metrics.trending);
                stored_metrics.rating = stored_metrics.rating.or(metrics.rating);
                stored_metrics.recent_rating =
                    stored_metrics.recent_rating.or(metrics.recent_rating);
                stored_metrics.popularity = stored_metrics.popularity.or(metrics.popularity);
                if season > existing.season {
                    existing.season = season;
                    existing.title = clean_title;
                    existing.stype = stype;
                    existing.release_year = release_year;
                    existing.cover_url = cover_url;
                }
                continue;
            }

            let raw_lower = item.title.to_lowercase();
            let is_dub = raw_lower.contains("[hindi]")
                || raw_lower.contains("[tamil]")
                || raw_lower.contains("[telugu]")
                || raw_lower.contains("[english]");

            if is_dub
                && self
                    .state
                    .search_results
                    .iter()
                    .any(|r| r.title == clean_title && r.stype == stype)
            {
                continue;
            }

            if self.state.search_results.iter().any(|r| {
                r.title == clean_title && r.release_year == release_year && r.stype == stype
            }) {
                continue;
            }

            if !id.is_empty() {
                self.state.browse_metrics.insert(id.clone(), metrics);
                self.state.search_results.push(SearchResult {
                    id,
                    title: clean_title,
                    stype,
                    release_year,
                    cover_url,
                    season,
                    episode: 1,
                    provider: item.id.provider,
                });
                count += 1;
            }
        }
        count
    }

    pub(super) fn sort_browse_results(&mut self) {
        let Some(preset) = self.state.active_browse_preset else {
            return;
        };
        let metrics = self.state.browse_metrics.clone();
        let metric = preset.metric();
        let descending = preset.descending();
        self.state.search_results.sort_by(|left, right| {
            let left_value = metrics
                .get(&left.id)
                .and_then(|values| values.value(metric));
            let right_value = metrics
                .get(&right.id)
                .and_then(|values| values.value(metric));
            let metric_order = compare_browse_values(left_value, right_value, descending);
            if left_value.is_none() && right_value.is_none() {
                metric_order
            } else {
                metric_order
                    .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            }
        });
    }

    pub(super) fn prefetch_visible_posters(&mut self) {
        if !self.state.image_supported || self.state.search_results.is_empty() {
            return;
        }
        let total = self.state.search_results.len();
        let selected = self.state.search_list_state.selected().unwrap_or(0);
        let offset = self.state.result_scroll;
        let visible = self.state.effective_visible_items().max(6);

        let base_start = offset.min(selected);
        let start = base_start.saturating_sub(2);
        let end = (offset + visible + 4).min(total);

        if start < end {
            let slice: Vec<(String, Option<String>, ProviderKind)> = self.state.search_results
                [start..end]
                .iter()
                .map(|r| (r.id.clone(), r.cover_url.clone(), r.provider))
                .collect();
            self.spawn_search_posters(slice);
        }
    }

    pub(super) fn spawn_search_posters(
        &mut self,
        results: Vec<(String, Option<String>, ProviderKind)>,
    ) {
        if !self.state.image_supported {
            return;
        }

        let mut to_fetch = Vec::new();
        for (id, cover_url, provider) in results {
            if self.state.search_posters.contains(&id) || self.state.failed_poster_recently(&id) {
                continue;
            }
            if !self.state.in_flight_posters.insert(id.clone()) {
                continue;
            }
            to_fetch.push((id, cover_url, provider));
        }

        if to_fetch.is_empty() {
            return;
        }

        let sender = self.action_sender.clone();
        let service = self.service.clone();
        let semaphore = self.state.poster_fetch_semaphore.clone();

        tokio::spawn(async move {
            let sem = semaphore;
            for (id, cover_url, provider) in to_fetch {
                let permit = sem.clone().acquire_owned().await.ok();
                let tx = sender.clone();
                let service = service.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let id_clone = id.clone();
                    if let Ok(Some(bytes)) = tokio::task::spawn_blocking({
                        let id_c = id_clone.clone();
                        move || crate::cache::get_namespaced_image_cache("posters", &id_c)
                    })
                    .await
                    {
                        if let Some(img) = network::decode_poster(bytes).await {
                            tx.send(Action::SearchPosterLoaded(id_clone, Some(img)))
                                .ok();
                            return;
                        }
                    }

                    let mut resolved_url = cover_url;
                    if resolved_url.is_none() {
                        if let Ok(details) = service.details_typed(provider, &id).await {
                            resolved_url = details.cover_url().map(|s| s.to_string());
                        }
                    }

                    if let Some(url) = resolved_url {
                        if !url.is_empty() {
                            if let Some(bytes) = service.fetch_poster_bytes(&url).await {
                                let bytes_clone = bytes.clone();
                                let id_c = id.clone();
                                let _ = tokio::task::spawn_blocking(move || {
                                    crate::cache::set_namespaced_image_cache(
                                        "posters",
                                        &id_c,
                                        &bytes_clone,
                                    );
                                })
                                .await;

                                if let Some(img) = network::decode_poster(bytes).await {
                                    tx.send(Action::SearchPosterLoaded(id, Some(img))).ok();
                                    return;
                                }
                            }
                        }
                    }

                    tx.send(Action::SearchPosterLoaded(id, None)).ok();
                });
            }
        });
    }
}
