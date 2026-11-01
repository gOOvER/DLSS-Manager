use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rayon::prelude::*;
use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::{DirEntry, WalkDir};

use crate::database::{import_legacy_configured_paths, load_legacy_db_game_roots};
use crate::models::{DetectedDll, DllType, Game, LauncherType};
use crate::pe_version::{extract_dll_version, preset_hint_for_dll};

#[derive(Debug, Clone)]
struct DiscoveredGameRoot {
    name: String,
    path: PathBuf,
    launcher: LauncherType,
    app_id: Option<String>,
    cover_url: Option<String>,
}

const SKIP_DIRS: &[&str] = &[
    "__pycache__",
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "logs",
    "log",
    "logbackups",
    "debuglogs",
    "saves",
    "save",
    "screenshots",
    "crash",
    "crashdumps",
    "dumps",
    "temp",
    "tmp",
    "cache",
    ".cache",
    "shader_cache",
    "shadercache",
    "gpucache",
    "webcache",
    ".dlss_backup",
    "arm64",
    "win32",
    "x86",
];

fn mounted_drives() -> Vec<char> {
    ('A'..='Z')
        .filter(|&d| PathBuf::from(format!("{}:\\", d)).exists())
        .collect()
}

fn parse_launcher_label(label: &str) -> LauncherType {
    let lower = label.to_ascii_lowercase();
    if lower.contains("steam") {
        LauncherType::Steam
    } else if lower.contains("epic") {
        LauncherType::Epic
    } else if lower.contains("gog") {
        LauncherType::Gog
    } else if lower.contains("ubisoft") {
        LauncherType::Ubisoft
    } else if lower.contains("ea") || lower.contains("origin") {
        LauncherType::Ea
    } else if lower.contains("battle") || lower.contains("blizzard") {
        LauncherType::Battlenet
    } else if lower.contains("xbox") {
        LauncherType::Xbox
    } else if lower.contains("rsi") || lower.contains("star citizen") {
        LauncherType::Rsi
    } else {
        LauncherType::Custom
    }
}

/// Scans all supported game launchers, drives, legacy databases, and custom folders in parallel using Rayon.
pub fn scan_all_games(
    custom_folders: &[String],
    ignored_games: &[String],
    blacklisted_games: &[String],
) -> Vec<Game> {
    let drives = mounted_drives();
    let mut roots: Vec<DiscoveredGameRoot> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    let mut add_root = |root: DiscoveredGameRoot| {
        if !root.path.exists() || !root.path.is_dir() {
            return;
        }
        let norm = fs::canonicalize(&root.path)
            .unwrap_or_else(|_| root.path.clone())
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_ascii_lowercase();
        if norm.len() <= 3 {
            // Never add a bare drive root like "c:" as a single game
            return;
        }
        if seen_paths.insert(norm) {
            roots.push(root);
        }
    };

    // 1. Discover from all supported launchers across registry + all mounted drives
    for g in discover_steam_games(&drives) {
        add_root(g);
    }
    for g in discover_epic_games(&drives) {
        add_root(g);
    }
    for g in discover_gog_games(&drives) {
        add_root(g);
    }
    for g in discover_ubisoft_games(&drives) {
        add_root(g);
    }
    for g in discover_ea_games(&drives) {
        add_root(g);
    }
    for g in discover_battlenet_games(&drives) {
        add_root(g);
    }
    for g in discover_xbox_games(&drives) {
        add_root(g);
    }
    for g in discover_rsi_star_citizen(&drives) {
        add_root(g);
    }
    for g in discover_generic_game_dirs(&drives) {
        add_root(g);
    }

    // 2. Import any games previously recorded in legacy dlss_updater.db / games.db
    for (name, path_str, launcher_str, app_id) in load_legacy_db_game_roots() {
        let p = PathBuf::from(&path_str);
        if p.exists() {
            let launcher = parse_launcher_label(&launcher_str);
            let cover_url = app_id.as_ref().map(|id| {
                format!(
                    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                    id
                )
            });
            add_root(DiscoveredGameRoot {
                name,
                path: p,
                launcher,
                app_id,
                cover_url,
            });
        }
    }

    // 3. Process user-configured custom folders + legacy config.toml launcher paths
    let mut all_custom: Vec<String> = custom_folders.to_vec();
    for leg in import_legacy_configured_paths() {
        if !all_custom.iter().any(|c| c.eq_ignore_ascii_case(&leg)) {
            all_custom.push(leg);
        }
    }

    for folder in &all_custom {
        for g in expand_custom_or_library_folder(Path::new(folder)) {
            add_root(g);
        }
    }

    let ignored_set: HashSet<String> = ignored_games.iter().map(|s| s.to_ascii_lowercase()).collect();
    let blacklist_set: HashSet<String> =
        blacklisted_games.iter().map(|s| s.to_ascii_lowercase()).collect();
    let now_str = Utc::now().to_rfc3339();

    let mut games: Vec<Game> = roots
        .into_par_iter()
        .map(|root| {
            let (dlls, anti_cheat) = scan_game_directory(&root.path);
            let path_str = root.path.to_string_lossy().to_string();
            let id = compute_game_id(&path_str);
            let raw_name = root.name.clone();
            let (pretty_name, inferred_app_id) = prettify_game_name(&raw_name, &root.path);
            let app_id = root.app_id.or(inferred_app_id);
            let cover_url = root.cover_url.or_else(|| {
                app_id.as_ref().map(|aid| {
                    format!(
                        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                        aid
                    )
                })
            });
            let key_lower = pretty_name.to_ascii_lowercase();
            let raw_lower = raw_name.to_ascii_lowercase();

            Game {
                id: id.clone(),
                name: pretty_name,
                path: path_str,
                launcher: root.launcher,
                launcher_name: root.launcher.display_name().to_string(),
                launcher_icon: root.launcher.icon_name().to_string(),
                app_id,
                cover_url,
                dlls,
                anti_cheat,
                ignored: ignored_set.contains(&id)
                    || ignored_set.contains(&key_lower)
                    || ignored_set.contains(&raw_lower),
                blacklisted: blacklist_set.contains(&id)
                    || blacklist_set.contains(&key_lower)
                    || blacklist_set.contains(&raw_lower),
                has_backups: false,
                last_scanned: now_str.clone(),
                original_name: Some(raw_name),
                custom_exe_path: None,
                dlss_sr_preset: None,
                dlss_rr_preset: None,
                dlss_fg_preset: None,
                db_profile: None,
            }
        })
        .collect();

    // Remove any parent container folder if one of its child folders is also in `games`
    let all_norm_paths: Vec<String> = games
        .iter()
        .map(|g| {
            format!(
                "{}\\",
                g.path
                    .replace('/', "\\")
                    .trim_end_matches('\\')
                    .to_ascii_lowercase()
            )
        })
        .collect();

    games.retain(|g| {
        let my_prefix = format!(
            "{}\\",
            g.path
                .replace('/', "\\")
                .trim_end_matches('\\')
                .to_ascii_lowercase()
        );
        !all_norm_paths
            .iter()
            .any(|other| other != &my_prefix && other.starts_with(&my_prefix))
    });

    // Only keep games that actually contain at least one upscaler DLL and a valid game executable (.exe)
    // This filters out uninstalled leftover/ghost directories that only left behind an orphaned DLL or backup file.
    games.retain(|g| !g.dlls.is_empty() && has_valid_game_executable(Path::new(&g.path)));

    games.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    games
}

