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

    let is_android = cfg!(target_os = "android")
        || std::path::Path::new("/system/bin/am").exists()
        || executable_on_path("termux-open")
        || executable_on_path("am");

    if is_android {
        players.push(PlayerKind::AndroidIntent);
    }

    players
}

pub fn supports_headers(kind: PlayerKind, headers: &[(String, String)]) -> bool {
    if kind == PlayerKind::AndroidIntent {
        return false;
    }
    #[cfg(target_os = "macos")]
    if kind == PlayerKind::Iina && !iina_cli_exists() {
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
    window: Option<(u32, u32)>,
) -> Command {
    match kind {
        PlayerKind::Mpv => mpv_command(url, subtitle, headers, false, window),
        PlayerKind::Iina => iina_command(url, subtitle, headers, window),
        PlayerKind::Vlc => vlc_command(url, subtitle, headers, window),
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
    window: Option<(u32, u32)>,
) -> Command {
    let fallback = if cfg!(target_os = "windows") {
        "mpv.exe"
    } else {
        "mpv"
    };
    let executable = mpv_executable().unwrap_or_else(|| fallback.into());
    let mut command = if executable.starts_with("flatpak run ") {
        let parts = executable.split(' ').collect::<Vec<_>>();
        let mut cmd = Command::new(parts.first().unwrap_or(&"flatpak"));
        if parts.len() > 1 && parts[1] == "run" {
            cmd.arg("run");
            cmd.arg("--file-forwarding");
            cmd.args(&parts[2..]);
        } else {
            cmd.args(&parts[1..]);
        }
        cmd
    } else {
        Command::new(&executable)
    };
    let prefix = if iina { "--mpv-" } else { "--" };

    if let Some((width, height)) = window {
        command.arg(format!("{prefix}autofit={width}x{height}"));
    }
    command.arg(format!("{prefix}geometry=50%:50%"));

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
        if executable.starts_with("flatpak run ") {
            let opt = if iina {
                "--mpv-sub-files"
            } else {
                "--sub-file"
            };
            command.arg(opt);
            command.arg("@@").arg(subtitle).arg("@@");
        } else {
            let opt = if iina {
                "--mpv-sub-files"
            } else {
                "--sub-file"
            };
            command.arg(format!("{}={}", opt, subtitle));
        }
    }

    command.arg(url);

    command
}

#[cfg(target_os = "macos")]
fn iina_command(
    url: &str,
    subtitle: Option<&str>,
    headers: &[(String, String)],
    window: Option<(u32, u32)>,
) -> Command {
    let configured = configured_executable("MOVIEBOX_IINA_PATH");
    let cli_global = std::path::Path::new("/Applications/IINA.app/Contents/MacOS/iina-cli");
    let cli_local = dirs::home_dir()
        .map(|h| h.join("Applications/IINA.app/Contents/MacOS/iina-cli"))
        .unwrap_or_default();

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
        let mut c = Command::new("open");
        c.arg("-a").arg("IINA").arg(url);
        return c;
    } else {
        Command::new("iina")
    };

    let mpv = mpv_command(url, subtitle, headers, true, window);
    for arg in mpv.get_args() {
        command.arg(arg);
    }
    command
}

#[cfg(not(target_os = "macos"))]
fn iina_command(
    url: &str,
    subtitle: Option<&str>,
    headers: &[(String, String)],
    window: Option<(u32, u32)>,
) -> Command {
    mpv_command(url, subtitle, headers, false, window)
}

fn vlc_command(
    url: &str,
    subtitle: Option<&str>,
    headers: &[(String, String)],
    window: Option<(u32, u32)>,
) -> Command {
    let fallback = if cfg!(target_os = "windows") {
        "vlc.exe"
    } else {
        "vlc"
    };
    let executable = vlc_executable().unwrap_or_else(|| fallback.into());
    let mut command = if executable.starts_with("flatpak run ") {
        let parts = executable.split(' ').collect::<Vec<_>>();
        let mut cmd = Command::new(parts.first().unwrap_or(&"flatpak"));
        if parts.len() > 1 && parts[1] == "run" {
            cmd.arg("run");
            cmd.arg("--file-forwarding");
            cmd.args(&parts[2..]);
        } else {
            cmd.args(&parts[1..]);
        }
        cmd
    } else {
        Command::new(&executable)
    };
    if let Some((width, height)) = window {
        command
            .arg(format!("--width={width}"))
            .arg(format!("--height={height}"));
    }
    command.arg("--play-and-exit");

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("referer") {
            command.arg(format!("--http-referrer={value}"));
        } else if name.eq_ignore_ascii_case("user-agent") {
            command.arg(format!("--http-user-agent={value}"));
        }
    }
    if let Some(subtitle) = subtitle {
        if executable.starts_with("flatpak run ") {
            command.arg("--sub-file").arg("@@").arg(subtitle).arg("@@");
        } else {
            command.arg(format!("--sub-file={subtitle}"));
        }
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

#[cfg(target_os = "macos")]
fn iina_cli_exists() -> bool {
    let cli_global = Path::new("/Applications/IINA.app/Contents/MacOS/iina-cli");
    let cli_local = dirs::home_dir()
        .map(|h| h.join("Applications/IINA.app/Contents/MacOS/iina-cli"))
        .unwrap_or_default();
    configured_executable("MOVIEBOX_IINA_PATH").is_some()
        || cli_global.exists()
        || cli_local.exists()
        || command_exists("iina")
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
    #[cfg(windows)]
    if std::path::Path::new(&format!("{name}.exe")).is_file()
        || std::path::Path::new(&format!("{name}.cmd")).is_file()
    {
        return true;
    }

    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        #[cfg(windows)]
        let candidates = vec![
            candidate.clone(),
            candidate.with_extension("exe"),
            candidate.with_extension("cmd"),
        ];
        #[cfg(not(windows))]
        let candidates = vec![candidate];

        candidates.into_iter().any(|candidate| {
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
    })
}
