//! In-App 1-Click Update Engine for New Loka
//!
//! Provides automatic update discovery from GitHub Releases, binary staging,
//! safe atomic replacement of running executables on Windows/Linux/macOS,
//! offline update bundle application, and server self-restart.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static SERVER_START_TIME: AtomicU64 = AtomicU64::new(0);

pub fn record_start_time() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    SERVER_START_TIME.store(now, Ordering::Relaxed);
}

pub fn uptime_seconds() -> u64 {
    let started = SERVER_START_TIME.load(Ordering::Relaxed);
    if started == 0 {
        return 0;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(started)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub tier: String,
    pub os: String,
    pub arch: String,
    pub exe_path: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub status: String, // "update_available", "up_to_date", "offline", "error"
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_name: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub html_url: Option<String>,
    pub asset_name: Option<String>,
    pub asset_url: Option<String>,
    pub asset_size: Option<u64>,
    pub os: String,
    pub arch: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyRequest {
    pub asset_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyResponse {
    pub status: String,
    pub message: String,
    pub requires_restart: bool,
    pub updated_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    html_url: Option<String>,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Retrieve basic system and process metadata.
pub fn get_system_info(tier: &str) -> SystemInfo {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        tier: tier.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        exe_path,
        uptime_seconds: uptime_seconds(),
    }
}

/// Query GitHub Releases API to check if a newer version is available.
pub async fn check_for_updates() -> UpdateCheckResponse {
    let current_ver = env!("CARGO_PKG_VERSION");
    let current_os = std::env::consts::OS;
    let current_arch = std::env::consts::ARCH;

    let client = match reqwest::Client::builder()
        .user_agent(format!("NewLoka-Updater/{}", current_ver))
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return UpdateCheckResponse {
                status: "error".to_string(),
                current_version: current_ver.to_string(),
                latest_version: None,
                update_available: false,
                release_name: None,
                release_notes: None,
                published_at: None,
                html_url: None,
                asset_name: None,
                asset_url: None,
                asset_size: None,
                os: current_os.to_string(),
                arch: current_arch.to_string(),
                message: format!("Failed to initialize HTTP client: {}", e),
            };
        }
    };

    let url = "https://api.github.com/repos/newloka/newloka/releases/latest";
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to query GitHub Releases API: {}", e);
            return UpdateCheckResponse {
                status: "offline".to_string(),
                current_version: current_ver.to_string(),
                latest_version: None,
                update_available: false,
                release_name: None,
                release_notes: None,
                published_at: None,
                html_url: None,
                asset_name: None,
                asset_url: None,
                asset_size: None,
                os: current_os.to_string(),
                arch: current_arch.to_string(),
                message: "Unable to reach update server. You may be offline or in an air-gapped environment. Offline update packaging is available.".to_string(),
            };
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        return UpdateCheckResponse {
            status: "offline".to_string(),
            current_version: current_ver.to_string(),
            latest_version: None,
            update_available: false,
            release_name: None,
            release_notes: None,
            published_at: None,
            html_url: None,
            asset_name: None,
            asset_url: None,
            asset_size: None,
            os: current_os.to_string(),
            arch: current_arch.to_string(),
            message: format!("GitHub Releases API returned status HTTP {}. If no public releases exist yet, you can upload offline update packages directly.", status),
        };
    }

    let release: GitHubRelease = match resp.json().await {
        Ok(rel) => rel,
        Err(e) => {
            return UpdateCheckResponse {
                status: "error".to_string(),
                current_version: current_ver.to_string(),
                latest_version: None,
                update_available: false,
                release_name: None,
                release_notes: None,
                published_at: None,
                html_url: None,
                asset_name: None,
                asset_url: None,
                asset_size: None,
                os: current_os.to_string(),
                arch: current_arch.to_string(),
                message: format!("Failed to parse release metadata: {}", e),
            };
        }
    };

    let latest_ver = release.tag_name.trim_start_matches('v').trim_start_matches('V');
    let is_newer = is_newer_version(latest_ver, current_ver);

    // Find best asset for this platform
    let matched_asset = find_matching_asset(&release.assets, current_os, current_arch);

    let (asset_name, asset_url, asset_size) = match matched_asset {
        Some(a) => (Some(a.name.clone()), Some(a.browser_download_url.clone()), Some(a.size)),
        None => (None, None, None),
    };

    if is_newer {
        UpdateCheckResponse {
            status: "update_available".to_string(),
            current_version: current_ver.to_string(),
            latest_version: Some(latest_ver.to_string()),
            update_available: true,
            release_name: release.name.or(Some(format!("New Loka v{}", latest_ver))),
            release_notes: release.body,
            published_at: release.published_at,
            html_url: release.html_url,
            asset_name,
            asset_url,
            asset_size,
            os: current_os.to_string(),
            arch: current_arch.to_string(),
            message: format!("A new version (v{}) is available for download.", latest_ver),
        }
    } else {
        UpdateCheckResponse {
            status: "up_to_date".to_string(),
            current_version: current_ver.to_string(),
            latest_version: Some(latest_ver.to_string()),
            update_available: false,
            release_name: release.name,
            release_notes: release.body,
            published_at: release.published_at,
            html_url: release.html_url,
            asset_name,
            asset_url,
            asset_size,
            os: current_os.to_string(),
            arch: current_arch.to_string(),
            message: format!("New Loka is up to date (v{}).", current_ver),
        }
    }
}

