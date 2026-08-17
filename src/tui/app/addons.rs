use super::App;
use crate::providers::addons::models::InstalledAddon;
use crate::tui::action::Action;
use crate::tui::overlay::NotificationKind;

impl App {
    fn reset_mode_state(&mut self) {
        self.state
            .fetch_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.fetch_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.provider_generation = self.state.provider_generation.wrapping_add(1);
        self.state.active_preview_request = self.state.active_preview_request.wrapping_add(1);

        self.state.active_browse_preset = None;
        self.state.browse_metrics.clear();
        self.state.tick_count = 0;
        self.reset_transient_overlays();
        self.state.input_mode = crate::tui::state::InputMode::Normal;
        self.state.is_loading = false;
        self.state.is_fetching_streams = false;
        self.state.pending_episode_fetch = None;
        self.state.selected_details = None;
        self.state.selected_resources = None;
        self.state.active_subject_id = None;
        self.state.search_suggestions.clear();
        self.state.suggest_index = None;
        self.state.search_preview = None;
        self.state.poster_image = None;
        self.state.poster_protocol = None;
        self.state.search_query.clear();
        self.state.search_results.clear();
    }

    pub(super) async fn handle_addons(&mut self, action: Action) -> Option<()> {
        match action {
            Action::ToggleAddonMode => {
                self.reset_mode_state();
                self.state.is_addon_mode = !self.state.is_addon_mode;
                if self.state.is_addon_mode {
                    self.state.is_tv_mode = false;
                    self.state.active_provider = crate::providers::models::ProviderKind::Addons;
                    self.load_installed_addons_from_config();
                    if self.state.installed_addons.is_empty() {
                        self.action_sender.send(Action::ShowAddonWizard).ok();
                    } else {
                        self.state.set_status("Addon mode active.".to_string(), 150);
                    }
                } else {
                    self.state.active_provider = crate::providers::models::ProviderKind::MovieBox;
                    self.state.set_status(
                        format!(
                            "Streaming mode active ({}).",
                            self.state.active_provider.label()
                        ),
                        150,
                    );
                }
            }

            Action::SwitchToStreamingMode => {
                self.reset_mode_state();
                self.state.is_addon_mode = false;
                self.state.is_tv_mode = false;
                self.state.active_provider = crate::providers::models::ProviderKind::MovieBox;
                self.state.set_status(
                    format!(
                        "Streaming mode active ({}).",
                        self.state.active_provider.label()
                    ),
                    150,
                );
            }

            Action::ShowAddonManager => {
                self.reset_transient_overlays();
                self.state.addon_manager_popup = true;
                self.state.addon_wizard_popup = false;
                self.state.input_mode = crate::tui::state::InputMode::Normal;
                self.state.addon_manager_selected = 1;
                self.state.addon_input_active = false;
                self.state.addon_input_buffer.clear();
                self.load_installed_addons_from_config();
            }

            Action::ShowAddonWizard => {
                self.reset_transient_overlays();
                self.state.addon_wizard_popup = true;
                self.state.addon_manager_popup = false;
                self.state.input_mode = crate::tui::state::InputMode::Normal;
                self.state.addon_wizard_selected = 0;
                self.state.addon_input_active = false;
                self.state.addon_input_buffer.clear();
            }

            Action::AddonAddManifest(manifest_url) => {
                let url = manifest_url.trim().to_string();
                if url.is_empty() {
                    return None;
                }

                self.state
                    .set_status("Verifying addon manifest...".to_string(), 200);
                let client = self.state.addon_client.clone();
                let sender = self.action_sender.clone();

                tokio::spawn(async move {
                    match client.fetch_manifest(&url).await {
                        Ok(manifest) => {
                            let installed = InstalledAddon::from_manifest(url.clone(), &manifest);
                            sender
                                .send(Action::SetStatus(format!(
                                    "Installed {} v{}",
                                    installed.name,
                                    installed.version.as_deref().unwrap_or("1.0.0")
                                )))
                                .ok();
                            let mut addons = crate::config::load_addons();
                            addons
                                .retain(|existing| existing.manifest_url != installed.manifest_url);
                            addons.push(installed);
                            crate::config::save_addons(&addons);
                            sender.send(Action::ShowAddonManager).ok();
                        }
                        Err(err) => {
                            sender
                                .send(Action::SetStatus(format!("Addon install failed: {err}")))
                                .ok();
                        }
                    }
                });
            }

            Action::AddonToggleEnabled(index) => {
                if index < self.state.installed_addons.len() {
                    if self.state.installed_addons[index].is_core() {
                        self.state.notify(
                            NotificationKind::Info,
                            "Core Provider",
                            "Cinemeta is the primary metadata provider and is locked enabled.",
                        );
                        return None;
                    }
                    self.state.installed_addons[index].enabled =
                        !self.state.installed_addons[index].enabled;
                    self.save_installed_addons();
                }
            }

            Action::AddonRemove(index) => {
                if index < self.state.installed_addons.len() {
                    if self.state.installed_addons[index].is_core() {
                        self.state.notify(
                            NotificationKind::Warning,
                            "Protected Addon",
                            "Cinemeta is the core metadata provider and cannot be uninstalled.",
                        );
                        return None;
                    }
                    let removed = self.state.installed_addons.remove(index);
                    self.save_installed_addons();
                    self.state.notify(
                        NotificationKind::Info,
                        "Addon Removed",
                        format!("Removed {}", removed.name),
                    );
                    if self.state.addon_manager_selected > self.state.installed_addons.len() {
                        self.state.addon_manager_selected = self.state.installed_addons.len();
                    }
                }
            }

            Action::AddonWizardSelect(index) => match index {
                0 => {
                    self.action_sender
                        .send(Action::AddonAddManifest(
                            "https://v3-cinemeta.strem.io/manifest.json".to_string(),
                        ))
                        .ok();
                }
                1 => {
                    self.action_sender
                        .send(Action::AddonAddManifest(
                            "https://anime-kitsu.strem.fun/manifest.json".to_string(),
                        ))
                        .ok();
                }
                2 => {
                    self.state.addon_input_active = true;
                    self.state.addon_input_buffer.clear();
                }
                _ => {}
            },

            Action::AddonInputToggle(active) => {
                self.state.addon_input_active = active;
                self.state.addon_input_buffer.clear();
            }

            _ => return None,
        }
        None
    }
}
