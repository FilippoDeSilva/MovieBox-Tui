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
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand {
    Settings,
    Browse,
    History,
    Favorites,
    Theme,
    Clear,
    Help,
    List,
    Exit,
}

impl SlashCommand {
    pub const ALL: [Self; 9] = [
        Self::Settings,
        Self::Browse,
        Self::History,
        Self::Favorites,
        Self::Theme,
        Self::Clear,
        Self::Help,
        Self::List,
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
            Self::Exit => "/exit",
        }
    }

    pub fn description(self, _state: &AppState) -> &'static str {
        match self {
            Self::Settings => "Interactive preferences, content modes & configuration",
            Self::Browse => "Curated, rated & most-watched views",
            Self::History => "Watch history",
            Self::Favorites => "Starred titles",
            Self::Theme => "Theme picker",
            Self::Clear => "Clear search results and return to landing",
            Self::Help => "Open interactive keybinding help menu",
            Self::List => "Show all TV channels",
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
            Self::Theme | Self::Clear | Self::Help | Self::Exit => true,
        }
    }

    pub fn suggest(state: &AppState, query: &str) -> Vec<String> {
        let lower = query.to_ascii_lowercase();
        let mut results = Vec::new();

        let candidates: [(&str, Self); 8] = [
            ("/settings", Self::Settings),
            ("/browse", Self::Browse),
            ("/history", Self::History),
            ("/favorites", Self::Favorites),
            ("/theme", Self::Theme),
            ("/clear", Self::Clear),
            ("/help", Self::Help),
            ("/list", Self::List),
        ];

        for (name, cmd) in candidates {
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

        if trimmed == "/pref"
            || trimmed == "/preferences"
            || trimmed == "/options"
            || trimmed == "/config"
        {
            return Some(Self::Settings.description(state));
        }
        if trimmed == "/?" {
            return Some(Self::Help.description(state));
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
            "/exit" | "/quit" | "/q" => Some(Self::Exit.description(state)),
            _ => None,
        }
    }

    pub fn parse(input: &str) -> Option<ParsedCommand> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let mut parts = trimmed.split_whitespace();
        let command_name = parts.next()?;

        match command_name.to_ascii_lowercase().as_str() {
            "/settings" | "/config" | "/pref" | "/preferences" | "/options" => {
                Some(ParsedCommand::Settings)
            }
            "/browse" => Some(ParsedCommand::Browse),
            "/history" => Some(ParsedCommand::History),
            "/favorites" => Some(ParsedCommand::Favorites),
            "/theme" => Some(ParsedCommand::Theme),
            "/clear" => Some(ParsedCommand::Clear),
            "/help" | "/?" => Some(ParsedCommand::Help),
            "/list" => Some(ParsedCommand::List),
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

        let tv_state = AppState {
            is_tv_mode: true,
            tv_enabled: true,
            ..Default::default()
        };
        let list_sug = SlashCommand::suggest(&tv_state, "/l");
        assert_eq!(list_sug, vec!["/list".to_string()]);
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
    fn test_core_commands_and_aliases_parse() {
        let state = AppState::default();
        assert_eq!(SlashCommand::ALL.len(), 9);
        assert_eq!(SlashCommand::PRIMARY.len(), 7);
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
            SlashCommand::parse("/settings"),
            Some(ParsedCommand::Settings)
        );
        assert_eq!(
            SlashCommand::parse("/config"),
            Some(ParsedCommand::Settings)
        );
        assert_eq!(SlashCommand::parse("/pref"), Some(ParsedCommand::Settings));
        assert_eq!(
            SlashCommand::parse("/preferences"),
            Some(ParsedCommand::Settings)
        );
        assert_eq!(
            SlashCommand::parse("/options"),
            Some(ParsedCommand::Settings)
        );

        assert_eq!(SlashCommand::parse("/list"), Some(ParsedCommand::List));
        assert_eq!(SlashCommand::parse("/browse"), Some(ParsedCommand::Browse));
        assert_eq!(
            SlashCommand::parse("/history"),
            Some(ParsedCommand::History)
        );

        assert_eq!(SlashCommand::parse("/toggle-tv"), None);
        assert_eq!(SlashCommand::parse("/enable-tv"), None);
        assert_eq!(SlashCommand::parse("/disable-tv"), None);
        assert_eq!(SlashCommand::parse("/toggle-addons"), None);
        assert_eq!(SlashCommand::parse("/download-dir"), None);
        assert_eq!(SlashCommand::parse("/download-dir ~/Movies"), None);
        assert_eq!(SlashCommand::parse("/update"), None);
        assert_eq!(SlashCommand::parse("/clear-cache"), None);
        assert_eq!(SlashCommand::parse("/github"), None);
        assert_eq!(SlashCommand::parse("/probe"), None);

        assert!(SlashCommand::suggest(&state, "/toggle").is_empty());
        assert!(SlashCommand::suggest(&state, "/toggle-").is_empty());
        assert!(SlashCommand::suggest(&state, "/probe").is_empty());
    }

    #[test]
    fn test_descriptions_and_aliases() {
        let state = AppState::default();
        assert_eq!(
            SlashCommand::description_for("/settings", &state),
            Some("Interactive preferences, content modes & configuration")
        );
        assert_eq!(
            SlashCommand::description_for("/config", &state),
            Some("Interactive preferences, content modes & configuration")
        );
        assert_eq!(
            SlashCommand::description_for("/pref", &state),
            Some("Interactive preferences, content modes & configuration")
        );
        assert_eq!(
            SlashCommand::description_for("/?", &state),
            Some("Open interactive keybinding help menu")
        );
        assert_eq!(SlashCommand::description_for("/download-dir", &state), None);
        assert_eq!(SlashCommand::description_for("/toggle-tv", &state), None);
    }
}
