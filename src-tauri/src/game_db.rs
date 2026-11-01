use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::database::get_app_data_dir;
use crate::models::{Game, GameDbManifest, GameProfileEntry, UnsupportedTechNote};
use crate::scanner::normalize_game_name_for_match;

pub const PRIMARY_DB_URL: &str = "https://dlssdb.goover.dev/api/v1/games.json";
pub const CLOUDFLARE_PAGES_DB_URL: &str =
    "https://dlss-manager-onlinedb.pages.dev/api/v1/games.json";
pub const GITHUB_RAW_DB_URL: &str =
    "https://raw.githubusercontent.com/gOOvER/DLSS-Manager-OnlineDB/main/api/v1/games.json";

fn get_game_db_cache_path() -> PathBuf {
    get_app_data_dir().join("game_profiles_cache.json")
}

/// Checks if the sibling `DLSS-Manager-OnlineDB/api/v1/games.json` repository exists locally in the workspace.
fn find_local_sibling_online_db() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("../DLSS-Manager-OnlineDB/api/v1/games.json"),
        PathBuf::from("../../DLSS-Manager-OnlineDB/api/v1/games.json"),
        PathBuf::from(r"X:\Github Workspace\DLSS-Manager-OnlineDB\api\v1\games.json"),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }
    None
}

fn parse_game_db_json(raw: &str) -> Option<Vec<GameProfileEntry>> {
    if let Ok(manifest) = serde_json::from_str::<GameDbManifest>(raw) {
        Some(manifest.games)
    } else if let Ok(list) = serde_json::from_str::<Vec<GameProfileEntry>>(raw) {
        Some(list)
    } else {
        None
    }
}

