use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::database::{get_backups_dir, get_cache_dir};
use crate::models::{AppConfig, BackupEntry, DllType, Game, GpuInfo, RemoteDllVersion};
use crate::pe_version::{extract_dll_version, is_version_newer, preset_hint_for_dll};

/// Shared, connection-pooled HTTP client used across all repository, Steam API, and Online-DB requests.
pub fn shared_http_client() -> &'static reqwest::Client {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
        reqwest::Client::builder()
            .user_agent(concat!("DLSS-Manager/", env!("CARGO_PKG_VERSION")))
            .pool_max_idle_per_host(8)
            .tcp_keepalive(Duration::from_secs(30))
            .timeout(Duration::from_secs(20))
            .build()
            .expect("Failed to initialize shared HTTP client")
    });
    &CLIENT
}

pub const DLSS_SWAPPER_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/beeradmoore/dlss-swapper-manifest-builder/main/manifest.json";
pub const NVIDIA_OFFICIAL_DLSS_RELEASE_API: &str =
    "https://api.github.com/repos/NVIDIA/DLSS/releases/latest";
pub const NVIDIA_OFFICIAL_DLSS_RAW_BASE: &str =
    "https://raw.githubusercontent.com/NVIDIA/DLSS/main/lib/Windows_x86_64/rel";

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

fn is_safe_stable_version(version: &str) -> bool {
    let trimmed = version.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("310.7")
        && !trimmed.to_ascii_lowercase().contains("dev")
        && !trimmed.to_ascii_lowercase().contains("alpha")
}

/// Scans the official local NVIDIA GeForce Driver NGX OTA model cache (`C:\ProgramData\NVIDIA\NGX\models\`)
/// for the latest official NVIDIA-signed full-featured production binaries (`dlss`, `dlssd`, `dlssg`).
fn find_best_local_nvidia_ngx_model(dll_name: &str) -> Option<(std::path::PathBuf, String, u64)> {
    let subdir = match dll_name.to_ascii_lowercase().as_str() {
        "nvngx_dlss.dll" => "dlss",
        "nvngx_dlssd.dll" => "dlssd",
        "nvngx_dlssg.dll" => "dlssg",
        _ => return None,
    };

    let versions_root = Path::new(r"C:\ProgramData\NVIDIA\NGX\models")
        .join(subdir)
        .join("versions");
    let entries = fs::read_dir(&versions_root).ok()?;

    let mut best: Option<(std::path::PathBuf, String, u64)> = None;

    for ver_dir in entries.flatten() {
        let files_dir = ver_dir.path().join("files");
        let Ok(files) = fs::read_dir(&files_dir) else {
            continue;
        };
        for file_entry in files.flatten() {
            let p = file_entry.path();
            if !p.is_file() {
                continue;
            }
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ext != "bin" && ext != "dll" {
                continue;
            }
            let Ok(meta) = fs::metadata(&p) else {
                continue;
            };
            let fsize = meta.len();
            if fsize < 65_536 {
                continue;
            }
            let Some(ver) = extract_dll_version(&p) else {
                continue;
            };
            if !is_safe_stable_version(&ver) {
                continue;
            }
            let take = match &best {
                None => true,
                Some((_, best_ver, best_size)) => {
                    is_version_newer(&ver, best_ver)
                        || (!is_version_newer(best_ver, &ver) && fsize > *best_size)
                }
            };
            if take {
                best = Some((p, ver, fsize));
            }
        }
    }

    best
}

