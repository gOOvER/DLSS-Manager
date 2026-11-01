use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::database::{get_backups_dir, get_cache_dir};
use crate::models::{AppConfig, BackupEntry, DllType, Game, GpuInfo, RemoteDllVersion};
use crate::pe_version::{extract_dll_version, is_version_newer, preset_hint_for_dll};

pub const GITHUB_DLL_REPO: &str = "Recol/DLSS-Updater-DLLs";
pub const DLL_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/Recol/DLSS-Updater-DLLs/main/manifest.json";
pub const DLL_RAW_BASE: &str =
    "https://raw.githubusercontent.com/Recol/DLSS-Updater-DLLs/main/dlls";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub version: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
}

pub type ManifestMap = HashMap<String, ManifestEntry>;

/// Fetches the latest `manifest.json` from `Recol/DLSS-Updater-DLLs` and caches it locally.
pub async fn fetch_or_load_manifest() -> Result<ManifestMap> {
    let cache_dir = get_cache_dir();
    let local_manifest_path = cache_dir.join("manifest.json");

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager-Rust/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    match client.get(DLL_MANIFEST_URL).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(bytes) = resp.bytes().await {
                if let Ok(parsed) = serde_json::from_slice::<ManifestMap>(&bytes) {
                    let _ = fs::write(&local_manifest_path, &bytes);
                    return Ok(parsed);
                }
            }
        }
        _ => {}
    }

    if local_manifest_path.exists() {
        let bytes = fs::read(&local_manifest_path)?;
        let parsed = serde_json::from_slice::<ManifestMap>(&bytes)?;
        return Ok(parsed);
    }

    Ok(default_fallback_manifest())
}

pub fn load_cached_manifest_sync() -> ManifestMap {
    let local_manifest_path = get_cache_dir().join("manifest.json");
    if let Ok(bytes) = fs::read(&local_manifest_path) {
        if let Ok(parsed) = serde_json::from_slice::<ManifestMap>(&bytes) {
            return parsed;
        }
    }
    default_fallback_manifest()
}

fn default_fallback_manifest() -> ManifestMap {
    let mut map = HashMap::new();
    let defaults = [
        ("nvngx_dlss.dll", "310.3.0.0"),
        ("nvngx_dlssg.dll", "310.3.0.0"),
        ("nvngx_dlssd.dll", "310.3.0.0"),
        ("nvngx_dlssnr.dll", "310.8.0.0"),
        ("amd_fidelityfx_dx12.dll", "1.0.1.41314"),
        ("amd_fidelityfx_vk.dll", "1.0.1.41314"),
        ("amd_fidelityfx_loader_dx12.dll", "1.0.2.44888"),
        ("amd_fidelityfx_upscaler_dx12.dll", "4.0.2.44888"),
        ("amd_fidelityfx_framegeneration_dx12.dll", "3.1.5.44888"),
        ("libxess.dll", "2.0.2.68"),
        ("libxess_dx11.dll", "2.0.2.68"),
        ("libxess_fg.dll", "1.2.2.118"),
        ("libxell.dll", "1.2.1.13"),
        ("dstorage.dll", "1.2.2504.401"),
        ("dstoragecore.dll", "1.2.2504.401"),
        ("sl.interposer.dll", "2.7.30.0"),
        ("sl.common.dll", "2.7.30.0"),
        ("sl.dlss.dll", "2.7.30.0"),
        ("sl.dlss_g.dll", "2.7.30.0"),
        ("sl.dlss_d.dll", "2.8.0.0"),
        ("sl.reflex.dll", "2.7.30.0"),
        ("sl.pcl.dll", "2.7.30.0"),
        ("sl.directsr.dll", "2.8.0.0"),
        ("sl.nis.dll", "2.8.0.0"),
    ];
    let cache_dir = get_cache_dir();
    for (fname, ver) in defaults {
        let cached_file = cache_dir.join(fname);
        let actual_ver = if cached_file.exists() {
            extract_dll_version(&cached_file).unwrap_or_else(|| ver.to_string())
        } else {
            ver.to_string()
        };
        map.insert(
            fname.to_string(),
            ManifestEntry {
                version: actual_ver,
                sha256: None,
                size: None,
                url: None,
                date: None,
            },
        );
    }
    map
}