/// Expands a custom or library path into one or more game roots.
/// If the path is a Steam root, library root (`steamapps/common`, `Epic Games`, `Games`, etc.),
/// its immediate child directories are added as games; if it is a standalone game folder, it is added directly.
fn expand_custom_or_library_folder(path: &Path) -> Vec<DiscoveredGameRoot> {
    let mut out = Vec::new();
    if !path.exists() || !path.is_dir() {
        return out;
    }

    // Check if path has steamapps/common
    let steam_common = path.join("steamapps").join("common");
    if steam_common.exists() {
        if let Ok(entries) = fs::read_dir(&steam_common) {
            for e in entries.flatten() {
                if e.path().is_dir() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if !is_redist_or_non_game(&name) {
                        out.push(DiscoveredGameRoot {
                            name,
                            path: e.path(),
                            launcher: LauncherType::Steam,
                            app_id: None,
                            cover_url: None,
                        });
                    }
                }
            }
        }
        return out;
    }

    let path_lower = path.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
    let inferred_launcher = if path_lower.contains("steam") {
        LauncherType::Steam
    } else if path_lower.contains("epic") {
        LauncherType::Epic
    } else if path_lower.contains("gog") {
        LauncherType::Gog
    } else if path_lower.contains("ubisoft") {
        LauncherType::Ubisoft
    } else if path_lower.contains("ea games") || path_lower.contains("origin") {
        LauncherType::Ea
    } else if path_lower.contains("battle.net") {
        LauncherType::Battlenet
    } else if path_lower.contains("xbox") {
        LauncherType::Xbox
    } else if path_lower.contains("starcitizen") || path_lower.contains("roberts space") {
        LauncherType::Rsi
    } else {
        LauncherType::Custom
    };

    // Does `path` itself look like a single game root (e.g. has .exe or Binaries/Engine)?
    let has_exe_directly = fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().is_file()
                    && e.file_name()
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .ends_with(".exe")
            })
        })
        .unwrap_or(false);
    let has_engine_or_bin = path.join("Binaries").exists()
        || path.join("Engine").exists()
        || path.join("bin").exists()
        || path.join("Bin64").exists();

    let default_cover = if inferred_launcher == LauncherType::Rsi {
        Some("assets/icons/starcitizen_cover.png".to_string())
    } else {
        None
    };

    if has_exe_directly || has_engine_or_bin {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        out.push(DiscoveredGameRoot {
            name,
            path: path.to_path_buf(),
            launcher: inferred_launcher,
            app_id: None,
            cover_url: default_cover,
        });
        return out;
    }

    // Otherwise treat `path` as a library container and enumerate its immediate child directories
    if let Ok(entries) = fs::read_dir(path) {
        for e in entries.flatten() {
            let child = e.path();
            if child.is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if !SKIP_DIRS.contains(&name.to_ascii_lowercase().as_str())
                    && !is_redist_or_non_game(&name)
                {
                    out.push(DiscoveredGameRoot {
                        name,
                        path: child,
                        launcher: inferred_launcher,
                        app_id: None,
                        cover_url: default_cover.clone(),
                    });
                }
            }
        }
    }

    out
}