/// Download and apply a release update from a URL.
pub async fn apply_update(asset_url: Option<String>) -> Result<UpdateApplyResponse> {
    let url = match asset_url {
        Some(u) if !u.trim().is_empty() => u,
        _ => {
            let check = check_for_updates().await;
            check
                .asset_url
                .ok_or_else(|| anyhow!("No download asset found for current platform."))?
        }
    };

    tracing::info!("Downloading update from: {}", url);
    let client = reqwest::Client::builder()
        .user_agent("NewLoka-Updater")
        .timeout(std::time::Duration::from_secs(180))
        .build()?;

    let resp = client.get(&url).send().await.context("Failed to download update file")?;
    if !resp.status().is_success() {
        return Err(anyhow!("Download failed with status HTTP {}", resp.status()));
    }

    let bytes = resp.bytes().await.context("Failed to read update stream")?;
    apply_binary_or_archive(&bytes)
}

/// Apply update from raw bytes (supports both downloaded and uploaded packages).
pub fn apply_binary_or_archive(bytes: &[u8]) -> Result<UpdateApplyResponse> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let target_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow!("Failed to get executable directory"))?;

    let mut updated_files = Vec::new();

    // Check if zip archive (starts with 'PK\x03\x04')
    let is_zip = bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04;

    if is_zip {
        tracing::info!("Extracting ZIP update archive...");
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).context("Invalid ZIP update archive")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = match file.enclosed_name() {
                Some(p) => p.file_name().map(|n| n.to_string_lossy().to_string()),
                None => continue,
            };

            let name = match file_name {
                Some(n) => n,
                None => continue,
            };

            // Only update executable binaries
            let is_target_bin = name.starts_with("newloka")
                && (name.ends_with(".exe") || !name.contains('.'));

            if is_target_bin {
                let mut content = Vec::new();
                std::io::copy(&mut file, &mut content)?;

                let dest_path = target_dir.join(&name);
                safe_replace_executable(&dest_path, &content)?;
                updated_files.push(name);
            }
        }

        if updated_files.is_empty() {
            return Err(anyhow!("Archive contained no recognizable New Loka binaries"));
        }
    } else {
        // Direct executable update
        tracing::info!("Writing direct executable update to {:?}", current_exe);
        safe_replace_executable(&current_exe, bytes)?;
        updated_files.push(
            current_exe
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "executable".to_string()),
        );
    }

    Ok(UpdateApplyResponse {
        status: "success".to_string(),
        message: format!(
            "Update applied successfully on disk ({:?}). Please restart the server to activate changes.",
            updated_files
        ),
        requires_restart: true,
        updated_files,
    })
}

/// Safely replaces an executable on disk.
/// On Windows: running files are locked against deletion, but CAN be renamed to .old!
fn safe_replace_executable(target: &Path, new_bytes: &[u8]) -> Result<()> {
    if target.exists() {
        let old_path = target.with_extension("exe.old");
        if old_path.exists() {
            let _ = std::fs::remove_file(&old_path);
        }
        // Rename running exe to .old
        std::fs::rename(target, &old_path)
            .context(format!("Failed to rename {:?} to {:?}", target, old_path))?;
    }

    // Write new executable
    std::fs::write(target, new_bytes)
        .context(format!("Failed to write new executable to {:?}", target))?;

    // On Unix, ensure execute permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(target)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(target, perms)?;
    }

    Ok(())
}

/// Restart the server process by spawning the new executable and cleanly exiting.
pub fn restart_server() -> Result<RestartResponse> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    tracing::info!("Spawning replacement process: {:?} with args {:?}", current_exe, args);

    let mut cmd = std::process::Command::new(&current_exe);
    cmd.args(&args);

    // Inherit existing environment variables
    for (k, v) in std::env::vars() {
        cmd.env(k, v);
    }

    cmd.spawn().context("Failed to spawn updated server process")?;

    // Schedule graceful exit after short delay to allow HTTP response to flush
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        tracing::info!("Exiting old server process for update restart.");
        std::process::exit(0);
    });

    Ok(RestartResponse {
        status: "restarting".to_string(),
        message: "Server is restarting. The web interface will reload automatically.".to_string(),
    })
}

/// Compares version string (e.g. "0.2.0" > "0.1.0").
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u32> {
        v.split('.')
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let lat = parse_parts(latest);
    let cur = parse_parts(current);

    for (l, c) in lat.iter().zip(cur.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    lat.len() > cur.len()
}

fn find_matching_asset<'a>(
    assets: &'a [GitHubAsset],
    current_os: &str,
    current_arch: &str,
) -> Option<&'a GitHubAsset> {
    let os_kw = match current_os {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        _ => current_os,
    };

    let arch_kw = match current_arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => current_arch,
    };

    // First try: both OS and arch in filename
    for a in assets {
        let name_lower = a.name.to_lowercase();
        if name_lower.contains(os_kw) && (name_lower.contains(arch_kw) || (arch_kw == "x86_64" && name_lower.contains("x64"))) {
            return Some(a);
        }
    }

    // Second try: matching OS
    for a in assets {
        let name_lower = a.name.to_lowercase();
        if name_lower.contains(os_kw) {
            return Some(a);
        }
    }

    // Third try: if windows and ends with .zip or .exe
    if current_os == "windows" {
        for a in assets {
            let name_lower = a.name.to_lowercase();
            if name_lower.ends_with(".zip") || name_lower.ends_with(".exe") {
                return Some(a);
            }
        }
    }

    assets.first()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.1.1", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_system_info() {
        let info = get_system_info("T0");
        assert_eq!(info.tier, "T0");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }
}
