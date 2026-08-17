use crate::tui::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommand {
    Browse,
    History,
    List,
    Reload,
    Config,
    Addons,
    DownloadDir,
    Theme,
    Update,
    ToggleUpdate,
    ClearCache,
    Github,
    EnableBdix,
    DisableBdix,
    EnableAddons,
    DisableAddons,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommand<'a> {
    Browse,
    History,
    List,
    Reload,
    Config,
    Addons,
    DownloadDir(&'a str),
    Theme,
    Update,
    ToggleUpdate,
    ClearCache,
    Github,
    EnableBdix,
    DisableBdix,
    EnableAddons,
    DisableAddons,
}

impl SlashCommand {
    pub const ALL: [Self; 16] = [
        Self::Browse,
        Self::History,
        Self::List,
        Self::Reload,
        Self::Config,
        Self::Addons,
        Self::DownloadDir,
        Self::Theme,
        Self::Update,
        Self::ToggleUpdate,
        Self::ClearCache,
        Self::Github,
        Self::EnableBdix,
        Self::DisableBdix,
        Self::EnableAddons,
        Self::DisableAddons,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Browse => "/browse",
            Self::History => "/history",
            Self::List => "/list",
            Self::Reload => "/reload",
            Self::Config => "/config",
            Self::Addons => "/addons",
            Self::DownloadDir => "/download-dir",
            Self::Theme => "/theme",
            Self::Update => "/update",
            Self::ToggleUpdate => "/toggle-update",
            Self::ClearCache => "/clear-cache",
            Self::Github => "/github",
            Self::EnableBdix => "/enable-bdix",
            Self::DisableBdix => "/disable-bdix",
            Self::EnableAddons => "/enable-addons",
            Self::DisableAddons => "/disable-addons",
        }
    }

    pub fn description(self, state: &AppState) -> &'static str {
        match self {
            Self::Browse => "Curated, rated & most-watched views",
            Self::History => "Watch history",
            Self::List => "Show all TV channels",
            Self::Reload => {
                if state.is_tv_mode {
                    "Reload IPTV playlists"
                } else {
                    "Refresh catalog & streams"
                }
            }
            Self::Config => "Configure IPTV playlists",
            Self::Addons => "Open Addon Manager dialog",
            Self::DownloadDir => "View or change download folder",
            Self::Theme => "Theme picker",
            Self::Update => "Check for newer release",
            Self::ToggleUpdate => "Toggle automatic update checks",
            Self::ClearCache => "Clear cached data",
            Self::Github => "Open project repository",
            Self::EnableBdix => "Enable BDIX FTP sources",
            Self::DisableBdix => "Disable BDIX FTP sources",
            Self::EnableAddons => "Enable Addon mode navigation",
            Self::DisableAddons => "Disable Addon mode navigation",
        }
    }

    pub fn is_available(self, state: &AppState) -> bool {
        match self {
            Self::Browse | Self::History => !state.is_tv_mode && !state.is_addon_mode,
            Self::List | Self::Config => state.is_tv_mode,
            Self::Reload => state.is_tv_mode,
            Self::Addons => state.is_addon_mode || (!state.is_tv_mode && state.addons_enabled),
            Self::EnableBdix => !state.is_tv_mode && !state.is_addon_mode && !state.bdix_enabled,
            Self::DisableBdix => !state.is_tv_mode && !state.is_addon_mode && state.bdix_enabled,
            Self::EnableAddons => !state.is_tv_mode && !state.addons_enabled,
            Self::DisableAddons => !state.is_tv_mode && state.addons_enabled,
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
            "/reload" => Some(ParsedCommand::Reload),
            "/config" => Some(ParsedCommand::Config),
            "/addons" => Some(ParsedCommand::Addons),
            "/download-dir" => Some(ParsedCommand::DownloadDir(arg)),
            "/theme" => Some(ParsedCommand::Theme),
            "/update" => Some(ParsedCommand::Update),
            "/toggle-update" => Some(ParsedCommand::ToggleUpdate),
            "/clear-cache" => Some(ParsedCommand::ClearCache),
            "/github" => Some(ParsedCommand::Github),
            "/enable-bdix" => Some(ParsedCommand::EnableBdix),
            "/disable-bdix" => Some(ParsedCommand::DisableBdix),
            "/enable-addons" => Some(ParsedCommand::EnableAddons),
            "/disable-addons" => Some(ParsedCommand::DisableAddons),
            _ => None,
        }
    }
}
