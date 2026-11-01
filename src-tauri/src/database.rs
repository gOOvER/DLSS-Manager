use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::{AppConfig, BackupEntry, Game, GameOverride};

/// Returns the shared `%LOCALAPPDATA%\Recol\DLSS-Updater` directory (preserving compatibility
/// with existing backups, caches, and user configurations).
pub fn get_app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("Recol").join("DLSS-Updater");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_cache_dir() -> PathBuf {
    let dir = get_app_data_dir().join("dll_cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_backups_dir() -> PathBuf {
    let dir = get_app_data_dir().join("backups");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_config_path() -> PathBuf {
    get_app_data_dir().join("dlss_manager_config.json")
}

/// Reads legacy `config.toml` and `config.ini` in `%LOCALAPPDATA%\Recol\DLSS-Updater`
/// to import any previously configured launcher paths (`steam_path`, `ea_path`, `epic_path`,
/// `gog_path`, `ubisoft_path`, `battle_net_path`, `xbox_path`, `custom_path_1`..`4`).
pub fn import_legacy_configured_paths() -> Vec<String> {
    let mut paths = Vec::new();
    let app_dir = get_app_data_dir();

    for filename in &["config.toml", "config.ini"] {
        let legacy_path = app_dir.join(filename);
        let Ok(content) = fs::read_to_string(&legacy_path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with('[') || !trimmed.contains('=') {
                continue;
            }
            let Some((key, raw_val)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            if key.ends_with("_path") || key.starts_with("custom_path") {
                let unquoted = raw_val
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .replace("\\\\", "\\");
                for part in unquoted.split(';') {
                    let clean = part.trim();
                    if !clean.is_empty()
                        && clean != "None"
                        && PathBuf::from(clean).exists()
                        && !paths.iter().any(|p: &String| p.eq_ignore_ascii_case(clean))
                    {
                        paths.push(clean.to_string());
                    }
                }
            }
        }
    }

    paths
}

/// Reads game directories previously stored in the legacy Python SQLite DB (`dlss_updater.db` / `games.db`).
pub fn load_legacy_db_game_roots() -> Vec<(String, String, String, Option<String>)> {
    let mut roots = Vec::new();
    let app_dir = get_app_data_dir();

    for db_name in &["dlss_updater.db", "games.db"] {
        let db_path = app_dir.join(db_name);
        if !db_path.exists() {
            continue;
        }
        let Ok(conn) = Connection::open(&db_path) else {
            continue;
        };
        let Ok(mut stmt) = conn.prepare(
            "SELECT COALESCE(display_name_override, name), path, launcher, CAST(COALESCE(override_steam_app_id, steam_app_id) AS TEXT) FROM games"
        ).or_else(|_| conn.prepare("SELECT name, path, launcher, CAST(steam_app_id AS TEXT) FROM games")) else {
            continue;
        };
        let Ok(rows) = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            let launcher: String = row.get(2).unwrap_or_else(|_| "Custom".to_string());
            let app_id: Option<String> = row.get(3).ok();
            Ok((name, path, launcher, app_id))
        }) else {
            continue;
        };
        for r in rows.flatten() {
            let p = PathBuf::from(&r.1);
            if p.exists() && crate::scanner::has_valid_game_executable(&p) {
                roots.push(r);
            }
        }
    }

    roots
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    let mut cfg = if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    // Merge any paths and Steam API credentials configured in legacy config.toml / config.ini
    for legacy_p in import_legacy_configured_paths() {
        if !cfg
            .custom_folders
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&legacy_p))
        {
            cfg.custom_folders.push(legacy_p);
        }
    }

    if cfg.steam_api_key.is_empty() || cfg.steam_id.is_empty() {
        let app_dir = get_app_data_dir();
        for filename in &["config.toml", "config.ini"] {
            if let Ok(content) = fs::read_to_string(app_dir.join(filename)) {
                for line in content.lines() {
                    if let Some((k, v)) = line.trim().split_once('=') {
                        let key = k.trim().to_ascii_lowercase();
                        let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() && val != "None" {
                            if (key == "steam_api_key" || key == "api_key")
                                && cfg.steam_api_key.is_empty()
                            {
                                cfg.steam_api_key = val.to_string();
                            } else if (key == "steam_id" || key == "auto_detected_steam_id")
                                && cfg.steam_id.is_empty()
                            {
                                cfg.steam_id = val.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    if cfg.steam_id.is_empty() {
        if let Some(detected_id) = crate::scanner::detect_steam_id_from_vdf() {
            cfg.steam_id = detected_id;
        }
    }

    if cfg.game_db_url.trim().is_empty()
        || cfg.game_db_url == "https://dlss-gamedb.pages.dev/api/v1/games.json"
        || cfg.game_db_url == "https://dlss-manager-onlinedb.pages.dev/api/v1/games.json"
    {
        cfg.game_db_url = crate::models::default_game_db_url();
    }

    cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(path, json)?;
    Ok(())
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let db_path = get_app_data_dir().join("dlss_manager_v2.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;

             CREATE TABLE IF NOT EXISTS games (
                 id TEXT PRIMARY KEY,
                 name TEXT NOT NULL,
                 path TEXT NOT NULL,
                 launcher TEXT NOT NULL,
                 payload_json TEXT NOT NULL,
                 last_scanned TEXT NOT NULL
             );

             CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(
                 id UNINDEXED,
                 name,
                 launcher,
                 path
             );

             CREATE TABLE IF NOT EXISTS backups (
                 id TEXT PRIMARY KEY,
                 game_id TEXT NOT NULL,
                 game_name TEXT NOT NULL,
                 original_dll_path TEXT NOT NULL,
                 backup_file_path TEXT NOT NULL,
                 payload_json TEXT NOT NULL,
                 created_at TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS game_overrides (
                 game_id TEXT PRIMARY KEY,
                 payload_json TEXT NOT NULL
             );",
        )?;

        Ok(Self { conn })
    }

    pub fn save_games(&mut self, games: &[Game]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM games", [])?;
        tx.execute("DELETE FROM games_fts", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO games (id, name, path, launcher, payload_json, last_scanned)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            let mut fts_stmt = tx.prepare(
                "INSERT INTO games_fts (id, name, launcher, path)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for g in games {
                let json = serde_json::to_string(g)?;
                stmt.execute(params![
                    g.id,
                    g.name,
                    g.path,
                    g.launcher_name,
                    json,
                    g.last_scanned
                ])?;
                fts_stmt.execute(params![g.id, g.name, g.launcher_name, g.path])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_games(&self) -> Result<Vec<Game>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload_json FROM games ORDER BY LOWER(name) ASC")?;
        let rows = stmt.query_map([], |row| {
            let s: String = row.get(0)?;
            Ok(s)
        })?;

        let mut games = Vec::new();
        for r in rows.flatten() {
            if let Ok(g) = serde_json::from_str::<Game>(&r) {
                if std::path::Path::new(&g.path).exists()
                    && crate::scanner::has_valid_game_executable(std::path::Path::new(&g.path))
                {
                    games.push(g);
                }
            }
        }
        Ok(games)
    }

    pub fn save_backup(&self, entry: &BackupEntry) -> Result<()> {
        let json = serde_json::to_string(entry)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO backups (id, game_id, game_name, original_dll_path, backup_file_path, payload_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id,
                entry.game_id,
                entry.game_name,
                entry.original_dll_path,
                entry.backup_file_path,
                json,
                entry.created_at
            ],
        )?;
        Ok(())
    }

    pub fn load_backups(&self) -> Result<Vec<BackupEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload_json FROM backups ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let s: String = row.get(0)?;
            Ok(s)
        })?;

        let mut list = Vec::new();
        for r in rows.flatten() {
            if let Ok(b) = serde_json::from_str::<BackupEntry>(&r) {
                list.push(b);
            }
        }
        Ok(list)
    }

    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM backups WHERE id = ?1", params![backup_id])?;
        Ok(())
    }

    pub fn save_override(&self, ov: &GameOverride) -> Result<()> {
        let json = serde_json::to_string(ov)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO game_overrides (game_id, payload_json) VALUES (?1, ?2)",
            params![ov.game_id, json],
        )?;
        Ok(())
    }

    pub fn load_overrides(&self) -> Result<HashMap<String, GameOverride>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_id, payload_json FROM game_overrides")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let s: String = row.get(1)?;
            Ok((id, s))
        })?;

        let mut map = HashMap::new();
        for (id, s) in rows.flatten() {
            if let Ok(ov) = serde_json::from_str::<GameOverride>(&s) {
                map.insert(id, ov);
            }
        }
        Ok(map)
    }

    pub fn delete_override(&self, game_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM game_overrides WHERE game_id = ?1",
            params![game_id],
        )?;
        Ok(())
    }

    pub fn apply_overrides_and_config(&self, games: &mut [Game], config: &AppConfig) {
        let overrides = self.load_overrides().unwrap_or_default();
        let ignored_set: HashSet<String> = config
            .ignored_games
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        let blacklist_set: HashSet<String> = config
            .blacklisted_games
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();

        for g in games.iter_mut() {
            if g.path.to_ascii_lowercase().contains("starcitizen")
                || g.launcher_name.eq_ignore_ascii_case("RSI Launcher")
                || g.name.to_ascii_lowercase().starts_with("star citizen")
            {
                g.launcher = crate::models::LauncherType::Rsi;
            }
            g.launcher_name = g.launcher.display_name().to_string();
            g.launcher_icon = g.launcher.icon_name().to_string();

            let (pretty_name, inferred_app_id) =
                crate::scanner::prettify_game_name(&g.name, std::path::Path::new(&g.path));
            if g.name.starts_with('_')
                || g.name.eq_ignore_ascii_case("live")
                || g.name.eq_ignore_ascii_case(&pretty_name.replace(' ', ""))
            {
                g.name = pretty_name;
            }
            if g.app_id.is_none() {
                g.app_id = inferred_app_id;
            }
            if g.launcher == crate::models::LauncherType::Rsi {
                let is_steam_cover = g
                    .cover_url
                    .as_deref()
                    .map(|u| u.contains("steamstatic.com"))
                    .unwrap_or(false);
                if g.cover_url.is_none() || is_steam_cover {
                    g.cover_url = Some("assets/icons/starcitizen_cover.png".to_string());
                }
            } else {
                if let Some(ref aid) = g.app_id {
                    if !aid.is_empty() && aid.chars().all(|c| c.is_ascii_digit()) {
                        let is_unhashed_default = g
                            .cover_url
                            .as_deref()
                            .map(|u| {
                                u.ends_with(&format!("/apps/{}/library_600x900.jpg", aid))
                                    || u.ends_with(&format!("/apps/{}/header.jpg", aid))
                            })
                            .unwrap_or(true);
                        if g.cover_url.is_none() || is_unhashed_default {
                            g.cover_url = Some(crate::scanner::resolve_steam_cover_url(aid));
                        }
                    }
                }
            }
            if g.original_name.is_none() {
                g.original_name = Some(g.name.clone());
            }
            if let Some(ov) = overrides.get(&g.id) {
                if let Some(ref cname) = ov.custom_name {
                    if !cname.trim().is_empty() {
                        g.name = cname.trim().to_string();
                    }
                }
                if let Some(ref app_id) = ov.custom_app_id {
                    if !app_id.trim().is_empty()
                        && g.launcher != crate::models::LauncherType::Rsi
                    {
                        g.app_id = Some(app_id.trim().to_string());
                    }
                }
                if let Some(ref cover) = ov.custom_cover_url {
                    let trimmed_cover = cover.trim();
                    let skip_wrong_steam_cover = g.launcher == crate::models::LauncherType::Rsi
                        && trimmed_cover.contains("steamstatic.com");
                    if !trimmed_cover.is_empty() && !skip_wrong_steam_cover {
                        // If override has a legacy unhashed URL for a newer Steam game, upgrade it via librarycache hash resolver
                        let resolved = if let Some(ref aid) = g.app_id {
                            if trimmed_cover.ends_with(&format!("/apps/{}/library_600x900.jpg", aid))
                                || trimmed_cover.ends_with(&format!("/apps/{}/header.jpg", aid))
                            {
                                crate::scanner::resolve_steam_cover_url(aid)
                            } else {
                                trimmed_cover.to_string()
                            }
                        } else {
                            trimmed_cover.to_string()
                        };
                        g.cover_url = Some(resolved);
                    }
                }
                if ov.custom_exe_path.is_some() {
                    g.custom_exe_path = ov.custom_exe_path.clone();
                }
                if ov.dlss_sr_preset.is_some() {
                    g.dlss_sr_preset = ov.dlss_sr_preset.clone();
                }
                if ov.dlss_rr_preset.is_some() {
                    g.dlss_rr_preset = ov.dlss_rr_preset.clone();
                }
                if ov.dlss_fg_preset.is_some() {
                    g.dlss_fg_preset = ov.dlss_fg_preset.clone();
                }
            }
            let name_lower = g.name.to_ascii_lowercase();
            let orig_lower = g
                .original_name
                .as_deref()
                .unwrap_or(&g.name)
                .to_ascii_lowercase();
            g.ignored = ignored_set.contains(&g.id.to_ascii_lowercase())
                || ignored_set.contains(&name_lower)
                || ignored_set.contains(&orig_lower);
            g.blacklisted = blacklist_set.contains(&g.id.to_ascii_lowercase())
                || blacklist_set.contains(&name_lower)
                || blacklist_set.contains(&orig_lower);
        }
    }
}