fn is_redist_or_non_game(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("steamworks common")
        || lower.starts_with("proton ")
        || lower.starts_with("steam linux runtime")
        || lower == "steam controller configs"
        || lower == "directx"
        || lower == "redist"
}

pub fn compute_game_id(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(
        path.replace('/', "\\")
            .trim_end_matches('\\')
            .to_ascii_lowercase()
            .as_bytes(),
    );
    let digest = hasher.finalize();
    hex::encode(&digest[..8])
}

fn should_descend(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    if entry.file_type().is_dir() {
        let lower = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if SKIP_DIRS.contains(&lower.as_str()) {
            return false;
        }
    }
    true
}

/// Walks a game folder up to depth 12 (skipping non-game caches/logs),
/// collecting all supported upscaler DLLs and detecting anti-cheat.
pub fn scan_game_directory(root: &Path) -> (Vec<DetectedDll>, Option<String>) {
    let mut dlls = Vec::new();
    let mut anti_cheat: Option<String> = None;

    // Fast path for Star Citizen channels (`LIVE`, `PTU`, `EPTU`, etc.) which contain a 150+ GB `Data.p4k`,
    // `user/`, `data/`, and `logbackups/`. All binaries reside in `Bin64/` and `EasyAntiCheat/`.
    let is_star_citizen_channel =
        root.join("Bin64").join("StarCitizen.exe").exists() || root.join("build_manifest.id").exists();
    if is_star_citizen_channel && root.join("EasyAntiCheat").exists() {
        anti_cheat = Some("EasyAntiCheat".to_string());
    }
    let scan_roots: Vec<PathBuf> = if is_star_citizen_channel && root.join("Bin64").exists() {
        vec![root.join("Bin64")]
    } else {
        vec![root.to_path_buf()]
    };

    for scan_root in scan_roots {
        for entry in WalkDir::new(&scan_root)
            .max_depth(12)
            .follow_links(false)
            .into_iter()
            .filter_entry(should_descend)
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy();
            let lower = file_name.to_ascii_lowercase();

            // Detect known Anti-Cheat binaries
            if anti_cheat.is_none() {
                if lower.contains("easyanticheat") {
                    anti_cheat = Some("EasyAntiCheat".to_string());
                } else if lower.contains("beservice")
                    || lower.contains("beclient")
                    || lower == "battleye.ini"
                {
                    anti_cheat = Some("BattlEye".to_string());
                } else if lower.contains("vgk.sys") || lower.contains("vgc.exe") {
                    anti_cheat = Some("Vanguard".to_string());
                } else if lower.contains("randgrid.sys") {
                    anti_cheat = Some("Ricochet".to_string());
                }
            }

            if !lower.ends_with(".dll") {
                continue;
            }

            let full_path = entry.path();
            if let Some(dll_type) = DllType::from_filename(&lower) {
                let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let version =
                    extract_dll_version(full_path).unwrap_or_else(|| "Unknown".to_string());
                let preset_label = preset_hint_for_dll(dll_type, &version);

                dlls.push(DetectedDll {
                    path: full_path.to_string_lossy().to_string(),
                    filename: file_name.to_string(),
                    dll_type,
                    type_label: dll_type.short_label().to_string(),
                    version,
                    file_size,
                    preset_label,
                    update_available: false,
                    latest_version: None,
                    gpu_supported: true,
                    technology_enabled: true,
                    bad_build_warning: None,
                });
            }
        }
    }

    (dlls, anti_cheat)
}

