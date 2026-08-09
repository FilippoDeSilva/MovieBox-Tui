use crate::providers::models::ProviderKind;
use std::path::PathBuf;

pub struct Config {
    pub auto_update: bool,
    pub last_update_check: u64,
    pub active_provider: ProviderKind,
    pub active_theme: String,
    pub bdix_enabled: bool,
    pub default_player: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_update: true,
            last_update_check: 0,
            active_provider: ProviderKind::MovieBox,
            active_theme: String::new(),
            bdix_enabled: false,
            default_player: None,
        }
    }
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("moviebox-tui").join("config.json"))
}

pub fn load() -> Config {
    let mut config = Config::default();
    let Some(path) = path() else {
        return config;
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return config;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return config;
    };
    if let Some(v) = value.get("auto_update").and_then(|v| v.as_bool()) {
        config.auto_update = v;
    }
    if let Some(v) = value.get("last_update_check").and_then(|v| v.as_u64()) {
        config.last_update_check = v;
    }
    if value.get("active_provider").and_then(|v| v.as_str())
        == Some(ProviderKind::FourKHdHub.cache_key())
    {
        config.active_provider = ProviderKind::FourKHdHub;
    }
    if let Some(v) = value.get("active_theme").and_then(|v| v.as_str()) {
        config.active_theme = v.to_string();
    }
    if let Some(v) = value.get("bdix_enabled").and_then(|v| v.as_bool()) {
        config.bdix_enabled = v;
    }
    if let Some(v) = value.get("default_player").and_then(|v| v.as_str()) {
        config.default_player = Some(v.to_string());
    }
    config
}

pub fn save(config: &Config) {
    let Some(path) = path() else {
        return;
    };
    if let Some(app_dir) = path.parent()
        && std::fs::create_dir_all(app_dir).is_err()
    {
        return;
    }
    let json = serde_json::json!({
        "auto_update": config.auto_update,
        "last_update_check": config.last_update_check,
        "active_provider": config.active_provider.cache_key(),
        "active_theme": config.active_theme,
        "bdix_enabled": config.bdix_enabled,
        "default_player": config.default_player
    });
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    if std::fs::write(&temporary, json.to_string()).is_ok()
        && std::fs::rename(&temporary, &path).is_err()
    {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::rename(&temporary, &path);
    }
}
