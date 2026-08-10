use super::{App, network};
use crate::providers::models::{ProviderKind, RequestContext};
use crate::tui::{
    action::Action,
    overlay::NotificationKind,
    state::{InputMode, Screen, SearchResult},
};

impl App {
    pub(super) fn handle_search_command(&mut self, query: &str, lower_query: &str) -> Option<bool> {
        if lower_query == "/clear-cache" {
            self.action_sender.send(Action::ClearCache).ok();
            self.state.search_query.clear();
            return Some(true);
        }
        if lower_query == "/github" {
            let _ = open::that("https://github.com/mesamirh/MovieBox-Tui");
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
            return Some(true);
        }
        if lower_query == "/update" {
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
            self.state.active_screen = Screen::Startup;
            self.state.update_available = None;
            self.state.manual_update_check = true;
            self.action_sender.send(Action::CheckForUpdates).ok();
            return Some(true);
        }
        if lower_query == "/theme" || lower_query == "/themes" {
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
            self.action_sender.send(Action::ToggleThemePopup).ok();
            return Some(true);
        }
        if lower_query == "/toggle-update" {
            self.state.auto_update = !self.state.auto_update;
            self.persist_config();
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
            self.state.notify(
                NotificationKind::Info,
                "Automatic updates",
                if self.state.auto_update {
                    "Enabled"
                } else {
                    "Disabled"
                },
            );
            return Some(true);
        }
        if lower_query == "/enable-bdix" || lower_query == "/disable-bdix" {
            let enable_req = lower_query == "/enable-bdix";
            if self.state.bdix_enabled == enable_req {
                self.state.search_query.clear();
                self.state.input_mode = InputMode::Normal;
                self.state.notify(
                    NotificationKind::Info,
                    "BDIX Providers",
                    if enable_req {
                        "Already Enabled"
                    } else {
                        "Already Disabled"
                    },
                );
                return Some(true);
            }

            self.state.bdix_enabled = enable_req;
            self.persist_config();
            self.state.search_query.clear();
            self.state.input_mode = InputMode::Normal;
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
            return Some(true);
        }

        if self.state.is_tv_mode {
            if lower_query == "/config" {
                self.action_sender.send(Action::ShowTvConfig).ok();
                self.state.search_query.clear();
                return Some(true);
            }
            if lower_query.starts_with('/') && lower_query != "/list" {
                self.state.set_status(
                    "Switch to streaming mode to use this command".to_string(),
                    150,
                );
                self.state.search_query.clear();
                return Some(true);
            }

            let q = lower_query.to_string();
            self.state.search_results = self
                .state
                .tv_channels
                .iter()
                .filter(|c| {
                    q == "/list"
                        || c.name.to_lowercase().contains(&q)
                        || c.group.to_lowercase().contains(&q)
                })
                .map(|c| SearchResult {
                    id: c.stream_url.clone(),
                    title: c.name.clone(),
                    stype: 3,
                    release_year: c.group.clone(),
                    cover_url: Some(c.logo.clone()),
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
                self.spawn_search_posters(
                    self.state
                        .search_results
                        .iter()
                        .take(15)
                        .map(|r| (r.id.clone(), r.cover_url.clone()))
                        .collect(),
                );
            }
            self.state.set_status(
                if self.state.search_results.is_empty() {
                    format!("No matches for '{}'.", query)
                } else {
                    format!("Found {} channels.", self.state.search_results.len())
                },
                150,
            );
            return Some(true);
        }

        if let Some(tid) = match lower_query {
            "/home" | "/discover" => Some("0"),
            "/movies" => Some("2"),
            "/shows" | "/tvshows" => Some("5"),
            "/anime" => Some("8"),
            _ => None,
        } {
            if self.state.active_provider != ProviderKind::MovieBox {
                self.state.set_status(
                    "This provider has no shared discover feed; enter a title to search.",
                    180,
                );
                return Some(true);
            }
            self.action_sender
                .send(Action::FetchHomepage {
                    tab_id: tid.to_string(),
                    page: 1,
                })
                .ok();
            return Some(true);
        }

        None
    }

    pub(super) fn prepare_search_request(&mut self, query: &str) -> RequestContext {
        self.state.is_homepage_mode = false;
        self.state.current_page = 1;
        self.state.active_screen = Screen::Home;
        self.state.selected_details = None;
        self.state.selected_resources = None;
        self.state.is_loading = true;
        self.state.search_error = None;
        self.state.search_list_state.select(Some(0));
        self.state.search_suggestions.clear();
        self.state.suggest_index = None;
        self.state.search_preview = None;
        self.state.preview_loading = false;
        self.state
            .set_status(format!("Searching for '{}'...", query), 150);
        self.request_context()
    }

    pub(super) fn run_search_request(
        &self,
        query: String,
        force_refresh: bool,
        context: RequestContext,
    ) {
        let page = 1;
        let sender = self.action_sender.clone();
        let client = self.client.clone();
        let fourk_client = self.fourk_client.clone();
        let circleftp_client = self.circleftp_client.clone();
        let dhakaflix_client = self.dhakaflix_client.clone();
        tokio::spawn(async move {
            if !force_refresh {
                let q = query.clone();
                let provider = context.provider;
                if let Ok(Some(cached)) = tokio::task::spawn_blocking(move || {
                    crate::cache::get_provider_search_cache(provider, &q)
                })
                .await
                {
                    sender
                        .send(Action::SearchSuccess {
                            context,
                            query: query.clone(),
                            page,
                            payload: cached,
                        })
                        .ok();
                    return;
                }
            }

            let result = network::provider_search(
                &client,
                &fourk_client,
                &circleftp_client,
                &dhakaflix_client,
                context.provider,
                &query,
                page,
            )
            .await;
            match result {
                Ok(res) => {
                    let q = query.clone();
                    let provider = context.provider;
                    let cached = res.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::cache::set_provider_search_cache(provider, &q, &cached);
                    });
                    sender
                        .send(Action::SearchSuccess {
                            context,
                            query,
                            page,
                            payload: res,
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(Action::SearchFailure(context, page, error))
                        .ok();
                }
            }
        });
    }

    pub(super) fn prepare_homepage_request(&mut self, tab_id: &str, page: usize) {
        self.state.is_homepage_mode = true;
        self.state.current_tab_id = tab_id.to_string();
        self.state.current_page = page;
        self.state.active_screen = Screen::Home;
        self.state.selected_details = None;
        self.state.selected_resources = None;
        self.state.is_loading = true;
        self.state.search_error = None;
        if page == 1 {
            self.state.search_results.clear();
            self.state.search_list_state.select(Some(0));
        }
        self.state.search_suggestions.clear();
        self.state.suggest_index = None;
        self.state
            .set_status("Loading discover tab...".to_string(), 150);
    }

    pub(super) fn run_homepage_request(&self, tab_id: String, page: usize) {
        let client = self.client.clone();
        let sender = self.action_sender.clone();
        tokio::spawn(async move {
            let t_clone = tab_id.clone();
            if let Ok(Some(cached)) = tokio::task::spawn_blocking(move || {
                crate::cache::get_homepage_cache(&t_clone, page)
            })
            .await
            {
                sender
                    .send(Action::HomepageSuccess {
                        tab_id: tab_id.clone(),
                        page,
                        payload: cached,
                    })
                    .ok();
                return;
            }

            match client.get_homepage(&tab_id, page).await {
                Ok(res) => {
                    let r_clone = res.clone();
                    let t_clone = tab_id.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::cache::set_homepage_cache(&t_clone, page, &r_clone);
                    });
                    sender
                        .send(Action::HomepageSuccess {
                            tab_id,
                            page,
                            payload: res,
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(Action::HomepageFailure(format!("{:?}", error)))
                        .ok();
                }
            }
        });
    }

    pub(super) fn extract_homepage_subjects(payload: &serde_json::Value) -> Vec<serde_json::Value> {
        let mut extracted_subjects = Vec::new();
        if let Some(items) = payload.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Some(banner) = item
                    .get("banner")
                    .and_then(|b| b.get("banners"))
                    .and_then(|b| b.as_array())
                {
                    for banner_item in banner {
                        if let Some(subject) = banner_item.get("subject") {
                            extracted_subjects.push(subject.clone());
                        }
                    }
                }
                if let Some(custom_data) = item
                    .get("customData")
                    .and_then(|c| c.get("items"))
                    .and_then(|i| i.as_array())
                {
                    for custom_item in custom_data {
                        if let Some(subject) = custom_item.get("subject") {
                            extracted_subjects.push(subject.clone());
                        }
                    }
                }
                if let Some(subjects) = item.get("subjects").and_then(|s| s.as_array()) {
                    for subject in subjects {
                        extracted_subjects.push(subject.clone());
                    }
                }
            }
        }
        extracted_subjects
    }

    pub(super) fn append_homepage_subjects(&mut self, subjects: Vec<serde_json::Value>) -> usize {
        let mut count = 0;
        for item in subjects {
            let id = item
                .get("subjectId")
                .and_then(|si| si.as_str())
                .unwrap_or("")
                .to_string();
            let raw_title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let clean_title = crate::providers::moviebox::clean_moviebox_title(&raw_title);
            let stype = item
                .get("subjectType")
                .and_then(|st| st.as_i64())
                .unwrap_or(0);
            let release_year = item
                .get("releaseDate")
                .and_then(|rd| rd.as_str())
                .unwrap_or("")
                .split('-')
                .next()
                .unwrap_or("")
                .to_string();
            let cover_url = item
                .get("cover")
                .and_then(|c| c.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());
            let season = item.get("season").and_then(|s| s.as_u64()).unwrap_or(0) as usize;

            if let Some(existing) = self.state.search_results.iter_mut().find(|r| r.id == id) {
                if season > existing.season {
                    existing.season = season;
                    existing.title = clean_title;
                    existing.stype = stype;
                    existing.release_year = release_year;
                    existing.cover_url = cover_url;
                }
                continue;
            }

            let raw_lower = raw_title.to_lowercase();
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
                self.state.search_results.push(SearchResult {
                    id,
                    title: clean_title,
                    stype,
                    release_year,
                    cover_url,
                    season,
                    episode: 1,
                    provider: ProviderKind::MovieBox,
                });
                count += 1;
            }
        }
        count
    }

    pub(super) fn spawn_search_posters(&self, results: Vec<(String, Option<String>)>) {
        let sender = self.action_sender.clone();
        let req_client = self.client.http_client().clone();
        tokio::spawn(async move {
            let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
            for (id, cover_url) in results {
                let Some(url) = cover_url else {
                    continue;
                };
                if url.is_empty() {
                    continue;
                }
                let permit = sem.clone().acquire_owned().await.ok();
                let tx = sender.clone();
                let client = req_client.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Some(bytes) = network::fetch_poster_bytes(&client, &url).await
                        && let Some(img) = network::decode_poster(bytes).await
                    {
                        tx.send(Action::SearchPosterLoaded(id, Some(img))).ok();
                    }
                });
            }
        });
    }
}