pub fn resolve_dll_download_url(dll_name: &str, entry: &ManifestEntry) -> String {
    let safe_name = Path::new(dll_name)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let fallback = format!("{}/{}", DLL_RAW_BASE, safe_name);

    if let Some(url) = &entry.url {
        if url.starts_with("https://github.com/Recol/DLSS-Updater-DLLs/")
            || url.starts_with("https://raw.githubusercontent.com/Recol/DLSS-Updater-DLLs/")
        {
            return url.clone();
        }
    }
    fallback
}

/// Converts the manifest map into a sorted list of `RemoteDllVersion` items supported by the user's GPU.
pub fn list_remote_dlls(manifest: &ManifestMap, gpu: &GpuInfo) -> Vec<RemoteDllVersion> {
    let cache_dir = get_cache_dir();
    let mut items = Vec::new();

    for (filename, info) in manifest {
        let Some(dll_type) = DllType::from_filename(filename) else {
            continue;
        };
        if !gpu.supports_dll_type(dll_type) {
            continue;
        }
        let cached_path = cache_dir.join(filename);
        let is_cached = cached_path.exists();
        let is_preview = filename.eq_ignore_ascii_case("amd_fidelityfx_radiancecache_dx12.dll");
        let preset_hint = preset_hint_for_dll(dll_type, &info.version);

        items.push(RemoteDllVersion {
            dll_type,
            type_label: dll_type.display_name().to_string(),
            filename: filename.clone(),
            version: info.version.clone(),
            download_url: resolve_dll_download_url(filename, info),
            sha256: info.sha256.clone(),
            file_size: info.size,
            release_date: info.date.clone(),
            is_preview,
            is_cached,
            preset_hint,
        });
    }

    items.sort_by(|a, b| a.filename.cmp(&b.filename));
    items
}

/// Known bad / watermarked DLL SHA-256 digests (e.g. DLSS 310.7 dev build with "development build - do not redistribute" watermark).
const KNOWN_BAD_DLL_SHA256: &[&str] = &[
    "83862f7848b780a5e92c3e2667a8a3b45017c83b18a49c019953bc48112d8a64",
    "c06d8351a4e0c0d7524bf564f13a6e43025517a7f37a104127a1d198a3b14518",
];

fn check_bad_dll_build(path: &Path, version: &str) -> Option<String> {
    if version.starts_with("310.7") {
        return Some("DLSS 310.7 development watermark build detected — update recommended".to_string());
    }
    if let Ok(bytes) = fs::read(path) {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha = hex::encode(hasher.finalize());
        if KNOWN_BAD_DLL_SHA256
            .iter()
            .any(|bad| bad.eq_ignore_ascii_case(&sha))
        {
            return Some("Known watermarked / faulty DLL build — update recommended".to_string());
        }
    }
    None
}

/// Filters and annotates each game's detected DLLs based on the user's GPU capabilities and manifest versions.
/// Never offers updates for DLL types that the detected GPU does not support.
pub fn annotate_games_with_manifest(games: &mut Vec<Game>, manifest: &ManifestMap, gpu: &GpuInfo) {
    let default_cfg = AppConfig::default();
    annotate_games_with_config(games, manifest, gpu, &default_cfg);
}