/// Prettifies raw folder/manifest names (e.g. Battle.net `_retail_` -> `Overwatch 2`,
/// Star Citizen `LIVE` -> `Star Citizen`, or CamelCase `ForzaHorizon5` -> `Forza Horizon 5`)
/// and infers well-known Steam App IDs for cover art.
pub fn prettify_game_name(raw_name: &str, full_path: &Path) -> (String, Option<String>) {
    let trimmed = raw_name.trim();
    let lower = trimmed.to_ascii_lowercase();
    let full_lower = full_path.to_string_lossy().to_ascii_lowercase();

    // Handle Star Citizen channel folders (`LIVE`, `PTU`, `EPTU`, `TECH-PREVIEW`, `HOTFIX`)
    if full_lower.contains("starcitizen")
        || full_path.join("Bin64").join("StarCitizen.exe").exists()
        || full_path.join("build_manifest.id").exists()
    {
        if lower == "live" || lower == "starcitizen" || lower == "star citizen" {
            return ("Star Citizen".to_string(), Some("starcitizen_live".to_string()));
        } else if matches!(
            lower.as_str(),
            "ptu" | "eptu" | "tech-preview" | "hotfix" | "4.0_preview"
        ) {
            return (
                format!("Star Citizen ({})", trimmed.to_ascii_uppercase()),
                Some(format!("starcitizen_{}", lower)),
            );
        }
    }

    // Handle Battle.net sub-flavor folders (`_retail_`, `_ptr_`, `_classic_`)
    if lower.starts_with('_') && lower.ends_with('_') {
        if let Some(parent_name) = full_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
        {
            let p_lower = parent_name.to_ascii_lowercase();
            if p_lower.contains("overwatch") {
                return ("Overwatch 2".to_string(), Some("2357570".to_string()));
            } else if p_lower.contains("world of warcraft") {
                return ("World of Warcraft".to_string(), None);
            } else if p_lower.contains("diablo iv") {
                return ("Diablo IV".to_string(), Some("2344520".to_string()));
            } else if !parent_name.is_empty() {
                return (parent_name, None);
            }
        }
    }

    let inferred_app_id = match lower.as_str() {
        "diablo iv" => Some("2344520".to_string()),
        "overwatch" | "overwatch 2" | "overwatch®" => Some("2357570".to_string()),
        "forzahorizon5" | "forza horizon 5" => Some("1551360".to_string()),
        "cyberpunk 2077" => Some("1091500".to_string()),
        "star wars outlaws" => Some("2842040".to_string()),
        "msfs2024" | "microsoft flight simulator 2024" => Some("2537590".to_string()),
        _ => None,
    };

    // If single word with CamelCase or trailing digits (e.g. "ForzaHorizon5", "Subnautica2", "SpaceEngineers2")
    if !trimmed.contains(' ') && trimmed.len() > 4 {
        let mut out = String::with_capacity(trimmed.len() + 4);
        let chars: Vec<char> = trimmed.chars().collect();
        for i in 0..chars.len() {
            let c = chars[i];
            if i > 0 {
                let prev = chars[i - 1];
                let next_is_lower = chars.get(i + 1).map(|n| n.is_ascii_lowercase()).unwrap_or(false);
                if (prev.is_ascii_lowercase() && c.is_ascii_uppercase())
                    || (prev.is_ascii_alphabetic() && c.is_ascii_digit() && prev != 'D' && prev != 'X')
                    || (prev.is_ascii_uppercase() && c.is_ascii_uppercase() && next_is_lower)
                {
                    out.push(' ');
                }
            }
            out.push(c);
        }
        return (out, inferred_app_id);
    }

    (trimmed.to_string(), inferred_app_id)
}

/// Finds the primary game executable (`.exe`) inside a game directory for launching or NVAPI DRS profile binding.
pub fn find_primary_game_executable(root: &Path) -> Option<PathBuf> {
    let ignore_substrings = [
        "unins",
        "crash",
        "prereq",
        "redist",
        "setup",
        "installer",
        "dxsetup",
        "vcredist",
        "easyanticheat",
        "beservice",
        "battleye",
        "unitycrashhandler",
        "cesium",
        "benchmark",
        "launcher",
        "updater",
        "dotnet",
    ];

    let mut candidates: Vec<(u64, PathBuf)> = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_descend)
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !fname.ends_with(".exe") {
            continue;
        }
        if ignore_substrings.iter().any(|s| fname.contains(s)) {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        // Prefer larger shipping binaries (e.g. > 500 KB)
        if size > 200_000 {
            candidates.push((size, entry.path().to_path_buf()));
        }
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().next().map(|(_, p)| p)
}

/// Checks whether a directory (or its immediate parent for sub-flavor folders like `_retail_`)
/// actually contains at least one valid game executable (`.exe`), ensuring uninstalled leftover folders
/// that only contain orphaned `.dll` files are not listed as installed games.
pub fn has_valid_game_executable(root: &Path) -> bool {
    if !root.exists() || !root.is_dir() {
        return false;
    }
    if root.join("Bin64").join("StarCitizen.exe").exists() {
        return true;
    }
    if find_primary_game_executable(root).is_some() {
        return true;
    }

    // Fallback: check for any non-uninstaller .exe inside root up to depth 6
    let has_exe_in_dir = |dir: &Path, max_d: usize| -> bool {
        for entry in WalkDir::new(dir)
            .max_depth(max_d)
            .follow_links(false)
            .into_iter()
            .filter_entry(should_descend)
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if fname.ends_with(".exe")
                && !fname.starts_with("unins")
                && !fname.contains("crash")
                && !fname.contains("redist")
                && !fname.contains("dxsetup")
            {
                if entry.metadata().map(|m| m.len()).unwrap_or(0) > 50_000 {
                    return true;
                }
            }
        }
        false
    };

    if has_exe_in_dir(root, 6) {
        return true;
    }

    // If root is a Battle.net or engine subfolder (e.g., `_retail_`, `_classic_`, `Binaries`, `Content`),
    // check 2 levels in the parent directory as well.
    if let Some(folder_name) = root.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()) {
        if (folder_name.starts_with('_') && folder_name.ends_with('_'))
            || matches!(
                folder_name.as_str(),
                "binaries" | "bin" | "bin64" | "x64" | "content" | "win64" | "gamelaunchhelper"
            )
        {
            if let Some(parent) = root.parent() {
                if has_exe_in_dir(parent, 2) {
                    return true;
                }
            }
        }
    }

    false
}

