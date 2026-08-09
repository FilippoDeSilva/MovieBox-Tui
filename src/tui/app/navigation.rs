use super::App;
use crate::providers::models::{ProviderKind, Release};
use crate::tui::{action::Action, state::Screen};

impl App {
    pub(super) fn switch_provider(&mut self, provider: ProviderKind) {
        if self.state.is_tv_mode {
            return;
        }
        if provider == self.state.active_provider {
            return;
        }
        self.prepare_image_refresh();
        self.state
            .fetch_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.fetch_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.provider_generation = self.state.provider_generation.wrapping_add(1);
        self.state.active_provider = provider;
        self.state.active_screen = Screen::Home;
        self.state.is_homepage_mode = false;
        self.state.is_tv_mode = false;
        self.state.is_loading = false;
        self.state.is_fetching_streams = false;
        self.state.stream_error = None;
        self.state.search_results.clear();
        self.state.search_suggestions.clear();
        self.state.search_preview = None;
        self.state.preview_loading = false;
        self.state.selected_details = None;
        self.state.selected_resources = None;
        self.state.active_subject_id = None;
        self.state.available_seasons.clear();
        self.state.available_episode_numbers.clear();
        self.state.stream_pool.clear();
        self.state.is_resolving_playback = false;
        self.state.search_posters.clear();
        self.state.search_poster_protocols.clear();
        self.state.image_cache.clear();
        self.state.preview_cache.clear();
        self.state.poster_image = None;
        self.state.poster_protocol = None;
        self.state.search_poster_protocols.clear();
        self.state.search_list_state.select(None);
        self.state.resource_list_state.select(None);
        self.state.set_status(
            format!(
                "{} selected. Search uses only this provider.",
                provider.label()
            ),
            180,
        );
        self.persist_config();
        if provider == ProviderKind::MovieBox {
            let client = self.client.clone();
            tokio::spawn(async move {
                let _ = client.init().await;
            });
        }
    }

    pub(super) fn cycle_provider(&mut self) {
        let available_providers: Vec<ProviderKind> = ProviderKind::ENABLED
            .into_iter()
            .filter(|p| !p.is_bdix() || self.state.bdix_enabled)
            .collect();

        if available_providers.is_empty() {
            return;
        }

        let current = available_providers
            .iter()
            .position(|provider| *provider == self.state.active_provider)
            .unwrap_or(0);
        let next = available_providers[(current + 1) % available_providers.len()];
        self.switch_provider(next);
    }

    pub(super) fn prepare_image_refresh(&mut self) {
        if self.state.image_picker.as_ref().is_some_and(|picker| {
            !matches!(
                picker.protocol_type(),
                ratatui_image::picker::ProtocolType::Halfblocks
            )
        }) {
            self.state.clear_terminal_before_draw = true;
        }
    }

    pub(super) fn prepare_image_soft_refresh(&mut self) {
        self.state.poster_protocol = None;
        self.state.search_poster_protocols.clear();
        self.state.dirty = true;
    }

    pub(super) fn cycle_details_pane(&mut self, forward: bool) {
        use crate::tui::state::DetailsPane;

        if self.state.active_screen != Screen::Details {
            return;
        }

        let has_languages = self
            .state
            .selected_details
            .as_ref()
            .and_then(|details| details.get("dubs"))
            .and_then(|dubs| dubs.as_array())
            .is_some_and(|dubs| dubs.len() > 1);
        let is_series = !self.state.available_seasons.is_empty();
        let mut panes = Vec::new();
        if has_languages {
            panes.push(DetailsPane::Languages);
        }
        if is_series {
            panes.push(DetailsPane::Seasons);
            panes.push(DetailsPane::Episodes);
        }
        panes.push(DetailsPane::Streams);

        let current = panes
            .iter()
            .position(|pane| *pane == self.state.details_pane)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % panes.len()
        } else if current == 0 {
            panes.len() - 1
        } else {
            current - 1
        };
        self.state.details_pane = panes[next];
    }

    pub(super) fn trigger_episode_fetch(&mut self) {
        if let Some(id) = self.state.active_subject_id.clone() {
            let stype = self
                .state
                .selected_details
                .as_ref()
                .map(crate::tui::state::stype)
                .unwrap_or(1);

            let (se, ep) = if stype == 2 {
                let se_idx = self.state.season_list_state.selected().unwrap_or(0);
                let ep_idx = self.state.episode_list_state.selected().unwrap_or(0);

                let season_num = self
                    .state
                    .available_seasons
                    .get(se_idx)
                    .and_then(|s| s.get("se"))
                    .and_then(|s| s.as_i64())
                    .unwrap_or(1) as usize;

                let ep_num =
                    if let Some(ep_numbers) = self.state.available_episode_numbers.get(se_idx) {
                        ep_numbers.get(ep_idx).copied().unwrap_or(ep_idx + 1)
                    } else {
                        ep_idx + 1
                    };
                (season_num, ep_num)
            } else {
                (0, 0)
            };

            self.state.selected_season = se;
            self.state.selected_episode = ep;
            self.state.resource_list_state.select(None);
            self.state.stream_error = None;
            self.state.active_resource_request = self.state.active_resource_request.wrapping_add(1);

            let memory_cached = self
                .state
                .stream_pool
                .get(&id)
                .and_then(|pool| pool.episode_index.get(&(se, ep)))
                .filter(|streams| !streams.is_empty())
                .cloned();

            if let Some(streams) = memory_cached {
                if let Some(pool) = self.state.stream_pool.get_mut(&id) {
                    pool.episode_index.insert((se, ep), streams.clone());
                }
                self.state.selected_resources = None;
                self.state.is_loading = true;
                self.state.is_fetching_streams = true;
                self.state.set_status("Loading streams...".to_string(), 90);
                self.state.pending_episode_fetch = None;
                let sender = self.action_sender.clone();
                let context = self.request_context();
                let request_id = self.state.active_resource_request;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                    sender
                        .send(Action::EpisodeStreamsReady(
                            context,
                            request_id,
                            id,
                            se,
                            ep,
                            serde_json::Value::Array(streams),
                        ))
                        .ok();
                });
            } else {
                self.state.selected_resources = None;
                self.state.is_loading = true;
                self.state.is_fetching_streams = true;
                self.state.set_status("Loading streams...".to_string(), 90);

                self.state.pending_episode_fetch = Some((id.clone(), se, ep));
                self.state.last_episode_nav = std::time::Instant::now();
            }
        }
    }

    pub(super) fn get_selected_link(&self) -> Option<String> {
        self.state
            .selected_resources
            .as_ref()
            .and_then(|res| res.get("list"))
            .and_then(|l| l.as_array())
            .and_then(|list| {
                let idx = self.state.resource_list_state.selected().unwrap_or(0);
                list.get(idx)
            })
            .and_then(|file| file.get("resourceLink"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
    }

    pub(super) fn get_selected_resource_id(&self) -> Option<String> {
        self.state
            .selected_resources
            .as_ref()
            .and_then(|res| res.get("list"))
            .and_then(|l| l.as_array())
            .and_then(|list| {
                let idx = self.state.resource_list_state.selected().unwrap_or(0);
                list.get(idx)
            })
            .and_then(|file| file.get("resourceId"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
    }

    pub(super) fn get_selected_release(&self) -> Option<Release> {
        self.state
            .selected_resources
            .as_ref()?
            .get("list")?
            .as_array()?
            .get(self.state.resource_list_state.selected().unwrap_or(0))?
            .get("_fourk_release")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}
