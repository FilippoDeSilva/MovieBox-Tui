const OWNER: &str = "mesamirh";
const REPOSITORY: &str = "MovieBox-Tui";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub version: String,
    pub tag_name: String,
    pub notes: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    MacosUniversal,
    LinuxX64,
    LinuxArm64,
    WindowsX64,
    WindowsArm64,
}

impl TargetPlatform {
    pub fn current() -> Option<Self> {
        Self::detect(
            std::env::consts::OS,
            std::env::consts::ARCH,
            is_termux_environment(),
        )
    }

    pub fn detect(os: &str, arch: &str, is_termux: bool) -> Option<Self> {
        if is_termux {
            if arch == "aarch64" || arch == "arm64" {
                return Some(Self::LinuxArm64);
            } else {
                return None;
            }
        }

        match os {
            "macos" | "darwin" => Some(Self::MacosUniversal),
            "linux" => match arch {
                "x86_64" | "x86-64" | "x64" | "amd64" => Some(Self::LinuxX64),
                "aarch64" | "arm64" => Some(Self::LinuxArm64),
                _ => None,
            },
            "windows" => match arch {
                "x86_64" | "x86-64" | "x64" | "amd64" => Some(Self::WindowsX64),
                "aarch64" | "arm64" => Some(Self::WindowsArm64),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn expected_asset_name(self) -> &'static str {
        match self {
            Self::MacosUniversal => "MovieBox_macOS_Universal.tar.gz",
            Self::LinuxX64 => "MovieBox_Linux_x64.tar.gz",
            Self::LinuxArm64 => "MovieBox_Linux_arm64.tar.gz",
            Self::WindowsX64 => "MovieBox_Windows_x64.zip",
            Self::WindowsArm64 => "MovieBox_Windows_arm64.zip",
        }
    }
}

pub fn is_termux_environment() -> bool {
    std::env::var("PREFIX").is_ok_and(|p| p.contains("com.termux"))
}

impl Release {
    pub fn find_compatible_asset(&self, platform: TargetPlatform) -> Option<&ReleaseAsset> {
        let expected = platform.expected_asset_name();
        self.assets.iter().find(|a| a.name == expected)
    }

    pub fn find_checksum_asset(&self) -> Option<&ReleaseAsset> {
        self.assets.iter().find(|a| a.name == "SHA256SUMS")
    }

    pub fn is_compatible_with_current_platform(&self) -> bool {
        TargetPlatform::current()
            .and_then(|p| self.find_compatible_asset(p))
            .is_some()
    }
}

pub async fn check_release(current: &str) -> Result<Option<Release>, String> {
    let release = match fetch_release().await {
        Ok(release) => release,
        Err(error) => {
            log::warn!("update check via API failed ({error}); falling back to release page");
            let tag = fetch_latest_tag().await?;
            log::info!("resolved latest release via redirect: {tag}");
            Release {
                version: tag.trim_start_matches('v').to_string(),
                tag_name: tag,
                notes: String::new(),
                assets: Vec::new(),
            }
        }
    };

    if !is_newer(current, &release.version) {
        return Ok(None);
    }

    Ok(Some(release))
}

pub async fn check(current: &str) -> Result<Option<(String, String)>, String> {
    let release = check_release(current).await?;
    Ok(release.map(|r| (r.version, r.notes)))
}

fn is_newer(current: &str, other: &str) -> bool {
    let parse = |v: &str| semver::Version::parse(v.trim_start_matches('v'));
    match (parse(current), parse(other)) {
        (Ok(cur), Ok(o)) => o > cur,
        _ => other != current,
    }
}

async fn fetch_release() -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{OWNER}/{REPOSITORY}/releases/latest");
    let client = http_client()?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GitHub request failed: {e}"))?;
    let status = resp.status();
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err(format!("GitHub API rate limited ({status})"));
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API {status}: {body}"));
    }

