use crate::providers::addons::models::InstalledAddon;
use crate::providers::models::ProviderKind;
use std::path::PathBuf;

pub struct Config {
    pub auto_update: bool,
    pub last_update_check: u64,
    pub active_provider: ProviderKind,
    pub active_theme: String,
    pub bdix_enabled: bool,
    pub addons_enabled: bool,
    pub default_player: Option<String>,
    pub download_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_update: true,
            last_update_check: 0,
            active_provider: ProviderKind::MovieBox,
            active_theme: String::new(),
            bdix_enabled: false,
            addons_enabled: false,
            default_player: None,
            download_dir: None,
        }
    }
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("moviebox-tui").join("config.json"))
}

pub fn addons_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("moviebox-tui").join("addons_config.json"))
}

pub fn load() -> Config {
    let mut config = Config::default();
    let Some(path) = path() else {
        return config;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return config;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        let _ = std::fs::remove_file(path);
        return config;
    };
    if let Some(v) = value.get("auto_update").and_then(|v| v.as_bool()) {
        config.auto_update = v;
    }
    if let Some(v) = value.get("last_update_check").and_then(|v| v.as_u64()) {
        config.last_update_check = v;
    }
    if let Some(provider) = value.get("active_provider").and_then(|v| v.as_str()) {
        config.active_provider = ProviderKind::parse(provider).unwrap_or(config.active_provider);
    }
    if let Some(v) = value.get("active_theme").and_then(|v| v.as_str()) {
        config.active_theme = v.to_string();
    }
    if let Some(v) = value.get("bdix_enabled").and_then(|v| v.as_bool()) {
        config.bdix_enabled = v;
    }
    if let Some(v) = value.get("addons_enabled").and_then(|v| v.as_bool()) {
        config.addons_enabled = v;
    }
    if let Some(v) = value.get("default_player").and_then(|v| v.as_str()) {
        config.default_player = Some(v.to_string());
    }
    if let Some(v) = value.get("download_dir").and_then(|v| v.as_str()) {
        config.download_dir = Some(v.to_string());
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
        "addons_enabled": config.addons_enabled,
        "default_player": config.default_player,
        "download_dir": config.download_dir
    });
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_extension(format!("tmp-{}-{stamp}", std::process::id()));
    if let Err(error) = std::fs::write(&temporary, json.to_string()) {
        log::warn!("failed to write config: {error}");
        return;
    }
    if std::fs::rename(&temporary, &path).is_err() {
        let _ = std::fs::remove_file(&path);
        if let Err(error) = std::fs::rename(&temporary, &path) {
            let _ = std::fs::remove_file(&temporary);
            log::warn!("failed to commit config: {error}");
        }
    }
}

pub fn load_addons() -> Vec<InstalledAddon> {
    let mut list = addons_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str::<Vec<InstalledAddon>>(&content).ok())
        .unwrap_or_default();

    if !list.iter().any(|a| a.is_core()) {
        list.insert(0, InstalledAddon::cinemeta_default());
        save_addons(&list);
    } else {
        for a in &mut list {
            if a.is_core() {
                a.enabled = true;
            }
        }
    }
    list
}

pub fn save_addons(addons: &[InstalledAddon]) {
    let Some(path) = addons_path() else {
        return;
    };
    if let Some(app_dir) = path.parent()
        && std::fs::create_dir_all(app_dir).is_err()
    {
        return;
    }
    let Ok(json) = serde_json::to_string_pretty(addons) else {
        return;
    };
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_extension(format!("tmp-addons-{}-{stamp}", std::process::id()));
    if let Err(error) = std::fs::write(&temporary, json) {
        log::warn!("failed to write addons config: {error}");
        return;
    }
    if std::fs::rename(&temporary, &path).is_err() {
        let _ = std::fs::remove_file(&path);
        if let Err(error) = std::fs::rename(&temporary, &path) {
            let _ = std::fs::remove_file(&temporary);
            log::warn!("failed to commit addons config: {error}");
        }
    }
}