pub fn annotate_games_with_config(
    games: &mut Vec<Game>,
    manifest: &ManifestMap,
    gpu: &GpuInfo,
    config: &AppConfig,
) {
    let cache_dir = get_cache_dir();
    for game in games.iter_mut() {
        // Retire dead paths whose files were removed by a game patch
        game.dlls.retain(|dll| Path::new(&dll.path).exists() && gpu.supports_dll_type(dll.dll_type));

        let skip_ac = config.skip_anti_cheat_games && game.anti_cheat.is_some();

        for dll in game.dlls.iter_mut() {
            dll.gpu_supported = gpu.supports_dll_type(dll.dll_type);
            let is_preview_rc = dll
                .filename
                .eq_ignore_ascii_case("amd_fidelityfx_radiancecache_dx12.dll");
            let tech_enabled = if is_preview_rc {
                config.allow_preview_dlls
            } else {
                config.enabled_technologies.is_empty()
                    || config
                        .enabled_technologies
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(&dll.type_label))
            };
            dll.technology_enabled = tech_enabled;

            let key = dll.filename.to_ascii_lowercase();
            let entry = manifest
                .get(&dll.filename)
                .or_else(|| manifest.get(&key));

            let cached_file = cache_dir.join(&dll.filename);
            let mut latest_ver = if cached_file.exists() {
                extract_dll_version(&cached_file)
                    .or_else(|| entry.map(|e| e.version.clone()))
            } else {
                entry.map(|e| e.version.clone())
            };

            // For FSR 3.1 monolith (amd_fidelityfx_dx12.dll / amd_fidelityfx_vk.dll) on FSR4-capable AMD GPUs,
            // check the FSR 4 upscaler version so FSR 3.1 -> FSR 4 upgrades are offered!
            if gpu.supports_fsr4
                && (key == "amd_fidelityfx_dx12.dll" || key == "amd_fidelityfx_vk.dll")
            {
                let upscaler_key = if key.ends_with("_vk.dll") {
                    "amd_fidelityfx_upscaler_vk.dll"
                } else {
                    "amd_fidelityfx_upscaler_dx12.dll"
                };
                if let Some(up_entry) = manifest.get(upscaler_key) {
                    latest_ver = Some(up_entry.version.clone());
                }
            }

            dll.bad_build_warning = if dll.version.starts_with("310.7") {
                check_bad_dll_build(Path::new(&dll.path), &dll.version)
            } else {
                None
            };

            if let Some(ver) = latest_ver {
                dll.latest_version = Some(ver.clone());
                let newer_or_bad =
                    is_version_newer(&ver, &dll.version) || dll.bad_build_warning.is_some();
                dll.update_available =
                    dll.gpu_supported && dll.technology_enabled && !skip_ac && newer_or_bad;
            } else {
                dll.update_available = false;
            }
        }
    }

    games.retain(|g| !g.dlls.is_empty());
}

/// Downloads a DLL from the repository into `%LOCALAPPDATA%\Recol\DLSS-Updater\dll_cache\`
/// and verifies its SHA-256 checksum if present in the manifest.
pub async fn ensure_dll_cached(dll_name: &str, manifest: &ManifestMap) -> Result<std::path::PathBuf> {
    let safe_name = Path::new(dll_name)
        .file_name()
        .ok_or_else(|| anyhow!("Invalid DLL filename: {}", dll_name))?
        .to_string_lossy()
        .to_string();

    if !safe_name.to_ascii_lowercase().ends_with(".dll") {
        return Err(anyhow!("Not a valid .dll file: {}", safe_name));
    }

    let cache_path = get_cache_dir().join(&safe_name);
    let info = manifest
        .get(&safe_name)
        .or_else(|| manifest.get(&safe_name.to_ascii_lowercase()))
        .cloned()
        .unwrap_or(ManifestEntry {
            version: String::new(),
            sha256: None,
            size: None,
            url: None,
            date: None,
        });

    // If cached and version is already up to date, return immediately
    if cache_path.exists() {
        if let Some(local_ver) = extract_dll_version(&cache_path) {
            if info.version.is_empty() || !is_version_newer(&info.version, &local_ver) {
                return Ok(cache_path);
            }
        }
    }

    let url = resolve_dll_download_url(&safe_name, &info);
    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager-Rust/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to download {}", url))?;

    if !resp.status().is_success() {
        return Err(anyhow!(
            "HTTP {} when downloading {}",
            resp.status(),
            safe_name
        ));
    }

    let bytes = resp.bytes().await?;
    if bytes.len() < 4096 || &bytes[0..2] != b"MZ" {
        return Err(anyhow!(
            "Downloaded file for {} is not a valid PE binary",
            safe_name
        ));
    }

    if let Some(expected_sha) = &info.sha256 {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_sha = hex::encode(hasher.finalize());
        if !actual_sha.eq_ignore_ascii_case(expected_sha) {
            return Err(anyhow!(
                "SHA-256 mismatch for {}: expected {}, got {}",
                safe_name,
                expected_sha,
                actual_sha
            ));
        }
    }

    let tmp_path = cache_path.with_extension("dll.part");
    fs::write(&tmp_path, &bytes)?;
    fs::rename(&tmp_path, &cache_path)?;

    Ok(cache_path)
}

