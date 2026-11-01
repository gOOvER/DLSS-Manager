pub mod database;
pub mod game_db;
pub mod gpu_nvapi;
pub mod models;
pub mod pe_version;
pub mod repository;
pub mod scanner;

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use chrono::Local;
use serde::Serialize;
use tauri::State;

use database::{
    get_app_data_dir, get_backups_dir, get_cache_dir, load_config, save_config, Database,
};
use game_db::{
    annotate_games_with_db_profiles, fetch_remote_game_db, load_cached_or_seed_game_db,
    report_games_to_remote_db,
};
use gpu_nvapi::{
    apply_nvapi_dlss_presets, detect_gpu_info, get_dlss_indicator_state, set_dlss_indicator_state,
};
use models::{
    AppConfig, AppUpdateInfo, BackupEntry, DashboardStats, DllType, Game, GameOverride,
    GameProfileEntry, GpuInfo, LogEntry, RemoteDllVersion, SteamSearchResult,
    SteamValidationResult,
};
use repository::{
    annotate_games_with_config, ensure_dll_cached, fetch_or_load_manifest, list_remote_dlls,
    load_cached_manifest_sync, restore_backup_entry, update_game_dll, ManifestMap,
};
use scanner::{
    detect_steam_id_from_vdf, find_primary_game_executable, normalize_game_name_for_match,
    scan_all_games, scan_game_directory,
};

pub struct AppState {
    pub db: Mutex<Database>,
    pub manifest: Mutex<ManifestMap>,
    pub config: Mutex<AppConfig>,
    pub gpu: Mutex<GpuInfo>,
    pub logs: Mutex<Vec<LogEntry>>,
    pub game_db: Mutex<Vec<GameProfileEntry>>,
}

#[derive(Serialize)]
pub struct InitialPayload {
    pub version: String,
    pub games: Vec<Game>,
    pub backups: Vec<BackupEntry>,
    pub catalog: Vec<RemoteDllVersion>,
    pub gpu: GpuInfo,
    pub config: AppConfig,
    pub stats: DashboardStats,
    pub logs: Vec<LogEntry>,
}

fn push_log(state: &AppState, level: &str, message: impl Into<String>) {
    if let Ok(mut logs) = state.logs.lock() {
        logs.push(LogEntry {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.into(),
        });
        if logs.len() > 250 {
            let excess = logs.len() - 250;
            logs.drain(0..excess);
        }
    }
}

fn get_logs_snapshot(state: &AppState) -> Vec<LogEntry> {
    state.logs.lock().map(|l| l.clone()).unwrap_or_default()
}

fn compute_stats(
    games: &[Game],
    backups: &[BackupEntry],
    catalog: &[RemoteDllVersion],
    game_db_count: usize,
) -> DashboardStats {
    let total_games = games.len();
    let games_with_dlls = games.iter().filter(|g| !g.dlls.is_empty()).count();
    let ignored_count = games.iter().filter(|g| g.ignored).count();
    let updates_available = games
        .iter()
        .filter(|g| !g.ignored && !g.blacklisted)
        .filter(|g| g.dlls.iter().any(|d| d.update_available && d.gpu_supported))
        .count();
    let total_backups = backups.len();
    let cached_dlls = catalog.iter().filter(|c| c.is_cached).count();

    let dir_size_mb = |dir: std::path::PathBuf| -> f64 {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum::<u64>() as f64
            / (1024.0 * 1024.0)
    };

    let cache_size_mb = dir_size_mb(get_cache_dir());
    let backup_size_mb = dir_size_mb(get_backups_dir());
    let last_scan_time = games.first().map(|g| g.last_scanned.clone());

    DashboardStats {
        total_games,
        games_with_dlls,
        updates_available,
        ignored_count,
        total_backups,
        cached_dlls,
        cache_size_mb,
        backup_size_mb,
        last_scan_time,
        app_data_dir: get_app_data_dir().to_string_lossy().to_string(),
        game_db_count,
    }
}

fn build_payload(
    state: &AppState,
    mut games: Vec<Game>,
    backups: Vec<BackupEntry>,
    manifest: &ManifestMap,
    gpu: GpuInfo,
    config: AppConfig,
) -> Result<InitialPayload, String> {
    let game_db_profiles = state
        .game_db
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| load_cached_or_seed_game_db());

    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.apply_overrides_and_config(&mut games, &config);
        annotate_games_with_config(&mut games, manifest, &gpu, &config);
        annotate_games_with_db_profiles(&mut games, &game_db_profiles);
        for g in games.iter_mut() {
            g.has_backups = backups.iter().any(|b| b.game_id == g.id);
        }
        let _ = db.save_games(&games);
    }

    let catalog = list_remote_dlls(manifest, &gpu);
    let stats = compute_stats(&games, &backups, &catalog, game_db_profiles.len());
    let logs = get_logs_snapshot(state);

    Ok(InitialPayload {
        version: env!("CARGO_PKG_VERSION").to_string(),
        games,
        backups,
        catalog,
        gpu,
        config,
        stats,
        logs,
    })
}