/// Auto-detects the user's 64-bit Steam ID (17-digit `7656119...`) from `<SteamPath>\config\loginusers.vdf`.
pub fn detect_steam_id_from_vdf() -> Option<String> {
    let mut steam_roots: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            if let Ok(p) = hkcu.get_value::<String, _>("SteamPath") {
                steam_roots.push(PathBuf::from(p));
            }
        }
        for reg_sub in &[
            "SOFTWARE\\WOW6432Node\\Valve\\Steam",
            "SOFTWARE\\Valve\\Steam",
        ] {
            if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_sub) {
                if let Ok(p) = hklm.get_value::<String, _>("InstallPath") {
                    steam_roots.push(PathBuf::from(p));
                }
            }
        }
    }
    steam_roots.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
    steam_roots.push(PathBuf::from("D:\\Steam"));

    let id_re = Regex::new(r#""(7656119\d{10})""#).ok()?;
    let recent_re = Regex::new(r#"(?is)"(7656119\d{10})"\s*\{[^}]*"MostRecent"\s*"1""#).ok()?;

    for root in steam_roots {
        let vdf_path = root.join("config").join("loginusers.vdf");
        if let Ok(content) = fs::read_to_string(&vdf_path) {
            if let Some(cap) = recent_re.captures(&content) {
                return Some(cap[1].to_string());
            }
            if let Some(cap) = id_re.captures(&content) {
                return Some(cap[1].to_string());
            }
        }
    }
    None
}

/// Normalizes a game title for Steam API / Owned Games matching (strips trademarks, punctuation, spaces, and leading "the").
pub fn normalize_game_name_for_match(name: &str) -> String {
    let lower = name
        .to_ascii_lowercase()
        .replace('™', "")
        .replace('®', "")
        .replace('©', "");
    let trimmed = lower.strip_prefix("the ").unwrap_or(&lower);
    trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

// =============================================================================
// 1. Steam Discovery (Registry + all drives + libraryfolders.vdf + acf + common/*)
// =============================================================================

fn discover_steam_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    let mut steam_roots: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            if let Ok(p) = hkcu.get_value::<String, _>("SteamPath") {
                steam_roots.push(PathBuf::from(p));
            }
        }
        for reg_sub in &[
            "SOFTWARE\\WOW6432Node\\Valve\\Steam",
            "SOFTWARE\\Valve\\Steam",
        ] {
            if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_sub) {
                if let Ok(p) = hklm.get_value::<String, _>("InstallPath") {
                    steam_roots.push(PathBuf::from(p));
                }
            }
        }
    }

    for &d in drives {
        for suffix in &[
            "Program Files (x86)\\Steam",
            "Program Files\\Steam",
            "Steam",
            "SteamLibrary",
            "Games\\Steam",
            "Games\\SteamLibrary",
            "Spiele\\Steam",
            "Spiele\\SteamLibrary",
        ] {
            let p = PathBuf::from(format!("{}:\\{}", d, suffix));
            if p.exists() {
                steam_roots.push(p);
            }
        }
    }

    let mut steamapps_dirs: Vec<PathBuf> = Vec::new();
    let path_re = Regex::new(r#"(?i)"path"\s+"([^"]+)""#).ok();

    for root in steam_roots {
        let root_str = root.to_string_lossy().replace('/', "\\");
        let sa = if root_str.to_ascii_lowercase().ends_with("\\steamapps") {
            root.clone()
        } else if root_str
            .to_ascii_lowercase()
            .ends_with("\\steamapps\\common")
        {
            root.parent().unwrap_or(&root).to_path_buf()
        } else {
            root.join("steamapps")
        };

        if sa.exists() {
            steamapps_dirs.push(sa.clone());
        }

        // Check both steamapps/libraryfolders.vdf and config/libraryfolders.vdf
        for vdf in &[
            sa.join("libraryfolders.vdf"),
            root.join("config").join("libraryfolders.vdf"),
        ] {
            if let (Ok(content), Some(re)) = (fs::read_to_string(vdf), &path_re) {
                for cap in re.captures_iter(&content) {
                    let lib_path = cap[1].replace("\\\\", "\\");
                    let lib_sa = PathBuf::from(&lib_path).join("steamapps");
                    if lib_sa.exists() {
                        steamapps_dirs.push(lib_sa);
                    } else if PathBuf::from(&lib_path).exists() {
                        steamapps_dirs.push(PathBuf::from(&lib_path));
                    }
                }
            }
        }
    }

    let appid_re = Regex::new(r#"(?i)"appid"\s+"(\d+)""#).unwrap();
    let name_re = Regex::new(r#"(?i)"name"\s+"([^"]+)""#).unwrap();
    let installdir_re = Regex::new(r#"(?i)"installdir"\s+"([^"]+)""#).unwrap();

    for sa_dir in steamapps_dirs {
        // 1a. Parse appmanifest_*.acf
        if let Ok(entries) = fs::read_dir(&sa_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !fname.starts_with("appmanifest_") || !fname.ends_with(".acf") {
                    continue;
                }
                let Ok(acf) = fs::read_to_string(entry.path()) else {
                    continue;
                };
                let app_id = appid_re.captures(&acf).map(|c| c[1].to_string());
                let Some(name) = name_re.captures(&acf).map(|c| c[1].to_string()) else {
                    continue;
                };
                let Some(installdir) = installdir_re.captures(&acf).map(|c| c[1].to_string())
                else {
                    continue;
                };

                if is_redist_or_non_game(&name) {
                    continue;
                }

                let game_path = sa_dir.join("common").join(installdir);
                if game_path.exists() {
                    let cover_url = app_id.as_ref().map(|id| {
                        format!(
                            "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                            id
                        )
                    });
                    results.push(DiscoveredGameRoot {
                        name,
                        path: game_path,
                        launcher: LauncherType::Steam,
                        app_id,
                        cover_url,
                    });
                }
            }
        }

        // 1b. Also enumerate every directory in steamapps/common/* directly
        // (catches games whose appmanifest is missing or on another mount)
        let common_dir = if sa_dir
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with("common")
        {
            sa_dir.clone()
        } else {
            sa_dir.join("common")
        };

        if let Ok(entries) = fs::read_dir(&common_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if is_redist_or_non_game(&name) {
                    continue;
                }
                results.push(DiscoveredGameRoot {
                    name,
                    path,
                    launcher: LauncherType::Steam,
                    app_id: None,
                    cover_url: None,
                });
            }
        }
    }

    results
}

// =============================================================================
// 2. Epic Games Discovery (Manifests + HKCR Registry + all drives)
// =============================================================================

fn discover_epic_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    let manifests_dir = PathBuf::from("C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests");
    if let Ok(entries) = fs::read_dir(manifests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            let Some(install_loc) = json.get("InstallLocation").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(display_name) = json.get("DisplayName").and_then(|v| v.as_str()) else {
                continue;
            };
            let app_name = json
                .get("AppName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let game_path = PathBuf::from(install_loc);
            if game_path.exists() {
                results.push(DiscoveredGameRoot {
                    name: display_name.to_string(),
                    path: game_path,
                    launcher: LauncherType::Epic,
                    app_id: app_name,
                    cover_url: None,
                });
            }
        }
    }

    for &d in drives {
        for suffix in &["Program Files\\Epic Games", "Epic Games", "Games\\Epic Games"] {
            let epic_root = PathBuf::from(format!("{}:\\{}", d, suffix));
            if let Ok(entries) = fs::read_dir(&epic_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.eq_ignore_ascii_case("Launcher") || name.eq_ignore_ascii_case("DirectXRedist") {
                        continue;
                    }
                    results.push(DiscoveredGameRoot {
                        name,
                        path,
                        launcher: LauncherType::Epic,
                        app_id: None,
                        cover_url: None,
                    });
                }
            }
        }
    }

    results
}