/// Backs up a game's existing DLL and replaces it with the cached version.
/// Supports all-or-nothing FSR 3.1 -> FSR 4 side-by-side upgrades for DX12 & Vulkan games.
pub async fn update_game_dll(
    game: &Game,
    target_dll_path: &str,
    manifest: &ManifestMap,
    create_backup: bool,
) -> Result<(String, Vec<BackupEntry>)> {
    let target = Path::new(target_dll_path);
    if !target.exists() {
        return Err(anyhow!(
            "File no longer exists — the game may have been patched. Run a rescan: {}",
            target_dll_path
        ));
    }

    let filename = target
        .file_name()
        .ok_or_else(|| anyhow!("Invalid target DLL path"))?
        .to_string_lossy()
        .to_string();
    let lower_fname = filename.to_ascii_lowercase();

    let dll_type = DllType::from_filename(&filename)
        .ok_or_else(|| anyhow!("Unsupported DLL type: {}", filename))?;

    // Check if this is an FSR 3.1 monolith (amd_fidelityfx_dx12.dll or amd_fidelityfx_vk.dll)
    // that should receive the FSR 4 loader + side-by-side upscaler & framegen DLLs
    let is_fsr_dx12_monolith = lower_fname == "amd_fidelityfx_dx12.dll";
    let is_fsr_vk_monolith = lower_fname == "amd_fidelityfx_vk.dll";

    let source_filename = if is_fsr_dx12_monolith
        && get_cache_dir()
            .join("amd_fidelityfx_loader_dx12.dll")
            .exists()
    {
        "amd_fidelityfx_loader_dx12.dll".to_string()
    } else if is_fsr_vk_monolith
        && get_cache_dir()
            .join("amd_fidelityfx_loader_vk.dll")
            .exists()
    {
        "amd_fidelityfx_loader_vk.dll".to_string()
    } else {
        filename.clone()
    };

    let cached_dll = ensure_dll_cached(&source_filename, manifest).await?;
    let new_version = extract_dll_version(&cached_dll).unwrap_or_else(|| "Latest".to_string());
    let old_version = extract_dll_version(target).unwrap_or_else(|| "Unknown".to_string());

    let mut backup_entries: Vec<BackupEntry> = Vec::new();
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let game_backup_dir = get_backups_dir().join(&game.id).join(&timestamp);

    if create_backup {
        fs::create_dir_all(&game_backup_dir)?;

        let mut path_hasher = Sha256::new();
        path_hasher.update(target_dll_path.to_ascii_lowercase().as_bytes());
        let path_hash = hex::encode(&path_hasher.finalize()[..4]);

        let backup_file_path = game_backup_dir.join(format!("{}_{}", path_hash, filename));
        fs::copy(target, &backup_file_path)
            .with_context(|| format!("Failed to create backup of {}", target_dll_path))?;

        let file_size = fs::metadata(&backup_file_path).map(|m| m.len()).unwrap_or(0);
        let backup_id = format!("{}_{}_{}_{}", game.id, path_hash, filename, timestamp);

        backup_entries.push(BackupEntry {
            id: backup_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            game_path: game.path.clone(),
            dll_type,
            filename: filename.clone(),
            original_version: old_version,
            replaced_with_version: new_version.clone(),
            backup_file_path: backup_file_path.to_string_lossy().to_string(),
            original_dll_path: target_dll_path.to_string(),
            created_at: Utc::now().to_rfc3339(),
            file_size,
            added_by_updater: false,
        });
    }

    // Clear Windows read-only attribute on target DLL if set before overwriting
    if let Ok(meta) = fs::metadata(target) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            let _ = fs::set_permissions(target, perms);
        }
    }

    fs::copy(&cached_dll, target).with_context(|| {
        format!(
            "File is in use — close {} and try again ({})",
            game.name, target_dll_path
        )
    })?;

    // If FSR 3.1 -> FSR 4 monolith upgrade, also place the companion upscaler & framegen DLLs alongside it
    if (is_fsr_dx12_monolith || is_fsr_vk_monolith) && source_filename != filename {
        if let Some(parent_dir) = target.parent() {
            let companions = if is_fsr_vk_monolith {
                [
                    "amd_fidelityfx_upscaler_vk.dll",
                    "amd_fidelityfx_framegeneration_vk.dll",
                ]
            } else {
                [
                    "amd_fidelityfx_upscaler_dx12.dll",
                    "amd_fidelityfx_framegeneration_dx12.dll",
                ]
            };

            for comp in companions {
                if let Ok(comp_cached) = ensure_dll_cached(comp, manifest).await {
                    let comp_target = parent_dir.join(comp);
                    let existed_before = comp_target.exists();
                    if !existed_before && create_backup {
                        let comp_id = format!("{}_added_{}_{}", game.id, comp, timestamp);
                        backup_entries.push(BackupEntry {
                            id: comp_id,
                            game_id: game.id.clone(),
                            game_name: game.name.clone(),
                            game_path: game.path.clone(),
                            dll_type,
                            filename: comp.to_string(),
                            original_version: "Added (FSR 4)".to_string(),
                            replaced_with_version: extract_dll_version(&comp_cached)
                                .unwrap_or_else(|| "4.1.2".to_string()),
                            backup_file_path: String::new(),
                            original_dll_path: comp_target.to_string_lossy().to_string(),
                            created_at: Utc::now().to_rfc3339(),
                            file_size: 0,
                            added_by_updater: true,
                        });
                    }
                    let _ = fs::copy(&comp_cached, &comp_target);
                }
            }
        }
    }

    Ok((new_version, backup_entries))
}

/// Restores a backed-up DLL to its original game location, or removes a side-by-side DLL added during an FSR 4 upgrade.
pub fn restore_backup_entry(entry: &BackupEntry) -> Result<()> {
    let target_path = Path::new(&entry.original_dll_path);

    if entry.added_by_updater {
        if target_path.exists() {
            if let Ok(meta) = fs::metadata(target_path) {
                let mut perms = meta.permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    let _ = fs::set_permissions(target_path, perms);
                }
            }
            let _ = fs::remove_file(target_path);
        }
        return Ok(());
    }

    let backup_path = Path::new(&entry.backup_file_path);

    if !backup_path.exists() {
        return Err(anyhow!(
            "Backup file no longer exists on disk: {}",
            entry.backup_file_path
        ));
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Ok(meta) = fs::metadata(target_path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            let _ = fs::set_permissions(target_path, perms);
        }
    }

    fs::copy(backup_path, target_path).with_context(|| {
        format!(
            "Failed to restore {} to {}",
            entry.backup_file_path, entry.original_dll_path
        )
    })?;

    let _ = fs::remove_file(backup_path);
    Ok(())
}