#[tauri::command]
async fn get_initial_state(state: State<'_, AppState>) -> Result<InitialPayload, String> {
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_else(|_| load_cached_manifest_sync());

    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let mut config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    config.dlss_indicator_enabled = get_dlss_indicator_state();

    let (games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let g = db.load_games().unwrap_or_default();
        let b = db.load_backups().unwrap_or_default();
        (g, b)
    };

    push_log(
        &state,
        "INFO",
        format!(
            "Initialized DLSS Manager v{} — GPU: {} ({}) — {} games loaded",
            env!("CARGO_PKG_VERSION"),
            gpu.name,
            gpu.architecture,
            games.len()
        ),
    );

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn scan_games(state: State<'_, AppState>) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    push_log(&state, "INFO", "Starting multi-launcher game library scan...");

    let custom_folders = config.custom_folders.clone();
    let ignored = config.ignored_games.clone();
    let blacklisted = config.blacklisted_games.clone();

    let games = tokio::task::spawn_blocking(move || {
        scan_all_games(&custom_folders, &ignored, &blacklisted)
    })
    .await
    .map_err(|e| e.to_string())?;

    let backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.load_backups().unwrap_or_default()
    };

    push_log(
        &state,
        "INFO",
        format!("Library scan complete: discovered {} games with upscaler DLLs", games.len()),
    );

    if config.auto_report_games_to_db && !config.game_db_api_key.trim().is_empty() {
        if let Ok(msg) = report_games_to_remote_db(
            &config.game_db_url,
            &config.game_db_api_key,
            &games,
            "auto_scan",
        )
        .await
        {
            push_log(&state, "INFO", format!("Auto-reported games to Online-DB: {}", msg));
        }
    }

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn refresh_dll_manifest(state: State<'_, AppState>) -> Result<Vec<RemoteDllVersion>, String> {
    let manifest = fetch_or_load_manifest()
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(mut m) = state.manifest.lock() {
        *m = manifest.clone();
    }
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());
    push_log(&state, "INFO", "Refreshed remote DLL repository manifest.");
    Ok(list_remote_dlls(&manifest, &gpu))
}

#[tauri::command]
async fn download_dll_to_cache(
    dll_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<RemoteDllVersion>, String> {
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());
    if let Some(dt) = DllType::from_filename(&dll_name) {
        if !gpu.supports_dll_type(dt) {
            return Err(format!(
                "{} is not supported by your {} GPU ({})",
                dt.display_name(),
                gpu.vendor,
                gpu.name
            ));
        }
    }
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    ensure_dll_cached(&dll_name, &manifest)
        .await
        .map_err(|e| e.to_string())?;
    push_log(&state, "INFO", format!("Downloaded and verified {} in local cache.", dll_name));
    Ok(list_remote_dlls(&manifest, &gpu))
}

#[tauri::command]
async fn update_single_game_dll(
    game_id: String,
    dll_path: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    if let Some(fname) = Path::new(&dll_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
    {
        if let Some(dt) = DllType::from_filename(&fname) {
            if !gpu.supports_dll_type(dt) {
                return Err(format!(
                    "Update blocked: {} is not supported by your {} GPU ({})",
                    dt.display_name(),
                    gpu.vendor,
                    gpu.name
                ));
            }
        }
    }

    let mut games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g = db.load_games().map_err(|e| e.to_string())?;
        db.apply_overrides_and_config(&mut g, &config);
        g
    };

    let game = games
        .iter()
        .find(|g| g.id == game_id)
        .cloned()
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let (new_ver, backup_vec) = update_game_dll(&game, &dll_path, &manifest, config.auto_backup)
        .await
        .map_err(|e| {
            push_log(&state, "ERROR", format!("Failed updating {} in {}: {}", dll_path, game.name, e));
            e.to_string()
        })?;

    push_log(
        &state,
        "INFO",
        format!("Updated {} -> v{} in {}", dll_path, new_ver, game.name),
    );

    if let Some(g_mut) = games.iter_mut().find(|g| g.id == game_id) {
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&g_mut.path));
        g_mut.dlls = rescanned_dlls;
    }

    let backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for b in backup_vec {
            let _ = db.save_backup(&b);
        }
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn update_all_for_game(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let mut games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g = db.load_games().map_err(|e| e.to_string())?;
        db.apply_overrides_and_config(&mut g, &config);
        g
    };
    annotate_games_with_config(&mut games, &manifest, &gpu, &config);

    let game = games
        .iter()
        .find(|g| g.id == game_id)
        .cloned()
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    let mut new_backups = Vec::new();
    let mut last_error: Option<String> = None;
    let mut updated_count = 0usize;

    for dll in &game.dlls {
        if dll.update_available && dll.gpu_supported && gpu.supports_dll_type(dll.dll_type) {
            match update_game_dll(&game, &dll.path, &manifest, config.auto_backup).await {
                Ok((_ver, b_vec)) => {
                    updated_count += 1;
                    new_backups.extend(b_vec);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }
    }

    if updated_count == 0 {
        if let Some(err) = last_error {
            push_log(&state, "ERROR", format!("Update failed for {}: {}", game.name, err));
            return Err(err);
        }
    } else {
        push_log(
            &state,
            "INFO",
            format!("Updated {} DLL(s) for {}", updated_count, game.name),
        );
    }

    if let Some(g_mut) = games.iter_mut().find(|g| g.id == game_id) {
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&g_mut.path));
        g_mut.dlls = rescanned_dlls;
    }

    let backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for b in new_backups {
            let _ = db.save_backup(&b);
        }
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn bulk_update_all_games(state: State<'_, AppState>) -> Result<InitialPayload, String> {
    bulk_update_selected_games(Vec::new(), None, state).await
}

