use crate::tui::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommand {
    Settings,
    Browse,
    History,
    Favorites,
    Theme,
    Clear,
    Help,
    List,
    Config,
    DownloadDir,
    Update,
    ClearCache,
    Github,
    ToggleUpdate,
    ToggleBdix,
    ToggleStreaming,
    ToggleTv,
    ToggleAddons,
    Probe,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand<'a> {
    Settings,
    Browse,
    History,
    Favorites,
    Theme,
    Clear,
    Help,
    List,
    Config,
    DownloadDir(&'a str),
    Update,
    ClearCache,
    Github,
    ToggleUpdate,
    ToggleBdix,
    ToggleStreaming,
    ToggleTv,
    ToggleAddons,
    Probe,
    Exit,
}

impl SlashCommand {
    pub const ALL: [Self; 20] = [
        Self::Settings,
        Self::Browse,
        Self::History,
        Self::Favorites,
        Self::Theme,
        Self::Clear,
        Self::Help,
        Self::List,
        Self::Config,
        Self::DownloadDir,
        Self::Update,
        Self::ClearCache,
        Self::Github,
        Self::ToggleUpdate,
        Self::ToggleBdix,
        Self::ToggleStreaming,
        Self::ToggleTv,
        Self::ToggleAddons,
        Self::Probe,
        Self::Exit,
    ];

    pub const PRIMARY: [Self; 7] = [
        Self::Settings,
        Self::Browse,
        Self::History,
        Self::Favorites,
        Self::Theme,
        Self::Clear,
        Self::Help,
    ];

    pub const TOGGLE_COMMANDS: [Self; 5] = [
        Self::ToggleUpdate,
        Self::ToggleBdix,
        Self::ToggleStreaming,
        Self::ToggleTv,
        Self::ToggleAddons,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Settings => "/settings",
            Self::Browse => "/browse",
            Self::History => "/history",
            Self::Favorites => "/favorites",
            Self::Theme => "/theme",
            Self::Clear => "/clear",
            Self::Help => "/help",
            Self::List => "/list",
            Self::Config => "/config",
            Self::DownloadDir => "/download-dir",
            Self::Update => "/update",
            Self::ClearCache => "/clear-cache",
            Self::Github => "/github",
            Self::ToggleUpdate => "/toggle-update",
            Self::ToggleBdix => "/toggle-bdix",
            Self::ToggleStreaming => "/toggle-streaming",
            Self::ToggleTv => "/toggle-tv",
            Self::ToggleAddons => "/toggle-addons",
            Self::Probe => "/probe",
            Self::Exit => "/exit",
        }
    }

    pub fn description(self, state: &AppState) -> &'static str {
        match self {
            Self::Settings => "Interactive preferences, content modes & configuration",
            Self::Browse => "Curated, rated & most-watched views",
            Self::History => "Watch history",
            Self::Favorites => "Starred titles",
            Self::Theme => "Theme picker",
            Self::Clear => "Clear search results and return to landing",
            Self::Help => "Open interactive keybinding help menu",
            Self::List => "Show all TV channels",
            Self::Config => {
                if state.is_addon_mode {
                    "Configure HTTP addons"
                } else if state.is_tv_mode {
                    "Configure IPTV playlists"
                } else {
                    "Interactive preferences and configuration"
                }
            }
            Self::DownloadDir => "View or change download folder",
            Self::Update => "Check for newer release",
            Self::ClearCache => "Clear cached data",
            Self::Github => "Open project repository",
            Self::ToggleUpdate => "Toggle automatic update checks",
            Self::ToggleBdix => "Toggle BDIX FTP sources",
            Self::ToggleStreaming => "Toggle Streaming mode navigation",
            Self::ToggleTv => "Toggle TV mode navigation",
            Self::ToggleAddons => "Toggle Addon mode navigation",
            Self::Probe => "Re-detect terminal graphics support",
            Self::Exit => "Exit application and restore terminal",
        }
    }

    pub fn is_available(self, state: &AppState) -> bool {
        match self {
            Self::Settings => true,
            Self::Browse => {
                (state.streaming_enabled && !state.is_tv_mode && !state.is_addon_mode)
                    || (state.addons_enabled && state.is_addon_mode)
            }
            Self::History => {
                (state.streaming_enabled && !state.is_tv_mode && !state.is_addon_mode)
                    || (state.addons_enabled && state.is_addon_mode)
            }
            Self::Favorites => state.favorites_available(),
            Self::List => state.tv_enabled && state.is_tv_mode,
            Self::Config => true,
            Self::ToggleBdix => !state.is_tv_mode && !state.is_addon_mode,
            Self::Theme
            | Self::Clear
            | Self::Help
            | Self::DownloadDir
            | Self::Update
            | Self::ClearCache
            | Self::Github
            | Self::ToggleUpdate
            | Self::ToggleStreaming
            | Self::ToggleTv
            | Self::ToggleAddons
            | Self::Probe
            | Self::Exit => true,
        }
    }

    pub fn suggest(state: &AppState, query: &str) -> Vec<String> {
        let lower = query.to_ascii_lowercase();
        let mut results = Vec::new();

        if lower.starts_with("/toggle-") {
            for cmd in Self::TOGGLE_COMMANDS {
                if !cmd.is_available(state) {
                    continue;
                }
                let name = cmd.name();
                if name.starts_with(&lower) {
                    results.push(name.to_string());
                }
            }
            return results;
        }

        let primary_suggestions: [(&str, Self); 7] = [
            ("/settings", Self::Settings),
            ("/browse", Self::Browse),
            ("/history", Self::History),
            ("/favorites", Self::Favorites),
            ("/theme", Self::Theme),
            ("/clear", Self::Clear),
            ("/help", Self::Help),
        ];

        for (name, cmd) in primary_suggestions {
            if !cmd.is_available(state) {
                continue;
            }
            if name.starts_with(&lower) {
                results.push(name.to_string());
            }
        }

        results
    }

    pub fn description_for(suggestion: &str, state: &AppState) -> Option<&'static str> {
        let trimmed = if suggestion.starts_with('/') {
            suggestion.trim()
        } else {
            return None;
        };

        if trimmed == "/download-dir reset" {
            return Some("Reset download folder to default");
        }
        if trimmed == "/pref" || trimmed == "/preferences" || trimmed == "/options" {
            return Some("Interactive preferences, content modes & configuration");
        }
        if trimmed == "/?" {
            return Some("Open interactive keybinding help menu");
        }

        match trimmed {
            "/settings" => Some(Self::Settings.description(state)),
            "/browse" => Some(Self::Browse.description(state)),
            "/history" => Some(Self::History.description(state)),
            "/favorites" => Some(Self::Favorites.description(state)),
            "/theme" => Some(Self::Theme.description(state)),
            "/clear" => Some(Self::Clear.description(state)),
            "/help" => Some(Self::Help.description(state)),
            "/list" => Some(Self::List.description(state)),
            "/config" => Some(Self::Config.description(state)),
            "/download-dir" => Some(Self::DownloadDir.description(state)),
            "/update" => Some(Self::Update.description(state)),
            "/clear-cache" => Some(Self::ClearCache.description(state)),
            "/github" => Some(Self::Github.description(state)),
            "/toggle-update" => Some(Self::ToggleUpdate.description(state)),
            "/toggle-bdix" => Some(Self::ToggleBdix.description(state)),
            "/toggle-streaming" => Some(Self::ToggleStreaming.description(state)),
            "/toggle-tv" => Some(Self::ToggleTv.description(state)),
            "/toggle-addons" => Some(Self::ToggleAddons.description(state)),
            "/probe" => Some(Self::Probe.description(state)),
            "/exit" | "/quit" | "/q" => Some(Self::Exit.description(state)),
            _ => None,
        }
    }

    pub fn parse(input: &str) -> Option<ParsedCommand<'_>> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let command_name = parts.next()?;
        let arg = parts.next().unwrap_or("").trim();

        match command_name.to_ascii_lowercase().as_str() {
            "/settings" | "/pref" | "/preferences" | "/options" => Some(ParsedCommand::Settings),
            "/browse" => Some(ParsedCommand::Browse),
            "/history" => Some(ParsedCommand::History),
            "/favorites" => Some(ParsedCommand::Favorites),
            "/theme" => Some(ParsedCommand::Theme),
            "/clear" => Some(ParsedCommand::Clear),
            "/help" | "/?" => Some(ParsedCommand::Help),
            "/list" => Some(ParsedCommand::List),
            "/config" => Some(ParsedCommand::Config),
            "/download-dir" => Some(ParsedCommand::DownloadDir(arg)),
            "/update" => Some(ParsedCommand::Update),
            "/clear-cache" => Some(ParsedCommand::ClearCache),
            "/github" => Some(ParsedCommand::Github),
            "/toggle-update" => Some(ParsedCommand::ToggleUpdate),
            "/toggle-bdix" | "/enable-bdix" | "/disable-bdix" => Some(ParsedCommand::ToggleBdix),
            "/toggle-streaming" | "/enable-streaming" | "/disable-streaming" => {
                Some(ParsedCommand::ToggleStreaming)
            }
            "/toggle-tv" | "/enable-tv" | "/disable-tv" => Some(ParsedCommand::ToggleTv),
            "/toggle-addons" | "/enable-addons" | "/disable-addons" => {
                Some(ParsedCommand::ToggleAddons)
            }
            "/probe" => Some(ParsedCommand::Probe),
            "/exit" | "/quit" | "/q" => Some(ParsedCommand::Exit),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_primary_commands_suggest() {
        let state = AppState::default();
        let suggestions = SlashCommand::suggest(&state, "/");
        assert_eq!(
            suggestions,
            vec![
                "/settings".to_string(),
                "/browse".to_string(),
                "/history".to_string(),
                "/favorites".to_string(),
                "/theme".to_string(),
                "/clear".to_string(),
                "/help".to_string(),
            ]
        );

        let s_sug = SlashCommand::suggest(&state, "/s");
        assert_eq!(s_sug, vec!["/settings".to_string()]);

        let c_sug = SlashCommand::suggest(&state, "/c");
        assert_eq!(c_sug, vec!["/clear".to_string()]);

        let p_sug = SlashCommand::suggest(&state, "/p");
        assert!(p_sug.is_empty());

        let d_sug = SlashCommand::suggest(&state, "/download-dir");
        assert!(d_sug.is_empty());
    }
    #[test]
    fn test_favorites_command_parses_and_is_available_by_default() {
        let state = AppState::default();
        assert_eq!(
            SlashCommand::parse("/favorites"),
            Some(ParsedCommand::Favorites)
        );
        assert!(SlashCommand::Favorites.is_available(&state));
        assert_eq!(SlashCommand::Favorites.name(), "/favorites");
    }

    #[test]
    fn test_favorites_command_unavailable_in_tv_mode() {
        let state = AppState {
            is_tv_mode: true,
            ..Default::default()
        };
        assert!(!SlashCommand::Favorites.is_available(&state));
    }

    #[test]
    fn test_toggle_commands_and_aliases_parse() {
        let state = AppState::default();
        assert_eq!(SlashCommand::ALL.len(), 20);
        assert_eq!(SlashCommand::parse("/exit"), Some(ParsedCommand::Exit));
        assert_eq!(SlashCommand::parse("/quit"), Some(ParsedCommand::Exit));
        assert_eq!(SlashCommand::parse("/q"), Some(ParsedCommand::Exit));
        assert!(SlashCommand::Exit.is_available(&state));
        assert_eq!(SlashCommand::Exit.name(), "/exit");
        assert_eq!(SlashCommand::parse("/clear"), Some(ParsedCommand::Clear));
        assert!(SlashCommand::Clear.is_available(&state));
        assert_eq!(SlashCommand::Clear.name(), "/clear");
        assert_eq!(SlashCommand::parse("/help"), Some(ParsedCommand::Help));
        assert_eq!(SlashCommand::parse("/?"), Some(ParsedCommand::Help));
        assert!(SlashCommand::Help.is_available(&state));
        assert_eq!(SlashCommand::Help.name(), "/help");

        assert_eq!(
            SlashCommand::parse("/toggle-tv"),
            Some(ParsedCommand::ToggleTv)
        );
        assert_eq!(
            SlashCommand::parse("/enable-tv"),
            Some(ParsedCommand::ToggleTv)
        );
        assert_eq!(
            SlashCommand::parse("/disable-tv"),
            Some(ParsedCommand::ToggleTv)
        );

        assert_eq!(
            SlashCommand::parse("/toggle-addons"),
            Some(ParsedCommand::ToggleAddons)
        );
        assert_eq!(
            SlashCommand::parse("/enable-addons"),
            Some(ParsedCommand::ToggleAddons)
        );
        assert_eq!(
            SlashCommand::parse("/disable-addons"),
            Some(ParsedCommand::ToggleAddons)
        );

        let toggle_no_dash = SlashCommand::suggest(&state, "/toggle");
        assert!(toggle_no_dash.is_empty());

        let toggle_sug = SlashCommand::suggest(&state, "/toggle-");
        assert!(toggle_sug.contains(&"/toggle-update".to_string()));
        assert!(toggle_sug.contains(&"/toggle-bdix".to_string()));

        let probe_sug = SlashCommand::suggest(&state, "/probe");
        assert!(probe_sug.is_empty());
    }

    #[test]
    fn test_download_dir_reset_description() {
        let state = AppState::default();
        assert_eq!(
            SlashCommand::description_for("/download-dir reset", &state),
            Some("Reset download folder to default")
        );
        assert_eq!(
            SlashCommand::description_for("/download-dir", &state),
            Some("View or change download folder")
        );
    }
}
