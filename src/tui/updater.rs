use self_update::backends::github::ReleaseList;
use std::{
    fs,
    path::{Path, PathBuf},
};

const OWNER: &str = "mesamirh";
const REPOSITORY: &str = "MovieBox-Tui";

pub fn check(current: &str) -> Result<Option<String>, String> {
    let releases = releases()?;
    let Some(release) = releases.first() else {
        return Err("GitHub returned no published releases".into());
    };

    if !self_update::version::bump_is_greater(current, &release.version)
        .map_err(|error| format!("invalid release version {}: {error}", release.version))?
    {
        return Ok(None);
    }

    let asset_name = asset_name()?;
    if !release.assets.iter().any(|asset| asset.name == asset_name) {
        return Err(format!(
            "release v{} has no compatible asset named {asset_name}",
            release.version
        ));
    }

    Ok(Some(release.version.clone()))
}

pub fn replacement_access() -> Result<(), String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("cannot locate executable: {error}"))?;
    let directory = executable
        .parent()
        .ok_or_else(|| "executable has no parent directory".to_string())?;
    let probe = directory.join(format!(".moviebox-update-access-{}", std::process::id()));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|error| format!("{} is not writable: {error}", directory.display()))?;
    fs::remove_file(probe).map_err(|error| format!("cannot clean update access check: {error}"))
}

pub fn install(version: &str, mut progress: impl FnMut(f64, &str)) -> Result<(), String> {
    let asset_name = asset_name()?;
    let release = releases()?
        .into_iter()
        .find(|release| release.version == version)
        .ok_or_else(|| format!("release v{version} was not found"))?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("release v{version} has no {asset_name} asset"))?;

    let temp_dir = update_temp_dir();
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("cannot create update directory: {error}"))?;

    let result = (|| {
        progress(0.1, "Downloading release...");
        let archive_path = temp_dir.join(&asset_name);
        let mut archive = fs::File::create(&archive_path)
            .map_err(|error| format!("cannot create update archive: {error}"))?;
        self_update::Download::from_url(&asset.download_url)
            .show_progress(false)
            .download_to(&mut archive)
            .map_err(|error| format!("release download failed: {error}"))?;

        progress(0.7, "Verifying package...");
        if archive
            .metadata()
            .map_err(|error| format!("cannot verify update archive: {error}"))?
            .len()
            == 0
        {
            return Err("downloaded update archive is empty".into());
        }

        let binary_name = if cfg!(target_os = "windows") {
            "MovieBox.exe"
        } else {
            "moviebox"
        };
        let archive_kind = if cfg!(target_os = "windows") {
            self_update::ArchiveKind::Zip
        } else {
            self_update::ArchiveKind::Tar(Some(self_update::Compression::Gz))
        };

        progress(0.82, "Extracting binary...");
        self_update::Extract::from_source(&archive_path)
            .archive(archive_kind)
            .extract_file(&temp_dir, Path::new(binary_name))
            .map_err(|error| format!("release extraction failed: {error}"))?;

        let new_binary = temp_dir.join(binary_name);
        if new_binary
            .metadata()
            .map_err(|error| format!("extracted binary is missing: {error}"))?
            .len()
            == 0
        {
            return Err("extracted binary is empty".into());
        }

        progress(0.94, "Replacing executable...");
        let current = std::env::current_exe()
            .map_err(|error| format!("cannot locate current executable: {error}"))?;
        self_update::self_replace::self_replace(&new_binary).map_err(|error| {
            let permission_hint = if cfg!(unix) {
                " If installed in /usr/local/bin, run the installer once with sudo."
            } else {
                ""
            };
            format!(
                "cannot replace {}: {error}.{permission_hint}",
                current.display()
            )
        })?;

        progress(1.0, "Update installed.");
        Ok(())
    })();

    let _ = fs::remove_dir_all(temp_dir);
    result
}

fn releases() -> Result<Vec<self_update::update::Release>, String> {
    ReleaseList::configure()
        .repo_owner(OWNER)
        .repo_name(REPOSITORY)
        .build()
        .and_then(ReleaseList::fetch)
        .map_err(|error| format!("GitHub release check failed: {error}"))
}

fn asset_name() -> Result<String, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64" | "x86_64") => Ok("MovieBox_macOS_Universal.tar.gz".into()),
        ("windows", "x86_64") => Ok("MovieBox_Windows_x64.zip".into()),
        ("linux", "x86_64") => Ok("MovieBox_Linux_x64.tar.gz".into()),
        ("linux", "aarch64") => Ok("MovieBox_Linux_arm64.tar.gz".into()),
        (os, arch) => Err(format!("automatic updates do not support {os}/{arch}")),
    }
}

fn update_temp_dir() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("moviebox-update-{}-{stamp}", std::process::id()))
}