#[tauri::command]
async fn bulk_update_selected_games(
    game_ids: Vec<String>,
    allowed_techs: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let mut games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g = db.load_games().map_err(|e| e.to_string())?;
        db.apply_overrides_and_config(&mut g, &config);
        g
    };
    annotate_games_with_config(&mut games, &manifest, &gpu, &config);

    let mut new_backups = Vec::new();
    let mut updated_games_count = 0usize;
    let mut updated_dlls_count = 0usize;

    for game in games.iter_mut() {
        if !game_ids.is_empty() {
            if !game_ids.iter().any(|id| id == &game.id) {
                continue;
            }
        } else if game.ignored || game.blacklisted {
            continue;
        }
        if config.skip_anti_cheat_games && game.anti_cheat.is_some() {
            continue;
        }

        let outdated: Vec<String> = game
            .dlls
            .iter()
            .filter(|d| {
                let tech_allowed = allowed_techs
                    .as_ref()
                    .map(|list| list.iter().any(|t| t.eq_ignore_ascii_case(&d.type_label)))
                    .unwrap_or(d.technology_enabled);
                d.update_available
                    && d.gpu_supported
                    && tech_allowed
                    && gpu.supports_dll_type(d.dll_type)
            })
            .map(|d| d.path.clone())
            .collect();

        if outdated.is_empty() {
            continue;
        }

        let g_clone = game.clone();
        let mut any_ok = false;
        for dll_path in outdated {
            if let Ok((_ver, b_vec)) =
                update_game_dll(&g_clone, &dll_path, &manifest, config.auto_backup).await
            {
                any_ok = true;
                updated_dlls_count += 1;
                new_backups.extend(b_vec);
            }
        }
        if any_ok {
            updated_games_count += 1;
        }
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&game.path));
        game.dlls = rescanned_dlls;
    }

    push_log(
        &state,
        "INFO",
        format!(
            "Batch update complete: updated {} DLL(s) across {} game(s).",
            updated_dlls_count, updated_games_count
        ),
    );

    let backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for b in new_backups {
            let _ = db.save_backup(&b);
        }
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn restore_backup(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (mut games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    let entry = backups
        .iter()
        .find(|b| b.id == backup_id)
        .cloned()
        .ok_or_else(|| format!("Backup not found: {}", backup_id))?;

    restore_backup_entry(&entry).map_err(|e| e.to_string())?;
    push_log(
        &state,
        "INFO",
        format!("Restored backup {} for {}", entry.filename, entry.game_name),
    );

    if let Some(g_mut) = games.iter_mut().find(|g| g.id == entry.game_id) {
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&g_mut.path));
        g_mut.dlls = rescanned_dlls;
    }

    let updated_backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let _ = db.delete_backup(&backup_id);
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, updated_backups, &manifest, gpu, config)
}

#[tauri::command]
async fn restore_all_for_game(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (mut games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    let game_backups: Vec<BackupEntry> = backups
        .iter()
        .filter(|b| b.game_id == game_id)
        .cloned()
        .collect();

    if game_backups.is_empty() {
        return Err("No active backups found for this game.".to_string());
    }

    let mut restored_ids = Vec::new();
    for entry in &game_backups {
        if restore_backup_entry(entry).is_ok() {
            restored_ids.push(entry.id.clone());
        }
    }

    if let Some(g_mut) = games.iter_mut().find(|g| g.id == game_id) {
        push_log(
            &state,
            "INFO",
            format!("Restored {} backup(s) for {}", restored_ids.len(), g_mut.name),
        );
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&g_mut.path));
        g_mut.dlls = rescanned_dlls;
    }

    let updated_backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for bid in restored_ids {
            let _ = db.delete_backup(&bid);
        }
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, updated_backups, &manifest, gpu, config)
}