/// Built-in seed database compiled into DLSS Manager (synced 1:1 with `DLSS-Manager-OnlineDB/api/v1/games.json`).
pub fn default_seed_game_db() -> Vec<GameProfileEntry> {
    vec![
        GameProfileEntry {
            id: "starcitizen".to_string(),
            title: "Star Citizen".to_string(),
            steam_app_ids: vec![
                "starcitizen_live".to_string(),
                "starcitizen_ptu".to_string(),
                "starcitizen_eptu".to_string(),
                "starcitizen_tech-preview".to_string(),
                "starcitizen_hotfix".to_string(),
            ],
            match_names: vec![
                "star citizen".to_string(),
                "star citizen (live)".to_string(),
                "star citizen (ptu)".to_string(),
                "star citizen (eptu)".to_string(),
                "star citizen (tech-preview)".to_string(),
                "star citizen (hotfix)".to_string(),
            ],
            match_executables: vec!["starcitizen.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec!["DLSS".to_string()],
            unsupported_techs: vec![
                UnsupportedTechNote {
                    tech: "FSR 3/4 & TSR".to_string(),
                    reason: "AMD FSR 2 and CIG TSR are compiled directly into StarCitizen.exe & Data.p4k shaders (no external FSR DLL in Bin64).".to_string(),
                },
                UnsupportedTechNote {
                    tech: "DLSS-G (Frame Gen)".to_string(),
                    reason: "DLSS Frame Generation (nvngx_dlssg.dll) is not natively supported by StarEngine yet.".to_string(),
                },
                UnsupportedTechNote {
                    tech: "DLSS-RR & XeSS".to_string(),
                    reason: "Ray Reconstruction (nvngx_dlssd.dll) and Intel XeSS (libxess.dll) are not natively integrated in Star Citizen.".to_string(),
                },
            ],
            warning_banner: Some(
                "EasyAntiCheat monitors Bin64: Official NVIDIA-signed nvngx_dlss.dll updates and NVIDIA Driver DRS Presets (Preset J/K/L) work safely. Do not inject unsigned third-party wrapper DLLs."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: None,
            recommended_settings: vec![
                "Upscaling: DLSS (Quality or Balanced at 1440p/4K)".to_string(),
                "NVIDIA DLSS Preset: Preset K (Transformer Gen 2 — sharper distant ship hulls & HUD MFD text)".to_string(),
                "Graphics Renderer: Vulkan".to_string(),
            ],
        },
        GameProfileEntry {
            id: "global-rescue".to_string(),
            title: "Global Rescue".to_string(),
            steam_app_ids: vec!["2873660".to_string()],
            match_names: vec!["global rescue".to_string(), "gr".to_string()],
            match_executables: vec!["gr.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec!["DStorage".to_string()],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: None,
            recommended_rr_preset: None,
            recommended_fg_preset: None,
            recommended_settings: vec![
                "DirectStorage: Active (dstorage.dll & dstoragecore.dll for fast map & tile streaming)".to_string(),
            ],
        },
        GameProfileEntry {
            id: "overwatch-2".to_string(),
            title: "Overwatch 2".to_string(),
            steam_app_ids: vec!["2357570".to_string()],
            match_names: vec![
                "overwatch 2".to_string(),
                "overwatch".to_string(),
                "overwatch®".to_string(),
            ],
            match_executables: vec!["overwatch.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![UnsupportedTechNote {
                tech: "DLSS-G (Frame Gen)".to_string(),
                reason: "Overwatch 2 supports DLSS Super Resolution, AMD FSR 3 & Intel XeSS, but does not ship with NVIDIA DLSS Frame Generation (nvngx_dlssg.dll).".to_string(),
            }],
            warning_banner: Some(
                "Blizzard Defense Matrix: Official NVIDIA-signed nvngx_dlss.dll updates and NVIDIA Driver DRS Presets are supported. Battle.net patches may reset nvngx_dlss.dll after seasonal updates."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: None,
            recommended_settings: vec![
                "High Quality Upsampling: NVIDIA DLSS Super Resolution (Quality)".to_string(),
                "NVIDIA Reflex: Enabled + Boost".to_string(),
                "Dynamic Render Scale: Off (for consistent crosshair & outline sharpness)".to_string(),
            ],
        },
        GameProfileEntry {
            id: "diablo-iv".to_string(),
            title: "Diablo IV".to_string(),
            steam_app_ids: vec!["2344520".to_string()],
            match_names: vec!["diablo iv".to_string(), "diablo 4".to_string()],
            match_executables: vec!["diablo iv.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "DStorage".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Resolution Scaling: NVIDIA DLSS Super Resolution (Quality / DLAA)".to_string(),
                "DLSS Frame Generation: Enabled (RTX 40/50 Series)".to_string(),
                "NVIDIA Reflex Low Latency: Enabled + Boost".to_string(),
            ],
        },
        GameProfileEntry {
            id: "cyberpunk-2077".to_string(),
            title: "Cyberpunk 2077".to_string(),
            steam_app_ids: vec!["1091500".to_string()],
            match_names: vec!["cyberpunk 2077".to_string()],
            match_executables: vec!["cyberpunk2077.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "DLSS Transformer Model: Enabled (Preset K / Preset M)".to_string(),
                "DLSS Ray Reconstruction: Enabled (Preset F) when Path Tracing / Ray Tracing is active".to_string(),
                "DLSS Frame Generation: Enabled (Multi Frame Gen on RTX 50 Series)".to_string(),
            ],
        },
        GameProfileEntry {
            id: "everspace-2".to_string(),
            title: "EVERSPACE 2".to_string(),
            steam_app_ids: vec!["1128920".to_string()],
            match_names: vec!["everspace 2".to_string()],
            match_executables: vec!["es2-win64-shipping.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Upscaler: NVIDIA DLSS Quality + Frame Generation".to_string(),
                "DirectX 12 required for DLSS Frame Generation & Lumen GI".to_string(),
            ],
        },
        GameProfileEntry {
            id: "longvinter".to_string(),
            title: "Longvinter".to_string(),
            steam_app_ids: vec!["1635450".to_string()],
            match_names: vec!["longvinter".to_string()],
            match_executables: vec!["longvinter-win64-shipping.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: Some(
                "EasyAntiCheat protected: Only official NVIDIA-signed DLSS & Streamline binaries should be used."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "NVIDIA DLSS: Quality".to_string(),
                "DLSS Preset: Preset K".to_string(),
            ],
        },
        GameProfileEntry {
            id: "forza-horizon-5".to_string(),
            title: "Forza Horizon 5".to_string(),
            steam_app_ids: vec!["1551360".to_string()],
            match_names: vec!["forza horizon 5".to_string(), "forzahorizon5".to_string()],
            match_executables: vec!["forzahorizon5.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "Streamline".to_string(),
                "DStorage".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "NVIDIA DLSS: Quality or DLAA (eliminates power-line & foliage shimmering with Preset K)".to_string(),
                "DLSS Frame Generation: Enabled".to_string(),
            ],
        },
        GameProfileEntry {
            id: "msfs-2024".to_string(),
            title: "Microsoft Flight Simulator 2024".to_string(),
            steam_app_ids: vec!["2537590".to_string()],
            match_names: vec![
                "microsoft flight simulator 2024".to_string(),
                "msfs2024".to_string(),
            ],
            match_executables: vec!["flightsimulator2024.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Anti-Aliasing: NVIDIA DLSS Super Resolution (Quality or DLAA)".to_string(),
                "DLSS Preset K significantly reduces glass cockpit speed/altitude tape ghosting".to_string(),
                "Frame Generation: NVIDIA DLSS G".to_string(),
            ],
        },
        GameProfileEntry {
            id: "call-of-duty-hq".to_string(),
            title: "Call of Duty / Warzone".to_string(),
            steam_app_ids: vec!["1938090".to_string()],
            match_names: vec![
                "call of duty".to_string(),
                "call of duty hq".to_string(),
                "warzone".to_string(),
                "black ops 6".to_string(),
            ],
            match_executables: vec!["cod.exe".to_string()],
            anticheat_status: "driver_override_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![UnsupportedTechNote {
                tech: "Manual DLL Swap".to_string(),
                reason: "Ricochet Anti-Cheat verifies game file hashes on launch. Use NVIDIA Driver DRS Presets (Preset K) instead of replacing files on disk.".to_string(),
            }],
            warning_banner: Some(
                "Ricochet Anti-Cheat Warning: Prefer NVIDIA Driver-Level DRS Presets (🎛️ DLSS Preset Settings) instead of modifying DLL files in the installation directory."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Use NVIDIA Driver DRS Override (Preset K) via DLSS Preset Settings".to_string(),
                "Upscaling: NVIDIA DLSS Quality or FidelityFX CAS".to_string(),
            ],
        },
        GameProfileEntry {
            id: "alan-wake-2".to_string(),
            title: "Alan Wake 2".to_string(),
            steam_app_ids: vec!["alanwake2".to_string()],
            match_names: vec!["alan wake 2".to_string(), "alanwake2".to_string()],
            match_executables: vec!["alanwake2.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "DStorage".to_string(),
            ],
            unsupported_techs: vec![UnsupportedTechNote {
                tech: "External FSR / XeSS DLL".to_string(),
                reason: "Northlight Engine integrates FSR 2 natively without external FidelityFX or XeSS DLLs.".to_string(),
            }],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "DLSS Ray Reconstruction: Enabled with Preset F (Transformer Gen 2 reduces wire & fence ghosting)".to_string(),
                "Resolution Upscaling: DLSS Quality or Balanced".to_string(),
                "DLSS Frame Generation: Enabled".to_string(),
            ],
        },
        GameProfileEntry {
            id: "black-myth-wukong".to_string(),
            title: "Black Myth: Wukong".to_string(),
            steam_app_ids: vec!["2358720".to_string()],
            match_names: vec![
                "black myth wukong".to_string(),
                "black myth: wukong".to_string(),
                "b1".to_string(),
            ],
            match_executables: vec!["b1-win64-shipping.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: Some("Preset F".to_string()),
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Super Resolution: DLSS (67% Quality or 75% DLAA) with Preset K".to_string(),
                "Full Ray Tracing: Enable DLSS Ray Reconstruction (Preset F) to eliminate water & foliage shimmer".to_string(),
                "Frame Generation: Enabled".to_string(),
            ],
        },
        GameProfileEntry {
            id: "stalker-2".to_string(),
            title: "S.T.A.L.K.E.R. 2: Heart of Chornobyl".to_string(),
            steam_app_ids: vec!["1643320".to_string()],
            match_names: vec![
                "stalker 2".to_string(),
                "s.t.a.l.k.e.r. 2".to_string(),
                "s.t.a.l.k.e.r. 2 heart of chornobyl".to_string(),
                "stalker2".to_string(),
            ],
            match_executables: vec!["stalker2-win64-shipping.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Upscaling Quality: DLSS Quality with Preset K (vastly improves foliage stability in the Zone)".to_string(),
                "DLSS Frame Generation: Enabled".to_string(),
            ],
        },
        GameProfileEntry {
            id: "marvel-rivals".to_string(),
            title: "Marvel Rivals".to_string(),
            steam_app_ids: vec!["2767030".to_string()],
            match_names: vec!["marvel rivals".to_string(), "marvelrivals".to_string()],
            match_executables: vec!["marvel-win64-shipping.exe".to_string()],
            anticheat_status: "signed_only".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "DLSS-RR".to_string(),
                "Streamline".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: Some(
                "Anti-Cheat Protected: Official NVIDIA-signed DLSS & Streamline updates and Driver DRS Presets (Preset K) are safe."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Anti-Aliasing and Super Resolution: NVIDIA DLSS Quality (Preset K)".to_string(),
                "NVIDIA Reflex Low Latency: Enabled + Boost".to_string(),
            ],
        },
        GameProfileEntry {
            id: "monster-hunter-wilds".to_string(),
            title: "Monster Hunter Wilds".to_string(),
            steam_app_ids: vec!["2246340".to_string()],
            match_names: vec![
                "monster hunter wilds".to_string(),
                "monsterhunterwilds".to_string(),
            ],
            match_executables: vec!["monsterhunterwilds.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec![
                "DLSS".to_string(),
                "DLSS-G".to_string(),
                "Streamline".to_string(),
                "DStorage".to_string(),
                "FSR 3/4".to_string(),
                "XeSS".to_string(),
            ],
            unsupported_techs: vec![],
            warning_banner: None,
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: Some("Latest".to_string()),
            recommended_settings: vec![
                "Upscaling: NVIDIA DLSS Quality or Balanced with Preset K (eliminates RE Engine hair/fur dithering)".to_string(),
                "Frame Generation: Enabled".to_string(),
                "DirectStorage: Keep updated to latest 1.2+ for faster texture streaming".to_string(),
            ],
        },
        GameProfileEntry {
            id: "red-dead-redemption-2".to_string(),
            title: "Red Dead Redemption 2".to_string(),
            steam_app_ids: vec!["1174180".to_string()],
            match_names: vec!["red dead redemption 2".to_string(), "rdr2".to_string()],
            match_executables: vec!["rdr2.exe".to_string()],
            anticheat_status: "safe".to_string(),
            supported_techs: vec!["DLSS".to_string(), "FSR 2".to_string()],
            unsupported_techs: vec![UnsupportedTechNote {
                tech: "DLSS-G (Frame Gen)".to_string(),
                reason: "RAGE Engine in Red Dead Redemption 2 only ships with DLSS 2 Super Resolution and FSR 2 (no native Frame Generation).".to_string(),
            }],
            warning_banner: Some(
                "Rockstar Games Launcher may revert nvngx_dlss.dll if 'Verify Integrity' is run. Avoid Red Dead Online when using modified files; Driver DRS Preset K works globally."
                    .to_string(),
            ),
            recommended_sr_preset: Some("Preset K".to_string()),
            recommended_rr_preset: None,
            recommended_fg_preset: None,
            recommended_settings: vec![
                "NVIDIA DLSS: Quality with Preset K (completely fixes RDR2's infamous horse-mane & tree-leaf sharpening artifacts)".to_string(),
                "DLSS Sharpening: 0 (Preset K handles edge clarity natively)".to_string(),
            ],
        },
        GameProfileEntry {
            id: "helldivers-2".to_string(),
            title: "HELLDIVERS 2".to_string(),
            steam_app_ids: vec!["553850".to_string()],
            match_names: vec!["helldivers 2".to_string()],
            match_executables: vec!["helldivers2.exe".to_string()],
            anticheat_status: "blocked".to_string(),
            supported_techs: vec![],
            unsupported_techs: vec![UnsupportedTechNote {
                tech: "DLSS / FSR / XeSS DLLs".to_string(),
                reason: "HELLDIVERS 2 uses its own built-in Stingray temporal upscaler and nProtect GameGuard blocks external DLL injection.".to_string(),
            }],
            warning_banner: Some(
                "nProtect GameGuard blocks external DLL modifications in bin/. Automatic DLL updates are disabled for safety."
                    .to_string(),
            ),
            recommended_sr_preset: None,
            recommended_rr_preset: None,
            recommended_fg_preset: None,
            recommended_settings: vec![
                "Render Scale: Native or Ultra Quality (built-in upscaler)".to_string(),
            ],
        },
    ]
}

