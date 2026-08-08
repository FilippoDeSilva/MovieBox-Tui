use crate::tui::state::PlayerKind;
use std::{path::Path, process::Command};

const MPV_WINDOWS: &str = r"C:\Program Files\mpv\mpv.exe";
const MPV_MACOS: &str = "/Applications/mpv.app/Contents/MacOS/mpv";
const VLC_WINDOWS: &str = r"C:\Program Files\VideoLAN\VLC\vlc.exe";
const VLC_WINDOWS_X86: &str = r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe";
const VLC_MACOS: &str = "/Applications/VLC.app/Contents/MacOS/VLC";

pub fn detect() -> Vec<PlayerKind> {
    let mut players = Vec::new();

    #[cfg(target_os = "macos")]
    if iina_available() {
        players.push(PlayerKind::Iina);
    }

    if mpv_executable().is_some() {
        players.push(PlayerKind::Mpv);
    }

    if vlc_executable().is_some() {
        players.push(PlayerKind::Vlc);
    }

    let is_android = cfg!(target_os = "android") || std::path::Path::new("/system/bin/am").exists();

    if is_android {
        players.push(PlayerKind::AndroidIntent);
    }

    players
}

pub fn supports_headers(kind: PlayerKind, headers: &[(String, String)]) -> bool {
    if kind == PlayerKind::AndroidIntent {
        return false;
    }
    kind != PlayerKind::Vlc
        || headers.iter().all(|(name, _)| {
            name.eq_ignore_ascii_case("referer") || name.eq_ignore_ascii_case("user-agent")
        })
}

pub fn command(
    kind: PlayerKind,
    url: &str,
    subtitle: Option<&str>,
    headers: &[(String, String)],
) -> Command {
    match kind {
        PlayerKind::Mpv => mpv_command(url, subtitle, headers, false),
        PlayerKind::Iina => iina_command(url, subtitle, headers),
        PlayerKind::Vlc => vlc_command(url, subtitle, headers),
        PlayerKind::AndroidIntent => android_intent_command(url),
    }
}

fn android_intent_command(url: &str) -> Command {
    let mut command;
    if executable_on_path("termux-open") {
        command = Command::new("termux-open");
        command
            .arg("--chooser")
            .arg("--content-type")
            .arg("video/*")
            .arg(url);
    } else {
        command = Command::new("am");
        command
            .arg("start")
            .arg("--user")
            .arg("0")
            .arg("-a")
            .arg("android.intent.action.VIEW")
            .arg("-d")
            .arg(url)
            .arg("-t")
            .arg("video/*");

        if cfg!(target_os = "android")
            || std::env::var("PREFIX")
                .unwrap_or_default()
                .contains("com.termux")
        {
            command.env_remove("LD_LIBRARY_PATH");
            command.env_remove("LD_PRELOAD");
        }
    }
    command
}

fn mpv_command(
    url: &str,
    subtitle: Option<&str>,
    headers: &[(String, String)],
    iina: bool,
) -> Command {
    let fallback = if cfg!(target_os = "windows") {
        "mpv.exe"
    } else {
        "mpv"
    };
    let executable = mpv_executable().unwrap_or_else(|| fallback.into());
    let mut command = if executable.starts_with("flatpak run ") {
        let mut parts = executable.split(' ');
        let mut cmd = Command::new(parts.next().unwrap_or("flatpak"));
        cmd.args(parts);
        cmd
    } else {
        Command::new(&executable)
    };
    let prefix = if iina { "--mpv-" } else { "--" };

    command
        .arg(format!("{prefix}autofit=960x540"))
        .arg(format!("{prefix}autofit-larger=640x360"))
        .arg(format!("{prefix}geometry=50%:50%"));

    if !iina {
        command.arg("--idle=no").arg("--keep-open=no");
    }

    if !headers.is_empty() {
        let fields = headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>()
            .join(",");
        command.arg(format!("{prefix}http-header-fields={fields}"));
    }
    if let Some(subtitle) = subtitle {
        let sub_arg = if executable.starts_with("flatpak run ") {
            format!("@@ {} @@", subtitle)
        } else {
            subtitle.to_string()
        };

        if iina {
            command.arg(format!("--mpv-sub-files={sub_arg}"));
        } else {
            command.arg(format!("--sub-file={sub_arg}"));
        }
    }

    command.arg(url);

    command
}

#[cfg(target_os = "macos")]
fn iina_command(url: &str, subtitle: Option<&str>, headers: &[(String, String)]) -> Command {
    let configured = configured_executable("MOVIEBOX_IINA_PATH");
    let cli_global = std::path::Path::new("/Applications/IINA.app/Contents/MacOS/iina-cli");
    let cli_local = dirs::home_dir()
        .map(|h| h.join("Applications/IINA.app/Contents/MacOS/iina-cli"))
        .unwrap_or_default();

    let mut is_open = false;
    let mut command = if let Some(executable) = configured {
        Command::new(executable)
    } else if cli_global.exists() {
        let mut c = Command::new(cli_global);
        c.arg("--keep-running").arg("--no-stdin");
        c
    } else if cli_local.exists() {
        let mut c = Command::new(cli_local);
        c.arg("--keep-running").arg("--no-stdin");
        c
    } else if iina_app_exists() {
        is_open = true;
        let mut c = Command::new("open");
        c.arg("-W").arg("-a").arg("IINA").arg(url).arg("--args");
        c
    } else {
        Command::new("iina")
    };

    let mpv = mpv_command(url, subtitle, headers, true);
    for arg in mpv.get_args() {
        if is_open && arg == std::ffi::OsStr::new(url) {
            continue;
        }
        command.arg(arg);
    }
    command
}