// =============================================================================
// 3. GOG Galaxy Discovery (Registry + all drives)
// =============================================================================

fn discover_gog_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(games_key) =
            RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\GOG.com\\Games")
        {
            for sub in games_key.enum_keys().flatten() {
                if let Ok(k) = games_key.open_subkey(&sub) {
                    let name = k
                        .get_value::<String, _>("gameName")
                        .or_else(|_| k.get_value::<String, _>("GAMENAME"));
                    let path = k
                        .get_value::<String, _>("path")
                        .or_else(|_| k.get_value::<String, _>("PATH"));
                    if let (Ok(name), Ok(path)) = (name, path) {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            results.push(DiscoveredGameRoot {
                                name,
                                path: p,
                                launcher: LauncherType::Gog,
                                app_id: Some(sub),
                                cover_url: None,
                            });
                        }
                    }
                }
            }
        }
    }

    for &d in drives {
        for suffix in &[
            "GOG Games",
            "GOG Galaxy\\Games",
            "Program Files (x86)\\GOG Galaxy\\Games",
            "Games\\GOG Games",
        ] {
            let gog_root = PathBuf::from(format!("{}:\\{}", d, suffix));
            if let Ok(entries) = fs::read_dir(&gog_root) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        results.push(DiscoveredGameRoot {
                            name,
                            path: entry.path(),
                            launcher: LauncherType::Gog,
                            app_id: None,
                            cover_url: None,
                        });
                    }
                }
            }
        }
    }

    results
}

// =============================================================================
// 4. Ubisoft Connect Discovery (Registry + all drives)
// =============================================================================