/// Parses the official DLSS Swapper manifest JSON (`beeradmoore/dlss-swapper-manifest-builder`)
/// and merges any newer STABLE (`is_developer_build == false`) versions into `base_manifest`.
/// When two builds share the exact same version (e.g. `310.9.16.0` trimmed SDK build vs full all-presets build),
/// prefers the full all-presets package (`new_size > existing_size`).
fn merge_dlss_swapper_stable_manifest(base_manifest: &mut ManifestMap, swapper_json: &serde_json::Value) {
    let section_map = [
        ("dlss", "nvngx_dlss.dll"),
        ("dlss_g", "nvngx_dlssg.dll"),
        ("dlss_d", "nvngx_dlssd.dll"),
        ("xess", "libxess.dll"),
        ("xess_dx11", "libxess_dx11.dll"),
        ("xess_fg", "libxess_fg.dll"),
        ("xell", "libxell.dll"),
    ];

    for (section_key, dll_filename) in section_map {
        let Some(arr) = swapper_json.get(section_key).and_then(|v| v.as_array()) else {
            continue;
        };

        for item in arr {
            let is_dev = item
                .get("is_developer_build")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_dev {
                continue;
            }
            let ver = item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !is_safe_stable_version(ver) {
                continue;
            }
            let dl_url = item
                .get("download_url")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !dl_url.starts_with("https://") {
                continue;
            }
            let item_size = item.get("file_size").and_then(|v| v.as_u64());

            let should_replace = match base_manifest.get(dll_filename) {
                Some(existing) => {
                    if is_version_newer(ver, &existing.version) {
                        true
                    } else if !is_version_newer(&existing.version, ver) {
                        match (item_size, existing.size) {
                            (Some(new_sz), Some(old_sz)) => new_sz > old_sz,
                            (Some(_), None) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                None => true,
            };

            if should_replace {
                base_manifest.insert(
                    dll_filename.to_string(),
                    ManifestEntry {
                        version: ver.to_string(),
                        sha256: None,
                        size: item_size,
                        url: Some(dl_url.to_string()),
                        date: None,
                    },
                );
            }
        }
    }
}

/// Fetches the latest stable DLL manifest using exclusively official NVIDIA upstream sources
/// (`NVIDIA/DLSS` GitHub repository & local NVIDIA NGX Driver Model cache) plus the
/// `dlss-swapper-manifest-builder` stable feed (never using `Recol/DLSS-Updater-DLLs`).
pub async fn fetch_or_load_manifest() -> Result<ManifestMap> {
    let cache_dir = get_cache_dir();
    let local_manifest_path = cache_dir.join("manifest.json");
    let client = shared_http_client();
    let req_timeout = Duration::from_secs(15);

    let mut merged: ManifestMap = if local_manifest_path.exists() {
        let mut loaded = fs::read(&local_manifest_path)
            .ok()
            .and_then(|b| serde_json::from_slice::<ManifestMap>(&b).ok())
            .unwrap_or_else(default_fallback_manifest);
        for (dll_name, entry) in loaded.iter_mut() {
            if entry
                .url
                .as_deref()
                .map(|u| u.contains("Recol/DLSS-Updater-DLLs"))
                .unwrap_or(false)
            {
                entry.url = Some(format!("{}/{}", NVIDIA_OFFICIAL_DLSS_RAW_BASE, dll_name));
            }
        }
        loaded
    } else {
        default_fallback_manifest()
    };

    let mut updated_online = false;

    // 1. Official NVIDIA DLSS SDK Repository (`https://github.com/NVIDIA/DLSS` -> `lib/Windows_x86_64/rel/`)
    if let Ok(resp) = client
        .get(NVIDIA_OFFICIAL_DLSS_RELEASE_API)
        .timeout(req_timeout)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let raw_tag = json
                    .get("tag_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .trim_start_matches('v')
                    .trim();
                if is_safe_stable_version(raw_tag) {
                    let dot_count = raw_tag.chars().filter(|&c| c == '.').count();
                    let normalized_ver = if dot_count == 2 {
                        format!("{}.0", raw_tag)
                    } else {
                        raw_tag.to_string()
                    };
                    let published = json
                        .get("published_at")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    for dll_name in &["nvngx_dlss.dll", "nvngx_dlssd.dll", "nvngx_dlssg.dll"] {
                        let use_nvidia_official = match merged.get(*dll_name) {
                            Some(existing) => !is_version_newer(&existing.version, &normalized_ver),
                            None => true,
                        };
                        if use_nvidia_official {
                            merged.insert(
                                (*dll_name).to_string(),
                                ManifestEntry {
                                    version: normalized_ver.clone(),
                                    sha256: None,
                                    size: None,
                                    url: Some(format!("{}/{}", NVIDIA_OFFICIAL_DLSS_RAW_BASE, dll_name)),
                                    date: published.clone(),
                                },
                            );
                        }
                    }
                    updated_online = true;
                }
            }
        }
    }

    // 2. Official Local NVIDIA GeForce Driver NGX Model Repository (`C:\ProgramData\NVIDIA\NGX\models\`)
    for dll_name in &["nvngx_dlss.dll", "nvngx_dlssd.dll", "nvngx_dlssg.dll"] {
        if let Some((ngx_bin_path, ngx_ver, ngx_size)) = find_best_local_nvidia_ngx_model(dll_name) {
            let should_sync = match merged.get(*dll_name) {
                Some(existing) => {
                    is_version_newer(&ngx_ver, &existing.version)
                        || (!is_version_newer(&existing.version, &ngx_ver)
                            && ngx_size >= existing.size.unwrap_or(0))
                }
                None => true,
            };
            if should_sync {
                let cached_path = cache_dir.join(dll_name);
                let needs_cache_update = if !cached_path.exists() {
                    true
                } else {
                    let cached_ver = extract_dll_version(&cached_path).unwrap_or_default();
                    let cached_size = fs::metadata(&cached_path).map(|m| m.len()).unwrap_or(0);
                    is_version_newer(&ngx_ver, &cached_ver)
                        || (!is_version_newer(&cached_ver, &ngx_ver) && ngx_size > cached_size)
                };
                if needs_cache_update {
                    let _ = fs::copy(&ngx_bin_path, &cached_path);
                }
                merged.insert(
                    (*dll_name).to_string(),
                    ManifestEntry {
                        version: ngx_ver,
                        sha256: None,
                        size: Some(ngx_size),
                        url: Some(format!("{}/{}", NVIDIA_OFFICIAL_DLSS_RAW_BASE, dll_name)),
                        date: None,
                    },
                );
                updated_online = true;
            }
        }
    }

    // 3. Secondary upstream check (`beeradmoore/dlss-swapper-manifest-builder` — selects any newer stable retail release)
    if let Ok(resp) = client
        .get(DLSS_SWAPPER_MANIFEST_URL)
        .timeout(req_timeout)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                merge_dlss_swapper_stable_manifest(&mut merged, &json);
                updated_online = true;
            }
        }
    }

    // 4. Official NVIDIA Streamline SDK (`https://github.com/NVIDIA-RTX/Streamline/releases/latest`)
    if let Ok(resp) = client
        .get("https://api.github.com/repos/NVIDIA-RTX/Streamline/releases/latest")
        .header("User-Agent", "DLSS-Manager")
        .timeout(req_timeout)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(sl_json) = resp.json::<serde_json::Value>().await {
                let tag = sl_json
                    .get("tag_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .trim_start_matches('v');
                let zip_url = sl_json
                    .get("assets")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        arr.iter().find_map(|a| {
                            let name = a.get("name").and_then(|n| n.as_str()).unwrap_or_default();
                            if name.starts_with("streamline-sdk-")
                                && name.ends_with(".zip")
                                && !name.contains("aarch64")
                                && !name.contains("arm64")
                            {
                                a.get("browser_download_url")
                                    .and_then(|u| u.as_str())
                                    .map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                    });
                if let (false, Some(url)) = (tag.is_empty(), zip_url) {
                    let sl_ver = format!("{}.0", tag);
                    for sl_dll in &[
                        "sl.interposer.dll",
                        "sl.common.dll",
                        "sl.dlss.dll",
                        "sl.dlss_g.dll",
                        "sl.dlss_d.dll",
                        "sl.reflex.dll",
                        "sl.pcl.dll",
                        "sl.directsr.dll",
                        "sl.nis.dll",
                    ] {
                        let should_set = merged
                            .get(*sl_dll)
                            .map(|e| is_version_newer(&sl_ver, &e.version) || e.url.is_none())
                            .unwrap_or(true);
                        if should_set {
                            merged.insert(
                                (*sl_dll).to_string(),
                                ManifestEntry {
                                    version: sl_ver.clone(),
                                    sha256: None,
                                    size: None,
                                    url: Some(url.clone()),
                                    date: None,
                                },
                            );
                        }
                    }
                    updated_online = true;
                }
            }
        }
    }

    // 5. Official Microsoft DirectStorage NuGet Package (`Microsoft.Direct3D.DirectStorage`)
    if let Ok(resp) = client
        .get("https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.directstorage/index.json")
        .timeout(req_timeout)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(ds_json) = resp.json::<serde_json::Value>().await {
                if let Some(latest_ver) = ds_json
                    .get("versions")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .filter(|v| !v.contains('-'))
                            .last()
                    })
                {
                    let nupkg_url = format!(
                        "https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.directstorage/{0}/microsoft.direct3d.directstorage.{0}.nupkg",
                        latest_ver
                    );
                    for ds_dll in &["dstorage.dll", "dstoragecore.dll"] {
                        let should_set = merged
                            .get(*ds_dll)
                            .map(|e| is_version_newer(latest_ver, &e.version) || e.url.is_none())
                            .unwrap_or(true);
                        if should_set {
                            merged.insert(
                                (*ds_dll).to_string(),
                                ManifestEntry {
                                    version: latest_ver.to_string(),
                                    sha256: None,
                                    size: None,
                                    url: Some(nupkg_url.clone()),
                                    date: None,
                                },
                            );
                        }
                    }
                    updated_online = true;
                }
            }
        }
    }

    if updated_online {
        if let Ok(serialized) = serde_json::to_vec_pretty(&merged) {
            let _ = fs::write(&local_manifest_path, serialized);
        }
    }

    Ok(merged)
}