#[tauri::command]
async fn restore_tech_group_for_game(
    game_id: String,
    tech_label: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (mut games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    let matching: Vec<BackupEntry> = backups
        .iter()
        .filter(|b| {
            b.game_id == game_id
                && b.dll_type.short_label().eq_ignore_ascii_case(&tech_label)
        })
        .cloned()
        .collect();

    if matching.is_empty() {
        return Err(format!("No active {} backups found for this game.", tech_label));
    }

    let mut restored_ids = Vec::new();
    for entry in &matching {
        if restore_backup_entry(entry).is_ok() {
            restored_ids.push(entry.id.clone());
        }
    }

    if let Some(g_mut) = games.iter_mut().find(|g| g.id == game_id) {
        push_log(
            &state,
            "INFO",
            format!("Restored {} {} backup(s) for {}", restored_ids.len(), tech_label, g_mut.name),
        );
        let (rescanned_dlls, _) = scan_game_directory(Path::new(&g_mut.path));
        g_mut.dlls = rescanned_dlls;
    }

    let updated_backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for bid in restored_ids {
            let _ = db.delete_backup(&bid);
        }
        db.load_backups().unwrap_or_default()
    };

    build_payload(&state, games, updated_backups, &manifest, gpu, config)
}

#[tauri::command]
async fn delete_backup_cmd(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<BackupEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let backups = db.load_backups().unwrap_or_default();
    if let Some(entry) = backups.iter().find(|b| b.id == backup_id) {
        if !entry.backup_file_path.is_empty() {
            let _ = fs::remove_file(&entry.backup_file_path);
        }
    }
    db.delete_backup(&backup_id).map_err(|e| e.to_string())?;
    db.load_backups().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_app_config(
    new_config: AppConfig,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    save_config(&new_config).map_err(|e| e.to_string())?;
    if let Ok(mut cfg) = state.config.lock() {
        *cfg = new_config.clone();
    }
    if let Ok(gpu) = state.gpu.lock() {
        if gpu.vendor.eq_ignore_ascii_case("NVIDIA") {
            let _ = apply_nvapi_dlss_presets(
                None,
                None,
                &new_config.global_dlss_preset,
                &new_config.global_dlss_rr_preset,
                &new_config.global_dlss_fg_preset,
            );
        }
    }
    push_log(&state, "INFO", "Saved application settings.");
    Ok(new_config)
}

#[tauri::command]
async fn toggle_dlss_indicator(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    set_dlss_indicator_state(enabled)?;
    let actual = get_dlss_indicator_state();
    if let Ok(mut cfg) = state.config.lock() {
        cfg.dlss_indicator_enabled = actual;
        let _ = save_config(&cfg);
    }
    push_log(
        &state,
        "INFO",
        format!(
            "NVIDIA DLSS On-Screen Overlay {}",
            if actual { "enabled" } else { "disabled" }
        ),
    );
    Ok(actual)
}

#[tauri::command]
async fn open_path_in_explorer(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    let dir = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(dir.as_os_str())
            .spawn()
            .map_err(|e| format!("Failed to open Explorer: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn launch_game_cmd(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let mut games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g = db.load_games().map_err(|e| e.to_string())?;
        db.apply_overrides_and_config(&mut g, &config);
        g
    };

    let game = games
        .iter_mut()
        .find(|g| g.id == game_id)
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    // 1. Custom exe override
    if let Some(ref exe) = game.custom_exe_path {
        let exe_path = Path::new(exe);
        if exe_path.exists() {
            std::process::Command::new(exe_path)
                .current_dir(exe_path.parent().unwrap_or_else(|| Path::new(&game.path)))
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", exe, e))?;
            push_log(&state, "INFO", format!("Launched {} via {}", game.name, exe));
            return Ok(format!("Launching {}...", game.name));
        }
    }

    // 2. Steam protocol if Steam game with numeric AppID
    if game.launcher == models::LauncherType::Steam {
        if let Some(ref app_id) = game.app_id {
            if !app_id.is_empty() && app_id.chars().all(|c| c.is_ascii_digit()) {
                let uri = format!("steam://rungameid/{}", app_id);
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("explorer.exe").arg(&uri).spawn();
                }
                push_log(&state, "INFO", format!("Launched {} via {}", game.name, uri));
                return Ok(format!("Launching {} via Steam...", game.name));
            }
        }
    }

    // 2b. Star Citizen via RSI Launcher (required for CIG auth token & EAC bootstrapper)
    if game.launcher == models::LauncherType::Rsi {
        let rsi_candidates = [
            "C:\\Program Files\\Roberts Space Industries\\RSI Launcher\\RSI Launcher.exe",
            "D:\\Program Files\\Roberts Space Industries\\RSI Launcher\\RSI Launcher.exe",
            "C:\\Roberts Space Industries\\RSI Launcher\\RSI Launcher.exe",
        ];
        for cand in &rsi_candidates {
            let p = Path::new(cand);
            if p.exists() {
                std::process::Command::new(p)
                    .current_dir(p.parent().unwrap_or(p))
                    .spawn()
                    .map_err(|e| format!("Failed to launch RSI Launcher: {}", e))?;
                push_log(
                    &state,
                    "INFO",
                    format!("Launched {} via RSI Launcher ({})", game.name, cand),
                );
                return Ok(format!("Launching {} via RSI Launcher...", game.name));
            }
        }
    }

    // 3. Auto-detect primary shipping executable in game folder
    if let Some(exe_path) = find_primary_game_executable(Path::new(&game.path)) {
        std::process::Command::new(&exe_path)
            .current_dir(exe_path.parent().unwrap_or_else(|| Path::new(&game.path)))
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", exe_path.display(), e))?;
        push_log(
            &state,
            "INFO",
            format!("Launched {} ({})", game.name, exe_path.display()),
        );
        return Ok(format!("Launching {}...", game.name));
    }

    Err(format!(
        "Could not locate game executable in {}. You can set a custom .exe in DLSS Settings.",
        game.path
    ))
}

#[tauri::command]
async fn toggle_ignore_game(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let mut config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    let game_name = games
        .iter()
        .find(|g| g.id == game_id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| game_id.clone());

    let existing_idx = config
        .ignored_games
        .iter()
        .position(|s| s.eq_ignore_ascii_case(&game_id) || s.eq_ignore_ascii_case(&game_name));

    let now_ignored = if let Some(idx) = existing_idx {
        config.ignored_games.remove(idx);
        false
    } else {
        config.ignored_games.push(game_id.clone());
        true
    };

    let _ = save_config(&config);
    if let Ok(mut cfg) = state.config.lock() {
        *cfg = config.clone();
    }

    push_log(
        &state,
        "INFO",
        format!(
            "{} {}",
            game_name,
            if now_ignored {
                "added to Ignore List"
            } else {
                "removed from Ignore List"
            }
        ),
    );

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn search_steam_store(query: String) -> Result<Vec<SteamSearchResult>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let encoded: String = trimmed
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect();

    let url = format!(
        "https://store.steampowered.com/api/storesearch/?term={}&l=english&cc=US",
        encoded
    );

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager-Rust/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        for item in items.iter().take(12) {
            let app_id = item
                .get("id")
                .and_then(|v| v.as_u64().map(|n| n.to_string()).or_else(|| v.as_str().map(|s| s.to_string())))
                .unwrap_or_default();
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !app_id.is_empty() && !name.is_empty() {
                let hero_url = format!(
                    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                    app_id
                );
                let header_url = format!(
                    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg",
                    app_id
                );
                out.push(SteamSearchResult {
                    app_id,
                    name,
                    hero_url,
                    header_url,
                });
            }
        }
    }

    Ok(out)
}