/// Loads cached game database from `%LOCALAPPDATA%\Recol\DLSS-Updater\game_profiles_cache.json`
/// and any local sibling `DLSS-Manager-OnlineDB/api/v1/games.json` repository,
/// merged with the built-in seed profiles.
pub fn load_cached_or_seed_game_db() -> Vec<GameProfileEntry> {
    let mut map: HashMap<String, GameProfileEntry> = HashMap::new();
    for entry in default_seed_game_db() {
        map.insert(entry.id.clone(), entry);
    }

    let cache_path = get_game_db_cache_path();
    if let Ok(content) = fs::read_to_string(&cache_path) {
        if let Some(entries) = parse_game_db_json(&content) {
            for entry in entries {
                map.insert(entry.id.clone(), entry);
            }
        }
    }

    // If the sibling `DLSS-Manager-OnlineDB` workspace exists locally, merge its latest `api/v1/games.json`
    if let Some(sibling_path) = find_local_sibling_online_db() {
        if let Ok(content) = fs::read_to_string(&sibling_path) {
            if let Some(entries) = parse_game_db_json(&content) {
                for entry in entries {
                    map.insert(entry.id.clone(), entry);
                }
            }
        }
    }

    let mut out: Vec<GameProfileEntry> = map.into_values().collect();
    out.sort_by(|a, b| a.title.to_ascii_lowercase().cmp(&b.title.to_ascii_lowercase()));
    out
}

