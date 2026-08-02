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
    if Path::new("/Applications/IINA.app").exists() || command_exists("iina") {
        players.push(PlayerKind::Iina);
    }

    if mpv_executable().is_some() {
        players.push(PlayerKind::Mpv);
    }
    if vlc_executable().is_some() {
        players.push(PlayerKind::Vlc);
    }

    players
}

pub fn supports_headers(kind: PlayerKind, headers: &[(String, String)]) -> bool {
    kind != PlayerKind::Vlc || headers.is_empty()
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
    }
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
        let mut cmd = Command::new(parts.next().unwrap());
        cmd.args(parts);
        cmd
    } else {
        Command::new(executable)
    };
    let prefix = if iina { "--mpv-" } else { "--" };

    command
        .arg(format!("{prefix}autofit=960x540"))
        .arg(format!("{prefix}autofit-larger=640x360"))
        .arg(format!("{prefix}geometry=50%:50%"))
        .arg(url);

    if !headers.is_empty() {
        let fields = headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>()
            .join(",");
        command.arg(format!("{prefix}http-header-fields={fields}"));
    }
    if let Some(subtitle) = subtitle {
        if iina {
            command.arg(format!("--mpv-sub-files={subtitle}"));
        } else {
            command.arg(format!("--sub-file={subtitle}"));
        }
    }

    command
}

#[cfg(target_os = "macos")]
fn iina_command(url: &str, subtitle: Option<&str>, headers: &[(String, String)]) -> Command {
    let mut command = if Path::new("/Applications/IINA.app").exists() {
        let mut c = Command::new("open");
        c.arg("-a").arg("IINA").arg("--args");
        c
    } else {
        Command::new("iina")
    };
    let mpv = mpv_command(url, subtitle, headers, true);
    command.args(mpv.get_args());
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
        let mut cmd = Command::new(parts.next().unwrap());
        cmd.args(parts);
        cmd
    } else {
        Command::new(executable)
    };
    command.arg("--width=960").arg("--height=540").arg(url);

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("referer") {
            command.arg(format!("--http-referrer={value}"));
        } else if name.eq_ignore_ascii_case("user-agent") {
            command.arg(format!("--http-user-agent={value}"));
        }
    }
    if let Some(subtitle) = subtitle {
        command.arg(format!("--sub-file={subtitle}"));
    }

    command
}

fn mpv_executable() -> Option<String> {
    let fallback = if cfg!(target_os = "windows") {
        "mpv.exe"
    } else {
        "mpv"
    };
    first_executable(&[MPV_WINDOWS, MPV_MACOS], fallback)
        .or_else(|| flatpak_executable("io.mpv.Mpv"))
}

fn vlc_executable() -> Option<String> {
    let fallback = if cfg!(target_os = "windows") {
        "vlc.exe"
    } else {
        "vlc"
    };

    let paths = vec![VLC_WINDOWS, VLC_WINDOWS_X86, VLC_MACOS];

    #[cfg(windows)]
    let windows_app_path = std::env::var("LOCALAPPDATA")
        .map(|l| format!(r"{}\Microsoft\WindowsApps\vlc.exe", l))
        .unwrap_or_default();

    #[allow(unused_mut)]
    let mut static_paths: Vec<&str> = paths.clone();

    #[cfg(windows)]
    if !windows_app_path.is_empty() {
        static_paths.push(&windows_app_path);
    }

    first_executable(&static_paths, fallback).or_else(|| flatpak_executable("org.videolan.VLC"))
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

fn first_executable(paths: &[&str], fallback: &str) -> Option<String> {
    paths
        .iter()
        .find(|path| Path::new(path).exists())
        .map(|path| (*path).to_string())
        .or_else(|| command_exists(fallback).then(|| fallback.to_string()))
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
    cmd.output().is_ok()
}
