use crate::tui::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommand {
    Browse,
    History,
    List,
    Config,
    DownloadDir,
    Theme,
    Update,
    ToggleUpdate,
    ClearCache,
    Github,
    EnableBdix,
    DisableBdix,
    EnableStreaming,
    DisableStreaming,
    EnableTv,
    DisableTv,
    EnableAddons,
    DisableAddons,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand<'a> {
    Browse,
    History,
    List,
    Config,
    DownloadDir(&'a str),
    Theme,
    Update,
    ToggleUpdate,
    ClearCache,
    Github,
    EnableBdix,
    DisableBdix,
    EnableStreaming,
    DisableStreaming,
    EnableTv,
    DisableTv,
    EnableAddons,
    DisableAddons,
}

impl SlashCommand {
    pub const ALL: [Self; 18] = [
        Self::Browse,
        Self::History,
        Self::List,
        Self::Config,
        Self::DownloadDir,
        Self::Theme,
        Self::Update,
        Self::ToggleUpdate,
        Self::ClearCache,
        Self::Github,
        Self::EnableBdix,
        Self::DisableBdix,
        Self::EnableStreaming,
        Self::DisableStreaming,
        Self::EnableTv,
        Self::DisableTv,
        Self::EnableAddons,
        Self::DisableAddons,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Browse => "/browse",
            Self::History => "/history",
            Self::List => "/list",
            Self::Config => "/config",
            Self::DownloadDir => "/download-dir",
            Self::Theme => "/theme",
            Self::Update => "/update",
            Self::ToggleUpdate => "/toggle-update",
            Self::ClearCache => "/clear-cache",
            Self::Github => "/github",
            Self::EnableBdix => "/enable-bdix",
            Self::DisableBdix => "/disable-bdix",
            Self::EnableStreaming => "/enable-streaming",
            Self::DisableStreaming => "/disable-streaming",
            Self::EnableTv => "/enable-tv",
            Self::DisableTv => "/disable-tv",
            Self::EnableAddons => "/enable-addons",
            Self::DisableAddons => "/disable-addons",
        }
    }

    pub fn description(self, _state: &AppState) -> &'static str {
        match self {
            Self::Browse => "Curated, rated & most-watched views",
            Self::History => "Watch history",
            Self::List => "Show all TV channels",
            Self::Config => "Configure IPTV playlists",
            Self::DownloadDir => "View or change download folder",
            Self::Theme => "Theme picker",
            Self::Update => "Check for newer release",
            Self::ToggleUpdate => "Toggle automatic update checks",
            Self::ClearCache => "Clear cached data",
            Self::Github => "Open project repository",
            Self::EnableBdix => "Enable BDIX FTP sources",
            Self::DisableBdix => "Disable BDIX FTP sources",
            Self::EnableStreaming => "Enable Streaming mode navigation",
            Self::DisableStreaming => "Disable Streaming mode navigation",
            Self::EnableTv => "Enable TV mode navigation",
            Self::DisableTv => "Disable TV mode navigation",
            Self::EnableAddons => "Enable Addon mode navigation",
            Self::DisableAddons => "Disable Addon mode navigation",
        }
    }

    pub fn is_available(self, state: &AppState) -> bool {
        match self {
            Self::Browse => {
                (state.streaming_enabled && !state.is_tv_mode && !state.is_addon_mode)
                    || (state.addons_enabled && state.is_addon_mode)
            }
            Self::History => state.streaming_enabled && !state.is_tv_mode && !state.is_addon_mode,
            Self::List | Self::Config => state.tv_enabled && state.is_tv_mode,
            Self::EnableBdix => {
                state.streaming_enabled
                    && !state.is_tv_mode
                    && !state.is_addon_mode
                    && !state.bdix_enabled
            }
            Self::DisableBdix => {
                state.streaming_enabled
                    && !state.is_tv_mode
                    && !state.is_addon_mode
                    && state.bdix_enabled
            }
            Self::EnableStreaming => !state.streaming_enabled,
            Self::DisableStreaming => state.streaming_enabled,
            Self::EnableTv => !state.tv_enabled,
            Self::DisableTv => state.tv_enabled,
            Self::EnableAddons => !state.addons_enabled,
            Self::DisableAddons => state.addons_enabled,
            Self::DownloadDir
            | Self::Theme
            | Self::Update
            | Self::ToggleUpdate
            | Self::ClearCache
            | Self::Github => true,
        }
    }

    pub fn suggest(state: &AppState, query: &str) -> Vec<Self> {
        let lower = query.to_ascii_lowercase();
        Self::ALL
            .iter()
            .copied()
            .filter(|cmd| cmd.is_available(state) && cmd.name().starts_with(&lower))
            .collect()
    }

    pub fn description_for(suggestion: &str, state: &AppState) -> Option<&'static str> {
        let trimmed = if suggestion.starts_with('/') {
            suggestion
        } else {
            return None;
        };

        for cmd in Self::ALL {
            if cmd.name() == trimmed {
                return Some(cmd.description(state));
            }
        }
        None
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
            "/browse" => Some(ParsedCommand::Browse),
            "/history" => Some(ParsedCommand::History),
            "/list" => Some(ParsedCommand::List),
            "/config" => Some(ParsedCommand::Config),
            "/download-dir" => Some(ParsedCommand::DownloadDir(arg)),
            "/theme" => Some(ParsedCommand::Theme),
            "/update" => Some(ParsedCommand::Update),
            "/toggle-update" => Some(ParsedCommand::ToggleUpdate),
            "/clear-cache" => Some(ParsedCommand::ClearCache),
            "/github" => Some(ParsedCommand::Github),
            "/enable-bdix" => Some(ParsedCommand::EnableBdix),
            "/disable-bdix" => Some(ParsedCommand::DisableBdix),
            "/enable-streaming" => Some(ParsedCommand::EnableStreaming),
            "/disable-streaming" => Some(ParsedCommand::DisableStreaming),
            "/enable-tv" => Some(ParsedCommand::EnableTv),
            "/disable-tv" => Some(ParsedCommand::DisableTv),
            "/enable-addons" => Some(ParsedCommand::EnableAddons),
            "/disable-addons" => Some(ParsedCommand::DisableAddons),
            _ => None,
        }
    }
}