/// Fetches the latest game compatibility & settings database from Cloudflare Pages (`url`),
/// authenticating with the user's personal `api_key` (`X-API-Key` / `Bearer`),
/// caches it locally, and returns the merged profile list.
pub async fn fetch_remote_game_db(url: &str, api_key: &str) -> Result<Vec<GameProfileEntry>, String> {
    let trimmed = url.trim();
    let trimmed_key = api_key.trim();
    let mut candidate_urls: Vec<String> = Vec::new();
    if !trimmed.is_empty() {
        candidate_urls.push(trimmed.to_string());
    }
    if !candidate_urls.iter().any(|u| u == PRIMARY_DB_URL) {
        candidate_urls.push(PRIMARY_DB_URL.to_string());
    }
    if !candidate_urls.iter().any(|u| u == CLOUDFLARE_PAGES_DB_URL) {
        candidate_urls.push(CLOUDFLARE_PAGES_DB_URL.to_string());
    }
    if !candidate_urls.iter().any(|u| u == GITHUB_RAW_DB_URL) {
        candidate_urls.push(GITHUB_RAW_DB_URL.to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let mut fetched_entries: Option<Vec<GameProfileEntry>> = None;
    let mut last_err = String::new();

    for cand in &candidate_urls {
        // Support local file path if user configured a file path
        let as_path = Path::new(cand);
        if as_path.exists() {
            if let Ok(body) = fs::read_to_string(as_path) {
                if let Some(entries) = parse_game_db_json(&body) {
                    let _ = fs::write(get_game_db_cache_path(), &body);
                    fetched_entries = Some(entries);
                    break;
                }
            }
        }

        let mut req = client.get(cand);
        if !trimmed_key.is_empty() {
            req = req
                .header("X-API-Key", trimmed_key)
                .header("Authorization", format!("Bearer {}", trimmed_key));
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.text().await {
                    if let Some(entries) = parse_game_db_json(&body) {
                        let _ = fs::write(get_game_db_cache_path(), &body);
                        fetched_entries = Some(entries);
                        break;
                    } else {
                        last_err = format!("Invalid JSON schema from {}", cand);
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() == 401 || status.as_u16() == 403 {
                    last_err = format!(
                        "HTTP {} from {} — Please enter a valid DLSS-DB API Key (create yours at https://dlssdb.goover.dev)",
                        status, cand
                    );
                } else {
                    last_err = format!("HTTP {} from {}", status, cand);
                }
            }
            Err(e) => {
                last_err = format!("{} ({})", e, cand);
            }
        }
    }

    // Also check local sibling `DLSS-Manager-OnlineDB/api/v1/games.json` for parallel local development
    if fetched_entries.is_none() {
        if let Some(sibling) = find_local_sibling_online_db() {
            if let Ok(body) = fs::read_to_string(&sibling) {
                if let Some(entries) = parse_game_db_json(&body) {
                    let _ = fs::write(get_game_db_cache_path(), &body);
                    fetched_entries = Some(entries);
                }
            }
        }
    }

    let Some(parsed_entries) = fetched_entries else {
        return Err(if last_err.is_empty() {
            "Could not reach Online Game DB".to_string()
        } else {
            last_err
        });
    };

    let mut map: HashMap<String, GameProfileEntry> = HashMap::new();
    for entry in default_seed_game_db() {
        map.insert(entry.id.clone(), entry);
    }
    for entry in parsed_entries {
        map.insert(entry.id.clone(), entry);
    }
    if let Some(sibling) = find_local_sibling_online_db() {
        if let Ok(body) = fs::read_to_string(&sibling) {
            if let Some(entries) = parse_game_db_json(&body) {
                for entry in entries {
                    map.insert(entry.id.clone(), entry);
                }
            }
        }
    }

    let mut out: Vec<GameProfileEntry> = map.into_values().collect();
    out.sort_by(|a, b| a.title.to_ascii_lowercase().cmp(&b.title.to_ascii_lowercase()));
    Ok(out)
}

/// Matches a scanned `Game` against the loaded `GameProfileEntry` database by Steam AppID,
/// normalized title, or path/executable.
pub fn match_game_profile(
    game: &Game,
    profiles: &[GameProfileEntry],
) -> Option<GameProfileEntry> {
    let game_norm = normalize_game_name_for_match(&game.name);
    let orig_norm = game
        .original_name
        .as_deref()
        .map(normalize_game_name_for_match)
        .unwrap_or_default();
    let path_lower = game.path.to_ascii_lowercase();

    for p in profiles {
        // 1. Match by App ID
        if let Some(ref aid) = game.app_id {
            if !aid.is_empty()
                && p.steam_app_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(aid))
            {
                return Some(p.clone());
            }
        }

        // 2. Match by normalized title
        if !game_norm.is_empty() {
            if normalize_game_name_for_match(&p.title) == game_norm {
                return Some(p.clone());
            }
            if p.match_names
                .iter()
                .any(|m| normalize_game_name_for_match(m) == game_norm)
            {
                return Some(p.clone());
            }
        }
        if !orig_norm.is_empty()
            && p.match_names
                .iter()
                .any(|m| normalize_game_name_for_match(m) == orig_norm)
        {
            return Some(p.clone());
        }

        // 3. Match by Star Citizen launcher or executable hint
        if p.id == "starcitizen"
            && (game.launcher == crate::models::LauncherType::Rsi
                || path_lower.contains("starcitizen"))
        {
            return Some(p.clone());
        }
    }

    None
}

/// Attaches matching `db_profile` entries to each game in `games`.
/// If a game's `db_profile` has `anticheat_status == "blocked"`, disables automatic DLL update flags.
pub fn annotate_games_with_db_profiles(games: &mut [Game], profiles: &[GameProfileEntry]) {
    for g in games.iter_mut() {
        if let Some(profile) = match_game_profile(g, profiles) {
            if profile.anticheat_status.eq_ignore_ascii_case("blocked") {
                for d in g.dlls.iter_mut() {
                    d.update_available = false;
                }
            }
            g.db_profile = Some(profile);
        } else {
            g.db_profile = None;
        }
    }
}

/// Optional user-supplied metadata when manually reporting a game's compatibility status.
#[derive(Debug, Clone, Default)]
pub struct UserGameReportInput {
    pub works_status: Option<String>,
    pub recommended_settings: Option<String>,
    pub recommended_sr_preset: Option<String>,
    pub notes: Option<String>,
}

/// Reports scanned or user-selected games to `https://dlssdb.goover.dev/api/v1/games/report`
/// using the user's personal `api_key`.
pub async fn report_games_to_remote_db(
    db_url: &str,
    api_key: &str,
    games: &[Game],
    source: &str,
    user_input: Option<UserGameReportInput>,
) -> Result<String, String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err(
            "Kein persönlicher DLSS-DB API Key hinterlegt. Bitte unter Settings -> Game Compatibility & Settings Database eintragen."
                .to_string(),
        );
    }

    let report_url = if db_url.contains("/api/v1/games.json") {
        db_url.replace("/api/v1/games.json", "/api/v1/games/report")
    } else {
        "https://dlssdb.goover.dev/api/v1/games/report".to_string()
    };

    let input = user_input.unwrap_or_default();
    let works_status = input
        .works_status
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| s == "works" || s == "broken")
        .unwrap_or_else(|| "works".to_string());

    let mut payload_games = Vec::new();
    for g in games {
        if g.dlls.is_empty() {
            continue;
        }
        let mut techs: Vec<String> = Vec::new();
        for d in &g.dlls {
            if !techs.iter().any(|t| t.eq_ignore_ascii_case(&d.type_label)) {
                techs.push(d.type_label.clone());
            }
        }
        let exe_name = g
            .custom_exe_path
            .as_deref()
            .and_then(|p| Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .or_else(|| {
                crate::scanner::find_primary_game_executable(Path::new(&g.path))
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            })
            .unwrap_or_default();

        let combined_notes = match (&input.notes, &g.anti_cheat) {
            (Some(n), Some(ac)) if !n.trim().is_empty() => {
                Some(format!("{} (Anti-Cheat: {})", n.trim(), ac))
            }
            (Some(n), None) if !n.trim().is_empty() => Some(n.trim().to_string()),
            (_, Some(ac)) => Some(format!("Anti-Cheat: {}", ac)),
            _ => None,
        };

        let sr_preset = input
            .recommended_sr_preset
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| g.dlss_sr_preset.clone());

        payload_games.push(serde_json::json!({
            "title": g.name,
            "app_id": g.app_id,
            "executable": exe_name,
            "supported_techs": techs,
            "launcher": g.launcher_name,
            "works_status": works_status,
            "recommended_settings": input.recommended_settings,
            "recommended_sr_preset": sr_preset,
            "anticheat_status": if works_status == "broken" {
                "blocked"
            } else if g.anti_cheat.is_some() {
                "signed_only"
            } else {
                "safe"
            },
            "notes": combined_notes,
        }));
    }

    if payload_games.is_empty() {
        return Err("Keine Spiele mit erkannten Upscaler-DLLs zum Melden gefunden.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(&report_url)
        .header("X-API-Key", trimmed_key)
        .header("Authorization", format!("Bearer {}", trimmed_key))
        .json(&serde_json::json!({
            "source": source,
            "games": payload_games,
        }))
        .send()
        .await
        .map_err(|e| format!("Verbindungsfehler zu {}: {}", report_url, e))?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(err_msg) = val.get("error").and_then(|v| v.as_str()) {
                return Err(err_msg.to_string());
            }
        }
        return Err(format!("HTTP {} von {}", status, report_url));
    }

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            return Ok(msg.to_string());
        }
    }

    Ok("Spiele erfolgreich an dlssdb.goover.dev gemeldet!".to_string())
}
