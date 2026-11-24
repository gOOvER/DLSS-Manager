use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const DISABLE_OVERRIDE_KEYS: [&str; 5] = [
    "Disable_FG_Override",
    "Disable_RR_Override",
    "Disable_SR_Override",
    "Disable_RR_Model_Override",
    "Disable_SR_Model_Override",
];

#[derive(Debug, Clone, Serialize, Default)]
pub struct NvAppWhitelistStatus {
    pub applicable: bool,
    pub blocked_flags: usize,
    pub blocked_apps: Vec<String>,
    pub fingerprint_files: usize,
    pub has_backup: bool,
    pub ops_unsupported_count: usize,
    pub ngx_ota_dlss_version: Option<String>,
    pub is_admin: bool,
}

pub fn is_running_as_admin() -> bool {
    #[cfg(windows)]
    {
        use libloading::{Library, Symbol};
        let Ok(lib) = (unsafe { Library::new("shell32.dll") }) else {
            return false;
        };
        type IsUserAnAdminFn = unsafe extern "system" fn() -> i32;
        let Ok(func): Result<Symbol<IsUserAnAdminFn>, _> = (unsafe { lib.get(b"IsUserAnAdmin\0") })
        else {
            return false;
        };
        unsafe { func() != 0 }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn relaunch_as_admin() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve current executable: {}", e))?;
        let exe_escaped = exe.to_string_lossy().replace('\'', "''");
        let ps_cmd = format!("Start-Process -FilePath '{}' -Verb RunAs", exe_escaped);
        let status = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("Failed to launch elevated instance: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("UAC elevation was cancelled or failed.".to_string())
        }
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

fn detect_ngx_ota_dlss_version() -> Option<String> {
    let models_dir = PathBuf::from(r"C:\ProgramData\NVIDIA\NGX\models\dlss\versions");
    let entries = fs::read_dir(&models_dir).ok()?;
    let mut best: Option<String> = None;
    for entry in entries.flatten() {
        let files_dir = entry.path().join("files");
        if let Ok(files) = fs::read_dir(&files_dir) {
            for f in files.flatten() {
                let p = f.path();
                if p.is_file() {
                    if let Some(ver) = crate::pe_version::extract_dll_version(&p) {
                        if best
                            .as_ref()
                            .map(|b| crate::pe_version::is_version_newer(&ver, b))
                            .unwrap_or(true)
                        {
                            best = Some(ver);
                        }
                    }
                }
            }
        }
    }
    best
}

fn nv_backend_dir() -> Option<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    let dir = PathBuf::from(local_app_data)
        .join("NVIDIA Corporation")
        .join("NVIDIA App")
        .join("NvBackend");
    if dir.exists() {
        Some(dir)
    } else {
        None
    }
}

fn collect_fingerprint_dbs(backend_dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in walkdir::WalkDir::new(backend_dir)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .map(|n| n.eq_ignore_ascii_case("fingerprint.db"))
                .unwrap_or(false)
        {
            result.push(entry.into_path());
        }
    }
    result
}