#[tauri::command]
async fn save_game_display_override(
    game_id: String,
    custom_name: Option<String>,
    custom_app_id: Option<String>,
    custom_cover_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut overrides = db.load_overrides().unwrap_or_default();
        let mut ov = overrides.remove(&game_id).unwrap_or(GameOverride {
            game_id: game_id.clone(),
            ..Default::default()
        });
        ov.custom_name = custom_name.filter(|s| !s.trim().is_empty());
        ov.custom_app_id = custom_app_id.filter(|s| !s.trim().is_empty());
        ov.custom_cover_url = custom_cover_url.filter(|s| !s.trim().is_empty());
        if ov.custom_cover_url.is_none() {
            if let Some(ref aid) = ov.custom_app_id {
                if aid.chars().all(|c| c.is_ascii_digit()) {
                    ov.custom_cover_url = Some(format!(
                        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                        aid
                    ));
                }
            }
        }
        db.save_override(&ov).map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    push_log(&state, "INFO", format!("Saved display override for game {}", game_id));
    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn reset_game_display_override(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (mut games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let _ = db.delete_override(&game_id);
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    if let Some(g) = games.iter_mut().find(|g| g.id == game_id) {
        if let Some(ref orig) = g.original_name {
            g.name = orig.clone();
        }
        let (pretty, aid) = scanner::prettify_game_name(&g.name, Path::new(&g.path));
        g.name = pretty;
        if g.launcher == models::LauncherType::Rsi {
            g.cover_url = Some("assets/icons/starcitizen_cover.png".to_string());
        } else if let Some(id) = aid {
            g.app_id = Some(id.clone());
            g.cover_url = Some(format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                id
            ));
        }
        g.dlss_sr_preset = None;
        g.dlss_rr_preset = None;
        g.dlss_fg_preset = None;
        g.custom_exe_path = None;
    }

    push_log(&state, "INFO", format!("Reset display overrides for game {}", game_id));
    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn save_game_dlss_presets(
    game_id: String,
    sr_preset: String,
    rr_preset: String,
    fg_preset: String,
    custom_exe_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let (games, backups, game_name_opt) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut overrides = db.load_overrides().unwrap_or_default();
        let mut ov = overrides.remove(&game_id).unwrap_or(GameOverride {
            game_id: game_id.clone(),
            ..Default::default()
        });
        ov.dlss_sr_preset = Some(sr_preset.clone());
        ov.dlss_rr_preset = Some(rr_preset.clone());
        ov.dlss_fg_preset = Some(fg_preset.clone());
        ov.custom_exe_path = custom_exe_path.clone().filter(|s| !s.trim().is_empty());
        db.save_override(&ov).map_err(|e| e.to_string())?;

        let g_list = db.load_games().unwrap_or_default();
        let gname = g_list
            .iter()
            .find(|g| g.id == game_id)
            .map(|g| g.name.clone());
        let b_list = db.load_backups().unwrap_or_default();
        (g_list, b_list, gname)
    };

    if gpu.vendor.eq_ignore_ascii_case("NVIDIA") {
        let msg = apply_nvapi_dlss_presets(
            game_name_opt.as_deref(),
            custom_exe_path.as_deref(),
            &sr_preset,
            &rr_preset,
            &fg_preset,
        )
        .unwrap_or_else(|e| e);
        push_log(
            &state,
            "INFO",
            format!(
                "{} for {}",
                msg,
                game_name_opt.unwrap_or_else(|| game_id.clone())
            ),
        );
    }

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn clear_app_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    if let Ok(mut logs) = state.logs.lock() {
        logs.clear();
    }
    Ok(Vec::new())
}

#[tauri::command]
async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let url = "https://api.github.com/repos/gOOvER/DLSS-Manager/releases/latest";

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager-Rust/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    if let Ok(resp) = client.get(url).send().await {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let tag = json
                    .get("tag_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current)
                    .trim_start_matches('v')
                    .to_string();
                let html_url = json
                    .get("html_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://github.com/gOOvER/DLSS-Manager/releases")
                    .to_string();
                let body = json
                    .get("body")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let update_available = pe_version::is_version_newer(&tag, &current);

                return Ok(AppUpdateInfo {
                    current_version: current,
                    latest_version: tag,
                    update_available,
                    release_url: html_url,
                    download_url: None,
                    release_notes: body,
                });
            }
        }
    }

    Ok(AppUpdateInfo {
        current_version: current.clone(),
        latest_version: current,
        update_available: false,
        release_url: "https://github.com/gOOvER/DLSS-Manager/releases".to_string(),
        download_url: None,
        release_notes: None,
    })
}