fn discover_ubisoft_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    let mut ubi_game_roots: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(installs_key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs")
        {
            for sub in installs_key.enum_keys().flatten() {
                if let Ok(k) = installs_key.open_subkey(&sub) {
                    if let Ok(install_dir) = k.get_value::<String, _>("InstallDir") {
                        let p = PathBuf::from(&install_dir);
                        if p.exists() {
                            let name = p
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| format!("Ubisoft Game {}", sub));
                            results.push(DiscoveredGameRoot {
                                name,
                                path: p,
                                launcher: LauncherType::Ubisoft,
                                app_id: Some(sub),
                                cover_url: None,
                            });
                        }
                    }
                }
            }
        }

        if let Ok(launcher_key) =
            RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher")
        {
            if let Ok(install_dir) = launcher_key.get_value::<String, _>("InstallDir") {
                ubi_game_roots.push(PathBuf::from(install_dir).join("games"));
            }
        }
    }

    for &d in drives {
        for suffix in &[
            "Program Files (x86)\\Ubisoft\\Ubisoft Game Launcher\\games",
            "Ubisoft Game Launcher\\games",
            "Games\\Ubisoft",
        ] {
            ubi_game_roots.push(PathBuf::from(format!("{}:\\{}", d, suffix)));
        }
    }

    for root in ubi_game_roots {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    results.push(DiscoveredGameRoot {
                        name,
                        path: entry.path(),
                        launcher: LauncherType::Ubisoft,
                        app_id: None,
                        cover_url: None,
                    });
                }
            }
        }
    }

    results
}

// =============================================================================
// 5. EA App / Origin Discovery
// =============================================================================