#[cfg(not(target_os = "macos"))]
fn iina_command(url: &str, subtitle: Option<&str>, headers: &[(String, String)]) -> Command {
    mpv_command(url, subtitle, headers, false)
}

fn vlc_command(url: &str, subtitle: Option<&str>, headers: &[(String, String)]) -> Command {
    let fallback = if cfg!(target_os = "windows") {
        "vlc.exe"
    } else {
        "vlc"
    };
    let executable = vlc_executable().unwrap_or_else(|| fallback.into());
    let mut command = if executable.starts_with("flatpak run ") {
        let mut parts = executable.split(' ');
        let mut cmd = Command::new(parts.next().unwrap_or("flatpak"));
        cmd.args(parts);
        cmd
    } else {
        Command::new(&executable)
    };
    command
        .arg("--width=960")
        .arg("--height=540")
        .arg("--play-and-exit");

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("referer") {
            command.arg(format!("--http-referrer={value}"));
        } else if name.eq_ignore_ascii_case("user-agent") {
            command.arg(format!("--http-user-agent={value}"));
        }
    }
    if let Some(subtitle) = subtitle {
        let sub_arg = if executable.starts_with("flatpak run ") {
            format!("@@ {} @@", subtitle)
        } else {
            subtitle.to_string()
        };
        command.arg(format!("--sub-file={sub_arg}"));
    }

    command.arg(url);
    command
}

fn mpv_executable() -> Option<String> {
    static CACHED: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    CACHED
        .get_or_init(|| {
            if let Some(executable) = configured_executable("MOVIEBOX_MPV_PATH") {
                return Some(executable);
            }
            let fallback = if cfg!(target_os = "windows") {
                "mpv.exe"
            } else {
                "mpv"
            };
            #[cfg_attr(not(any(target_os = "macos", windows)), allow(unused_mut))]
            let mut paths = vec![MPV_WINDOWS.to_string(), MPV_MACOS.to_string()];
            #[cfg(target_os = "macos")]
            if let Some(home) = dirs::home_dir() {
                paths.push(
                    home.join("Applications/mpv.app/Contents/MacOS/mpv")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
            #[cfg(windows)]
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                paths.push(format!(r"{local}\Programs\mpv\mpv.exe"));
            }
            first_executable(&paths, fallback).or_else(|| flatpak_executable("io.mpv.Mpv"))
        })
        .clone()
}

fn vlc_executable() -> Option<String> {
    static CACHED: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    CACHED
        .get_or_init(|| {
            if let Some(executable) = configured_executable("MOVIEBOX_VLC_PATH") {
                return Some(executable);
            }
            let fallback = if cfg!(target_os = "windows") {
                "vlc.exe"
            } else {
                "vlc"
            };

            #[cfg_attr(not(any(target_os = "macos", windows)), allow(unused_mut))]
            let mut paths = vec![
                VLC_WINDOWS.to_string(),
                VLC_WINDOWS_X86.to_string(),
                VLC_MACOS.to_string(),
            ];

            #[cfg(target_os = "macos")]
            if let Some(home) = dirs::home_dir() {
                paths.push(
                    home.join("Applications/VLC.app/Contents/MacOS/VLC")
                        .to_string_lossy()
                        .into_owned(),
                );
            }

            #[cfg(windows)]
            let windows_app_path = std::env::var("LOCALAPPDATA")
                .map(|l| format!(r"{}\Microsoft\WindowsApps\vlc.exe", l))
                .unwrap_or_default();

            #[cfg(windows)]
            if !windows_app_path.is_empty() {
                paths.push(windows_app_path);
            }

            first_executable(&paths, fallback).or_else(|| flatpak_executable("org.videolan.VLC"))
        })
        .clone()
}

#[cfg(target_os = "macos")]
fn iina_available() -> bool {
    configured_executable("MOVIEBOX_IINA_PATH").is_some()
        || iina_app_exists()
        || command_exists("iina")
}

#[cfg(target_os = "macos")]
fn iina_app_exists() -> bool {
    Path::new("/Applications/IINA.app").exists()
        || dirs::home_dir().is_some_and(|home| home.join("Applications/IINA.app").exists())
}

fn flatpak_executable(app_id: &str) -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    if command_exists("flatpak") {
        let mut cmd = Command::new("flatpak");
        cmd.arg("info")
            .arg(app_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if cmd.output().map(|o| o.status.success()).unwrap_or(false) {
            return Some(format!("flatpak run {}", app_id));
        }
    }
    None
}

fn first_executable(paths: &[String], fallback: &str) -> Option<String> {
    paths
        .iter()
        .find(|path| Path::new(path).exists())
        .cloned()
        .or_else(|| command_exists(fallback).then(|| fallback.to_string()))
}

fn configured_executable(variable: &str) -> Option<String> {
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .filter(|value| Path::new(value).exists() || command_exists(value))
}

fn command_exists(command: &str) -> bool {
    let mut cmd = Command::new(command);
    cmd.arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.output().is_ok_and(|output| output.status.success())
}

fn executable_on_path(name: &str) -> bool {
    if std::path::Path::new(name).is_file() {
        return true;
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        if !candidate.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            candidate
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    })
}