#[tauri::command]
async fn validate_and_save_steam_api(
    api_key: String,
    steam_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<SteamValidationResult, String> {
    let trimmed_key = api_key.trim().to_string();
    if trimmed_key.is_empty() {
        let mut cfg = state.config.lock().map(|c| c.clone()).unwrap_or_default();
        cfg.steam_api_key.clear();
        let _ = save_config(&cfg);
        if let Ok(mut lock) = state.config.lock() {
            *lock = cfg;
        }
        return Ok(SteamValidationResult {
            valid: false,
            steam_id: None,
            owned_games_count: 0,
            games_to_improve: 0,
            message: "Steam API key cleared.".to_string(),
        });
    }

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let validate_url = format!(
        "https://api.steampowered.com/ISteamWebAPIUtil/GetSupportedAPIList/v1/?key={}",
        trimmed_key
    );
    let resp = client
        .get(&validate_url)
        .send()
        .await
        .map_err(|e| format!("Network error validating Steam API key: {}", e))?;

    if !resp.status().is_success() {
        return Ok(SteamValidationResult {
            valid: false,
            steam_id: None,
            owned_games_count: 0,
            games_to_improve: 0,
            message: format!("Invalid Steam API key (HTTP {})", resp.status()),
        });
    }

    let resolved_steam_id = steam_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(detect_steam_id_from_vdf)
        .unwrap_or_default();

    let mut owned_count = 0usize;
    if !resolved_steam_id.is_empty() {
        let owned_url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1&skip_unvetted_apps=0",
            trimmed_key, resolved_steam_id
        );
        if let Ok(o_resp) = client.get(&owned_url).send().await {
            if let Ok(json) = o_resp.json::<serde_json::Value>().await {
                if let Some(arr) = json
                    .get("response")
                    .and_then(|r| r.get("games"))
                    .and_then(|g| g.as_array())
                {
                    owned_count = arr.len();
                }
            }
        }
    }

    let mut cfg = state.config.lock().map(|c| c.clone()).unwrap_or_default();
    cfg.steam_api_key = trimmed_key;
    if !resolved_steam_id.is_empty() {
        cfg.steam_id = resolved_steam_id.clone();
    }
    let _ = save_config(&cfg);
    if let Ok(mut lock) = state.config.lock() {
        *lock = cfg;
    }

    let games_to_improve = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let games = db.load_games().unwrap_or_default();
        games
            .iter()
            .filter(|g| g.app_id.as_ref().map(|id| id.is_empty()).unwrap_or(true))
            .count()
    };

    push_log(
        &state,
        "INFO",
        format!(
            "Validated Steam Web API key (Steam ID: {}, {} owned games).",
            if resolved_steam_id.is_empty() {
                "not set"
            } else {
                &resolved_steam_id
            },
            owned_count
        ),
    );

    Ok(SteamValidationResult {
        valid: true,
        steam_id: if resolved_steam_id.is_empty() {
            None
        } else {
            Some(resolved_steam_id)
        },
        owned_games_count: owned_count,
        games_to_improve,
        message: format!(
            "Connected! {} owned games found ({} library games can be enhanced).",
            owned_count, games_to_improve
        ),
    })
}