fn count_blocked_in_json(
    val: &Value,
    blocked_flags: &mut usize,
    ops_unsupported: &mut usize,
    blocked_apps: &mut BTreeSet<String>,
) {
    match val {
        Value::Object(map) => {
            let app_obj = map.get("Application").and_then(|v| v.as_object());
            let label = app_obj
                .and_then(|a| a.get("DisplayName").or_else(|| a.get("ShortName")))
                .or_else(|| map.get("DisplayName").or_else(|| map.get("ShortName")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut obj_blocked = false;
            for key in DISABLE_OVERRIDE_KEYS {
                if map.get(key).and_then(|v| v.as_bool()) == Some(true) {
                    *blocked_flags += 1;
                    obj_blocked = true;
                }
            }
            if map.get("IsOpsSupported").and_then(|v| v.as_bool()) == Some(false) {
                *ops_unsupported += 1;
                obj_blocked = true;
            }
            if obj_blocked {
                if let Some(name) = label {
                    blocked_apps.insert(name);
                }
            }
            for v in map.values() {
                if v.is_object() || v.is_array() {
                    count_blocked_in_json(v, blocked_flags, ops_unsupported, blocked_apps);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                count_blocked_in_json(item, blocked_flags, ops_unsupported, blocked_apps);
            }
        }
        _ => {}
    }
}

fn unlock_json_recursive(
    val: &mut Value,
    unlock_ops: bool,
    modified_flags: &mut usize,
    modified_apps: &mut BTreeSet<String>,
) {
    match val {
        Value::Object(map) => {
            let label = map
                .get("Application")
                .and_then(|v| v.as_object())
                .and_then(|a| a.get("DisplayName").or_else(|| a.get("ShortName")))
                .or_else(|| map.get("DisplayName").or_else(|| map.get("ShortName")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut changed_here = false;
            for key in DISABLE_OVERRIDE_KEYS {
                if let Some(entry) = map.get_mut(key) {
                    if entry.as_bool() == Some(true) {
                        *entry = Value::Bool(false);
                        *modified_flags += 1;
                        changed_here = true;
                    }
                }
            }
            if unlock_ops {
                if let Some(entry) = map.get_mut("IsOpsSupported") {
                    if entry.as_bool() == Some(false) {
                        *entry = Value::Bool(true);
                        *modified_flags += 1;
                        changed_here = true;
                    }
                }
            }
            if changed_here {
                if let Some(name) = label {
                    modified_apps.insert(name);
                }
            }
            for v in map.values_mut() {
                if v.is_object() || v.is_array() {
                    unlock_json_recursive(v, unlock_ops, modified_flags, modified_apps);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                unlock_json_recursive(item, unlock_ops, modified_flags, modified_apps);
            }
        }
        _ => {}
    }
}

/// Inspects NVIDIA App's `ApplicationStorage.json` and `fingerprint.db` files (read-only)
/// to report how many `Disable_*_Override` flags are currently blocking DLSS overrides.
pub fn get_nvidia_app_whitelist_status() -> NvAppWhitelistStatus {
    let is_admin = is_running_as_admin();
    let ngx_ota_dlss_version = detect_ngx_ota_dlss_version();
    let Some(backend_dir) = nv_backend_dir() else {
        return NvAppWhitelistStatus {
            is_admin,
            ngx_ota_dlss_version,
            ..Default::default()
        };
    };

    let app_storage = backend_dir.join("ApplicationStorage.json");
    let fp_dbs = collect_fingerprint_dbs(&backend_dir);
    let applicable = app_storage.is_file() || !fp_dbs.is_empty();
    let backup_path = backend_dir.join("ApplicationStorage.json.dlss_manager.bak");

    let mut blocked_flags = 0usize;
    let mut ops_unsupported_count = 0usize;
    let mut blocked_apps = BTreeSet::new();

    if app_storage.is_file() {
        if let Ok(raw) = fs::read_to_string(&app_storage) {
            if let Ok(json) = serde_json::from_str::<Value>(&raw) {
                count_blocked_in_json(
                    &json,
                    &mut blocked_flags,
                    &mut ops_unsupported_count,
                    &mut blocked_apps,
                );
            }
        }
    }

    for fp in &fp_dbs {
        if let Ok(xml) = fs::read_to_string(fp) {
            for key in DISABLE_OVERRIDE_KEYS {
                let needle = format!("<{}>1</{}>", key, key);
                blocked_flags += xml.matches(&needle).count();
            }
            ops_unsupported_count += xml.matches("<IsAutomatable>0</IsAutomatable>").count();
            ops_unsupported_count += xml
                .matches("<DLSS_Override_No_OPS>0</DLSS_Override_No_OPS>")
                .count();
        }
    }

    NvAppWhitelistStatus {
        applicable,
        blocked_flags,
        blocked_apps: blocked_apps.into_iter().take(25).collect(),
        fingerprint_files: fp_dbs.len(),
        has_backup: backup_path.is_file(),
        ops_unsupported_count,
        ngx_ota_dlss_version,
        is_admin,
    }
}

/// Unlocks DLSS overrides for all games in NVIDIA App (`ApplicationStorage.json` and
/// `fingerprint.db`) by flipping `Disable_*_Override` flags to `false` / `0`
/// (and optionally `IsOpsSupported` to `true`).
pub fn apply_nvidia_app_whitelist(unlock_ops_supported: bool) -> Result<(usize, usize), String> {
    let backend_dir = nv_backend_dir()
        .ok_or_else(|| "NVIDIA App NvBackend directory not found".to_string())?;

    let mut flags_cleared = 0usize;
    let mut apps_unlocked = BTreeSet::new();

    let app_storage = backend_dir.join("ApplicationStorage.json");
    if app_storage.is_file() {
        let backup_path = backend_dir.join("ApplicationStorage.json.dlss_manager.bak");
        if !backup_path.exists() {
            let _ = fs::copy(&app_storage, &backup_path);
        }
        let raw = fs::read_to_string(&app_storage)
            .map_err(|e| format!("Failed to read ApplicationStorage.json: {}", e))?;
        let mut json: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse ApplicationStorage.json: {}", e))?;
        let before = flags_cleared;
        unlock_json_recursive(
            &mut json,
            unlock_ops_supported,
            &mut flags_cleared,
            &mut apps_unlocked,
        );
        if flags_cleared > before {
            let serialized = serde_json::to_string(&json)
                .map_err(|e| format!("Failed to serialize ApplicationStorage.json: {}", e))?;
            fs::write(&app_storage, serialized)
                .map_err(|e| format!("Failed to write ApplicationStorage.json: {}", e))?;
        }
    }

    for fp in collect_fingerprint_dbs(&backend_dir) {
        if let Ok(mut xml) = fs::read_to_string(&fp) {
            let mut changed = false;
            for key in DISABLE_OVERRIDE_KEYS {
                let needle = format!("<{}>1</{}>", key, key);
                let replacement = format!("<{}>0</{}>", key, key);
                let count = xml.matches(&needle).count();
                if count > 0 {
                    xml = xml.replace(&needle, &replacement);
                    flags_cleared += count;
                    changed = true;
                }
            }
            if unlock_ops_supported {
                for (needle, replacement) in [
                    (
                        "<IsAutomatable>0</IsAutomatable>",
                        "<IsAutomatable>1</IsAutomatable>",
                    ),
                    (
                        "<DLSS_Override_No_OPS>0</DLSS_Override_No_OPS>",
                        "<DLSS_Override_No_OPS>1</DLSS_Override_No_OPS>",
                    ),
                ] {
                    let count = xml.matches(needle).count();
                    if count > 0 {
                        xml = xml.replace(needle, replacement);
                        flags_cleared += count;
                        changed = true;
                    }
                }
            }
            if changed {
                let bak = fp.with_extension("db.dlss_manager.bak");
                if !bak.exists() {
                    let _ = fs::copy(&fp, &bak);
                }
                let _ = fs::write(&fp, xml);
            }
        }
    }

    Ok((flags_cleared, apps_unlocked.len()))
}

/// Restarts NVIDIA Container LocalSystem service (`NvContainerLocalSystem`) so NVIDIA App
/// picks up `ApplicationStorage.json` changes without requiring a full reboot.
/// Re-applies the whitelist while the service is stopped so a shutdown flush cannot revert flags.
pub fn restart_nvidia_services() -> Result<String, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let stop_out = Command::new("net")
            .args(["stop", "NvContainerLocalSystem", "/y"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to stop NvContainerLocalSystem: {}", e))?;

        if stop_out.status.success()
            || String::from_utf8_lossy(&stop_out.stderr).contains("2182")
            || String::from_utf8_lossy(&stop_out.stderr).contains("3521")
        {
            // Ensure ApplicationStorage.json & fingerprint.db stay unlocked after container shutdown flush
            let _ = apply_nvidia_app_whitelist(true);
            std::thread::sleep(std::time::Duration::from_millis(500));
            let start_out = Command::new("net")
                .args(["start", "NvContainerLocalSystem"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Failed to start NvContainerLocalSystem: {}", e))?;
            if start_out.status.success() {
                return Ok("NVIDIA Container Dienst (NvContainerLocalSystem) neu gestartet.".to_string());
            }
        }

        // Fall back to UAC-elevated Start-Process when not running as administrator
        let ps_cmd = "Start-Process -FilePath 'cmd.exe' -ArgumentList '/c net stop NvContainerLocalSystem /y & timeout /t 1 /nobreak >nul & net start NvContainerLocalSystem' -Verb RunAs -WindowStyle Hidden -Wait";
        let elevated = Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to elevate service restart: {}", e))?;

        if elevated.status.success() {
            let _ = apply_nvidia_app_whitelist(true);
            Ok("NVIDIA Container Dienst (NvContainerLocalSystem) mit Admin-Rechten neu gestartet.".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&elevated.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                "UAC-Bestätigung abgebrochen oder Administrator-Rechte erforderlich (alternativ PC neu starten).".to_string()
            } else {
                stderr
            })
        }
    }
    #[cfg(not(windows))]
    {
        Err("Only supported on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn counts_and_unlocks_ops_locks_even_when_override_flags_are_zero() {
        let mut doc = json!({
            "Applications": [
                {
                    "Application": {
                        "DisplayName": "Star Citizen",
                        "Disable_FG_Override": false,
                        "Disable_RR_Override": false,
                        "Disable_SR_Override": false,
                        "Disable_RR_Model_Override": false,
                        "Disable_SR_Model_Override": false,
                        "IsOpsSupported": false
                    }
                }
            ]
        });

        let mut blocked = 0;
        let mut ops = 0;
        let mut apps = BTreeSet::new();
        count_blocked_in_json(&doc, &mut blocked, &mut ops, &mut apps);
        assert_eq!(blocked, 0);
        assert_eq!(ops, 1);
        assert!(apps.contains("Star Citizen"));

        let mut modified = 0;
        let mut modified_apps = BTreeSet::new();
        unlock_json_recursive(&mut doc, true, &mut modified, &mut modified_apps);
        assert_eq!(modified, 1);
        assert!(modified_apps.contains("Star Citizen"));

        let mut blocked_after = 0;
        let mut ops_after = 0;
        let mut apps_after = BTreeSet::new();
        count_blocked_in_json(&doc, &mut blocked_after, &mut ops_after, &mut apps_after);
        assert_eq!(blocked_after, 0);
        assert_eq!(ops_after, 0);
        assert!(apps_after.is_empty());
    }
}