    let item: serde_json::Value = resp.json().await.map_err(|e| format!("bad JSON: {e}"))?;
    let tag = item["tag_name"].as_str().ok_or("missing tag_name")?;
    let notes = item["body"].as_str().unwrap_or("").to_string();
    let assets = if let Some(arr) = item["assets"].as_array() {
        arr.iter()
            .filter_map(|a| {
                let name = a["name"].as_str()?.to_string();
                let download_url = a["browser_download_url"].as_str()?.to_string();
                let size = a["size"].as_u64();
                Some(ReleaseAsset {
                    name,
                    download_url,
                    size,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Release {
        version: tag.trim_start_matches('v').to_string(),
        tag_name: tag.to_string(),
        notes,
        assets,
    })
}

async fn fetch_latest_tag() -> Result<String, String> {
    let url = format!("https://github.com/{OWNER}/{REPOSITORY}/releases/latest");
    let client = http_client()?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GitHub release page failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("GitHub release page {status}"));
    }

    let path = resp.url().path();
    let tag = path.rsplit('/').next().unwrap_or("");
    if tag.is_empty() || tag == "latest" {
        return Err("could not resolve the latest release tag".into());
    }
    Ok(tag.to_string())
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("MovieBox-Tui")
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_comprehensive_matrix() {
        assert!(is_newer("0.1.9", "0.1.10"));
        assert!(is_newer("0.1.10", "0.1.11"));
        assert!(is_newer("0.1.12", "0.1.13"));
        assert!(is_newer("1.9.0", "1.10.0"));
        assert!(is_newer("0.9.9", "1.0.0"));

        assert!(!is_newer("0.1.12", "0.1.12"));
        assert!(!is_newer("1.0.0", "1.0.0"));

        assert!(!is_newer("0.1.13", "0.1.12"));
        assert!(!is_newer("1.10.0", "1.9.0"));
        assert!(!is_newer("2.0.0", "1.9.9"));

        assert!(is_newer("v0.1.9", "v0.1.10"));
        assert!(is_newer("0.1.9", "v0.1.10"));
        assert!(is_newer("v0.1.9", "0.1.10"));
        assert!(!is_newer("v0.1.12", "v0.1.12"));
        assert!(!is_newer("v0.1.13", "v0.1.12"));

        assert!(is_newer("1.0.0-beta", "1.0.0"));
        assert!(is_newer("1.0.0-rc.1", "1.0.0"));
        assert!(is_newer("1.0.0-beta.1", "1.0.0-beta.2"));
        assert!(!is_newer("1.0.0", "1.0.0-beta"));
    }

    #[test]
    fn test_is_newer_non_semver_fallback() {
        assert!(is_newer("0.1.12", "custom-build-1"));
        assert!(!is_newer("custom-build-1", "custom-build-1"));
    }

    #[test]
    fn test_release_json_deserialization() {
        let raw_json = "{\"tag_name\": \"v0.1.13\", \"body\": \"### New Features\\n- Added cool feature\\n- Fixed bugs\"}";
        let item: serde_json::Value = serde_json::from_str(raw_json).unwrap();
        let tag = item["tag_name"].as_str().unwrap();
        let notes = item["body"].as_str().unwrap_or("").to_string();
        let release = Release {
            version: tag.trim_start_matches('v').to_string(),
            tag_name: tag.to_string(),
            notes,
            assets: Vec::new(),
        };
        assert_eq!(release.version, "0.1.13");
        assert!(release.notes.contains("### New Features"));
    }

    #[test]
    fn test_release_json_missing_body_graceful() {
        let raw_json = "{\"tag_name\": \"v0.1.13\"}";
        let item: serde_json::Value = serde_json::from_str(raw_json).unwrap();
        let tag = item["tag_name"].as_str().unwrap();
        let notes = item["body"].as_str().unwrap_or("").to_string();
        assert_eq!(notes, "");
        assert_eq!(tag.trim_start_matches('v'), "0.1.13");
    }

    fn sample_release_with_all_assets() -> Release {
        Release {
            version: "0.1.13".to_string(),
            tag_name: "v0.1.13".to_string(),
            notes: "Bug fixes and improvements".to_string(),
            assets: vec![
                ReleaseAsset {
                    name: "MovieBox_macOS_Universal.tar.gz".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/MovieBox_macOS_Universal.tar.gz".to_string(),
                    size: Some(15_000_000),
                },
                ReleaseAsset {
                    name: "MovieBox_Linux_x64.tar.gz".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/MovieBox_Linux_x64.tar.gz".to_string(),
                    size: Some(12_000_000),
                },
                ReleaseAsset {
                    name: "MovieBox_Linux_arm64.tar.gz".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/MovieBox_Linux_arm64.tar.gz".to_string(),
                    size: Some(11_500_000),
                },
                ReleaseAsset {
                    name: "MovieBox_Windows_x64.zip".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/MovieBox_Windows_x64.zip".to_string(),
                    size: Some(13_000_000),
                },
                ReleaseAsset {
                    name: "MovieBox_Windows_arm64.zip".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/MovieBox_Windows_arm64.zip".to_string(),
                    size: Some(12_500_000),
                },
                ReleaseAsset {
                    name: "SHA256SUMS".to_string(),
                    download_url: "https://github.com/mesamirh/MovieBox-Tui/releases/download/v0.1.13/SHA256SUMS".to_string(),
                    size: Some(512),
                },
            ],
        }
    }

    #[test]
    fn test_update_asset_selection_for_current_platform() {
        let release = sample_release_with_all_assets();

        let mac_platform = TargetPlatform::detect("macos", "aarch64", false).unwrap();
        let mac_asset = release.find_compatible_asset(mac_platform).unwrap();
        assert_eq!(mac_asset.name, "MovieBox_macOS_Universal.tar.gz");

        let linux_x64 = TargetPlatform::detect("linux", "x86_64", false).unwrap();
        let linux_x64_asset = release.find_compatible_asset(linux_x64).unwrap();
        assert_eq!(linux_x64_asset.name, "MovieBox_Linux_x64.tar.gz");

        let linux_arm64 = TargetPlatform::detect("linux", "aarch64", false).unwrap();
        let linux_arm64_asset = release.find_compatible_asset(linux_arm64).unwrap();
        assert_eq!(linux_arm64_asset.name, "MovieBox_Linux_arm64.tar.gz");

        let win_x64 = TargetPlatform::detect("windows", "x86_64", false).unwrap();
        let win_x64_asset = release.find_compatible_asset(win_x64).unwrap();
        assert_eq!(win_x64_asset.name, "MovieBox_Windows_x64.zip");

        let win_arm64 = TargetPlatform::detect("windows", "arm64", false).unwrap();
        let win_arm64_asset = release.find_compatible_asset(win_arm64).unwrap();
        assert_eq!(win_arm64_asset.name, "MovieBox_Windows_arm64.zip");

        let checksum = release.find_checksum_asset().unwrap();
        assert_eq!(checksum.name, "SHA256SUMS");
    }

    #[test]
    fn test_update_asset_missing_for_current_platform() {
        let partial_release = Release {
            version: "0.1.13".to_string(),
            tag_name: "v0.1.13".to_string(),
            notes: "Partial release".to_string(),
            assets: vec![ReleaseAsset {
                name: "MovieBox_Linux_x64.tar.gz".to_string(),
                download_url: "https://...".to_string(),
                size: Some(1000),
            }],
        };

        let mac = TargetPlatform::detect("macos", "arm64", false).unwrap();
        assert!(partial_release.find_compatible_asset(mac).is_none());

        let win = TargetPlatform::detect("windows", "x86_64", false).unwrap();
        assert!(partial_release.find_compatible_asset(win).is_none());
    }

    #[test]
    fn test_update_asset_rejects_wrong_architecture() {
        assert!(TargetPlatform::detect("linux", "mips", false).is_none());
        assert!(TargetPlatform::detect("linux", "riscv64", false).is_none());
        assert!(TargetPlatform::detect("windows", "ia64", false).is_none());
    }

    #[test]
    fn test_update_asset_rejects_wrong_platform() {
        assert!(TargetPlatform::detect("freebsd", "x86_64", false).is_none());
        assert!(TargetPlatform::detect("openbsd", "x86_64", false).is_none());
        assert!(TargetPlatform::detect("solaris", "x86_64", false).is_none());
    }

    #[test]
    fn test_update_asset_termux_arm64_selection() {
        let release = sample_release_with_all_assets();
        let termux_arm64 = TargetPlatform::detect("linux", "aarch64", true).unwrap();
        assert_eq!(termux_arm64, TargetPlatform::LinuxArm64);

        let asset = release.find_compatible_asset(termux_arm64).unwrap();
        assert_eq!(asset.name, "MovieBox_Linux_arm64.tar.gz");

        assert!(TargetPlatform::detect("linux", "x86_64", true).is_none());
    }

    #[test]
    fn test_update_asset_multiple_candidates_deterministic() {
        let release = sample_release_with_all_assets();
        let platform = TargetPlatform::LinuxX64;
        let a1 = release.find_compatible_asset(platform);
        let a2 = release.find_compatible_asset(platform);
        assert_eq!(a1, a2);
        assert_eq!(a1.unwrap().name, "MovieBox_Linux_x64.tar.gz");
    }
}