#[tauri::command]
async fn reresolve_games_via_steam_api(
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());

    let client = reqwest::Client::builder()
        .user_agent("DLSS-Manager/1.0.0-beta.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // 1. Fetch owned games map (normalized_name -> (app_id, official_name)) if API key + SteamID configured
    let mut owned_map: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    let steam_id = if !config.steam_id.trim().is_empty() {
        config.steam_id.trim().to_string()
    } else {
        detect_steam_id_from_vdf().unwrap_or_default()
    };

    if !config.steam_api_key.trim().is_empty() && !steam_id.is_empty() {
        let owned_url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1&skip_unvetted_apps=0",
            config.steam_api_key.trim(),
            steam_id
        );
        if let Ok(resp) = client.get(&owned_url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = json
                    .get("response")
                    .and_then(|r| r.get("games"))
                    .and_then(|g| g.as_array())
                {
                    for item in arr {
                        let aid = item
                            .get("appid")
                            .and_then(|v| v.as_u64().map(|n| n.to_string()))
                            .unwrap_or_default();
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        if !aid.is_empty() && !name.is_empty() {
                            let norm = normalize_game_name_for_match(&name);
                            if !norm.is_empty() {
                                owned_map.insert(norm, (aid, name));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g = db.load_games().unwrap_or_default();
        db.apply_overrides_and_config(&mut g, &config);
        g
    };

    let mut resolved_count = 0usize;
    let mut new_overrides: Vec<GameOverride> = Vec::new();

    for g in games.iter_mut() {
        // Never overwrite Star Citizen with a Steam Store search result (Star Citizen is not on Steam)
        if g.launcher == models::LauncherType::Rsi {
            if g.cover_url.is_none() {
                g.cover_url = Some("assets/icons/starcitizen_cover.png".to_string());
            }
            continue;
        }

        let has_valid_appid = g
            .app_id
            .as_ref()
            .map(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false);
        if has_valid_appid && g.cover_url.is_some() {
            continue;
        }

        let norm = normalize_game_name_for_match(&g.name);
        let mut found: Option<(String, String)> = owned_map.get(&norm).cloned();

        // Fallback to Steam Store search API if not in owned games
        if found.is_none() {
            if let Ok(results) = search_steam_store(g.name.clone()).await {
                if let Some(first) = results.into_iter().next() {
                    found = Some((first.app_id, first.name));
                }
            }
        }

        if let Some((app_id, _official_name)) = found {
            let hero_url = format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                app_id
            );
            g.app_id = Some(app_id.clone());
            g.cover_url = Some(hero_url.clone());
            new_overrides.push(GameOverride {
                game_id: g.id.clone(),
                custom_name: None,
                custom_app_id: Some(app_id),
                custom_cover_url: Some(hero_url),
                custom_exe_path: g.custom_exe_path.clone(),
                dlss_sr_preset: g.dlss_sr_preset.clone(),
                dlss_rr_preset: g.dlss_rr_preset.clone(),
                dlss_fg_preset: g.dlss_fg_preset.clone(),
            });
            resolved_count += 1;
        }
    }

    let backups = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for ov in new_overrides {
            let _ = db.save_override(&ov);
        }
        db.load_backups().unwrap_or_default()
    };

    push_log(
        &state,
        "INFO",
        format!(
            "Steam API Re-resolution finished: enhanced artwork/AppIDs for {} game(s).",
            resolved_count
        ),
    );

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn sync_game_database(
    custom_url: Option<String>,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let mut config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();

    let mut changed = false;
    if let Some(u) = custom_url {
        let trimmed = u.trim().to_string();
        if !trimmed.is_empty() && trimmed != config.game_db_url {
            config.game_db_url = trimmed;
            changed = true;
        }
    }
    if let Some(k) = api_key {
        let trimmed_k = k.trim().to_string();
        if trimmed_k != config.game_db_api_key {
            config.game_db_api_key = trimmed_k;
            changed = true;
        }
    }
    if changed {
        let _ = save_config(&config);
        if let Ok(mut cfg_lock) = state.config.lock() {
            *cfg_lock = config.clone();
        }
    }

    let url = config.game_db_url.clone();
    let key = config.game_db_api_key.clone();
    match fetch_remote_game_db(&url, &key).await {
        Ok(profiles) => {
            let count = profiles.len();
            if let Ok(mut lock) = state.game_db.lock() {
                *lock = profiles;
            }
            push_log(
                &state,
                "INFO",
                format!("Synced Game Compatibility DB from {} ({} profiles).", url, count),
            );
        }
        Err(err) => {
            let fallback = load_cached_or_seed_game_db();
            let count = fallback.len();
            if let Ok(mut lock) = state.game_db.lock() {
                *lock = fallback;
            }
            push_log(
                &state,
                "WARN",
                format!(
                    "Remote Game DB unreachable ({}), using local/built-in DB ({} profiles).",
                    err, count
                ),
            );
        }
    }

    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());
    let (games, backups) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        (
            db.load_games().unwrap_or_default(),
            db.load_backups().unwrap_or_default(),
        )
    };

    if config.auto_report_games_to_db && !config.game_db_api_key.trim().is_empty() && !games.is_empty() {
        if let Ok(msg) = report_games_to_remote_db(&url, &key, &games, "auto_sync").await {
            push_log(&state, "INFO", format!("Auto-reported games to Online-DB: {}", msg));
        }
    }

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[tauri::command]
async fn report_games_to_db(
    game_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();

    let games = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut list = db.load_games().unwrap_or_default();
        db.apply_overrides_and_config(&mut list, &config);
        list
    };

    let target_games: Vec<Game> = if let Some(ref gid) = game_id {
        games
            .into_iter()
            .filter(|g| g.id == *gid)
            .collect()
    } else {
        games
    };

    if target_games.is_empty() {
        return Err("Kein Spiel zum Melden gefunden.".to_string());
    }

    let source = if game_id.is_some() { "manual_card" } else { "manual_batch" };
    let msg = report_games_to_remote_db(
        &config.game_db_url,
        &config.game_db_api_key,
        &target_games,
        source,
    )
    .await?;

    push_log(&state, "INFO", format!("Reported games to Online-DB: {}", msg));
    Ok(msg)
}

#[tauri::command]
async fn apply_recommended_game_profile(
    game_id: String,
    state: State<'_, AppState>,
) -> Result<InitialPayload, String> {
    let config = state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();
    let manifest = state
        .manifest
        .lock()
        .map(|m| m.clone())
        .unwrap_or_default();
    let gpu = state
        .gpu
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| detect_gpu_info());
    let profiles = state
        .game_db
        .lock()
        .map(|p| p.clone())
        .unwrap_or_else(|_| load_cached_or_seed_game_db());

    let (games, backups, matched_profile, game_name, exe_override) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut g_list = db.load_games().unwrap_or_default();
        db.apply_overrides_and_config(&mut g_list, &config);
        annotate_games_with_db_profiles(&mut g_list, &profiles);

        let game = g_list
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| "Game not found.".to_string())?;
        let prof = game
            .db_profile
            .clone()
            .ok_or_else(|| "No database profile found for this game.".to_string())?;
        let gname = game.name.clone();
        let exe_opt = game.custom_exe_path.clone();

        let sr = prof
            .recommended_sr_preset
            .clone()
            .unwrap_or_else(|| "Default".to_string());
        let rr = prof
            .recommended_rr_preset
            .clone()
            .unwrap_or_else(|| "Default".to_string());
        let fg = prof
            .recommended_fg_preset
            .clone()
            .unwrap_or_else(|| "Default".to_string());

        let mut overrides = db.load_overrides().unwrap_or_default();
        let mut ov = overrides.remove(&game_id).unwrap_or(GameOverride {
            game_id: game_id.clone(),
            ..Default::default()
        });
        ov.dlss_sr_preset = Some(sr);
        ov.dlss_rr_preset = Some(rr);
        ov.dlss_fg_preset = Some(fg);
        db.save_override(&ov).map_err(|e| e.to_string())?;

        let b_list = db.load_backups().unwrap_or_default();
        (g_list, b_list, prof, gname, exe_opt)
    };

    if gpu.vendor.eq_ignore_ascii_case("NVIDIA") {
        let sr = matched_profile
            .recommended_sr_preset
            .as_deref()
            .unwrap_or("Default");
        let rr = matched_profile
            .recommended_rr_preset
            .as_deref()
            .unwrap_or("Default");
        let fg = matched_profile
            .recommended_fg_preset
            .as_deref()
            .unwrap_or("Default");
        let _ = apply_nvapi_dlss_presets(Some(&game_name), exe_override.as_deref(), sr, rr, fg);
    }

    push_log(
        &state,
        "INFO",
        format!("Applied recommended database profile for {}", game_name),
    );

    build_payload(&state, games, backups, &manifest, gpu, config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::open().expect("Failed to initialize SQLite database");
    let manifest = load_cached_manifest_sync();
    let config = load_config();
    let gpu = detect_gpu_info();
    let game_db = load_cached_or_seed_game_db();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(db),
            manifest: Mutex::new(manifest),
            config: Mutex::new(config),
            gpu: Mutex::new(gpu),
            logs: Mutex::new(Vec::new()),
            game_db: Mutex::new(game_db),
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_state,
            scan_games,
            refresh_dll_manifest,
            download_dll_to_cache,
            update_single_game_dll,
            update_all_for_game,
            bulk_update_all_games,
            bulk_update_selected_games,
            restore_backup,
            restore_all_for_game,
            restore_tech_group_for_game,
            delete_backup_cmd,
            update_app_config,
            toggle_dlss_indicator,
            open_path_in_explorer,
            launch_game_cmd,
            toggle_ignore_game,
            search_steam_store,
            save_game_display_override,
            reset_game_display_override,
            save_game_dlss_presets,
            clear_app_logs,
            check_app_update,
            validate_and_save_steam_api,
            reresolve_games_via_steam_api,
            sync_game_database,
            report_games_to_db,
            apply_recommended_game_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DLSS Manager");
}