fn discover_ea_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        for reg_path in &[
            "SOFTWARE\\Electronic Arts\\EA Games",
            "SOFTWARE\\WOW6432Node\\Electronic Arts\\EA Games",
        ] {
            if let Ok(ea_key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_path) {
                for sub in ea_key.enum_keys().flatten() {
                    if let Ok(k) = ea_key.open_subkey(&sub) {
                        if let Ok(install_dir) = k.get_value::<String, _>("Install Dir") {
                            let p = PathBuf::from(install_dir);
                            if p.exists() {
                                results.push(DiscoveredGameRoot {
                                    name: sub,
                                    path: p,
                                    launcher: LauncherType::Ea,
                                    app_id: None,
                                    cover_url: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    for &d in drives {
        for suffix in &[
            "Program Files\\EA Games",
            "EA Games",
            "Origin Games",
            "Games\\EA Games",
        ] {
            let p = PathBuf::from(format!("{}:\\{}", d, suffix));
            if let Ok(entries) = fs::read_dir(p) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        results.push(DiscoveredGameRoot {
                            name,
                            path: entry.path(),
                            launcher: LauncherType::Ea,
                            app_id: None,
                            cover_url: None,
                        });
                    }
                }
            }
        }
    }
    results
}

// =============================================================================
// 6. Battle.net Discovery
// =============================================================================

fn discover_battlenet_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        for reg_path in &[
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ] {
            if let Ok(uninstall) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_path) {
                for sub in uninstall.enum_keys().flatten() {
                    if let Ok(k) = uninstall.open_subkey(&sub) {
                        let publisher = k.get_value::<String, _>("Publisher").unwrap_or_default();
                        if publisher.contains("Blizzard") {
                            let name = k.get_value::<String, _>("DisplayName").unwrap_or_default();
                            let loc =
                                k.get_value::<String, _>("InstallLocation").unwrap_or_default();
                            if !name.is_empty() && !loc.is_empty() && !name.contains("Battle.net") {
                                let p = PathBuf::from(loc);
                                if p.exists() {
                                    results.push(DiscoveredGameRoot {
                                        name,
                                        path: p,
                                        launcher: LauncherType::Battlenet,
                                        app_id: Some(sub),
                                        cover_url: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for &d in drives {
        for suffix in &["Battle.net", "Games\\Battle.net"] {
            let bnet_dir = PathBuf::from(format!("{}:\\{}", d, suffix));
            if let Ok(entries) = fs::read_dir(bnet_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.eq_ignore_ascii_case("Battle.net") {
                            results.push(DiscoveredGameRoot {
                                name,
                                path,
                                launcher: LauncherType::Battlenet,
                                app_id: None,
                                cover_url: None,
                            });
                        }
                    }
                }
            }
        }
    }

    results
}

// =============================================================================
// 7. Xbox / Microsoft Store Discovery (XboxGames across all drives)
// =============================================================================

fn discover_xbox_games(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    for &d in drives {
        for suffix in &["XboxGames", "Games\\XboxGames", "ModifiableWindowsApps"] {
            let xbox_dir = PathBuf::from(format!("{}:\\{}", d, suffix));
            let Ok(entries) = fs::read_dir(&xbox_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let content_dir = path.join("Content");
                let actual_path = if content_dir.exists() {
                    content_dir
                } else {
                    path
                };
                results.push(DiscoveredGameRoot {
                    name,
                    path: actual_path,
                    launcher: LauncherType::Xbox,
                    app_id: None,
                    cover_url: None,
                });
            }
        }
    }
    results
}

// =============================================================================
// 8. Star Citizen Discovery (LIVE, PTU, EPTU, TECH-PREVIEW, HOTFIX, 4.0_PREVIEW)
// =============================================================================

fn discover_rsi_star_citizen(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    let mut base_dirs: Vec<PathBuf> = Vec::new();
    let mut direct_channel_dirs: Vec<(String, PathBuf)> = Vec::new();

    if let Some(appdata) = dirs::config_dir() {
        let rsi_log = appdata.join("rsilauncher").join("logs").join("log.log");
        if let Ok(log_text) = fs::read_to_string(&rsi_log) {
            if let Ok(launch_re) = Regex::new(
                r"\[Launcher::launch\]\s+Launching Star Citizen\s+([A-Za-z0-9_-]+)\s+from\s+\(([^)]+)\)",
            ) {
                for cap in launch_re.captures_iter(&log_text) {
                    let chan = cap[1].trim().to_string();
                    let raw_path = cap[2].trim().replace("\\\\", "\\");
                    let p = PathBuf::from(&raw_path);
                    if let Some(parent) = p.parent() {
                        base_dirs.push(parent.to_path_buf());
                    }
                    direct_channel_dirs.push((chan, p));
                }
            }
            if let Ok(re) =
                Regex::new(r#"([A-Za-z]:\\(?:\\\\|[^\\\r\n:*?<>|"]+[\\]+)*StarCitizen)"#)
            {
                for cap in re.captures_iter(&log_text) {
                    let cleaned = cap[1].replace("\\\\", "\\");
                    base_dirs.push(PathBuf::from(cleaned));
                }
            }
        }
    }

    for &d in drives {
        for suffix in &[
            "Program Files\\Roberts Space Industries\\StarCitizen",
            "Roberts Space Industries\\StarCitizen",
            "Games\\Roberts Space Industries\\StarCitizen",
            "Games\\StarCitizen",
            "Spiele\\Roberts Space Industries\\StarCitizen",
            "Spiele\\StarCitizen",
            "StarCitizen",
        ] {
            base_dirs.push(PathBuf::from(format!("{}:\\{}", d, suffix)));
        }
    }

    let known_channels = [
        "LIVE",
        "PTU",
        "EPTU",
        "TECH-PREVIEW",
        "HOTFIX",
        "4.0_PREVIEW",
    ];
    let mut seen_chan_paths: HashSet<String> = HashSet::new();

    let mut add_channel = |channel: &str, chan_dir: PathBuf| {
        if !chan_dir.exists() || !chan_dir.is_dir() {
            return;
        }
        if !chan_dir.join("Bin64").exists() && !chan_dir.join("build_manifest.id").exists() {
            return;
        }
        let norm = chan_dir
            .to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_ascii_lowercase();
        if !seen_chan_paths.insert(norm) {
            return;
        }

        let chan_upper = channel.to_ascii_uppercase();
        let label = if chan_upper == "LIVE" {
            "Star Citizen".to_string()
        } else {
            format!("Star Citizen ({})", chan_upper)
        };

        results.push(DiscoveredGameRoot {
            name: label,
            path: chan_dir,
            launcher: LauncherType::Rsi,
            app_id: Some(format!("starcitizen_{}", channel.to_ascii_lowercase())),
            cover_url: Some("assets/icons/starcitizen_cover.png".to_string()),
        });
    };

    for (chan, dir) in direct_channel_dirs {
        add_channel(&chan, dir);
    }

    for base in base_dirs {
        if !base.exists() || !base.is_dir() {
            continue;
        }
        for channel in &known_channels {
            add_channel(channel, base.join(channel));
        }
        // Also discover any custom/future channel directory inside `StarCitizen/`
        if let Ok(entries) = fs::read_dir(&base) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    add_channel(&folder_name, p);
                }
            }
        }
    }

    results
}

// =============================================================================
// 9. Generic `<Drive>:\Games\*` & `<Drive>:\Spiele\*` Discovery
// =============================================================================

fn discover_generic_game_dirs(drives: &[char]) -> Vec<DiscoveredGameRoot> {
    let mut results = Vec::new();
    let container_names = [
        "steam",
        "steamlibrary",
        "epic games",
        "gog games",
        "gog galaxy",
        "ubisoft",
        "ubisoft game launcher",
        "ea games",
        "origin games",
        "battle.net",
        "xboxgames",
        "starcitizen",
        "roberts space industries",
    ];

    for &d in drives {
        for folder_name in &["Games", "Spiele", "PC Games"] {
            let root = PathBuf::from(format!("{}:\\{}", d, folder_name));
            let Ok(entries) = fs::read_dir(&root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                if container_names.contains(&lower.as_str())
                    || SKIP_DIRS.contains(&lower.as_str())
                    || is_redist_or_non_game(&name)
                {
                    continue;
                }
                results.push(DiscoveredGameRoot {
                    name,
                    path,
                    launcher: LauncherType::Custom,
                    app_id: None,
                    cover_url: None,
                });
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_machine_game_scan() {
        let games = scan_all_games(&[], &[], &[]);
        println!("Total discovered games: {}", games.len());
        let with_dlls: Vec<_> = games.iter().filter(|g| !g.dlls.is_empty()).collect();
        println!("Games with upscaler DLLs: {}", with_dlls.len());
        for g in &with_dlls {
            println!(
                "  - [{}] {} ({}) -> {} DLLs",
                g.launcher_name,
                g.name,
                g.path,
                g.dlls.len()
            );
        }
    }
}