pub fn load_cached_manifest_sync() -> ManifestMap {
    let local_manifest_path = get_cache_dir().join("manifest.json");
    if let Ok(bytes) = fs::read(&local_manifest_path) {
        if let Ok(mut parsed) = serde_json::from_slice::<ManifestMap>(&bytes) {
            for (dll_name, entry) in parsed.iter_mut() {
                if entry
                    .url
                    .as_deref()
                    .map(|u| u.contains("Recol/DLSS-Updater-DLLs"))
                    .unwrap_or(false)
                {
                    entry.url = Some(format!("{}/{}", NVIDIA_OFFICIAL_DLSS_RAW_BASE, dll_name));
                }
            }
            return parsed;
        }
    }
    default_fallback_manifest()
}

fn default_fallback_manifest() -> ManifestMap {
    let mut map = HashMap::new();
    let defaults = [
        ("nvngx_dlss.dll", "310.9.16.0"),
        ("nvngx_dlssg.dll", "310.9.16.0"),
        ("nvngx_dlssd.dll", "310.9.16.0"),
        ("nvngx_dlssnr.dll", "310.8.0.0"),
        ("libxess.dll", "2.0.2.68"),
        ("libxess_dx11.dll", "2.0.2.68"),
        ("libxess_fg.dll", "1.3.1.78"),
        ("libxell.dll", "1.2.1.13"),
        ("dstorage.dll", "1.3.0"),
        ("dstoragecore.dll", "1.3.0"),
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
    let fallback = format!("{}/{}", NVIDIA_OFFICIAL_DLSS_RAW_BASE, safe_name);

    if let Some(url) = &entry.url {
        if url.starts_with("https://") && !url.contains("Recol/DLSS-Updater-DLLs") {
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

/// Downloads a DLL from the official NVIDIA upstream / manifest into `%LOCALAPPDATA%\gOOvER\DLSS-Manager\dll_cache\`
/// (or copies the official local NVIDIA GeForce Driver NGX OTA model binary when available)
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

    // Prefer the official local NVIDIA GeForce Driver NGX OTA model binary (`C:\ProgramData\NVIDIA\NGX\models\`)
    // when its version is >= the manifest version (ensuring the full 74.1 MB all-presets NVIDIA binary is used)
    if let Some((ngx_bin_path, ngx_ver, ngx_size)) = find_best_local_nvidia_ngx_model(&safe_name) {
        if info.version.is_empty() || !is_version_newer(&info.version, &ngx_ver) {
            let needs_copy = if !cache_path.exists() {
                true
            } else {
                let cached_ver = extract_dll_version(&cache_path).unwrap_or_default();
                let cached_size = fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
                is_version_newer(&ngx_ver, &cached_ver)
                    || (!is_version_newer(&cached_ver, &ngx_ver) && ngx_size > cached_size)
            };
            if needs_copy {
                let _ = fs::copy(&ngx_bin_path, &cache_path);
            }
            if cache_path.exists() {
                return Ok(cache_path);
            }
        }
    }

    // If cached, version is already up to date, and cached file size is not smaller than manifest's full package size, return immediately
    if cache_path.exists() {
        if let Some(local_ver) = extract_dll_version(&cache_path) {
            let cached_size = fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
            let size_ok = info.size.map(|sz| cached_size >= sz).unwrap_or(true);
            if (info.version.is_empty() || !is_version_newer(&info.version, &local_ver)) && size_ok {
                return Ok(cache_path);
            }
        }
    }

    let url = resolve_dll_download_url(&safe_name, &info);
    let client = shared_http_client();

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(300))
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

    let raw_bytes = resp.bytes().await?;

    // Support both raw PE `.dll` downloads (`MZ`) and `.zip` packages (`PK\x03\x04`, used by DLSS Swapper manifest)
    let dll_bytes: Vec<u8> = if raw_bytes.len() >= 4 && &raw_bytes[0..4] == b"PK\x03\x04" {
        let reader = std::io::Cursor::new(&raw_bytes);
        let mut archive = zip::ZipArchive::new(reader)
            .with_context(|| format!("Failed to open ZIP archive for {}", safe_name))?;
        let mut extracted: Option<Vec<u8>> = None;
        for idx in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(idx) {
                let entry_name = file
                    .name()
                    .map(|n| n.to_ascii_lowercase())
                    .unwrap_or_default();
                if entry_name.ends_with(".dll") {
                    let mut buf = Vec::with_capacity(file.size() as usize);
                    std::io::Read::read_to_end(&mut file, &mut buf)?;
                    extracted = Some(buf);
                    break;
                }
            }
        }
        extracted.ok_or_else(|| anyhow!("ZIP archive for {} did not contain a .dll file", safe_name))?
    } else {
        raw_bytes.to_vec()
    };

    if dll_bytes.len() < 4096 || &dll_bytes[0..2] != b"MZ" {
        return Err(anyhow!(
            "Downloaded file for {} is not a valid PE binary",
            safe_name
        ));
    }

    if let Some(expected_sha) = &info.sha256 {
        let mut hasher = Sha256::new();
        hasher.update(&dll_bytes);
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
    fs::write(&tmp_path, &dll_bytes)?;
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
    if matches!(
        dll_type,
        DllType::Dlss | DllType::DlssD | DllType::DlssG | DllType::Streamline
    ) {
        crate::pe_version::verify_authenticode_signature(&cached_dll)
            .map_err(|e| anyhow!("Digital signature check failed: {}", e))?;
    }
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

