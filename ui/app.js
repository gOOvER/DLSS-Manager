async function invoke(cmd, args = {}) {
  const fn =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI_INTERNALS__?.invoke;
  if (!fn) {
    throw new Error("Tauri IPC bridge not available");
  }
  return await fn(cmd, args);
}

function openExternalUrl(url) {
  const fn = window.__TAURI__?.opener?.openUrl;
  if (fn) {
    fn(url);
  } else {
    window.open(url, "_blank");
  }
}

async function openFolderDialog() {
  const fn = window.__TAURI__?.dialog?.open;
  if (!fn) {
    showToast("Folder dialog plugin not available — please paste the path manually.", true);
    return null;
  }
  return await fn({ directory: true, multiple: false });
}

const ALL_TECH_GROUPS = [
  { label: "DLSS", title: "DLSS Super Resolution" },
  { label: "DLSS-G", title: "DLSS Frame Generation" },
  { label: "DLSS-RR", title: "DLSS Ray Reconstruction" },
  { label: "DLSS 5", title: "DLSS 5 Neural Rendering" },
  { label: "Streamline", title: "NVIDIA Streamline" },
  { label: "DStorage", title: "Microsoft DirectStorage" },
  { label: "XeSS", title: "Intel XeSS" },
  { label: "XeSS-FG", title: "Intel XeSS Frame Gen" },
  { label: "XeLL", title: "Intel Xe Low Latency" },
];

let appState = {
  version: "1.0.0",
  games: [],
  backups: [],
  catalog: [],
  gpu: null,
  config: null,
  stats: null,
  logs: [],
  selectedLauncher: "all",
  statusFilter: "all", // 'all' | 'needs_update' | 'up_to_date' | 'has_backups'
  sortMode: "name_asc", // 'name_asc' | 'outdated_first' | 'name_desc' | 'dll_count'
  gridDensity: "comfortable", // 'compact' | 'comfortable' | 'large'
  searchQuery: "",
  activeModalGameId: null,
  selectMode: false,
  selectedGameIds: new Set(),
  appUpdateInfo: null,
};

const I18N = {
  de: {
    nav_hub: "Hub",
    nav_games: "Spiele",
    nav_backups: "Backups",
    nav_settings: "Einstellungen",
    nav_info: "Info",
    gpu_detecting: "ERKENNE GPU",
    gpu_scanning: "Scanne Hardware...",
    btn_scan_games: "🔍 Spiele scannen",
    btn_scanning: "⏳ Scanne...",
    search_placeholder: "Spiele durchsuchen...",
    update_available: "⬆️ Update verfügbar",
    dlss_overlay: "DLSS-Overlay",
    btn_logs: "📋 Logs",
    btn_update_all: "Alle aktualisieren",
    stat_installed_title: "Installierte Spiele",
    stat_installed_sub: "Mit kompatiblen Upscaler-DLLs",
    stat_needs_update_title: "Update verfügbar",
    stat_needs_update_sub: "Spiele mit neueren DLL-Versionen",
    stat_uptodate_title: "Aktuell",
    stat_uptodate_sub: "Auf neuester Upscaler-Version",
    stat_backups_title: "Gesicherte Spiele",
    stat_backups_sub: "Bereit für 1-Klick-Wiederherstellung",
    steam_api_hub_title: "Steam Web API Integration",
    steam_api_hub_sub: "Lädt offizielle Cover-Bilder & App-IDs für alle Launcher nach",
    steam_api_key_label: "Steam Web API Key",
    steam_get_key_link: "Key bei Steam erstellen ↗",
    steam_key_placeholder: "32-stelligen Steam Web API Key einfügen...",
    steam_id_label: "Steam64 ID (Wird automatisch aus lokaler Steam-Installation erkannt)",
    steam_id_placeholder: "z.B. 76561198000000000 (automatisch erkannt)",
    steam_api_feedback_default: "Gleicht eigene Steam-Spiele (`IPlayerService/GetOwnedGames`) + Steam-Store-Suche für alle Launcher ab.",
    btn_reresolve_covers: "🔄 Cover & App-IDs neu abgleichen",
    btn_validate_save_key: "✓ Prüfen & Speichern",
    hub_latest_versions: "Aktuell verfügbare Upscaler-Versionen",
    btn_check_newer_dlls: "🔄 Auf neue DLLs prüfen",
    hub_detected_launchers: "Erkannte Launcher",
    hub_detected_launchers_sub: "Klicke auf einen Launcher zum Filtern · Klicke auf 📂 für Pfade",
    sort_name_asc: "Name (A → Z)",
    sort_outdated_first: "Updates zuerst",
    sort_name_desc: "Name (Z → A)",
    sort_dll_count: "Meiste Technologien",
    btn_select_outdated: "Veraltete auswählen",
    btn_select_all_visible: "Alle sichtbaren auswählen",
    btn_clear_selection: "Auswahl aufheben",
    btn_update_selected: "Ausgewählte aktualisieren",
    backups_title: "Automatische Backups & Wiederherstellung",
    backups_sub: "Originale Spiel-DLLs werden vor jedem Update automatisch gesichert. Stelle ein komplettes Spiel oder einzelne Technologien mit einem Klick wieder her.",
    settings_safety_title: "Update- & Sicherheitseinstellungen",
    settings_lang_label: "Sprache / Language",
    settings_lang_sub: "Sprache der Benutzeroberfläche (Deutsch / English)",
    settings_autobackup_title: "Automatisches Backup vor dem Ersetzen",
    settings_autobackup_sub: "Originale Spiel-DLLs vor jedem Update im lokalen Backup-Tresor sichern",
    settings_skip_ac_title: "Anti-Cheat-geschützte Spiele bei Massen-Updates überspringen",
    settings_skip_ac_sub: "EasyAntiCheat-, BattlEye-, Vanguard- & Ricochet-Titel bei „Alle aktualisieren“ auslassen",
    settings_preview_title: "Experimentelle / Preview-DLLs erlauben",
    settings_preview_sub: "Vorabversionen von DLSS / FSR / XeSS bei der Update-Prüfung berücksichtigen",
    settings_hide_ignored_title: "Ignorierte Spiele in der Bibliothek ausblenden",
    settings_hide_ignored_sub: "Als ignoriert markierte Spiele komplett verstecken, sofern der Filter nicht aktiv ist",
    settings_steam_title: "Steam Web API & Cover-Auflösung",
    settings_steam_sub: "Hinterlege deinen Steam Web API Key, um Steam-App-IDs und hochauflösende Cover-Bilder automatisch zuzuordnen.",
    btn_save_steam_api: "💾 Steam API speichern",
    settings_db_title: "☁️ Spiele-Kompatibilitäts- & Einstellungs-Datenbank (dlssdb.goover.dev)",
    settings_db_sub: "Online-Datenbank für Upscaler-Kompatibilität, Anti-Cheat-Hinweise und empfohlene DLSS-Presets.",
    settings_db_key_label: "Persönlicher DLSS-DB API-Key",
    settings_db_create_key: "API-Key erstellen ↗",
    settings_db_url_label: "Datenbank-Endpunkt-URL (games.json)",
    settings_db_auto_report: "Gescannte Spiele & erkannte Technologien automatisch an dlssdb.goover.dev senden, sobald ein API-Key hinterlegt ist",
    btn_open_web_portal: "🌐 Web-Portal öffnen",
    btn_send_scanned_games: "📤 Gescannte Spiele an DB senden",
    btn_save_sync_db: "🔄 Key speichern & DB synchronisieren",
    settings_nv_presets_title: "Globale NVIDIA DLSS-Presets (Treiber-DRS)",
    settings_nv_presets_sub: "Globale DLSS 4 / 4.5 Treiber-Presets (nvapi64.dll). Über das Kartenmenü (⋮) können Presets auch pro Spiel überschrieben werden.",
    hub_nvapp_whitelist_title: "🛡️ NVIDIA App Override-Status (Treiber-DRS & Whitelist)",
    settings_nvapp_whitelist_title: "NVIDIA App Override-Whitelist (für nicht unterstützte Spiele)",
    btn_nvapp_whitelist: "🔓 Alle Spiele freischalten",
    btn_restart_nv_services: "🔄 NVIDIA Dienst neu starten",
    settings_tech_scope_title: "Aktivierte Upscaler-Technologien",
    settings_tech_scope_sub: "Wähle aus, welche Technologiegruppen auf Updates geprüft und bei Massen-Updates berücksichtigt werden sollen.",
    settings_custom_folders_title: "Eigene Spieleordner",
    settings_custom_folders_sub: "Füge eigene Verzeichnisse hinzu, wenn du Standalone-Spiele außerhalb der Standard-Launcher speicherst.",
    custom_folder_placeholder: "z.B. D:\\Games oder E:\\Standalone",
    btn_browse_folder: "📂 Durchsuchen",
    btn_add_path: "Pfad hinzufügen",
    settings_ignored_title: "Ignorierte & Blacklist-Spiele",
    settings_ignored_sub: "Verwalte vom Update ausgeschlossene Spiele oder füge Ordnernamen zur Blacklist hinzu.",
    blacklist_placeholder: "Spielname für Blacklist eingeben (z.B. Valorant)",
    btn_add_blacklist: "Zur Blacklist hinzufügen",
    "info.subtitle": "Universeller Upscaler- & Frame-Generation-Manager für NVIDIA DLSS 4.5 / 5, Intel XeSS 2, NVIDIA Streamline & Microsoft DirectStorage.",
    "info.github_repo": "⭐ GitHub Repository",
    "info.wiki": "📚 Wiki & Dokumentation",
    "info.releases": "📦 Releases & Changelog",
    "info.report_issue": "🐞 Fehler melden",
    "info.diag_title": "System- & Speicher-Diagnose",
    "info.open_appdata": "📂 AppData-Ordner öffnen",
    "info.open_cache": "📂 DLL-Cache öffnen",
    "info.open_backups": "📂 Backup-Tresor öffnen",
    "info.check_update": "🔄 Nach App-Update suchen",
    "info.catalog_title": "Unterstützte Upscaler-Technologien & Katalog",
    "info.catalog_desc": "Aktive Manifest-Versionen kompatibel mit deiner erkannten GPU-Hardware:",
    "info.launchers_safety_title": "Unterstützte Launcher & Sicherheitsarchitektur",
    "info.launchers_sub": "9 Automatisch erkannte Spiele-Launcher",
    "info.launchers_list": "• <strong>Steam</strong> (Registry + <code>libraryfolders.vdf</code> + <code>appmanifest_*.acf</code> + <code>loginusers.vdf</code>)<br />• <strong>Star Citizen</strong> (<code>%APPDATA%\\rsilauncher\\logs\\log.log</code> + <code>LIVE</code>, <code>PTU</code>, <code>EPTU</code>, <code>TECH-PREVIEW</code>, <code>HOTFIX</code>)<br />• <strong>Battle.net</strong> (<code>product.db</code> Protobuf + Windows Uninstall Registry)<br />• <strong>Epic Games</strong> (<code>ProgramData\\Epic\\...\\Manifests\\*.item</code>)<br />• <strong>GOG Galaxy</strong>, <strong>EA App / Origin</strong>, <strong>Ubisoft Connect</strong>, <strong>Xbox / Game Pass</strong> &amp; Eigene Ordner",
    "info.safety_sub": "Integritäts- & Anti-Cheat-Schutz",
    "info.safety_list": "• <strong>PE-Binärprüfung</strong>: Jede heruntergeladene DLL wird vor dem Kopieren auf gültige <code>MZ</code>-Header und Win32 <code>VS_FIXEDFILEINFO</code>-Versionsressourcen geprüft.<br />• <strong>Wasserzeichen-Schutz</strong>: Erkennt und warnt automatisch vor fehlerhaften DLSS <code>310.7.0</code>-Builds.<br />• <strong>Anti-Cheat-Erkennung</strong>: Markiert automatisch Spiele mit EasyAntiCheat, BattlEye, Vanguard &amp; Ricochet.",
  },
  en: {
    nav_hub: "Hub",
    nav_games: "Games",
    nav_backups: "Backups",
    nav_settings: "Settings",
    nav_info: "Info",
    gpu_detecting: "DETECTING GPU",
    gpu_scanning: "Scanning hardware...",
    btn_scan_games: "🔍 Scan Games",
    btn_scanning: "⏳ Scanning...",
    search_placeholder: "Search games by title, launcher, or DLL...",
    update_available: "⬆️ Update Available",
    dlss_overlay: "DLSS Overlay",
    btn_logs: "📋 Logs",
    btn_update_all: "Update All",
    stat_installed_title: "Installed Games",
    stat_installed_sub: "With compatible upscaler DLLs",
    stat_needs_update_title: "Needs Update",
    stat_needs_update_sub: "Games with newer DLL versions",
    stat_uptodate_title: "Up to Date",
    stat_uptodate_sub: "Running latest upscaler builds",
    stat_backups_title: "Protected Games",
    stat_backups_sub: "Ready for 1-click restore",
    steam_api_hub_title: "Steam Web API Integration",
    steam_api_hub_sub: "Auto-resolves official cover artwork & App IDs across all launchers",
    steam_api_key_label: "Steam Web API Key",
    steam_get_key_link: "Get key from Steam ↗",
    steam_key_placeholder: "Paste 32-character Steam Web API Key...",
    steam_id_label: "Steam64 ID (Auto-detected from local Steam installation)",
    steam_id_placeholder: "e.g. 76561198000000000 (auto-detected)",
    steam_api_feedback_default: "Matches owned Steam games (`IPlayerService/GetOwnedGames`) + Steam Store search for all launchers.",
    btn_reresolve_covers: "🔄 Re-Resolve Covers & App IDs",
    btn_validate_save_key: "✓ Validate & Save Key",
    hub_latest_versions: "Latest Available Upscaler Versions",
    btn_check_newer_dlls: "🔄 Check for Newer DLLs",
    hub_detected_launchers: "Detected Launchers",
    hub_detected_launchers_sub: "Click a launcher to filter · Click 📂 to inspect paths",
    sort_name_asc: "Name (A → Z)",
    sort_outdated_first: "Needs Update First",
    sort_name_desc: "Name (Z → A)",
    sort_dll_count: "Most Technologies",
    btn_select_outdated: "Select Outdated",
    btn_select_all_visible: "Select All Visible",
    btn_clear_selection: "Clear Selection",
    btn_update_selected: "Update Selected",
    backups_title: "Automatic Backups & Restore",
    backups_sub: "Original game DLLs are backed up before every update. Restore an entire game or individual technologies with one click.",
    settings_safety_title: "Update & Safety Preferences",
    settings_lang_label: "Language / Sprache",
    settings_lang_sub: "User interface language (Deutsch / English)",
    settings_autobackup_title: "Automatic Backup Before Replace",
    settings_autobackup_sub: "Save original game DLLs to local vault before applying updates",
    settings_skip_ac_title: "Skip Anti-Cheat Protected Games in Bulk Updates",
    settings_skip_ac_sub: "Exclude EasyAntiCheat, BattlEye, Vanguard & Ricochet titles from Update All",
    settings_preview_title: "Allow Experimental / Preview DLLs",
    settings_preview_sub: "Include pre-release DLSS / FSR / XeSS builds when checking for updates",
    settings_hide_ignored_title: "Hide Ignored Games from Library",
    settings_hide_ignored_sub: "Completely hide games marked as Ignored unless filter is toggled",
    settings_steam_title: "Steam Web API & Artwork Resolution",
    settings_steam_sub: "Configure your Steam Web API key to automatically match Steam App IDs and high-res cover art.",
    btn_save_steam_api: "💾 Save Steam API",
    settings_db_title: "☁️ Game Compatibility & Settings Database (dlssdb.goover.dev)",
    settings_db_sub: "Cloud-synced compatibility database for upscaler support, anti-cheat rules, and optimal DLSS presets.",
    settings_db_key_label: "Personal DLSS-DB API Key",
    settings_db_create_key: "Create API Key ↗",
    settings_db_url_label: "Database Endpoint URL (games.json)",
    settings_db_auto_report: "Automatically report scanned games & detected technologies to dlssdb.goover.dev when API Key is set",
    btn_open_web_portal: "🌐 Open Web Portal",
    btn_send_scanned_games: "📤 Send Scanned Games to DB",
    btn_save_sync_db: "🔄 Save Key & Sync DB",
    settings_nv_presets_title: "Global NVIDIA DLSS Presets (Driver DRS)",
    settings_nv_presets_sub: "Global DLSS 4 / 4.5 driver overrides (nvapi64.dll). You can also override presets per game from any game card menu (⋮).",
    hub_nvapp_whitelist_title: "🛡️ NVIDIA App Override Status (Driver DRS & Whitelist)",
    settings_nvapp_whitelist_title: "NVIDIA App Override Whitelist (Unlock Unsupported Games)",
    btn_nvapp_whitelist: "🔓 Unlock All Games",
    btn_restart_nv_services: "🔄 Restart NVIDIA Service",
    settings_tech_scope_title: "Enabled Upscaler Technologies",
    settings_tech_scope_sub: "Choose which technology groups should be checked for updates and included in batch updates.",
    settings_custom_folders_title: "Custom Game Folders",
    settings_custom_folders_sub: "Add custom directories if you keep standalone games outside standard launchers.",
    custom_folder_placeholder: "e.g. D:\\Games or E:\\Standalone",
    btn_browse_folder: "📂 Browse",
    btn_add_path: "Add Path",
    settings_ignored_title: "Ignored & Blacklisted Games",
    settings_ignored_sub: "Manage games excluded from updates or add custom folder/title patterns to the blacklist.",
    blacklist_placeholder: "Enter game title to blacklist (e.g. Valorant)",
    btn_add_blacklist: "Add to Blacklist",
    "info.subtitle": "Universal Upscaler & Frame Generation Manager for NVIDIA DLSS 4.5 / 5, Intel XeSS 2, NVIDIA Streamline & Microsoft DirectStorage.",
    "info.github_repo": "⭐ GitHub Repository",
    "info.wiki": "📚 Wiki & Documentation",
    "info.releases": "📦 Releases & Changelog",
    "info.report_issue": "🐞 Report Issue",
    "info.diag_title": "System & Storage Diagnostics",
    "info.open_appdata": "📂 Open AppData Folder",
    "info.open_cache": "📂 Open DLL Cache",
    "info.open_backups": "📂 Open Backups Vault",
    "info.check_update": "🔄 Check App Update",
    "info.catalog_title": "Supported Upscaler Technologies & Catalog",
    "info.catalog_desc": "Active manifest versions compatible with your detected GPU hardware:",
    "info.launchers_safety_title": "Supported Launchers & Safety Architecture",
    "info.launchers_sub": "9 Auto-Detected Game Launchers",
    "info.launchers_list": "• <strong>Steam</strong> (Registry + <code>libraryfolders.vdf</code> + <code>appmanifest_*.acf</code> + <code>loginusers.vdf</code>)<br />• <strong>Star Citizen</strong> (<code>%APPDATA%\\rsilauncher\\logs\\log.log</code> + <code>LIVE</code>, <code>PTU</code>, <code>EPTU</code>, <code>TECH-PREVIEW</code>, <code>HOTFIX</code>)<br />• <strong>Battle.net</strong> (<code>product.db</code> Protobuf + Windows Uninstall Registry)<br />• <strong>Epic Games</strong> (<code>ProgramData\\Epic\\...\\Manifests\\*.item</code>)<br />• <strong>GOG Galaxy</strong>, <strong>EA App / Origin</strong>, <strong>Ubisoft Connect</strong>, <strong>Xbox / Game Pass</strong> &amp; Custom Folders",
    "info.safety_sub": "Integrity & Anti-Cheat Protection",
    "info.safety_list": "• <strong>PE Binary Verification</strong>: Every downloaded DLL is validated for <code>MZ</code> headers and Win32 <code>VS_FIXEDFILEINFO</code> version resources before touching a game directory.<br />• <strong>Watermark Protection</strong>: Automatically detects and warns about faulty DLSS <code>310.7.0</code> builds.<br />• <strong>Anti-Cheat Detection</strong>: Automatically flags EasyAntiCheat, BattlEye, Vanguard &amp; Ricochet games.",
  },
};

function getLang() {
  const l = (appState.config?.language || "de").toLowerCase();
  return l === "en" ? "en" : "de";
}

function isDe() {
  return getLang() === "de";
}

function t(key) {
  const lang = getLang();
  return I18N[lang]?.[key] ?? I18N.de?.[key] ?? key;
}

function applyLanguage() {
  const lang = getLang();
  document.documentElement.lang = lang;

  const langBtnLabel = document.getElementById("lang-toggle-label");
  if (langBtnLabel) {
    langBtnLabel.innerHTML =
      lang === "de"
        ? `<svg width="18" height="12" viewBox="0 0 18 12" style="border-radius:2px;flex-shrink:0"><rect width="18" height="4" y="0" fill="#111"/><rect width="18" height="4" y="4" fill="#dd0000"/><rect width="18" height="4" y="8" fill="#ffce00"/></svg> DE`
        : `<svg width="18" height="12" viewBox="0 0 18 12" style="border-radius:2px;flex-shrink:0"><rect width="18" height="12" fill="#012169"/><path d="M0,0 L18,12 M18,0 L0,12" stroke="#fff" stroke-width="2.2"/><path d="M0,0 L18,12 M18,0 L0,12" stroke="#c8102e" stroke-width="1.1"/><path d="M9,0 V12 M0,6 H18" stroke="#fff" stroke-width="3.6"/><path d="M9,0 V12 M0,6 H18" stroke="#c8102e" stroke-width="2"/></svg> EN`;
  }
  const cfgLang = document.getElementById("cfg-language");
  if (cfgLang && cfgLang.value !== lang) {
    cfgLang.value = lang;
  }

  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const k = el.getAttribute("data-i18n");
    if (k && I18N[lang]?.[k]) {
      el.textContent = I18N[lang][k];
    }
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    const k = el.getAttribute("data-i18n-html");
    if (k && I18N[lang]?.[k]) {
      el.innerHTML = I18N[lang][k];
    }
  });
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const k = el.getAttribute("data-i18n-placeholder");
    if (k && I18N[lang]?.[k]) {
      el.setAttribute("placeholder", I18N[lang][k]);
    }
  });
}

function showToast(message, isError = false) {
  const container = document.getElementById("toast-container");
  if (!container) return;
  const el = document.createElement("div");
  el.className = "toast";
  if (isError) el.style.borderLeftColor = "#ef4444";
  el.textContent = message;
  container.appendChild(el);
  setTimeout(() => el.remove(), 4200);
}

function escapeHtml(str) {
  return String(str || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeJsStr(str) {
  return String(str || "")
    .replace(/\\/g, "\\\\")
    .replace(/'/g, "\\'");
}

function applyThemeMode(mode) {
  const clean = mode === "light" ? "light" : "dark";
  document.documentElement.setAttribute("data-theme", clean);
  const icon = document.getElementById("theme-toggle-icon");
  if (icon) {
    icon.textContent = clean === "light" ? "🌙" : "☀️";
  }
}

function applyPayload(payload) {
  if (!payload) return;
  appState.version = payload.version || appState.version;
  appState.games = payload.games || [];
  appState.backups = payload.backups || [];
  appState.catalog = payload.catalog || [];
  appState.gpu = payload.gpu || appState.gpu;
  appState.config = payload.config || appState.config;
  appState.stats = payload.stats || appState.stats;
  if (payload.logs) {
    appState.logs = payload.logs;
  }

  if (appState.config?.theme_mode) {
    applyThemeMode(appState.config.theme_mode);
  }

  renderAll();
  if (appState.activeModalGameId) {
    const stillExists = appState.games.find((g) => g.id === appState.activeModalGameId);
    if (stillExists) {
      openGameDllModal(appState.activeModalGameId);
    } else {
      closeGameDllModal();
    }
  }
}

/**
 * Groups a game's supported DLLs by Technology Group (e.g. DLSS, DLSS-G, DLSS-RR, Streamline, DStorage, FSR 3/4, XeSS)
 */
function getGameTechGroups(game) {
  const groups = new Map();
  for (const d of game.dlls || []) {
    if (d.gpu_supported === false) continue;
    const key = d.type_label;
    if (!groups.has(key)) {
      groups.set(key, {
        typeLabel: key,
        version: d.version,
        latestVersion: d.latest_version,
        presetLabel: d.preset_label,
        updateAvailable: false,
        technologyEnabled: d.technology_enabled !== false,
        badBuildWarning: d.bad_build_warning || null,
        dlls: [],
      });
    }
    const g = groups.get(key);
    g.dlls.push(d);
    if (d.bad_build_warning && !g.badBuildWarning) {
      g.badBuildWarning = d.bad_build_warning;
    }
    if (d.update_available) {
      g.updateAvailable = true;
      g.version = d.version;
      if (d.latest_version) g.latestVersion = d.latest_version;
    }
  }
  return Array.from(groups.values());
}

function getGameUpdateCounts(game) {
  const groups = getGameTechGroups(game);
  const outdated = game.ignored || game.blacklisted
    ? 0
    : groups.filter((grp) => grp.updateAvailable).length;
  const current = groups.length - outdated;
  return { total: groups.length, outdated, current, groups };
}

function buildTechSummaryLine(game) {
  const groups = getGameTechGroups(game);
  return (
    groups
      .map((grp) => {
        const shortVer =
          grp.version && grp.version !== "Unknown"
            ? ` ${grp.version.split(".").slice(0, 3).join(".")}`
            : "";
        if (
          grp.typeLabel === "DLSS" ||
          grp.typeLabel === "DLSS 5" ||
          grp.typeLabel === "FSR 3/4" ||
          grp.typeLabel === "XeSS"
        ) {
          return `${grp.typeLabel}${shortVer}`;
        }
        return grp.typeLabel;
      })
      .join(" · ") || "Upscaler Ready"
  );
}

function renderAll() {
  applyLanguage();

  document.getElementById("app-version-badge").textContent = `v${appState.version}`;
  const infoVer = document.getElementById("info-version-badge");
  if (infoVer) infoVer.textContent = `v${appState.version}`;
  document.getElementById("nav-games-count").textContent = appState.games.length;

  const uniqueBackupGames = new Set(appState.backups.map((b) => b.game_id)).size;
  document.getElementById("nav-backups-count").textContent = uniqueBackupGames;

  const needsUpdateGames = appState.games.filter(
    (g) => !g.ignored && !g.blacklisted && getGameUpdateCounts(g).outdated > 0
  ).length;
  const upToDateGames = Math.max(
    0,
    appState.games.filter((g) => !g.ignored && !g.blacklisted).length - needsUpdateGames
  );

  document.getElementById("stat-total-games").textContent = appState.games.length;
  document.getElementById("stat-updates").textContent = needsUpdateGames;
  document.getElementById("stat-dll-games").textContent = upToDateGames;
  document.getElementById("stat-backups").textContent = uniqueBackupGames;
  document.getElementById("bulk-update-count").textContent = needsUpdateGames;

  if (appState.gpu) {
    const vendor = (appState.gpu.vendor || "GPU").toUpperCase();
    document.getElementById("gpu-vendor-badge").textContent = isDe()
      ? `ERKANNTE ${vendor} GPU`
      : `DETECTED ${vendor} GPU`;
    document.getElementById("gpu-name").textContent = appState.gpu.name;
    const drv = appState.gpu.driver_version
      ? isDe()
        ? ` · Treiber ${appState.gpu.driver_version}`
        : ` · Driver ${appState.gpu.driver_version}`
      : "";
    document.getElementById("gpu-arch").textContent = `${appState.gpu.architecture}${drv}`;

    const caps = document.getElementById("gpu-caps");
    caps.innerHTML = "";
    if (appState.gpu.supports_dlss) {
      caps.innerHTML += `<span class="cap-tag">DLSS SR</span>`;
      caps.innerHTML += `<span class="cap-tag">Streamline</span>`;
    }
    if (appState.gpu.supports_dlss_fg) caps.innerHTML += `<span class="cap-tag">DLSS Frame Gen</span>`;
    if (appState.gpu.supports_dlss_rr) caps.innerHTML += `<span class="cap-tag">Ray Recon</span>`;
    if (appState.gpu.supports_dlss_nr) caps.innerHTML += `<span class="cap-tag">DLSS 5 NR</span>`;
    if (appState.gpu.supports_fsr4) caps.innerHTML += `<span class="cap-tag">AMD FSR 3/4</span>`;
    if (appState.gpu.supports_xess) caps.innerHTML += `<span class="cap-tag">Intel XeSS 2</span>`;
    caps.innerHTML += `<span class="cap-tag">DirectStorage</span>`;

    const overlayPill = document.getElementById("dlss-overlay-pill");
    if (overlayPill) {
      overlayPill.style.display = vendor === "NVIDIA" ? "flex" : "none";
    }
    const nvPresetsCard = document.getElementById("settings-nvidia-presets-card");
    if (nvPresetsCard) {
      nvPresetsCard.style.display = vendor === "NVIDIA" ? "block" : "none";
    }
  }

  if (appState.config) {
    document.getElementById("toggle-dlss-overlay").checked = !!appState.config.dlss_indicator_enabled;
    document.getElementById("cfg-auto-backup").checked = !!appState.config.auto_backup;
    document.getElementById("cfg-skip-anticheat").checked = !!appState.config.skip_anti_cheat_games;
    document.getElementById("cfg-allow-preview").checked = !!appState.config.allow_preview_dlls;
    const hideIgnoredEl = document.getElementById("cfg-hide-ignored");
    if (hideIgnoredEl) hideIgnoredEl.checked = !!appState.config.hide_ignored_games;

    const srSel = document.getElementById("cfg-global-sr-preset");
    const rrSel = document.getElementById("cfg-global-rr-preset");
    const fgSel = document.getElementById("cfg-global-fg-preset");
    if (srSel) srSel.value = appState.config.global_dlss_preset || "Default";
    if (rrSel) rrSel.value = appState.config.global_dlss_rr_preset || "Default";
    if (fgSel) fgSel.value = appState.config.global_dlss_fg_preset || "Default";

    renderCustomFolders();
    renderSettingsTechScope();
    renderIgnoredAndBlacklistedSettings();
  }

  renderHubTechStrip();
  renderHubLaunchers();
  renderSteamApiStatus();
  renderStatusChips();
  renderLauncherSelect();
  renderBatchSelectionBar();
  renderGamesGrid();
  renderBackups();
  renderInfoPage();
}

function renderSteamApiStatus() {
  const key = appState.config?.steam_api_key || "";
  const sid = appState.config?.steam_id || "";
  const hasKey = key.trim().length > 0;

  const k2 = document.getElementById("settings-steam-api-key");
  const s2 = document.getElementById("settings-steam-id");
  if (k2 && document.activeElement !== k2) k2.value = key;
  if (s2 && document.activeElement !== s2) s2.value = sid;

  const pill2 = document.getElementById("settings-steam-status-pill");
  const label = hasKey
    ? isDe()
      ? `● Verbunden${sid ? ` · ID ${sid}` : ""}`
      : `● Connected${sid ? ` · ID ${sid}` : ""}`
    : isDe()
      ? "○ Nicht konfiguriert"
      : "○ Not configured";
  if (pill2) {
    pill2.textContent = label;
    pill2.classList.toggle("connected", hasKey);
  }

  const dbUrlEl = document.getElementById("settings-game-db-url");
  if (dbUrlEl && document.activeElement !== dbUrlEl) {
    dbUrlEl.value =
      appState.config?.game_db_url || "https://dlssdb.goover.dev/api/v1/games.json";
  }
  const dbKeyEl = document.getElementById("settings-game-db-api-key");
  if (dbKeyEl && document.activeElement !== dbKeyEl) {
    dbKeyEl.value = appState.config?.game_db_api_key || "";
  }
  const autoRepEl = document.getElementById("cfg-auto-report-db");
  if (autoRepEl) {
    autoRepEl.checked = appState.config?.auto_report_games_to_db !== false;
  }
  const dbPill = document.getElementById("settings-game-db-status-pill");
  if (dbPill) {
    const cnt = appState.stats?.game_db_count || 0;
    const hasDbKey = !!(appState.config?.game_db_api_key || "").trim();
    dbPill.textContent = hasDbKey
      ? isDe()
        ? `● API-Key aktiv · ${cnt} Profile geladen`
        : `● API Key Active · ${cnt} Profiles Loaded`
      : isDe()
        ? `● ${cnt} Profile (Lokal / Seed-DB)`
        : `● ${cnt} Profiles (Local / Seed DB)`;
  }
}

window.toggleRevealGameDbKey = function () {
  const el = document.getElementById("settings-game-db-api-key");
  if (!el) return;
  el.type = el.type === "password" ? "text" : "password";
};

window.syncGameDatabaseNow = async function () {
  const customUrl = document.getElementById("settings-game-db-url")?.value || null;
  const apiKey = document.getElementById("settings-game-db-api-key")?.value ?? null;
  try {
    showToast(
      isDe()
        ? "Synchronisiere Spiele-Kompatibilitäts- & Einstellungs-Datenbank..."
        : "Syncing Game Compatibility & Settings Database..."
    );
    const payload = await invoke("sync_game_database", { customUrl, apiKey });
    applyPayload(payload);
    showToast(
      isDe()
        ? `✓ Datenbank synchronisiert (${payload.stats?.game_db_count || 0} Profile aktiv)!`
        : `✓ Game Database synced (${payload.stats?.game_db_count || 0} profiles active)!`
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

window.reportGamesToDbNow = async function (gameId = null) {
  if (gameId) {
    openReportGameModal(gameId);
    return;
  }
  try {
    showToast(
      isDe()
        ? "Sende alle gescannten Spiele & erkannten Technologien an dlssdb.goover.dev..."
        : "Sending all scanned games & detected technologies to dlssdb.goover.dev..."
    );
    const msg = await invoke("report_games_to_db", {
      gameId: null,
      worksStatus: null,
      recommendedSettings: null,
      recommendedSrPreset: null,
      notes: null,
    });
    showToast(`✓ ${msg}`);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.openReportGameModal = function (gameId) {
  const game = (appState.games || []).find((g) => g.id === gameId);
  if (!game) return;

  const backdrop = document.getElementById("secondary-modal-backdrop");
  const content = document.getElementById("secondary-modal-content");
  if (!backdrop || !content) return;

  const techs = Array.from(new Set((game.dlls || []).map((d) => d.type_label)));
  const existingSettings = (game.db_profile?.recommended_settings || []).join("\n");
  const defaultPreset = game.dlss_sr_preset || game.db_profile?.recommended_sr_preset || "";
  const de = isDe();

  const techBadgesHtml =
    techs.length > 0
      ? techs.map((t) => `<span class="dll-badge">${escapeHtml(t)}</span>`).join("")
      : `<span class="dll-badge">DLSS</span>`;

  content.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>${de ? "📤 Spiel an Online-DB melden" : "📤 Report Game to Online-DB"} — ${escapeHtml(game.name)}</h3>
        <div class="muted" style="font-size:0.78rem;margin-top:3px">
          ${de
      ? `Melde Kompatibilität & optimale Einstellungen für <strong>${escapeHtml(game.name)}</strong> an <code>dlssdb.goover.dev</code>. <strong>Pflichtangabe:</strong> Geht oder Geht nicht — alles andere ist optional.`
      : `Report compatibility & optimal settings for <strong>${escapeHtml(game.name)}</strong> to <code>dlssdb.goover.dev</code>. <strong>Required:</strong> Works or Doesn't Work — everything else is optional.`
    }
        </div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>

    <div class="modal-body" style="display:flex;flex-direction:column;gap:16px">
      <div class="dll-row" style="padding:12px 16px">
        <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap">
          <img src="assets/icons/${escapeHtml(game.launcher_icon || "folder.svg")}" style="width:20px;height:20px;flex-shrink:0" alt="" />
          <strong style="font-size:0.92rem">${escapeHtml(game.name)}</strong>
          <span class="default-preset-chip">${escapeHtml(game.launcher_name)}</span>
          ${game.anti_cheat
      ? `<span class="anticheat-badge">🛡️ ${escapeHtml(game.anti_cheat)}</span>`
      : ""
    }
        </div>
        <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
          <span class="muted" style="font-size:0.76rem;margin-right:2px">${de ? "Erkannte DLLs:" : "Detected DLLs:"}</span>
          ${techBadgesHtml}
        </div>
      </div>

      <div class="preset-field">
        <label style="font-size:0.82rem;font-weight:700;color:var(--text-primary)">
          ${de
      ? `Status: Funktioniert das Spiel mit DLSS / Upscaler-Update? <span style="color:#a3e635">* (Pflichtfeld)</span>`
      : `Status: Does this game work with DLSS / Upscaler updates? <span style="color:#a3e635">* (Required)</span>`
    }
        </label>
        <input type="hidden" id="modal-rep-works-status" value="works" />
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:4px">
          <button
            type="button"
            id="modal-rep-btn-works"
            class="btn"
            onclick="setModalReportStatus('works')"
            style="justify-content:center;padding:12px 14px;border:1px solid rgba(118,185,0,0.75);background:rgba(118,185,0,0.18);color:#d9f99d;font-weight:700;font-size:0.86rem"
          >
            ${de ? "✅ Geht (Funktioniert)" : "✅ Works"}
          </button>
          <button
            type="button"
            id="modal-rep-btn-broken"
            class="btn btn-secondary"
            onclick="setModalReportStatus('broken')"
            style="justify-content:center;padding:12px 14px;opacity:0.78;font-size:0.86rem"
          >
            ${de ? "❌ Geht nicht (Probleme / Blockiert)" : "❌ Doesn't Work (Broken / Blocked)"}
          </button>
        </div>
      </div>

      <div class="preset-field">
        <label>
          ${de ? "Empfohlenes DLSS Preset" : "Recommended DLSS Preset"} <span style="opacity:0.75">(Optional)</span>
        </label>
        <select id="modal-rep-preset" class="toolbar-select" style="width:100%">
          <option value="" ${!defaultPreset || defaultPreset === "Default" ? "selected" : ""}>${de ? "— Keine Angabe (Optional) —" : "— Not specified (Optional) —"}</option>
          <option value="Preset K" ${defaultPreset === "Preset K" ? "selected" : ""}>Preset K (DLSS 4.5 Transformer Gen 2 · Standard-Empfehlung)</option>
          <option value="Preset J" ${defaultPreset === "Preset J" ? "selected" : ""}>Preset J (DLSS 4.0 Transformer Gen 1)</option>
          <option value="Preset L" ${defaultPreset === "Preset L" ? "selected" : ""}>Preset L (DLSS 4.5 Transformer Gen 2 · 4K Performance)</option>
          <option value="Preset M" ${defaultPreset === "Preset M" ? "selected" : ""}>Preset M (DLSS 4.5 Transformer Gen 2 · Max Quality / DLAA)</option>
          <option value="Latest" ${defaultPreset === "Latest" ? "selected" : ""}>Latest (Vom NVIDIA-Treiber gesteuert)</option>
          <option value="Preset E" ${defaultPreset === "Preset E" ? "selected" : ""}>Preset E (DLSS 3.7+ CNN)</option>
          <option value="Preset C" ${defaultPreset === "Preset C" ? "selected" : ""}>Preset C (CNN)</option>
        </select>
      </div>

      <div class="preset-field">
        <label>
          ${de ? "Empfohlene In-Game Einstellungen" : "Recommended In-Game Settings"} <span style="opacity:0.75">${de ? "(Optional, eine Einstellung pro Zeile)" : "(Optional, one setting per line)"}</span>
        </label>
        <textarea
          id="modal-rep-settings"
          class="toolbar-select"
          rows="4"
          style="width:100%"
          placeholder="${de ? "Optional, z.B.&#10;Graphics Renderer: Vulkan&#10;Upscaling: DLSS Quality" : "Optional, e.g.&#10;Graphics Renderer: Vulkan&#10;Upscaling: DLSS Quality"}"
        >${escapeHtml(existingSettings)}</textarea>
      </div>

      <div class="preset-field">
        <label>
          ${de ? "Hinweise / Fehlerbeschreibung" : "Notes / Issue Description"} <span style="opacity:0.75">(Optional)</span>
        </label>
        <input
          type="text"
          id="modal-rep-notes"
          class="toolbar-select"
          style="width:100%"
          placeholder="${de ? "Optional: z.B. Läuft super / Anti-Cheat blockiert / Flackern bei..." : "Optional: e.g. Runs great / Anti-Cheat blocks swap / Flickering..."}"
        />
      </div>
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="closeSecondaryModal()">${de ? "Abbrechen" : "Cancel"}</button>
      <button class="btn btn-primary" onclick="submitModalGameReport('${escapeJsStr(game.id)}')">
        ${de ? "📤 Jetzt an Datenbank melden" : "📤 Submit Report to Database"}
      </button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.setModalReportStatus = function (status) {
  const val = status === "broken" ? "broken" : "works";
  const hidden = document.getElementById("modal-rep-works-status");
  if (hidden) hidden.value = val;
  const btnW = document.getElementById("modal-rep-btn-works");
  const btnB = document.getElementById("modal-rep-btn-broken");
  if (!btnW || !btnB) return;
  if (val === "works") {
    btnW.style.border = "1px solid rgba(118,185,0,0.75)";
    btnW.style.background = "rgba(118,185,0,0.2)";
    btnW.style.color = "#d9f99d";
    btnW.style.fontWeight = "700";
    btnW.style.opacity = "1";
    btnB.style.border = "";
    btnB.style.background = "";
    btnB.style.color = "";
    btnB.style.fontWeight = "500";
    btnB.style.opacity = "0.75";
  } else {
    btnB.style.border = "1px solid rgba(239,68,68,0.75)";
    btnB.style.background = "rgba(239,68,68,0.2)";
    btnB.style.color = "#fca5a5";
    btnB.style.fontWeight = "700";
    btnB.style.opacity = "1";
    btnW.style.border = "";
    btnW.style.background = "";
    btnW.style.color = "";
    btnW.style.fontWeight = "500";
    btnW.style.opacity = "0.75";
  }
};

window.submitModalGameReport = async function (gameId) {
  const worksStatus = document.getElementById("modal-rep-works-status")?.value || "works";
  const recommendedSrPreset = document.getElementById("modal-rep-preset")?.value || null;
  const recommendedSettings = document.getElementById("modal-rep-settings")?.value?.trim() || null;
  const notes = document.getElementById("modal-rep-notes")?.value?.trim() || null;

  try {
    showToast(isDe() ? "Sende Meldung an dlssdb.goover.dev..." : "Submitting report to dlssdb.goover.dev...");
    const msg = await invoke("report_games_to_db", {
      gameId,
      worksStatus,
      recommendedSettings,
      recommendedSrPreset,
      notes,
    });
    closeSecondaryModal();
    showToast(`✓ ${msg}`);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.applyRecommendedProfileNow = async function (gameId) {
  try {
    const payload = await invoke("apply_recommended_game_profile", { gameId });
    applyPayload(payload);
    showToast(
      isDe()
        ? "✨ Empfohlene DLSS-Presets aus der Datenbank angewendet!"
        : "✨ Applied recommended DLSS presets from Game Database!"
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

window.toggleRevealSteamApiKey = function () {
  const el = document.getElementById("settings-steam-api-key") || document.getElementById("input-steam-api-key");
  if (!el) return;
  el.type = el.type === "password" ? "text" : "password";
};

window.validateAndSaveSteamApiNow = async function () {
  const apiKey = document.getElementById("input-steam-api-key")?.value || "";
  const steamId = document.getElementById("input-steam-id")?.value || null;
  await runSteamValidation(apiKey, steamId);
};

window.validateAndSaveSteamApiFromSettings = async function () {
  const apiKey = document.getElementById("settings-steam-api-key")?.value || "";
  const steamId = document.getElementById("settings-steam-id")?.value || null;
  await runSteamValidation(apiKey, steamId);
};

async function runSteamValidation(apiKey, steamId) {
  const fb = document.getElementById("steam-api-feedback");
  if (fb) fb.textContent = isDe() ? "Prüfe Steam Web API Key..." : "Validating Steam Web API key...";
  try {
    const res = await invoke("validate_and_save_steam_api", {
      apiKey,
      steamId,
    });
    const initial = await invoke("get_initial_state");
    applyPayload(initial);
    if (fb) fb.textContent = res.message || "";
    showToast(res.message || (isDe() ? "Steam API aktualisiert." : "Steam API updated."), !res.valid && apiKey.trim().length > 0);
  } catch (e) {
    if (fb) fb.textContent = String(e);
    showToast(String(e), true);
  }
}

window.reresolveGamesViaSteamNow = async function () {
  const fb = document.getElementById("steam-api-feedback");
  if (fb)
    fb.textContent = isDe()
      ? "Gleiche App-IDs & Cover-Bilder über Steam API neu ab..."
      : "Re-resolving game App IDs & artwork via Steam API...";
  try {
    showToast(isDe() ? "Gleiche Spiele über Steam Web API ab..." : "Re-resolving games via Steam Web API...");
    const payload = await invoke("reresolve_games_via_steam_api");
    applyPayload(payload);
    if (fb)
      fb.textContent = isDe()
        ? "Abgleich abgeschlossen! Cover und App-IDs aktualisiert."
        : "Re-resolution complete! Artwork and App IDs updated.";
    showToast(isDe() ? "✓ Steam API Abgleich abgeschlossen!" : "✓ Steam API re-resolution complete!");
  } catch (e) {
    if (fb) fb.textContent = String(e);
    showToast(String(e), true);
  }
};

function renderInfoPage() {
  const diagGrid = document.getElementById("info-diagnostics-grid");
  if (diagGrid) {
    const gpu = appState.gpu || {};
    const stats = appState.stats || {};
    const de = isDe();
    const items = [
      { label: de ? "Erkannte GPU" : "Detected GPU", val: gpu.name || (de ? "Unbekannt" : "Unknown") },
      { label: de ? "GPU-Architektur" : "GPU Architecture", val: gpu.architecture || "—" },
      { label: de ? "Grafiktreiber" : "Graphics Driver", val: gpu.driver_version || "—" },
      { label: de ? "Empfohlenes Preset" : "Recommended Preset", val: gpu.recommended_preset || "—" },
      {
        label: de ? "Zwischengespeicherte DLLs" : "Cached Repository DLLs",
        val: `${stats.cached_dlls || 0} ${de ? "Dateien" : "files"} (${(stats.cache_size_mb || 0).toFixed(1)} MB)`,
      },
      {
        label: de ? "Aktive Spiel-Backups" : "Active Game Backups",
        val: `${stats.total_backups || 0} ${de ? "Dateien" : "files"} (${(stats.backup_size_mb || 0).toFixed(1)} MB)`,
      },
      {
        label: de ? "Spiele-Datenbank" : "Game Settings DB",
        val: `${stats.game_db_count || 0} ${de ? "Profile (Cloudflare Pages + Lokal)" : "Profiles (Cloudflare Pages + Local)"}`,
      },
      { label: "Steam64 ID", val: appState.config?.steam_id || (de ? "Automatisch beim Scan erkannt" : "Auto-detected on scan") },
      { label: de ? "AppData-Speicherpfad" : "AppData Storage Path", val: stats.app_data_dir || "%LOCALAPPDATA%\\gOOvER\\DLSS-Manager" },
    ];
    diagGrid.innerHTML = items
      .map(
        (i) => `
      <div class="info-kv-item" title="${escapeHtml(i.val)}">
        <span class="info-kv-label">${escapeHtml(i.label)}</span>
        <span class="info-kv-val">${escapeHtml(i.val)}</span>
      </div>`
      )
      .join("");
  }

  const catTable = document.getElementById("info-catalog-table");
  if (catTable) {
    const grouped = new Map();
    for (const c of appState.catalog || []) {
      if (!grouped.has(c.type_label)) {
        grouped.set(c.type_label, {
          typeLabel: c.type_label,
          version: c.version,
          presetHint: c.preset_hint,
          files: [],
        });
      }
      grouped.get(c.type_label).files.push(c.filename);
    }
    catTable.innerHTML = Array.from(grouped.values())
      .map(
        (g) => `
      <div class="dll-row">
        <div class="dll-row-info">
          <div style="display:flex;align-items:center;gap:8px">
            <span class="dll-badge">${escapeHtml(g.typeLabel)}</span>
            <span class="dll-ver-text">v${escapeHtml(g.version)}</span>
            ${g.presetHint ? `<span class="muted" style="font-size:0.76rem">· ${escapeHtml(g.presetHint)}</span>` : ""}
          </div>
          <div class="dll-preset-sub"><code>${escapeHtml(g.files.join(", "))}</code></div>
        </div>
      </div>`
      )
      .join("");
  }

  const refMatrixEl = document.getElementById("info-reference-matrix");
  if (refMatrixEl) {
    refMatrixEl.innerHTML = buildVersionReferenceTablesHtml();
  }

  const hubRefBtn = document.getElementById("btn-hub-version-ref");
  if (hubRefBtn) {
    hubRefBtn.textContent = isDe()
      ? "📊 Versions- & Preset-Referenz"
      : "📊 Version & Preset Reference";
  }
}

window.openAppDataDir = async function (sub) {
  const base = appState.stats?.app_data_dir;
  if (!base) return;
  let target = base;
  if (sub === "cache") target = `${base}\\dll_cache`;
  else if (sub === "backups") target = `${base}\\backups`;
  await openFolderInExplorer(target);
};

window.manualCheckAppUpdate = async function () {
  try {
    showToast(isDe() ? "Prüfe GitHub auf neue DLSS Manager Versionen..." : "Checking GitHub for newer DLSS Manager releases...");
    const info = await invoke("check_app_update");
    if (info && info.update_available) {
      showToast(
        isDe()
          ? `Neue Version v${info.latest_version} verfügbar! Öffne Release-Seite...`
          : `New release v${info.latest_version} available! Opening release page...`
      );
      openExternalUrl(info.release_url);
    } else {
      showToast(
        isDe()
          ? `✓ Du nutzt bereits die neueste Version (v${appState.version}).`
          : `✓ You are running the latest version (v${appState.version}).`
      );
    }
  } catch (e) {
    showToast(String(e), true);
  }
};

function renderHubTechStrip() {
  const strip = document.getElementById("hub-tech-strip");
  if (!strip) return;

  const grouped = new Map();
  for (const item of appState.catalog || []) {
    if (!grouped.has(item.type_label)) {
      grouped.set(item.type_label, item);
    }
  }

  strip.innerHTML = Array.from(grouped.values())
    .map(
      (c) => `
      <div class="tech-group-pill">
        <div class="tech-name">${escapeHtml(c.type_label)}</div>
        <div class="tech-ver">v${escapeHtml(c.version)}</div>
        <div class="tech-sub">${escapeHtml(c.preset_hint || (isDe() ? "Offizielles Release" : "Latest official release"))}</div>
      </div>`
    )
    .join("");
}

function renderHubLaunchers() {
  const grid = document.getElementById("hub-launcher-grid");
  const counts = {};
  for (const g of appState.games) {
    const key = g.launcher_name;
    if (!counts[key]) counts[key] = { count: 0, outdatedCount: 0, icon: g.launcher_icon, samplePath: g.path };
    counts[key].count++;
    if (!g.ignored && !g.blacklisted && getGameUpdateCounts(g).outdated > 0) {
      counts[key].outdatedCount++;
    }
  }

  const de = isDe();
  const entries = Object.entries(counts);
  if (entries.length === 0) {
    grid.innerHTML = `<div class="muted">${de
      ? "Noch keine Spiele mit kompatiblen Upscaler-DLLs erkannt. Klicke links auf <strong>Spiele scannen</strong>."
      : "No games with compatible upscaler DLLs detected yet. Click <strong>Scan Games</strong> on the left."
      }</div>`;
    return;
  }

  grid.innerHTML = entries
    .map(
      ([name, info]) => `
      <div class="launcher-card" onclick="selectLauncherFromHub('${escapeJsStr(name)}')">
        <img src="assets/icons/${escapeHtml(info.icon)}" alt="${escapeHtml(name)}" />
        <div style="flex:1;min-width:0">
          <div style="font-weight:600">${escapeHtml(name)}</div>
          <div class="muted">${info.count} ${de ? "Spiele" : "Games"} · ${info.outdatedCount > 0
          ? `<span style="color:#fbbf24">${info.outdatedCount} ${de ? "Update verfügbar" : "need update"}</span>`
          : `<span style="color:#a3e635">${de ? "Alle aktuell" : "All up to date"}</span>`
        }</div>
        </div>
        <button class="btn btn-secondary btn-card-action" style="padding:5px 8px"
                onclick="event.stopPropagation(); openLauncherDetailsModal('${escapeJsStr(name)}')"
                title="${de ? `Erkannte ${escapeHtml(name)}-Ordner anzeigen` : `View detected ${escapeHtml(name)} folders`}">
          📂
        </button>
      </div>`
    )
    .join("");
}

window.selectLauncherFromHub = function (launcherName) {
  appState.selectedLauncher = launcherName;
  switchView("games");
  renderLauncherSelect();
  renderGamesGrid();
};

window.openGamesWithFilter = function (statusFilter) {
  appState.statusFilter = statusFilter;
  switchView("games");
  renderStatusChips();
  renderGamesGrid();
};

function renderStatusChips() {
  const container = document.getElementById("status-filter-chips");
  if (!container) return;

  let needsUpdate = 0;
  let upToDate = 0;
  let withBackups = 0;
  let ignoredCount = 0;

  for (const g of appState.games) {
    if (g.ignored) {
      ignoredCount++;
      if (appState.config?.hide_ignored_games) continue;
    }
    if (g.has_backups) withBackups++;
    if (!g.ignored && !g.blacklisted && getGameUpdateCounts(g).outdated > 0) needsUpdate++;
    else upToDate++;
  }

  const visibleTotal = appState.config?.hide_ignored_games
    ? appState.games.filter((g) => !g.ignored).length
    : appState.games.length;

  const de = isDe();
  const chips = [
    { id: "all", label: de ? `Alle (${visibleTotal})` : `All (${visibleTotal})` },
    { id: "needs_update", label: de ? `● Update verfügbar (${needsUpdate})` : `● Needs Update (${needsUpdate})` },
    { id: "up_to_date", label: de ? `● Aktuell (${upToDate})` : `● Up to Date (${upToDate})` },
    { id: "has_backups", label: `🛡️ Backups (${withBackups})` },
  ];

  let html = chips
    .map(
      (c) => `
      <button class="status-chip ${appState.statusFilter === c.id ? "active" : ""}"
              onclick="setStatusFilter('${c.id}')">
        ${c.label}
      </button>`
    )
    .join("");

  if (ignoredCount > 0) {
    const hiding = !!appState.config?.hide_ignored_games;
    html += `
      <button class="status-chip ${!hiding ? "active" : ""}"
              onclick="toggleHideIgnoredChip()"
              title="${de ? "Sichtbarkeit ignorierter Spiele umschalten" : "Toggle visibility of ignored games"}">
        👁️ ${hiding
        ? de
          ? `Ignorierte zeigen (${ignoredCount})`
          : `Show Ignored (${ignoredCount})`
        : de
          ? `Ignorierte sichtbar (${ignoredCount})`
          : `Ignored Shown (${ignoredCount})`
      }
      </button>`;
  }

  container.innerHTML = html;
}

window.toggleHideIgnoredChip = async function () {
  if (!appState.config) return;
  appState.config.hide_ignored_games = !appState.config.hide_ignored_games;
  await saveConfig();
  renderAll();
};

window.setStatusFilter = function (filterId) {
  appState.statusFilter = filterId;
  renderStatusChips();
  renderGamesGrid();
};

function getLauncherEmoji(name) {
  const n = (name || "").toLowerCase();
  if (n.includes("steam")) return "🎮";
  if (n.includes("star citizen")) return "🚀";
  if (n.includes("battle")) return "⚔️";
  if (n.includes("xbox")) return "🟩";
  if (n.includes("epic")) return "🛒";
  if (n.includes("gog")) return "🌌";
  if (n.includes("ubisoft")) return "🌀";
  if (n.includes("ea")) return "⚡";
  if (n.includes("custom")) return "📁";
  return "🎯";
}

function renderLauncherSelect() {
  const sel = document.getElementById("select-launcher-filter");
  if (!sel) return;
  const counts = {};
  for (const g of appState.games) {
    counts[g.launcher_name] = (counts[g.launcher_name] || 0) + 1;
  }
  const launchers = ["all", ...new Set(appState.games.map((g) => g.launcher_name))];
  const de = isDe();
  sel.innerHTML = launchers
    .map((l) => {
      const label =
        l === "all"
          ? `🌐 ${de ? "Alle Launcher" : "All Launchers"} (${appState.games.length})`
          : `${getLauncherEmoji(l)} ${l} (${counts[l] || 0})`;
      return `<option value="${escapeHtml(l)}" ${appState.selectedLauncher === l ? "selected" : ""}>${escapeHtml(label)}</option>`;
    })
    .join("");
}

window.resetLauncherAndStatusFilters = function () {
  appState.selectedLauncher = "all";
  appState.statusFilter = "all";
  renderLauncherSelect();
  renderStatusChips();
  renderGamesGrid();
};

function updateActiveLauncherBanner(filteredCount) {
  const banner = document.getElementById("active-launcher-banner");
  const textEl = document.getElementById("active-launcher-banner-text");
  if (!banner || !textEl) return;
  const de = isDe();
  if (appState.selectedLauncher !== "all") {
    banner.style.display = "flex";
    const emoji = getLauncherEmoji(appState.selectedLauncher);
    textEl.textContent = de
      ? `${emoji} Aktiver Launcher: ${appState.selectedLauncher} (${filteredCount} ${filteredCount === 1 ? "Spiel" : "Spiele"})`
      : `${emoji} Active Launcher: ${appState.selectedLauncher} (${filteredCount} ${filteredCount === 1 ? "game" : "games"})`;
  } else {
    banner.style.display = "none";
  }
}

function getFilteredGames() {
  const q = appState.searchQuery.toLowerCase().trim();
  const hideIgnored = !!appState.config?.hide_ignored_games;

  let filtered = appState.games.filter((g) => {
    if (hideIgnored && g.ignored) return false;
    if (appState.selectedLauncher !== "all" && g.launcher_name !== appState.selectedLauncher) {
      return false;
    }
    const counts = getGameUpdateCounts(g);
    if (appState.statusFilter === "needs_update" && counts.outdated === 0) return false;
    if (appState.statusFilter === "up_to_date" && counts.outdated > 0) return false;
    if (appState.statusFilter === "has_backups" && !g.has_backups) return false;

    if (q) {
      const hay = `${g.name} ${g.launcher_name} ${g.path} ${g.dlls
        .map((d) => d.filename + " " + d.type_label)
        .join(" ")}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });

  filtered.sort((a, b) => {
    const ca = getGameUpdateCounts(a);
    const cb = getGameUpdateCounts(b);
    if (appState.sortMode === "outdated_first") {
      if ((cb.outdated > 0) !== (ca.outdated > 0)) {
        return cb.outdated > 0 ? 1 : -1;
      }
      if (cb.outdated !== ca.outdated) {
        return cb.outdated - ca.outdated;
      }
      return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    } else if (appState.sortMode === "name_desc") {
      return b.name.toLowerCase().localeCompare(a.name.toLowerCase());
    } else if (appState.sortMode === "dll_count") {
      return cb.total - ca.total || a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    }
    return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
  });

  return filtered;
}

function renderBatchSelectionBar() {
  const bar = document.getElementById("batch-selection-bar");
  const btnToggle = document.getElementById("btn-toggle-select-mode");
  if (!bar) return;

  const de = isDe();
  const count = appState.selectedGameIds.size;
  if (btnToggle) {
    btnToggle.classList.toggle("btn-primary", appState.selectMode);
    btnToggle.classList.toggle("btn-secondary", !appState.selectMode);
    btnToggle.textContent = appState.selectMode
      ? de
        ? "✕ Auswahl beenden"
        : "✕ Exit Select"
      : de
        ? "☑ Auswählen"
        : "☑ Select";
  }

  if (appState.selectMode || count > 0) {
    bar.style.display = "flex";
    document.getElementById("batch-selected-label").textContent = de
      ? `${count} ${count === 1 ? "Spiel" : "Spiele"} ausgewählt`
      : `${count} ${count === 1 ? "game" : "games"} selected`;
    document.getElementById("btn-update-selected-count").textContent = count;
    document.getElementById("btn-update-selected").disabled = count === 0;
  } else {
    bar.style.display = "none";
  }
}

window.toggleSelectGame = function (event, gameId) {
  if (event) event.stopPropagation();
  if (appState.selectedGameIds.has(gameId)) {
    appState.selectedGameIds.delete(gameId);
  } else {
    appState.selectedGameIds.add(gameId);
  }
  renderBatchSelectionBar();
  renderGamesGrid();
};

window.selectOutdatedVisibleGames = function () {
  for (const g of getFilteredGames()) {
    if (!g.ignored && !g.blacklisted && getGameUpdateCounts(g).outdated > 0) {
      appState.selectedGameIds.add(g.id);
    }
  }
  renderBatchSelectionBar();
  renderGamesGrid();
};

window.selectAllVisibleGames = function () {
  for (const g of getFilteredGames()) {
    appState.selectedGameIds.add(g.id);
  }
  renderBatchSelectionBar();
  renderGamesGrid();
};

window.clearSelectedGames = function () {
  appState.selectedGameIds.clear();
  renderBatchSelectionBar();
  renderGamesGrid();
};

window.updateSelectedGamesNow = async function () {
  const ids = Array.from(appState.selectedGameIds);
  if (ids.length === 0) return;
  try {
    showToast(
      isDe()
        ? `Aktualisiere ${ids.length} ausgewählte(s) Spiel(e)...`
        : `Updating ${ids.length} selected game(s)...`
    );
    const payload = await invoke("bulk_update_selected_games", {
      gameIds: ids,
      allowedTechs: null,
    });
    appState.selectedGameIds.clear();
    appState.selectMode = false;
    applyPayload(payload);
    showToast(
      isDe()
        ? `✓ ${ids.length} ausgewählte(s) Spiel(e) aktualisiert!`
        : `✓ Updated ${ids.length} selected game(s)!`
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

function renderGamesGrid() {
  const grid = document.getElementById("games-grid");
  grid.className = `games-hero-grid density-${appState.gridDensity}`;

  const filtered = getFilteredGames();
  updateActiveLauncherBanner(filtered.length);
  const de = isDe();

  if (filtered.length === 0) {
    grid.innerHTML = `<div class="muted">${de
      ? "Keine passenden Spiele für den aktiven Filter gefunden."
      : "No matching games found for the active filter."
      }</div>`;
    return;
  }

  grid.innerHTML = filtered
    .map((g) => {
      const counts = getGameUpdateCounts(g);
      const hasUpdate = counts.outdated > 0;
      const techLine = buildTechSummaryLine(g);
      const isSelected = appState.selectedGameIds.has(g.id);

      const heroImgUrl =
        g.cover_url ||
        (g.app_id && /^\d+$/.test(g.app_id)
          ? `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${g.app_id}/library_hero.jpg`
          : "");
      const fallbackHeaderUrl =
        g.app_id && /^\d+$/.test(g.app_id)
          ? `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${g.app_id}/header.jpg`
          : "";

      const artTag = heroImgUrl
        ? `<img src="${heroImgUrl}" class="hero-card-art" alt="${escapeHtml(g.name)}"
                onerror="if(this.dataset.fb && this.src!==this.dataset.fb){this.src=this.dataset.fb;}else{this.style.display='none';}"
                data-fb="${fallbackHeaderUrl}" loading="lazy" />`
        : "";

      const statusText = g.ignored
        ? de
          ? "Ignoriert"
          : "Ignored"
        : hasUpdate
          ? de
            ? "Update verfügbar"
            : "Needs update"
          : de
            ? "Aktuell"
            : "Up to date";
      const statusDotClass = g.ignored ? "" : hasUpdate ? "outdated" : "current";
      const statusColor = g.ignored
        ? "#94a3b8"
        : hasUpdate
          ? "#fbbf24"
          : "#a3e635";

      const badgeText = g.ignored
        ? de
          ? `👁️ Ignoriert`
          : `👁️ Ignored`
        : hasUpdate
          ? de
            ? `⚡ ${counts.outdated}/${counts.total} veraltet`
            : `⚡ ${counts.outdated}/${counts.total} outdated`
          : de
            ? `✓ Aktuell`
            : `✓ Up to date`;
      const badgeClass = g.ignored
        ? ""
        : hasUpdate
          ? "badge-outdated"
          : "badge-current";

      const hasSrTech = (g.dlls || []).some((d) => d.type_label === "DLSS");
      const hasRrTech = (g.dlls || []).some((d) => d.type_label === "DLSS-RR");
      const hasFgTech = (g.dlls || []).some((d) => d.type_label === "DLSS-G");

      const activePresetParts = [];
      if (hasSrTech && g.dlss_sr_preset && g.dlss_sr_preset !== "Default") {
        activePresetParts.push(`SR: ${g.dlss_sr_preset}`);
      }
      if (hasRrTech && g.dlss_rr_preset && g.dlss_rr_preset !== "Default") {
        activePresetParts.push(`RR: ${g.dlss_rr_preset}`);
      }
      if (hasFgTech && g.dlss_fg_preset && g.dlss_fg_preset !== "Default") {
        activePresetParts.push(`FG: ${g.dlss_fg_preset}`);
      }
      const hasCustomPreset = activePresetParts.length > 0;
      const presetBadgeLabel =
        activePresetParts.length === 1
          ? activePresetParts[0].replace(/^SR:\s*/, "")
          : activePresetParts.map((p) => p.replace(/Preset\s+/i, "")).join(" · ");
      const presetBadgeTooltip = activePresetParts.join(" | ");

      return `
      <div class="hero-game-card ${hasUpdate ? "needs-update" : ""} ${g.ignored ? "is-ignored" : ""
        } ${isSelected ? "is-selected" : ""}"
           oncontextmenu="openCardContextMenu(event, '${g.id}')">
        <!-- Hero Banner -->
        <div class="hero-card-banner" onclick="${appState.selectMode
          ? `toggleSelectGame(event, '${g.id}')`
          : `openGameDllModal('${g.id}')`
        }" title="${escapeHtml(g.name)}\n${escapeHtml(g.path)}">
          <div class="hero-card-fallback-icon">
            <img src="assets/icons/${escapeHtml(g.launcher_icon)}" alt="" />
          </div>
          ${artTag}
          <div class="hero-top-badges">
            <div style="display:flex;align-items:center;gap:6px;pointer-events:auto;flex-shrink:0">
              ${appState.selectMode || isSelected
          ? `<label class="card-select-pill" onclick="event.stopPropagation()">
                       <input type="checkbox" ${isSelected ? "checked" : ""}
                              onchange="toggleSelectGame(event, '${g.id}')" />
                     </label>`
          : ""
        }
              <span class="hero-launcher-pill">
                <img src="assets/icons/${escapeHtml(g.launcher_icon)}" alt="" />
                ${escapeHtml(g.launcher_name)}
              </span>
            </div>
            <div class="hero-top-right-group">
              ${g.anti_cheat
          ? `<span class="anticheat-badge" style="overflow:hidden;text-overflow:ellipsis;max-width:140px" title="🛡️ ${escapeHtml(g.anti_cheat)}">🛡️ ${escapeHtml(g.anti_cheat)}</span>`
          : ""
        }
              <button class="card-menu-btn" onclick="openCardContextMenu(event, '${g.id}')" title="${de ? "Weitere Aktionen" : "More actions"}">⋮</button>
            </div>
          </div>
          <div class="hero-scrim">
            <div class="hero-game-title">${escapeHtml(g.name)}</div>
            <div class="hero-status-line">
              <span>${escapeHtml(g.launcher_name)}</span>
              <span>·</span>
              ${statusDotClass ? `<span class="status-dot ${statusDotClass}"></span>` : ""}
              <span style="color:${statusColor};font-weight:600">${statusText}</span>
            </div>
            <div class="hero-tech-line">
              <span style="overflow:hidden;text-overflow:ellipsis">${escapeHtml(techLine)}</span>
              ${hasCustomPreset
          ? `<span class="card-preset-pill" title="${escapeHtml(presetBadgeTooltip)}">🎛️ ${escapeHtml(
            presetBadgeLabel
          )}</span>`
          : ""
        }
            </div>
          </div>
        </div>

        <!-- Fixed 52px Footer -->
        <div class="hero-card-footer">
          <button class="footer-dll-badge ${badgeClass}" onclick="openGameDllModal('${g.id}')" title="${de ? "Klicken, um Technologien zu prüfen" : "Click to inspect technology groups"}">
            <span>${badgeText}</span>
          </button>
          <div class="footer-actions">
            ${g.has_backups
          ? `<button class="btn btn-secondary btn-card-action" onclick="restoreAllGameBackups('${g.id}')" title="${de
            ? `Originale DLLs für ${escapeHtml(g.name)} wiederherstellen`
            : `Restore original DLLs for ${escapeHtml(g.name)}`
          }">
                     ⏪ ${de ? "Backup" : "Restore"}
                   </button>`
          : ""
        }
            ${hasUpdate
          ? `<button class="btn btn-primary btn-card-action" id="btn-update-card-${g.id}" onclick="updateAllGameDlls('${g.id}')" title="${de
            ? `Alle veralteten Technologien in ${escapeHtml(g.name)} aktualisieren`
            : `Update all outdated technologies in ${escapeHtml(g.name)}`
          }">
                     ⚡ Update
                   </button>`
          : `<button class="btn btn-secondary btn-card-action" onclick="openGameDllModal('${g.id}')">
                     Details
                   </button>`
        }
          </div>
        </div>
      </div>`;
    })
    .join("");
}

/* ==========================================================================
   Card Context Menu (`⋮` & Right-Click)
   ========================================================================== */
window.openCardContextMenu = function (event, gameId) {
  event.preventDefault();
  event.stopPropagation();
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;

  const menu = document.getElementById("card-context-menu");
  const counts = getGameUpdateCounts(game);
  const isNvidia = (appState.gpu?.vendor || "").toUpperCase() === "NVIDIA";
  const de = isDe();

  const acStatus = (game.db_profile?.anticheat_status || "").toLowerCase();
  const isDrsOnlyGame =
    acStatus === "driver_override_only" || acStatus === "drs_only" || acStatus === "blocked";

  menu.innerHTML = `
    ${!isDrsOnlyGame && counts.outdated > 0
      ? `<button class="context-menu-item" onclick="closeCardContextMenu(); updateAllGameDlls('${game.id}', false)">
             <span>⚡</span><span>${de ? `Veraltete DLLs aktualisieren (${counts.outdated})` : `Update Outdated DLLs (${counts.outdated})`}</span>
           </button>`
      : ""
    }
    ${!isDrsOnlyGame
      ? `<button class="context-menu-item" onclick="closeCardContextMenu(); updateAllGameDlls('${game.id}', true)">
             <span>🔄</span><span>${de ? `Alle DLLs neu installieren (${counts.total})` : `Reinstall All DLLs (${counts.total})`}</span>
           </button>`
      : ""
    }
    <button class="context-menu-item" onclick="closeCardContextMenu(); openGameDllModal('${game.id}')">
      <span>🔍</span><span>${de ? "Technologien & Backups prüfen" : "Inspect Technologies & Backups"}</span>
    </button>
    ${isNvidia
      ? `<button class="context-menu-item" onclick="closeCardContextMenu(); openGameDlssPresetsModal('${game.id}')">
             <span>🎛️</span><span>${de ? "DLSS Preset-Einstellungen (DRS)" : "DLSS Preset Settings (DRS)"}</span>
           </button>`
      : ""
    }
    <button class="context-menu-item" onclick="closeCardContextMenu(); openEditDisplayModal('${game.id}')">
      <span>✏️</span><span>${de ? "Name & Cover-Bild bearbeiten" : "Edit Name & Cover Art"}</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); reportGamesToDbNow('${game.id}')">
      <span>📤</span><span>${de ? "Spiel an Online-DB melden" : "Report Game to Online-DB"}</span>
    </button>
    <div class="context-menu-divider"></div>
    <button class="context-menu-item" onclick="closeCardContextMenu(); launchGameNow('${game.id}')">
      <span>🚀</span><span>${de ? "Spiel starten" : "Launch Game"}</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); openFolderInExplorer('${escapeJsStr(game.path)}')">
      <span>📂</span><span>${de ? "Spielordner öffnen" : "Open Game Folder"}</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); copyTextToClipboard('${escapeJsStr(game.path)}', '${de ? "Spielpfad in Zwischenablage kopiert" : "Copied game path to clipboard"}')">
      <span>📋</span><span>${de ? "Installationspfad kopieren" : "Copy Install Path"}</span>
    </button>
    <div class="context-menu-divider"></div>
    <button class="context-menu-item" onclick="closeCardContextMenu(); toggleIgnoreGameNow('${game.id}')">
      <span>👁️</span><span>${game.ignored
      ? de
        ? "Nicht mehr ignorieren"
        : "Unignore Game"
      : de
        ? "Spiel ignorieren"
        : "Ignore Game"
    }</span>
    </button>
  `;

  menu.style.display = "flex";
  const rect = menu.getBoundingClientRect();
  const x = Math.min(event.clientX, window.innerWidth - (rect.width || 240) - 12);
  const y = Math.min(event.clientY, window.innerHeight - (rect.height || 290) - 12);
  menu.style.left = `${Math.max(8, x)}px`;
  menu.style.top = `${Math.max(8, y)}px`;
};

window.closeCardContextMenu = function () {
  const menu = document.getElementById("card-context-menu");
  if (menu) menu.style.display = "none";
};

window.openFolderInExplorer = async function (path) {
  try {
    await invoke("open_path_in_explorer", { path });
  } catch (e) {
    showToast(String(e), true);
  }
};

window.copyTextToClipboard = async function (text, toastMsg = null) {
  const msg = toastMsg || (isDe() ? "In Zwischenablage kopiert" : "Copied to clipboard");
  try {
    await navigator.clipboard.writeText(text);
    showToast(msg);
  } catch (_) {
    showToast((isDe() ? "Kopiert: " : "Copied: ") + text);
  }
};

window.launchGameNow = async function (gameId) {
  try {
    const msg = await invoke("launch_game_cmd", { gameId });
    showToast(msg);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.toggleIgnoreGameNow = async function (gameId) {
  try {
    const payload = await invoke("toggle_ignore_game", { gameId });
    applyPayload(payload);
    showToast(isDe() ? "Ignorier-Liste aktualisiert." : "Updated Ignore List.");
  } catch (e) {
    showToast(String(e), true);
  }
};

/* ==========================================================================
   Game Details Modal (Per-Tech Update, Applied Presets & Compatibility)
   ========================================================================== */
function translateDbText(raw) {
  if (!raw || !isDe()) return raw || "";
  const exactMap = {
    "EasyAntiCheat (base.bin) checks the exact file hash of Bin64\\nvngx_dlss.dll: Replacing the file on disk disables DLSS in-game ('GPU: DLSS Support = Unavailable'). Keep the original DLL in Bin64 and activate DLSS 4.5 (v310.9.16.0 · Preset K) via the NVIDIA Driver DRS Override above!":
      "EasyAntiCheat (base.bin) prüft den exakten Datei-Hash von Bin64\\nvngx_dlss.dll: Ein Ersetzen der Datei auf der Festplatte deaktiviert DLSS im Spiel ('GPU: DLSS Support = Unavailable'). Belasse die Original-DLL in Bin64 und aktiviere DLSS 4.5 (v310.9.16.0 · Preset K) oben über den NVIDIA Treiber-DRS-Override!",
    "EasyAntiCheat (EasyAntiCheat\\Certificates\\base.bin) verifies the exact file hash of Bin64\\nvngx_dlss.dll. Replacing the DLL on disk causes EAC to block it at launch ('GPU: DLSS Support = Unavailable'). Use the NVIDIA Driver DRS Override (Preset K) instead!":
      "EasyAntiCheat (EasyAntiCheat\\Certificates\\base.bin) verifiziert den exakten Hash von Bin64\\nvngx_dlss.dll. Ein Überschreiben der Datei führt dazu, dass EAC das Laden beim Start blockiert ('GPU: DLSS Support = Unavailable'). Nutze stattdessen den NVIDIA Treiber-DRS-Override (Preset K)!",
    "EasyAntiCheat monitors Bin64: Only official NVIDIA-signed nvngx_dlss.dll builds (full all-presets production build) and NVIDIA Driver DRS Presets (Preset K) are supported.":
      "EasyAntiCheat überwacht Bin64: Nur offiziell NVIDIA-signierte nvngx_dlss.dll-Versionen (vollständiger 74,18 MB All-Presets-Build) sowie NVIDIA Treiber-DRS-Presets werden unterstützt.",
    "EasyAntiCheat monitors Bin64: Official NVIDIA-signed nvngx_dlss.dll updates and NVIDIA Driver DRS Presets (Preset J/K/L) work safely. Do not inject unsigned third-party wrapper DLLs.":
      "EasyAntiCheat überwacht Bin64: Offiziell NVIDIA-signierte nvngx_dlss.dll-Updates sowie NVIDIA Treiber-DRS-Presets (Preset J/K/L) funktionieren sicher. Keine unsignierten Drittanbieter-Wrapper-DLLs injizieren.",
    "AMD FSR 2 and CIG TSR are compiled directly into StarCitizen.exe & Data.p4k shaders (no external FSR DLL in Bin64).":
      "AMD FSR 2 und CIG TSR sind direkt in StarCitizen.exe & Data.p4k kompiliert (keine externe FSR-DLL im Bin64-Ordner).",
    "DLSS Frame Generation (nvngx_dlssg.dll) is not natively supported by StarEngine yet.":
      "DLSS Frame Generation (nvngx_dlssg.dll) wird von der StarEngine noch nicht nativ unterstützt.",
    "Ray Reconstruction (nvngx_dlssd.dll) and Intel XeSS (libxess.dll) are not natively integrated in Star Citizen.":
      "Ray Reconstruction (nvngx_dlssd.dll) und Intel XeSS (libxess.dll) sind in Star Citizen nicht nativ integriert.",
    "Upscaling: DLSS (Quality or Balanced at 1440p/4K)":
      "Upscaling: DLSS (Qualität oder Ausgeglichen bei 1440p/4K)",
    "NVIDIA DLSS Preset: Preset K (Transformer Gen 2 — sharper distant ship hulls & HUD MFD text)":
      "NVIDIA DLSS Preset: Preset K (Transformer Gen 2 — schärfere Schiffshüllen & HUD/MFD-Schrift)",
    "NVIDIA DLSS Preset: Preset K (Transformer Gen 2 — sharper distant ship & HUD details)":
      "NVIDIA DLSS Preset: Preset K (Transformer Gen 2 — schärfere Schiffshüllen & HUD-Details)",
    "Graphics Renderer: Vulkan": "Grafik-Renderer: Vulkan",
    "Graphics Renderer: DirectX 11 or Vulkan (Beta)": "Grafik-Renderer: Vulkan",
    "EasyAntiCheat (EasyAntiCheat\\Certificates\\base.bin) verifies the exact file hash of Bin64\\nvngx_dlss.dll (v310.2.1.0). Replacing the file on disk causes EAC to block it at launch ('GPU: DLSS Support = Unavailable').":
      "EasyAntiCheat (EasyAntiCheat\\Certificates\\base.bin) prüft den exakten Datei-Hash von Bin64\\nvngx_dlss.dll (v310.2.1.0). Ein Überschreiben auf der Festplatte blockiert DLSS beim Spielstart ('GPU: DLSS Support = Unavailable').",
    "DirectStorage: Active (dstorage.dll & dstoragecore.dll for fast map & tile streaming)":
      "DirectStorage: Aktiv (dstorage.dll & dstoragecore.dll für schnelles Karten-Streaming)",
    "Overwatch 2 supports DLSS Super Resolution, AMD FSR 3 & Intel XeSS, but does not ship with NVIDIA DLSS Frame Generation (nvngx_dlssg.dll).":
      "Overwatch 2 unterstützt DLSS Super Resolution, AMD FSR 3 & Intel XeSS, liefert jedoch keine NVIDIA DLSS Frame Generation (nvngx_dlssg.dll) mit.",
    "Blizzard Defense Matrix: Official NVIDIA-signed nvngx_dlss.dll updates and NVIDIA Driver DRS Presets are supported. Battle.net patches may reset nvngx_dlss.dll after seasonal updates.":
      "Blizzard Defense Matrix: Offiziell signierte nvngx_dlss.dll-Updates und Treiber-DRS-Presets werden unterstützt. Battle.net-Patches können die DLL nach Updates zurücksetzen.",
    "EasyAntiCheat protected: Only official NVIDIA-signed DLSS & Streamline binaries should be used.":
      "Durch EasyAntiCheat geschützt: Nur offiziell NVIDIA-signierte DLSS- & Streamline-Dateien verwenden.",
  };
  if (exactMap[raw]) return exactMap[raw];

  return String(raw)
    .replace(/^EasyAntiCheat protected:/i, "EasyAntiCheat-geschützt:")
    .replace(/^BattlEye protected:/i, "BattlEye-geschützt:")
    .replace(/^Ricochet Anti-Cheat:/i, "Ricochet Anti-Cheat:")
    .replace(/^Javelin /i, "Javelin ")
    .replace(/\bEnabled \+ Boost\b/gi, "Aktiviert + Boost")
    .replace(/\bEnabled\b/g, "Aktiviert")
    .replace(/\bQuality or Balanced\b/gi, "Qualität oder Ausgeglichen")
    .replace(/\bis not natively supported\b/gi, "wird nicht nativ unterstützt")
    .replace(/\bnot natively integrated\b/gi, "nicht nativ integriert")
    .replace(/\bOnly official NVIDIA-signed\b/gi, "Nur offiziell NVIDIA-signierte")
    .replace(/\bsharper distant ship & HUD details\b/gi, "schärfere Schiffshüllen & HUD-Details")
    .replace(/\bDirectX 11 or Vulkan \(Beta\)\b/gi, "Vulkan");
}

const DLSS_PRESET_META = {
  sr: {
    Default: {
      opt_de: "Standard (Default) — Vom Spiel gesteuert",
      opt_en: "Default — Controlled by Game",
      desc_de: "Nutzt das vom Spiel bzw. der DLL standardmäßig vorgesehene DLSS-Profil ohne Treiber-Eingriff.",
      desc_en: "Uses the game's built-in default DLSS profile without driver override.",
    },
    Latest: {
      opt_de: "Latest — Automatisch neuestes NVIDIA-Profil",
      opt_en: "Latest — Automatically use newest NVIDIA profile",
      desc_de: "Erzwingt über den NVIDIA-Treiber immer automatisch das jeweils neueste verfügbare DLSS-Transformer-Profil.",
      desc_en: "Always forces the newest available DLSS Transformer profile via the NVIDIA driver.",
    },
    "Preset J": {
      opt_de: "Preset J — DLSS 4 Transformer Gen 1 (Hohe Bewegungsschärfe)",
      opt_en: "Preset J — DLSS 4 Transformer Gen 1 (High Motion Clarity)",
      desc_de: "Erstes DLSS 4 Transformer-Modell: Sehr hohe Bewegungsschärfe und minimales Ghosting, kann bei feinen Gittern leicht schimmern.",
      desc_en: "First-generation DLSS 4 Transformer model: Very sharp in motion with minimal ghosting, may show slight shimmer on fine meshes.",
    },
    "Preset K": {
      opt_de: "Preset K — DLSS 4.5 Transformer Gen 2 (Empfohlen · Beste Bildruhe & Schärfe)",
      opt_en: "Preset K — DLSS 4.5 Transformer Gen 2 (Recommended · Best Stability & Sharpness)",
      desc_de: "Weiterentwickeltes Transformer-Modell (Gen 2): Beste Balance aus maximaler Detailschärfe (HUD/Schrift/Ferne) und höchster temporaler Bildruhe.",
      desc_en: "Refined Transformer Gen 2 model: Best overall balance of fine detail sharpness (HUD/text/distance) and temporal stability.",
    },
    "Preset L": {
      opt_de: "Preset L — DLSS 4.5 Transformer (Optimiert für Performance / Ultra-Perf.)",
      opt_en: "Preset L — DLSS 4.5 Transformer (Optimized for Performance / Ultra-Perf.)",
      desc_de: "Speziell für stärkere Skalierung (Performance- & Ultra-Performance-Modus bei 4K) abgestimmtes DLSS 4.5 Transformer-Profil.",
      desc_en: "DLSS 4.5 Transformer profile specifically tuned for stronger scaling ratios (4K Performance / Ultra Performance modes).",
    },
    "Preset M": {
      opt_de: "Preset M — DLSS 4.5 Transformer Max Quality (Optimiert für Quality / DLAA)",
      opt_en: "Preset M — DLSS 4.5 Transformer Max Quality (Optimized for Quality / DLAA)",
      desc_de: "Maximale Bildqualität für DLAA und Quality-Modus auf RTX 40/50 GPUs (nur in DLSS 4.5 ab v310.9.0+ verfügbar).",
      desc_en: "Maximum image quality profile for DLAA and Quality modes on RTX 40/50 GPUs (only available in DLSS 4.5 v310.9.0+).",
    },
    "Preset E": {
      opt_de: "Preset E — DLSS 3.7+ CNN (Bestes klassisches CNN-Modell)",
      opt_en: "Preset E — DLSS 3.7+ CNN (Best Classic CNN Model)",
      desc_de: "Bestes klassisches Convolutional-Neural-Network (CNN) Profil (Standard in DLSS 3.7/3.8 sowie als CNN-Fallback in v310.x).",
      desc_en: "Best classic Convolutional Neural Network (CNN) profile (default in DLSS 3.7/3.8 and CNN fallback in v310.x).",
    },
    "Preset C": {
      opt_de: "Preset C — DLSS 3.x CNN (Schnelle Bewegung)",
      opt_en: "Preset C — DLSS 3.x CNN (Fast Motion)",
      desc_de: "Klassisches CNN-Profil mit Fokus auf Bewegungsschärfe (DLSS 3.1–3.8).",
      desc_en: "Classic CNN profile favoring motion clarity (DLSS 3.1–3.8).",
    },
    "Preset D": {
      opt_de: "Preset D — DLSS 3.x CNN (Temporale Bildruhe)",
      opt_en: "Preset D — DLSS 3.x CNN (Temporal Stability)",
      desc_de: "Klassisches CNN-Profil mit Fokus auf temporale Stabilität (DLSS 3.1–3.8).",
      desc_en: "Classic CNN profile favoring temporal stability (DLSS 3.1–3.8).",
    },
    "Preset F": {
      opt_de: "Preset F — DLSS 3.x CNN (Ultra-Performance / DLAA)",
      opt_en: "Preset F — DLSS 3.x CNN (Ultra-Performance / DLAA)",
      desc_de: "Klassisches CNN-Profil für Ultra-Performance und DLAA (DLSS 3.1–3.8).",
      desc_en: "Classic CNN profile for Ultra-Performance and DLAA (DLSS 3.1–3.8).",
    },
  },
  rr: {
    Default: {
      opt_de: "Standard (Default) — Vom Spiel gesteuert",
      opt_en: "Default — Controlled by Game",
      desc_de: "Nutzt das Standard-Ray-Reconstruction-Profil des Spiels ohne Treiber-Eingriff.",
      desc_en: "Uses the game's default Ray Reconstruction profile without driver override.",
    },
    Latest: {
      opt_de: "Latest — Automatisch neuestes Ray-Reconstruction-Profil",
      opt_en: "Latest — Automatically use newest Ray Reconstruction profile",
      desc_de: "Erzwingt über den Treiber automatisch das neueste verfügbare Transformer-Denoiser-Modell.",
      desc_en: "Forces the newest available Transformer denoiser model via the driver.",
    },
    "Preset D": {
      opt_de: "Preset D — Transformer Ray Reconstruction (Gen 1)",
      opt_en: "Preset D — Transformer Ray Reconstruction (Gen 1)",
      desc_de: "Erstes Transformer-Denoiser-Modell für Raytracing und Pathtracing.",
      desc_en: "First-generation Transformer denoiser model for Ray Tracing and Path Tracing.",
    },
    "Preset F": {
      opt_de: "Preset F — DLSS 4.5 Transformer RR Gen 2 (Empfohlen · Minimales Rauschen & Ghosting)",
      opt_en: "Preset F — DLSS 4.5 Transformer RR Gen 2 (Recommended · Minimal Noise & Ghosting)",
      desc_de: "Neuestes Transformer-Denoiser-Modell (Gen 2): Deutlich ruhigere Reflexionen, Schatten und weniger Ghosting an Zäunen/Drähten.",
      desc_en: "Newest Transformer Gen 2 denoiser: Significantly cleaner reflections, shadows, and reduced wire/fence ghosting.",
    },
  },
  fg: {
    Default: {
      opt_de: "Standard (Default) — Vom Spiel gesteuert",
      opt_en: "Default — Controlled by Game",
      desc_de: "Nutzt das Standard-Frame-Generation-Profil des Spiels ohne Treiber-Eingriff.",
      desc_en: "Uses the game's default Frame Generation profile without driver override.",
    },
    Latest: {
      opt_de: "Latest — Neuestes Optical-Flow / Flip-Metering Modell (Empfohlen)",
      opt_en: "Latest — Newest Optical-Flow / Flip-Metering Model (Recommended)",
      desc_de: "Nutzt das neueste DLSS 4/4.5 Frame-Generation-Modell mit geringerem VRAM-Bedarf und gleichmäßigerem Frame-Pacing.",
      desc_en: "Uses the newest DLSS 4/4.5 Frame Generation model with lower VRAM usage and smoother frame pacing.",
    },
    "Preset A": {
      opt_de: "Preset A — Frame Generation Profil A (Standard)",
      opt_en: "Preset A — Frame Generation Profile A (Standard)",
      desc_de: "Standard-Profil für DLSS 3/4 Frame Generation.",
      desc_en: "Standard profile for DLSS 3/4 Frame Generation.",
    },
    "Preset B": {
      opt_de: "Preset B — Frame Generation Profil B (Alternativ bei HUD-Artefakten)",
      opt_en: "Preset B — Frame Generation Profile B (Alternative for HUD Artifacts)",
      desc_de: "Alternatives Frame-Generation-Profil, falls bei schnellen Kameraschwenks UI- oder Fadenkreuz-Artefakte auftreten.",
      desc_en: "Alternative Frame Generation profile if UI or crosshair artifacts occur during fast camera pans.",
    },
  },
};

function parseVerParts(verStr) {
  if (!verStr) return [310, 9, 16];
  const nums = String(verStr)
    .split(/[^0-9]+/)
    .filter(Boolean)
    .map((n) => parseInt(n, 10));
  return [nums[0] || 0, nums[1] || 0, nums[2] || 0];
}

function getSupportedPresetsForDllVersion(cat, dllVersion) {
  const [major, minor] = parseVerParts(dllVersion);
  if (cat === "sr") {
    if (major >= 310) {
      if (minor >= 9) {
        return ["Default", "Latest", "Preset J", "Preset K", "Preset L", "Preset M", "Preset E"];
      }
      if (minor >= 2) {
        // v310.2.x – v310.8.x (e.g. Star Citizen stock 310.2.1.0): NO Preset L or M (DLSS 4.5 310.9+ only!)
        return ["Default", "Latest", "Preset J", "Preset K", "Preset E"];
      }
      // v310.1.x: only Preset J + CNN
      return ["Default", "Latest", "Preset J", "Preset E"];
    }
    if (major === 3 && minor >= 7) {
      return ["Default", "Preset E", "Preset C", "Preset D", "Preset F"];
    }
    if (major === 3 && minor >= 1) {
      return ["Default", "Preset C", "Preset D", "Preset F"];
    }
    return ["Default"];
  }
  if (cat === "rr") {
    if (major >= 310) {
      if (minor >= 4) {
        return ["Default", "Latest", "Preset D", "Preset F"];
      }
      return ["Default", "Latest", "Preset D"];
    }
    return ["Default"];
  }
  if (cat === "fg") {
    if (major >= 310) {
      return ["Default", "Latest", "Preset A", "Preset B"];
    }
    return ["Default"];
  }
  return ["Default"];
}

function getPresetOptionLabel(cat, opt, recVal = null, dllVersion = null) {
  const de = isDe();
  const [major, minor] = parseVerParts(dllVersion);
  const meta = DLSS_PRESET_META[cat]?.[opt];
  let base = meta ? (de ? meta.opt_de : meta.opt_en) : opt;

  // If the installed DLL is DLSS 4.0 (v310.2.x – v310.3.x, e.g. Star Citizen stock v310.2.1.0),
  // Preset K is DLSS 4.0 Transformer Gen 1 — NOT DLSS 4.5!
  if (cat === "sr" && opt === "Preset K" && major === 310 && minor < 4) {
    base = de
      ? "Preset K — DLSS 4.0 Transformer Gen 1 (Empfohlen für v310.2/310.3 · Kein DLSS 4.5)"
      : "Preset K — DLSS 4.0 Transformer Gen 1 (Recommended for v310.2/310.3 · Not DLSS 4.5)";
  }
  return recVal === opt ? `★ ${base}` : base;
}

function getAllPresetsForCategory(cat) {
  if (cat === "sr") {
    return ["Default", "Latest", "Preset J", "Preset K", "Preset L", "Preset M", "Preset E", "Preset C", "Preset D", "Preset F"];
  }
  if (cat === "rr") return ["Default", "Latest", "Preset D", "Preset F"];
  if (cat === "fg") return ["Default", "Latest", "Preset A", "Preset B"];
  return ["Default"];
}

function getPresetMinDllHint(cat, opt, de) {
  if (cat === "sr") {
    if (opt === "Preset L" || opt === "Preset M") return de ? "Benötigt v310.9.0+ / DLSS 4.5" : "Requires v310.9.0+ / DLSS 4.5";
    if (opt === "Preset K") return de ? "Benötigt v310.2.1+" : "Requires v310.2.1+";
    if (opt === "Preset J" || opt === "Latest") return de ? "Benötigt v310.1.0+" : "Requires v310.1.0+";
    if (opt === "Preset E") return de ? "Benötigt v3.7.0+" : "Requires v3.7.0+";
    return de ? "Benötigt v3.1.1+" : "Requires v3.1.1+";
  }
  if (cat === "rr") {
    if (opt === "Preset F") return de ? "Benötigt v310.4.0+" : "Requires v310.4.0+";
    return de ? "Benötigt v310.1.0+" : "Requires v310.1.0+";
  }
  return de ? "Benötigt v310.1.0+" : "Requires v310.1.0+";
}

function buildPresetOptionsHtml(cat, selectedVal, recVal, dllVersion) {
  const de = isDe();
  const supported = getSupportedPresetsForDllVersion(cat, dllVersion);
  const all = getAllPresetsForCategory(cat);
  if (selectedVal && !all.includes(selectedVal)) all.push(selectedVal);
  return all
    .map((o) => {
      const isSupported = supported.includes(o);
      const label = getPresetOptionLabel(cat, o, recVal, dllVersion);
      if (!isSupported) {
        const hint = getPresetMinDllHint(cat, o, de);
        return `<option value="${o}" disabled style="color:#6b7280" ${selectedVal === o ? "selected" : ""}>${escapeHtml(
          `${label} [${de ? "Nicht unterstützt" : "Unsupported"} · ${hint}]`
        )}</option>`;
      }
      return `<option value="${o}" ${selectedVal === o ? "selected" : ""}>${escapeHtml(label)}</option>`;
    })
    .join("");
}

function getPresetDescription(cat, opt, dllVersion = null) {
  const de = isDe();
  const [major, minor] = parseVerParts(dllVersion);
  if (cat === "sr" && opt === "Preset K" && major === 310 && minor < 4) {
    return de
      ? `DLSS 4.0 Transformer-Modell (Gen 1) deiner aktuell installierten DLL (v${dllVersion}): Beste Schärfe & Bildruhe für v310.2/310.3. Hinweis: Die Original-DLL unterstützt noch KEIN DLSS 4.5 (Gen 2 / Preset L/M) — dafür oben auf v310.9.16 aktualisieren!`
      : `DLSS 4.0 Transformer model (Gen 1) of your currently installed DLL (v${dllVersion}): Best sharpness & stability for v310.2/310.3. Note: Stock DLL does NOT support DLSS 4.5 (Gen 2 / Preset L/M) — update to v310.9.16 above for DLSS 4.5!`;
  }
  const meta = DLSS_PRESET_META[cat]?.[opt] || DLSS_PRESET_META[cat]?.Default;
  return meta ? (de ? meta.desc_de : meta.desc_en) : "";
}

function buildDlssVersionMatrixTableHtml() {
  const de = isDe();
  return `
    <div class="ref-table-wrap">
      <table class="ref-matrix-table">
        <thead>
          <tr>
            <th>${de ? "DLL-Version" : "DLL Version"}</th>
            <th>${de ? "Generation & Architektur" : "Generation & Architecture"}</th>
            <th>${de ? "Super Resolution (SR) Presets" : "Super Resolution (SR) Presets"}</th>
            <th>${de ? "Ray Recon (RR)" : "Ray Recon (RR)"}</th>
            <th>${de ? "Frame Gen (FG)" : "Frame Gen (FG)"}</th>
            <th>${de ? "Hinweise & Hardware" : "Notes & Hardware"}</th>
          </tr>
        </thead>
        <tbody>
          <tr class="ref-row-highlight">
            <td><span class="ref-ver-pill">v310.9.0 – 310.9.16+</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss45">★ DLSS 4.5 (Transformer Gen 2)</span></td>
            <td><strong>Preset J, K ★, L, M</strong><br/><span class="muted">${de
      ? "+ CNN Preset A–E (im 74,18 MB Full-Build)"
      : "+ CNN Preset A–E (in 74.18 MB Full Build)"
    }</span></td>
            <td><strong>Preset D, F ★</strong></td>
            <td><strong>Preset A, B</strong><br/><span class="muted">${de ? "Dynamic MFG bis 6×" : "Dynamic MFG up to 6×"
    }</span></td>
            <td>${de
      ? "Unterstützt alle neuen DLSS 4.5 Presets (J/K/L/M & RR Preset F). SR/RR ab RTX 20+, FG ab RTX 40+, MFG 3×–6× auf RTX 50."
      : "Supports all DLSS 4.5 presets (J/K/L/M & RR Preset F). SR/RR on RTX 20+, FG on RTX 40+, MFG 3×–6× on RTX 50."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v310.4.0 – 310.8.0</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss45">DLSS 4.5 (Early Gen 2)</span></td>
            <td><strong>Preset J, K ★</strong><br/><span class="muted">${de ? "Kein Preset L / M" : "No Preset L / M"
    }</span></td>
            <td><strong>Preset D, F ★</strong></td>
            <td><strong>Preset A, B</strong><br/><span class="muted">MFG 3× / 4×</span></td>
            <td>${de
      ? "Einführung des überarbeiteten Transformer Gen 2 Modells. ⚠️ v310.7.x ist ein Dev-Build mit Wasserzeichen (wird blockiert)."
      : "Introduced refined Transformer Gen 2 weights. ⚠️ v310.7.x is a watermarked dev build (automatically blocked)."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v310.2.1 – 310.3.0</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss40">DLSS 4.0 (Transformer Gen 1)</span></td>
            <td><strong>Preset J, K</strong> + <strong>CNN A–E</strong><br/><span class="muted" style="color:#fbbf24">${de
      ? "Kein DLSS 4.5 · Kein Preset L / M"
      : "No DLSS 4.5 · No Preset L / M"
    }</span></td>
            <td><strong>Preset D, E</strong><br/><span class="muted">${de ? "Kein Preset F" : "No Preset F"
    }</span></td>
            <td><strong>Preset A, B</strong><br/><span class="muted">MFG 3× / 4×</span></td>
            <td>${de
      ? "Erste DLSS 4.0 Releases (z. B. Standard-DLL v310.2.1.0 in Star Citizen). Unterstützt Transformer Gen 1 (J/K) & CNN (E), aber noch kein DLSS 4.5 (L/M)."
      : "Initial DLSS 4.0 releases (e.g. stock v310.2.1.0 in Star Citizen). Supports Transformer Gen 1 (J/K) & CNN (E), but no DLSS 4.5 (L/M)."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v310.1.0</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss40">DLSS 4.0 (Launch Build)</span></td>
            <td><strong>Preset J</strong> + <strong>CNN A–E</strong><br/><span class="muted">${de ? "Noch kein Preset K / L / M" : "No Preset K / L / M yet"
    }</span></td>
            <td><strong>Preset D</strong></td>
            <td><strong>Preset A</strong></td>
            <td>${de
      ? "Erster DLSS 4.0 Release: Enthält nur das erste Transformer-Modell (Preset J), noch kein Preset K."
      : "Initial DLSS 4.0 launch binary: Only includes the first Transformer model (Preset J), no Preset K yet."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v3.7.0 – 3.8.10</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss3">DLSS 3.7 / 3.8 (CNN Final)</span></td>
            <td><strong>Preset A, B, C, D, E ★, F</strong><br/><span class="muted" style="color:#fbbf24">${de ? "Kein Transformer (J/K/L/M)" : "No Transformer (J/K/L/M)"
    }</span></td>
            <td><strong>CNN (A–C)</strong></td>
            <td><strong>DLSS 3.x FG</strong><br/><span class="muted">${de ? "Nur 2× (RTX 40/50)" : "2× only (RTX 40/50)"
    }</span></td>
            <td>${de
      ? "Letzte reine CNN-Generation. Einführung von Preset E (bestes CNN-Profil). Unterstützt keine Transformer-Presets."
      : "Final pure CNN generation. Introduced Preset E (best CNN profile). Does not support Transformer presets."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v3.1.1 – 3.6.0</span></td>
            <td><span class="ref-gen-badge ref-gen-dlss3">DLSS 3.1 – 3.6 (CNN)</span></td>
            <td><strong>Preset A, B, C, D, F</strong><br/><span class="muted">${de ? "Noch kein Preset E oder J–M" : "No Preset E or J–M"
    }</span></td>
            <td><strong>CNN (ab 3.5)</strong></td>
            <td><strong>DLSS 3.x FG</strong><br/><span class="muted">Optical Flow</span></td>
            <td>${de
      ? "Einführung des Preset-Systems (A–F) ab v3.1.1 und Ray Reconstruction ab v3.5.0."
      : "Introduced the DLSS preset system (A–F) in v3.1.1 and Ray Reconstruction in v3.5.0."
    }</td>
          </tr>
          <tr>
            <td><span class="ref-ver-pill">v2.0.0 – 2.5.1</span></td>
            <td><span class="ref-gen-badge ref-gen-legacy">DLSS 2.x (Legacy CNN)</span></td>
            <td><span class="muted">${de ? "Festes Modell (keine Presets)" : "Fixed model (no presets)"
    }</span></td>
            <td>—</td>
            <td>—</td>
            <td>${de
      ? "Klassisches DLSS 2 Super Resolution auf RTX 20/30/40/50. Kann direkt auf v310.9.16 (DLSS 4.5) aktualisiert werden."
      : "Classic DLSS 2 Super Resolution on RTX 20/30/40/50. Can be upgraded directly to v310.9.16 (DLSS 4.5)."
    }</td>
          </tr>
        </tbody>
      </table>
    </div>`;
}

function buildVersionReferenceTablesHtml() {
  const de = isDe();
  return `
    <div class="ref-section-block">
      <div class="ref-section-title">
        <span>📊 ${de
      ? "1. NVIDIA DLSS Versions-Historie & Unterstützte Presets (Was unterstützt welche Version?)"
      : "1. NVIDIA DLSS Version History & Supported Presets (Which version supports what?)"
    }</span>
        <span class="muted" style="font-size:0.74rem;font-weight:500">${de
      ? "Offizieller NVIDIA Upstream (NVIDIA/DLSS & NGX Driver Cache)"
      : "Official NVIDIA Upstream (NVIDIA/DLSS & NGX Driver Cache)"
    }</span>
      </div>
      ${buildDlssVersionMatrixTableHtml()}
    </div>

    <div class="ref-section-block">
      <div class="ref-section-title">
        <span>🎛️ ${de
      ? "2. NVIDIA DLSS Presets im Detail — Architektur, Mindestversion & Einsatzzweck"
      : "2. NVIDIA DLSS Presets Reference — Architecture, Minimum Version & Use Case"
    }</span>
      </div>
      <div class="ref-table-wrap">
        <table class="ref-matrix-table">
          <thead>
            <tr>
              <th>Preset</th>
              <th>${de ? "Technologie" : "Technology"}</th>
              <th>${de ? "Architektur" : "Architecture"}</th>
              <th>${de ? "Mindest-DLL" : "Min. DLL Version"}</th>
              <th>${de ? "Optimaler Modus" : "Optimal Quality Mode"}</th>
              <th>${de ? "Eigenschaften & Empfehlung" : "Characteristics & Recommendation"}</th>
            </tr>
          </thead>
          <tbody>
            <tr class="ref-row-highlight">
              <td><span class="ref-ver-pill">Preset K ★</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-dlss45">Transformer Gen 2</span></td>
              <td><code>v310.2.1+</code><br/><span class="muted">${de ? "(Gen 2 ab 310.4+)" : "(Gen 2 in 310.4+)"}</span></td>
              <td><strong>Quality / Balanced / DLAA</strong></td>
              <td>${de
      ? "★ Standard-Empfehlung für fast alle Spiele: Schärfste Details (HUD, Schrift, entfernte Objekte) bei höchster temporaler Bildruhe ohne Schimmern."
      : "★ Recommended default for almost all games: Sharpest details (HUD, text, distant geometry) with highest temporal stability and minimal shimmering."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">Preset J</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-dlss40">Transformer Gen 1</span></td>
              <td><code>v310.1.0+</code></td>
              <td><strong>Quality / Balanced</strong></td>
              <td>${de
      ? "Erstes Transformer-Modell: Extrem hohe Bewegungsschärfe und sehr wenig Ghosting, neigt auf feinen Gittern/Zäunen aber etwas eher zu Schimmern als Preset K."
      : "First Transformer model: Very high motion clarity and minimal ghosting, but slightly more prone to shimmering on fine meshes/fences than Preset K."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">Preset L</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-dlss45">Transformer Gen 2 (DLSS 4.5)</span></td>
              <td><code>v310.9.0+</code></td>
              <td><strong>Performance / Ultra-Perf. (4K)</strong></td>
              <td>${de
      ? "Speziell in DLSS 4.5 für starke Skalierungsfaktoren (2× / 3× im Performance- & Ultra-Performance-Modus bei 4K/UHD) trainiertes Transformer-Profil."
      : "Specifically trained in DLSS 4.5 for aggressive scaling ratios (2× / 3× in 4K Performance & Ultra-Performance modes)."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">Preset M</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-dlss45">Transformer Gen 2 (DLSS 4.5)</span></td>
              <td><code>v310.9.0+</code></td>
              <td><strong>DLAA / Quality (RTX 40/50)</strong></td>
              <td>${de
      ? "Maximale Bildqualität und feinste Kantenglättung für DLAA & Quality in DLSS 4.5 (etwas höherer Tensor-Rechenaufwand als Preset K)."
      : "Maximum image quality and anti-aliasing stability for DLAA & Quality in DLSS 4.5 (slightly higher Tensor compute cost than Preset K)."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">Preset E</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-dlss3">CNN (Final)</span></td>
              <td><code>v3.7.0+</code><br/><span class="muted">${de ? "(im 74 MB 310.9.16)" : "(in 74 MB 310.9.16)"}</span></td>
              <td><strong>Quality / Balanced / Perf.</strong></td>
              <td>${de
      ? "Bestes klassisches Convolutional-Neural-Network (CNN) Profil. Geringerer GPU-Aufwand auf älteren GPUs und Fallback für Spiele wie Star Citizen (r.TSR.UpscalingPreset=0)."
      : "Best classic Convolutional Neural Network (CNN) profile. Lower GPU overhead on older cards and default fallback for games like Star Citizen (r.TSR.UpscalingPreset=0)."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">Preset A – D / F</span></td>
              <td>DLSS Super Resolution</td>
              <td><span class="ref-gen-badge ref-gen-legacy">CNN (Legacy)</span></td>
              <td><code>v3.1.1+</code></td>
              <td><strong>Legacy / Ultra-Perf. (F)</strong></td>
              <td>${de
      ? "Ältere CNN-Profile aus DLSS 3.1–3.6 (C = schnelle Bewegung, D = Bildruhe, F = Ultra-Performance). Heute durch Preset K und L abgelöst."
      : "Older CNN profiles from DLSS 3.1–3.6 (C = fast motion, D = stability, F = Ultra-Performance). Superseded today by Presets K and L."
    }</td>
            </tr>
            <tr class="ref-row-highlight">
              <td><span class="ref-ver-pill">RR · Preset F ★</span></td>
              <td>DLSS Ray Reconstruction</td>
              <td><span class="ref-gen-badge ref-gen-dlss45">Transformer Gen 2</span></td>
              <td><code>v310.4.0+</code></td>
              <td><strong>Raytracing / Pathtracing</strong></td>
              <td>${de
      ? "★ Empfohlen für alle Spiele mit Ray Reconstruction (z. B. Cyberpunk 2077, Alan Wake 2): Deutlich schärfere Reflexionen und weniger Ghosting als Preset D."
      : "★ Recommended for all Ray Reconstruction games (e.g. Cyberpunk 2077, Alan Wake 2): Much cleaner reflections and less ghosting than Preset D."
    }</td>
            </tr>
            <tr>
              <td><span class="ref-ver-pill">RR · Preset D</span></td>
              <td>DLSS Ray Reconstruction</td>
              <td><span class="ref-gen-badge ref-gen-dlss40">Transformer Gen 1</span></td>
              <td><code>v310.1.0+</code></td>
              <td><strong>Raytracing / Pathtracing</strong></td>
              <td>${de
      ? "Erstes Transformer-Denoiser-Modell für Ray Reconstruction aus DLSS 4.0."
      : "First-generation Transformer denoiser model for Ray Reconstruction from DLSS 4.0."
    }</td>
            </tr>
            <tr class="ref-row-highlight">
              <td><span class="ref-ver-pill">FG · Latest / A / B</span></td>
              <td>DLSS Frame Generation</td>
              <td><span class="ref-gen-badge ref-gen-dlss45">AI Frame Gen + Flip Metering</span></td>
              <td><code>v310.1.0+</code></td>
              <td><strong>2× (RTX 40) / 3×–6× (RTX 50)</strong></td>
              <td>${de
      ? "Neuestes KI-Frame-Generation-Modell (~30% weniger VRAM-Bedarf, höhere FPS). Preset B dient als Alternative, falls HUD-/Fadenkreuz-Artefakte auftreten."
      : "Newest AI Frame Generation model (~30% lower VRAM usage, higher FPS). Preset B serves as an alternative if HUD/crosshair artifacts appear."
    }</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="ref-section-block">
      <div class="ref-section-title">
        <span>🖥️ ${de
      ? "3. Alle Upscaler- & Streaming-Technologien — DLL-Dateien & Hardware-Kompatibilität"
      : "3. All Upscaler & Streaming Technologies — DLL Files & Hardware Compatibility"
    }</span>
      </div>
      <div class="ref-table-wrap">
        <table class="ref-matrix-table">
          <thead>
            <tr>
              <th>${de ? "Technologie" : "Technology"}</th>
              <th>${de ? "Aktuelle Version" : "Latest Version"}</th>
              <th>${de ? "DLL-Dateien im Spielordner" : "DLL Files in Game Folder"}</th>
              <th>${de ? "Unterstützte Grafikkarten (Hardware)" : "Supported Graphics Cards (Hardware)"}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>DLSS Super Resolution (SR)</strong></td>
              <td><span class="ref-ver-pill">v310.9.16.0</span> (DLSS 4.5)</td>
              <td><code>nvngx_dlss.dll</code></td>
              <td>NVIDIA GeForce <strong>RTX 20, RTX 30, RTX 40, RTX 50</strong> (Turing, Ampere, Ada, Blackwell)</td>
            </tr>
            <tr>
              <td><strong>DLSS Ray Reconstruction (RR)</strong></td>
              <td><span class="ref-ver-pill">v310.9.16.0</span> (DLSS 4.5)</td>
              <td><code>nvngx_dlssd.dll</code></td>
              <td>NVIDIA GeForce <strong>RTX 20, RTX 30, RTX 40, RTX 50</strong></td>
            </tr>
            <tr>
              <td><strong>DLSS Frame Generation (FG / MFG)</strong></td>
              <td><span class="ref-ver-pill">v310.9.16.0</span> (DLSS 4.5)</td>
              <td><code>nvngx_dlssg.dll</code></td>
              <td><strong>RTX 40 & RTX 50</strong> (2× FG) · <strong>RTX 50 Blackwell</strong> (3×/4×/6× Multi Frame Gen)</td>
            </tr>
            <tr>
              <td><strong>DLSS 5 Neural Rendering (NR)</strong></td>
              <td><span class="ref-ver-pill">v310.8.0.0</span></td>
              <td><code>nvngx_dlssnr.dll</code></td>
              <td>NVIDIA GeForce <strong>RTX 50 Serie (Blackwell)</strong></td>
            </tr>
            <tr>
              <td><strong>NVIDIA Streamline SDK</strong></td>
              <td><span class="ref-ver-pill">v2.14.16.0</span></td>
              <td><code>sl.interposer.dll</code>, <code>sl.common.dll</code>, <code>sl.dlss.dll</code>, <code>sl.dlss_g.dll</code>, <code>sl.dlss_d.dll</code>, <code>sl.reflex.dll</code>, <code>sl.pcl.dll</code></td>
              <td>NVIDIA GeForce <strong>RTX 20 / 30 / 40 / 50</strong> (Reflex & PCL auch auf GTX 900/10/16)</td>
            </tr>
            <tr>
              <td><strong>Intel XeSS 2 (SR / FG / XeLL)</strong></td>
              <td><span class="ref-ver-pill">v2.0.2.68</span></td>
              <td><code>libxess.dll</code>, <code>libxess_dx11.dll</code>, <code>libxess_fg.dll</code>, <code>libxell.dll</code></td>
              <td><strong>XeSS SR:</strong> Alle GPUs (XMX auf Intel Arc, DP4a auf NVIDIA/AMD) · <strong>XeSS-FG:</strong> Intel Arc</td>
            </tr>
            <tr>
              <td><strong>Microsoft DirectStorage</strong></td>
              <td><span class="ref-ver-pill">v1.2.2504+</span></td>
              <td><code>dstorage.dll</code>, <code>dstoragecore.dll</code></td>
              <td><strong>Alle DirectX-12-fähigen GPUs</strong> (NVIDIA, AMD, Intel) + NVMe SSD</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>`;
}

window.openVersionReferenceModal = function () {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  if (!backdrop || !modal) return;
  const de = isDe();

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>📊 ${de
      ? "Versions-, Preset- & Hardware-Referenzübersicht"
      : "Version, Preset & Hardware Reference Matrix"
    }</h3>
        <div class="muted" style="font-size:0.78rem">
          ${de
      ? "Komplette tabellarische Übersicht: Welche DLL-Version unterstützt welche DLSS-Generation, welche Presets (A–M) und welche Grafikkarte?"
      : "Complete tabular reference: Which DLL version supports which DLSS generation, presets (A–M), and GPU architecture?"
    }
        </div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"
    }</button>
    </div>
    <div class="modal-body" style="gap:16px">
      ${buildVersionReferenceTablesHtml()}
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick="closeSecondaryModal()">✓ ${de ? "Verstanden" : "Got it"
    }</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

function buildPresetGuideHtml(hasSr, hasRr, hasFg) {
  const de = isDe();
  const items = [];
  if (hasSr) {
    for (const k of ["Preset K", "Preset J", "Preset L", "Preset M", "Latest", "Default"]) {
      const m = DLSS_PRESET_META.sr[k];
      items.push({
        badge: `SR · ${k}`,
        rec: k === "Preset K",
        desc: de ? m.desc_de : m.desc_en,
      });
    }
  }
  if (hasRr) {
    for (const k of ["Preset F", "Preset D"]) {
      const m = DLSS_PRESET_META.rr[k];
      items.push({
        badge: `RR · ${k}`,
        rec: k === "Preset F",
        desc: de ? m.desc_de : m.desc_en,
      });
    }
  }
  if (hasFg) {
    for (const k of ["Latest", "Preset A", "Preset B"]) {
      const m = DLSS_PRESET_META.fg[k];
      items.push({
        badge: `FG · ${k}`,
        rec: k === "Latest",
        desc: de ? m.desc_de : m.desc_en,
      });
    }
  }
  return `
    <details class="preset-guide-accordion">
      <summary>
        <span>📖 ${de
      ? "Alle DLSS-Presets & Versions-Kompatibilität erklärt (Welche Version unterstützt was?)"
      : "All DLSS Presets & Version Compatibility Explained (Which version supports what?)"
    }</span>
        <span style="font-size:0.72rem;opacity:0.85">${de ? "Tabelle & Übersicht öffnen ▾" : "Show table & guide ▾"}</span>
      </summary>
      <div class="preset-guide-list">
        ${items
      .map(
        (it) => `
          <div class="preset-guide-item" style="${it.rec ? "border-color:rgba(118,185,0,0.42);background:rgba(118,185,0,0.06)" : ""}">
            <div style="display:flex;align-items:center;justify-content:space-between;gap:6px">
              <strong style="color:${it.rec ? "#a3e635" : "var(--text-primary)"};font-size:0.76rem">${escapeHtml(it.badge)}</strong>
              ${it.rec ? `<span class="preset-slot-rec">★ ${de ? "Empfohlen" : "Recommended"}</span>` : ""}
            </div>
            <span class="muted">${escapeHtml(it.desc)}</span>
          </div>`
      )
      .join("")}
      </div>
      <div style="margin-top:12px;padding-top:10px;border-top:1px solid var(--border-subtle);display:flex;flex-direction:column;gap:8px">
        <div style="display:flex;align-items:center;justify-content:space-between;gap:8px;flex-wrap:wrap">
          <strong style="font-size:0.77rem;color:var(--text-primary)">📊 ${de
      ? "DLSS DLL-Versionen & Unterstützte Presets (Referenztabelle)"
      : "DLSS DLL Versions & Supported Presets (Reference Table)"
    }</strong>
          <button type="button" class="btn btn-secondary btn-card-action" onclick="openVersionReferenceModal()" style="padding:3px 9px;font-size:0.72rem">
            📊 ${de ? "Komplette Referenz öffnen" : "Open Full Reference"}
          </button>
        </div>
        ${buildDlssVersionMatrixTableHtml()}
      </div>
    </details>`;
}

function gameHasRrTech(game) {
  return (game.dlls || []).some(
    (d) => d.type_label === "DLSS-RR" || (d.filename || "").toLowerCase() === "sl.dlss_d.dll"
  );
}

function gameHasFgTech(game) {
  return (game.dlls || []).some(
    (d) => d.type_label === "DLSS-G" || (d.filename || "").toLowerCase() === "sl.dlss_g.dll"
  );
}

window.quickSaveInlineGamePresets = async function (gameId, reset = false) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;
  const hasSrTech = (game.dlls || []).some((d) => d.type_label === "DLSS");
  const hasRrTech = gameHasRrTech(game);
  const hasFgTech = gameHasFgTech(game);

  const srPreset =
    reset || !hasSrTech
      ? "Default"
      : document.getElementById("inline-sr-preset")?.value || game.dlss_sr_preset || "Default";
  const rrPreset =
    reset || !hasRrTech
      ? "Default"
      : document.getElementById("inline-rr-preset")?.value || game.dlss_rr_preset || "Default";
  const fgPreset =
    reset || !hasFgTech
      ? "Default"
      : document.getElementById("inline-fg-preset")?.value || game.dlss_fg_preset || "Default";
  const fgMode =
    reset || !hasFgTech
      ? "Default"
      : document.getElementById("inline-fg-mode")?.value || game.dlss_fg_mode || "Default";
  const fgMultRaw = document.getElementById("inline-fg-mult")?.value;
  const fgMultiplier =
    reset || !hasFgTech ? null : parseInt(fgMultRaw || game.dlss_fg_multiplier || "2", 10) || 2;
  const fgFpsRaw = document.getElementById("inline-fg-target-fps")?.value;
  const fgTargetFps =
    reset || !hasFgTech
      ? null
      : fgFpsRaw !== undefined
        ? parseInt(fgFpsRaw, 10) || null
        : game.dlss_fg_target_fps || null;

  try {
    const payload = await invoke("save_game_dlss_presets", {
      gameId,
      srPreset,
      rrPreset,
      fgPreset,
      fgMode,
      fgMultiplier,
      fgTargetFps,
      customExePath: game.custom_exe_path || null,
    });
    applyPayload(payload);
    const activeParts = [];
    if (hasSrTech) activeParts.push(`SR: ${srPreset}`);
    if (hasRrTech) activeParts.push(`RR: ${rrPreset}`);
    if (hasFgTech) activeParts.push(`FG: ${fgPreset}`);
    const summaryStr = activeParts.join(" · ") || `SR: ${srPreset}`;
    showToast(
      reset
        ? isDe()
          ? "↺ DLSS-Presets auf Treiber-Standard zurückgesetzt."
          : "↺ Reset DLSS presets to Driver Default."
        : isDe()
          ? `✓ NVIDIA Treiber-Presets aktiv (${summaryStr})`
          : `✓ Applied NVIDIA Driver Presets (${summaryStr})`
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

window.openGameDllModal = function (gameId) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;
  appState.activeModalGameId = gameId;

  const backdrop = document.getElementById("dll-modal-backdrop");
  const modal = document.getElementById("dll-modal-content");
  const counts = getGameUpdateCounts(game);
  const isNvidia = (appState.gpu?.vendor || "").toUpperCase() === "NVIDIA";
  const de = isDe();
  const dbProf = game.db_profile;

  const hasSrTech = counts.groups.some((g) => g.typeLabel === "DLSS");
  const hasRrTech = gameHasRrTech(game);
  const hasFgTech = gameHasFgTech(game);

  const gameSr = hasSrTech ? game.dlss_sr_preset || "Default" : "Default";
  const gameRr = hasRrTech ? game.dlss_rr_preset || "Default" : "Default";
  const gameFg = hasFgTech ? game.dlss_fg_preset || "Default" : "Default";
  const globalSr = appState.config?.global_dlss_preset || "Default";
  const globalRr = appState.config?.global_dlss_rr_preset || "Default";
  const globalFg = appState.config?.global_dlss_fg_preset || "Default";

  const effectiveSr = !hasSrTech
    ? "Default"
    : gameSr !== "Default"
      ? gameSr
      : globalSr !== "Default"
        ? `${globalSr} (Global)`
        : "Default";
  const effectiveRr = !hasRrTech
    ? "Default"
    : gameRr !== "Default"
      ? gameRr
      : globalRr !== "Default"
        ? `${globalRr} (Global)`
        : "Default";
  const effectiveFg = !hasFgTech
    ? "Default"
    : gameFg !== "Default"
      ? gameFg
      : globalFg !== "Default"
        ? `${globalFg} (Global)`
        : "Default";

  const isDrsOnlyGame =
    (dbProf?.anticheat_status || "").toLowerCase() === "driver_override_only" ||
    (dbProf?.anticheat_status || "").toLowerCase() === "drs_only" ||
    (dbProf?.anticheat_status || "").toLowerCase() === "blocked";

  // 1. Installed Upscaler DLL Rows (with active preset badge right inside the row)
  const groupRows = counts.groups
    .map((grp) => {
      const fullFileList = grp.dlls.map((d) => d.filename).join(", ");
      const shortFileList =
        grp.dlls.length <= 2
          ? fullFileList
          : `${grp.dlls.length} ${de ? "Dateien" : "files"} (${grp.dlls
            .slice(0, 2)
            .map((d) => d.filename)
            .join(", ")}, …)`;
      const encodedType = escapeJsStr(grp.typeLabel);

      const techBackupsCount = appState.backups.filter(
        (b) =>
          b.game_id === game.id &&
          grp.dlls.some((d) => d.filename.toLowerCase() === b.filename.toLowerCase())
      ).length;

      let activeTechPresetBadge = "";
      if (isNvidia) {
        if (grp.typeLabel === "DLSS") {
          activeTechPresetBadge =
            effectiveSr !== "Default"
              ? `<span class="active-preset-chip">🎛️ ${de ? "Aktiv:" : "Active:"} ${escapeHtml(effectiveSr)}</span>`
              : `<span class="default-preset-chip">🎛️ ${de ? "Preset: Standard" : "Preset: Default"}</span>`;
        } else if (grp.typeLabel === "DLSS-RR") {
          activeTechPresetBadge =
            effectiveRr !== "Default"
              ? `<span class="active-preset-chip">🎛️ ${de ? "Aktiv:" : "Active:"} ${escapeHtml(effectiveRr)}</span>`
              : `<span class="default-preset-chip">🎛️ ${de ? "Preset: Standard" : "Preset: Default"}</span>`;
        } else if (grp.typeLabel === "DLSS-G") {
          activeTechPresetBadge =
            effectiveFg !== "Default"
              ? `<span class="active-preset-chip">🎛️ ${de ? "Aktiv:" : "Active:"} ${escapeHtml(effectiveFg)}</span>`
              : `<span class="default-preset-chip">🎛️ ${de ? "Preset: Standard" : "Preset: Default"}</span>`;
        }
      }

      return `
      <div class="dll-row" style="${!grp.technologyEnabled ? "opacity:0.58" : ""}">
        <div class="dll-row-info">
          <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
            <span class="dll-badge">${escapeHtml(grp.typeLabel)}</span>
            <span class="dll-ver-text">v${escapeHtml(grp.version)}</span>
            ${grp.updateAvailable
          ? `<span style="color:#76b900;font-weight:600;font-size:0.82rem">→ v${escapeHtml(grp.latestVersion)}</span>`
          : isDrsOnlyGame
            ? `<span class="muted" style="font-size:0.74rem;margin-left:2px">🛡️ ${de ? "Original-DLL (EAC-geschützt)" : "Stock DLL (EAC Protected)"
            }</span>`
            : !grp.technologyEnabled
              ? `<span class="muted" style="font-size:0.74rem;margin-left:4px">${de ? "(Deaktiviert)" : "(Disabled)"
              }</span>`
              : `<span class="muted" style="font-size:0.75rem;margin-left:2px">✓ ${de ? "Aktuell" : "Up to date"}</span>`
        }
            ${activeTechPresetBadge}
            ${grp.badBuildWarning
          ? `<span class="anticheat-badge" title="${escapeHtml(grp.badBuildWarning)}">⚠️ ${de ? "Wasserzeichen-Build" : "Watermark Build"
          }</span>`
          : ""
        }
          </div>
          <div class="dll-preset-sub" title="${escapeHtml(fullFileList)}">
            ${grp.presetLabel ? escapeHtml(grp.presetLabel) + " · " : ""}<code>${escapeHtml(shortFileList)}</code>
          </div>
        </div>
        <div class="dll-row-action" style="gap:6px">
          ${techBackupsCount > 0
          ? `<button class="btn btn-secondary btn-card-action" onclick="restoreTechGroupForGame('${game.id}', '${encodedType}')" title="${de
            ? `Originale ${escapeHtml(grp.typeLabel)}-Dateien wiederherstellen`
            : `Restore original ${escapeHtml(grp.typeLabel)} files`
          }">
                   ⏪ ${de ? "Backup" : "Restore"}
                 </button>`
          : ""
        }
          ${isDrsOnlyGame
          ? `<span class="compat-status-pill compat-warn" title="${de
            ? "EasyAntiCheat prüft den Datei-Hash dieser DLL auf der Festplatte. Nutze das Treiber-DRS-Preset darunter!"
            : "Anti-Cheat verifies the file hash of this DLL on disk. Use the Driver DRS Preset below!"
          }">🛡️ ${de ? "Nur Treiber-DRS" : "Driver DRS Only"}</span>`
          : grp.updateAvailable
            ? `<button class="btn btn-primary btn-card-action" onclick="updateTechGroupForGame('${game.id}', '${encodedType}', false)">
                   ⚡ Update
                 </button>`
            : `<button class="btn btn-secondary btn-card-action" onclick="updateTechGroupForGame('${game.id}', '${encodedType}', true)" title="${de
              ? `${escapeHtml(grp.typeLabel)}-DLLs neu installieren`
              : `Reinstall ${escapeHtml(grp.typeLabel)} DLLs`
            }">
                   🔄 ${de ? "Neu installieren" : "Reinstall"}
                 </button>`
        }
        </div>
      </div>`;
    })
    .join("");

  // 2. Dedicated Applied NVIDIA DLSS Presets Bar (DRS)
  let appliedPresetsPanel = "";
  const hasDlssTech = counts.groups.some((g) =>
    ["DLSS", "DLSS-RR", "DLSS-G", "Streamline"].includes(g.typeLabel)
  );
  if (isNvidia && (hasDlssTech || dbProf?.recommended_sr_preset)) {
    const srDllVer = counts.groups.find((g) => g.typeLabel === "DLSS")?.version || "310.9.16.0";
    const rrDllVer = counts.groups.find((g) => g.typeLabel === "DLSS-RR")?.version || "310.9.16.0";
    const fgDllVer = counts.groups.find((g) => g.typeLabel === "DLSS-G")?.version || "310.9.16.0";

    const srOptions = getSupportedPresetsForDllVersion("sr", srDllVer);
    const rrOptions = getSupportedPresetsForDllVersion("rr", rrDllVer);
    const fgOptions = getSupportedPresetsForDllVersion("fg", fgDllVer);
    if (!srOptions.includes(gameSr)) srOptions.push(gameSr);
    if (!rrOptions.includes(gameRr)) rrOptions.push(gameRr);
    if (!fgOptions.includes(gameFg)) fgOptions.push(gameFg);

    const recSr = dbProf?.recommended_sr_preset || null;
    const recRr = dbProf?.recommended_rr_preset || null;
    const recFg = dbProf?.recommended_fg_preset || null;
    const hasRec = !!(recSr || recRr || recFg);

    const isOptimalApplied =
      hasRec &&
      (!recSr || gameSr === recSr) &&
      (!recRr || gameRr === recRr) &&
      (!recFg || gameFg === recFg);
    const hasAnyCustom = gameSr !== "Default" || gameRr !== "Default" || gameFg !== "Default";

    const statusBadge = isOptimalApplied
      ? `<span class="compat-status-pill compat-safe">✓ ${de ? "Optimales Preset aktiv" : "Optimal Preset Active"} (${escapeHtml(recSr || gameSr)})</span>`
      : hasAnyCustom
        ? `<span class="compat-status-pill compat-safe">● ${de ? "Eigenes Preset aktiv" : "Custom Preset Active"}</span>`
        : `<span class="default-preset-chip">○ ${de ? "Treiber-Standard (Default)" : "Driver Default"}</span>`;

    const liveDrsBadge = `<span id="live-drs-readback-badge" class="default-preset-chip" style="margin-left:6px">🔎 ${de ? "Treiber-DRS prüfen…" : "Checking Driver DRS…"}</span>`;

    const slotCardsHtml = [];
    if (hasSrTech || (!hasRrTech && !hasFgTech)) {
      slotCardsHtml.push(`
        <div class="preset-slot-card ${gameSr !== "Default" ? "is-active" : ""}">
          <div class="preset-slot-top">
            <span class="preset-slot-title">DLSS Super Resolution (SR) · <code style="font-size:0.72rem;opacity:0.85">v${escapeHtml(srDllVer)}</code></span>
            ${recSr
          ? `<span class="preset-slot-rec" title="${de ? "Empfohlenes Preset" : "Recommended Preset"}">★ ${de ? "Empfohlen:" : "Recommended:"} ${escapeHtml(recSr)}</span>`
          : ""
        }
          </div>
          <select id="inline-sr-preset" class="preset-slot-select" onchange="quickSaveInlineGamePresets('${game.id}', false)">
            ${buildPresetOptionsHtml("sr", gameSr, recSr, srDllVer)}
          </select>
          <div class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("sr", gameSr, srDllVer))}</div>
        </div>
      `);
    }
    if (hasRrTech) {
      slotCardsHtml.push(`
        <div class="preset-slot-card ${gameRr !== "Default" ? "is-active" : ""}">
          <div class="preset-slot-top">
            <span class="preset-slot-title">DLSS Ray Reconstruction (RR) · <code style="font-size:0.72rem;opacity:0.85">v${escapeHtml(rrDllVer)}</code></span>
            ${recRr
          ? `<span class="preset-slot-rec" title="${de ? "Empfohlenes Preset" : "Recommended Preset"}">★ ${de ? "Empfohlen:" : "Recommended:"} ${escapeHtml(recRr)}</span>`
          : ""
        }
          </div>
          <select id="inline-rr-preset" class="preset-slot-select" onchange="quickSaveInlineGamePresets('${game.id}', false)">
            ${buildPresetOptionsHtml("rr", gameRr, recRr, rrDllVer)}
          </select>
          <div class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("rr", gameRr, rrDllVer))}</div>
        </div>
      `);
    }
    if (hasFgTech) {
      slotCardsHtml.push(`
        <div class="preset-slot-card ${gameFg !== "Default" ? "is-active" : ""}">
          <div class="preset-slot-top">
            <span class="preset-slot-title">DLSS Frame Generation (FG) · <code style="font-size:0.72rem;opacity:0.85">v${escapeHtml(fgDllVer)}</code></span>
            ${recFg
          ? `<span class="preset-slot-rec" title="${de ? "Empfohlenes Preset" : "Recommended Preset"}">★ ${de ? "Empfohlen:" : "Recommended:"} ${escapeHtml(recFg)}</span>`
          : ""
        }
          </div>
          <select id="inline-fg-preset" class="preset-slot-select" onchange="quickSaveInlineGamePresets('${game.id}', false)">
            ${buildPresetOptionsHtml("fg", gameFg, recFg, fgDllVer)}
          </select>
          <div style="display:grid;grid-template-columns:1fr 90px 100px;gap:8px;margin-top:8px">
            <select id="inline-fg-mode" class="preset-slot-select" title="${de ? "DLSS-G MFG Modus (RTX 50)" : "DLSS-G MFG Mode (RTX 50)"}" onchange="quickSaveInlineGamePresets('${game.id}', false)">
              <option value="Default" ${!game.dlss_fg_mode || game.dlss_fg_mode === "Default" ? "selected" : ""}>MFG: Default</option>
              <option value="Fixed" ${game.dlss_fg_mode === "Fixed" ? "selected" : ""}>MFG: Fixed</option>
              <option value="Dynamic" ${game.dlss_fg_mode === "Dynamic" ? "selected" : ""}>MFG: Dynamic</option>
              <option value="Off" ${game.dlss_fg_mode === "Off" ? "selected" : ""}>MFG: Off</option>
            </select>
            <select id="inline-fg-mult" class="preset-slot-select" title="${de ? "Multiplikator" : "Multiplier"}" onchange="quickSaveInlineGamePresets('${game.id}', false)">
              ${[2, 3, 4, 5, 6].map((m) => `<option value="${m}" ${(game.dlss_fg_multiplier || 2) === m ? "selected" : ""}>${m}x</option>`).join("")}
            </select>
            <input type="number" id="inline-fg-target-fps" class="preset-slot-select" min="0" max="1000" value="${game.dlss_fg_target_fps || 0}" title="${de ? "Ziel-FPS (0 = Auto)" : "Target FPS (0 = Auto)"}" onchange="quickSaveInlineGamePresets('${game.id}', false)" />
          </div>
          <div class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("fg", gameFg, fgDllVer))}</div>
        </div>
      `);
    }

    appliedPresetsPanel = `
      <div class="applied-presets-box ${hasAnyCustom ? "has-active-preset" : ""}">
        <div class="compat-db-header" style="display:flex;align-items:center;justify-content:space-between;gap:10px;flex-wrap:wrap">
          <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
            <strong style="font-size:0.88rem">🎛️ ${de ? "Angewendete NVIDIA DLSS Presets (Treiber-DRS)" : "Applied NVIDIA DLSS Presets (Driver DRS)"
      }</strong>
            ${statusBadge}
            ${liveDrsBadge}
          </div>
          <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
            ${hasRec && !isOptimalApplied
        ? `<button class="btn btn-accent btn-card-action" onclick="applyRecommendedProfileNow('${game.id}')">
                     ✨ ${de ? "Optimales Preset anwenden" : "Apply Optimal Preset"} (${escapeHtml(recSr || "Preset K")})
                   </button>`
        : ""
      }
            ${hasAnyCustom
        ? `<button class="btn btn-secondary btn-card-action" onclick="quickSaveInlineGamePresets('${game.id}', true)" title="${de ? "Alle Presets auf Treiber-Standard zurücksetzen" : "Reset all presets to Driver Default"
        }">
                     ↺ ${de ? "Auf Standard zurücksetzen" : "Reset to Default"}
                   </button>`
        : ""
      }
          </div>
        </div>
        <div class="preset-slots-grid">
          ${slotCardsHtml.join("")}
        </div>
        ${buildPresetGuideHtml(hasSrTech || (!hasRrTech && !hasFgTech), hasRrTech, hasFgTech)}
      </div>
    `;
  }

  // 3. Clean & Compact Compatibility & In-Game Settings Box
  let dbCompatibilityPanel = "";
  if (dbProf || game.anti_cheat) {
    const status = (dbProf?.anticheat_status || "signed_only").toLowerCase();
    const statusMeta =
      status === "safe"
        ? {
          label: de
            ? "🟢 Sicher (Voller DLL-Tausch & DRS)"
            : "🟢 Safe (Full DLL Swap & DRS)",
          cls: "compat-safe",
        }
        : status === "driver_override_only"
          ? {
            label: de
              ? "🟠 Nur Treiber-DRS empfohlen"
              : "🟠 Driver DRS Override Recommended",
            cls: "compat-warn",
          }
          : status === "blocked"
            ? {
              label: de
                ? "🔴 Durch Anti-Cheat blockiert"
                : "🔴 Blocked by Anti-Cheat",
              cls: "compat-danger",
            }
            : {
              label: de
                ? "🟡 Nur signierte DLLs & Treiber-DRS"
                : "🟡 Signed DLLs & Driver DRS Only",
              cls: "compat-info",
            };

    const warnBanner = dbProf?.warning_banner
      ? `<div class="compat-warning-banner">⚠️ ${escapeHtml(translateDbText(dbProf.warning_banner))}</div>`
      : game.anti_cheat
        ? `<div class="compat-warning-banner">🛡️ <strong>${escapeHtml(game.anti_cheat)}:</strong> ${de
          ? "Nur offiziell signierte Upscaler-DLLs oder NVIDIA Treiber-DRS-Presets verwenden."
          : "Only use official signed upscaler DLLs or NVIDIA Driver DRS Presets."
        }</div>`
        : "";

    const recSettingsPills = (dbProf?.recommended_settings || [])
      .map((s) => {
        const localized = translateDbText(s);
        const colonIdx = localized.indexOf(":");
        if (colonIdx > 0 && colonIdx < 38) {
          const k = localized.slice(0, colonIdx).trim();
          const v = localized.slice(colonIdx + 1).trim();
          return `
            <div class="compat-setting-pill">
              <span class="compat-setting-key">${escapeHtml(k)}</span>
              <span class="compat-setting-val">${escapeHtml(v)}</span>
            </div>`;
        }
        return `
          <div class="compat-setting-pill">
            <span class="compat-setting-val">${escapeHtml(localized)}</span>
          </div>`;
      })
      .join("");

    const unsupportedList = dbProf?.unsupported_techs || [];
    const unsupportedInlineTags = unsupportedList
      .map(
        (u) =>
          `<span class="compat-unsupported-tag" style="background:rgba(245,158,11,0.12);padding:2px 7px;border-radius:999px;border:1px solid rgba(245,158,11,0.3)">🚫 ${escapeHtml(
            u.tech
          )}</span>`
      )
      .join("");

    const unsupportedDetailsRows = unsupportedList
      .map(
        (u) => `
        <div class="compat-unsupported-item">
          <span class="compat-unsupported-tag">🚫 ${escapeHtml(u.tech)}</span>
          <span class="muted" style="font-size:0.75rem">${escapeHtml(translateDbText(u.reason))}</span>
        </div>`
      )
      .join("");

    dbCompatibilityPanel = `
      <div class="compat-db-box">
        <div class="compat-db-header">
          <strong style="font-size:0.86rem">🎯 ${de ? "Kompatibilität & In-Game Empfehlungen" : "Compatibility & In-Game Recommendations"
      }</strong>
          <span class="compat-status-pill ${statusMeta.cls}">${statusMeta.label}</span>
        </div>
        ${warnBanner}
        ${recSettingsPills
        ? `<div class="compat-settings-grid">${recSettingsPills}</div>`
        : ""
      }
        ${unsupportedList.length > 0
        ? `<details class="compat-details-accordion">
                 <summary>
                   <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
                     <span>ℹ️ ${de ? "Nicht unterstützte Technologien:" : "Unsupported technologies:"
        }</span>
                     ${unsupportedInlineTags}
                   </div>
                   <span style="font-size:0.71rem;opacity:0.8">${de ? "Gründe anzeigen ▾" : "Show reasons ▾"
        }</span>
                 </summary>
                 <div class="compat-details-body">
                   ${unsupportedDetailsRows}
                 </div>
               </details>`
        : ""
      }
      </div>
    `;
  }

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;flex-wrap:wrap">
          <img src="assets/icons/${escapeHtml(game.launcher_icon)}" style="width:20px;height:20px;flex-shrink:0" alt="" />
          <h3 style="font-size:1.15rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${escapeHtml(game.name)}</h3>
          ${game.anti_cheat ? `<span class="anticheat-badge">🛡️ ${escapeHtml(game.anti_cheat)}</span>` : ""}
        </div>
        <div class="muted" style="font-size:0.75rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis" title="${escapeHtml(game.path)}">${escapeHtml(game.path)}</div>
      </div>
      <div style="display:flex;align-items:center;gap:6px;flex-shrink:0">
        <button class="btn btn-secondary btn-card-action" onclick="reportGamesToDbNow('${game.id}')" title="${de ? "Spiel & Status an dlssdb.goover.dev melden" : "Report Game & Detected Techs to dlssdb.goover.dev"}">📤</button>
        <button class="btn btn-secondary btn-card-action" onclick="openFolderInExplorer('${escapeJsStr(game.path)}')" title="${de ? "Spielordner öffnen" : "Open Game Folder"}">📂</button>
        <button class="btn btn-secondary btn-card-action" onclick="openEditDisplayModal('${game.id}')" title="${de ? "Name & Cover bearbeiten" : "Edit Name & Cover"}">✏️</button>
        ${isNvidia
      ? `<button class="btn btn-secondary btn-card-action" onclick="openGameDlssPresetsModal('${game.id}')" title="${de ? "Erweiterte DLSS Preset- & EXE-Einstellungen" : "Advanced DLSS Preset & EXE Settings"}">🎛️</button>`
      : ""
    }
        <button class="btn btn-secondary btn-card-action" onclick="launchGameNow('${game.id}')" title="${de ? "Spiel starten" : "Launch Game"}">🚀</button>
        <button class="btn btn-secondary btn-card-action" onclick="closeGameDllModal()">✕ ${de ? "Schließen" : "Close"}</button>
      </div>
    </div>
    <div class="modal-body" style="gap:10px">
      <div class="modal-section-header">
        <span>📦 ${de ? "Erkannte Upscaler-DLLs im Spielordner" : "Detected Upscaler DLLs in Game Folder"}</span>
        <span>${counts.total} ${counts.total === 1 ? (de ? "Technologie" : "Technology") : (de ? "Technologien" : "Technologies")}</span>
      </div>
      ${groupRows}
      ${appliedPresetsPanel}
      ${dbCompatibilityPanel}
    </div>
    <div class="modal-footer">
      ${game.has_backups
      ? `<button class="btn btn-secondary" onclick="restoreAllGameBackups('${game.id}')">
               ⏪ ${de ? "Alle Backups wiederherstellen" : "Restore All Backups"}
             </button>`
      : ""
    }
      ${!isDrsOnlyGame
      ? `<button class="btn btn-secondary" onclick="updateAllGameDlls('${game.id}', true)">
               🔄 ${de ? `Alle neu installieren (${counts.total})` : `Reinstall All (${counts.total})`}
             </button>`
      : ""
    }
      ${!isDrsOnlyGame && counts.outdated > 0
      ? `<button class="btn btn-accent" onclick="updateAllGameDlls('${game.id}', false)">
               ⚡ ${de ? `Alle aktualisieren (${counts.outdated})` : `Update All (${counts.outdated})`}
             </button>`
      : ""
    }
    </div>
  `;

  backdrop.style.display = "flex";

  const drsBadgeEl = document.getElementById("live-drs-readback-badge");
  if (drsBadgeEl) {
    const exeHint = game.custom_exe_path || (game.dlls && game.dlls[0] ? game.dlls[0].path : game.path);
    invoke("read_live_drs_status_cmd", { exeName: exeHint })
      .then((drs) => {
        if (!drs || !drs.nvapi_available) {
          drsBadgeEl.style.display = "none";
          return;
        }
        if (drs.is_blocked_by_zero) {
          drsBadgeEl.className = "compat-status-pill compat-warn";
          drsBadgeEl.textContent = de
            ? "⚠️ Treiber-DRS: Durch NVIDIA App blockiert (0)"
            : "⚠️ Driver DRS: Blocked by NVIDIA App (0)";
        } else if (drs.sr_override === 1 && drs.sr_preset) {
          drsBadgeEl.className = "compat-status-pill compat-safe";
          const fgPart =
            hasFgTech && drs.fg_mode && drs.fg_mode !== "Default"
              ? ` · FG ${drs.fg_mode}${drs.fg_multiplier && drs.fg_multiplier > 1 ? ` ${drs.fg_multiplier}x` : ""}`
              : "";
          drsBadgeEl.textContent = `${de ? "Treiber-Live" : "Driver Live"}: ${drs.sr_preset}${fgPart}`;
        } else {
          drsBadgeEl.className = "default-preset-chip";
          drsBadgeEl.textContent = `${de ? "Treiber-Live" : "Driver Live"}: Default`;
        }
      })
      .catch(() => {
        drsBadgeEl.style.display = "none";
      });
  }
};

window.updateTechGroupForGame = async function (gameId, typeLabel, forceReinstall = false) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;
  const targetDlls = (game.dlls || []).filter(
    (d) => d.type_label === typeLabel && (forceReinstall || d.update_available)
  );
  if (targetDlls.length === 0) return;

  try {
    showToast(
      isDe()
        ? `${forceReinstall ? "Installiere" : "Aktualisiere"} ${typeLabel} in ${game.name}${forceReinstall ? " neu..." : "..."}`
        : `${forceReinstall ? "Reinstalling" : "Updating"} ${typeLabel} in ${game.name}...`
    );
    let latestPayload = null;
    for (const d of targetDlls) {
      latestPayload = await invoke("update_single_game_dll", {
        gameId,
        dllPath: d.path,
      });
    }
    if (latestPayload) applyPayload(latestPayload);
    showToast(
      isDe()
        ? `✓ ${typeLabel} ${forceReinstall ? "neu installiert" : "aktualisiert"}!`
        : `✓ ${typeLabel} ${forceReinstall ? "reinstalled" : "updated"}!`
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

window.restoreTechGroupForGame = async function (gameId, techLabel) {
  try {
    showToast(isDe() ? `Stelle ${techLabel}-Backup wieder her...` : `Restoring ${techLabel} backups...`);
    const payload = await invoke("restore_tech_group_for_game", {
      gameId,
      techLabel,
    });
    applyPayload(payload);
    showToast(
      isDe()
        ? `✓ Originale ${techLabel}-Dateien wiederhergestellt!`
        : `✓ Restored original ${techLabel} files!`
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

window.closeGameDllModal = function () {
  appState.activeModalGameId = null;
  const backdrop = document.getElementById("dll-modal-backdrop");
  if (backdrop) backdrop.style.display = "none";
};

window.closeSecondaryModal = function () {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  if (backdrop) backdrop.style.display = "none";
};

/* ==========================================================================
   Edit Display Modal (Custom Name, Cover URL & Live Steam Store Search)
   ========================================================================== */
window.openEditDisplayModal = function (gameId) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;

  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const de = isDe();
  const isSteamGame =
    String(game.launcher || "").toLowerCase() === "steam" || game.launcher_name === "Steam";

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>✏️ ${de ? "Anzeige & Cover bearbeiten" : "Edit Display & Artwork"} — ${escapeHtml(game.name)}</h3>
        <div class="muted" style="font-size:0.76rem">${isSteamGame
      ? de
        ? "Suche im Steam-Store nach offiziellem Cover oder lege einen eigenen Anzeigenamen fest."
        : "Search Steam for official cover art or set a custom display title."
      : de
        ? "Lege einen eigenen Anzeigenamen oder eine eigene Cover-Bild-URL fest."
        : "Set a custom display title or custom cover image URL."
    }</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>
    <div class="modal-body" style="gap:14px">
      ${isSteamGame
      ? `<div class="preset-field">
               <label>${de ? "Im Steam-Store nach offiziellem Cover & App-ID suchen" : "Search Steam Store for Official Cover & App ID"}</label>
               <div style="display:flex;gap:8px">
                 <input type="text" id="edit-steam-query" class="toolbar-select" style="flex:1"
                        value="${escapeHtml(game.name)}" placeholder="${de ? "z.B. Overwatch 2, Diablo IV, Cyberpunk 2077" : "e.g. Overwatch 2, Diablo IV, Cyberpunk 2077"}" />
                 <button class="btn btn-primary" onclick="runSteamStoreSearch()">🔎 ${de ? "Steam durchsuchen" : "Search Steam"}</button>
               </div>
             </div>
             <div id="steam-search-results" class="steam-search-grid"></div>`
      : ""
    }

      <div class="preset-field">
        <label>${de ? "Anzeigename" : "Display Name"}</label>
        <input type="text" id="edit-custom-name" class="toolbar-select" value="${escapeHtml(game.name)}" />
      </div>
      ${isSteamGame
      ? `<div class="preset-select-grid">
               <div class="preset-field">
                 <label>${de ? "Steam App-ID (für Hero- & Cover-Artwork)" : "Steam App ID (for Hero & Portrait Artwork)"}</label>
                 <input type="text" id="edit-custom-appid" class="toolbar-select" value="${escapeHtml(game.app_id || "")}" placeholder="z.B. 2357570" />
               </div>
               <div class="preset-field">
                 <label>${de ? "Eigene Cover-Bild-URL (Optional)" : "Custom Cover Image URL (Optional)"}</label>
                 <input type="text" id="edit-custom-cover" class="toolbar-select" value="${escapeHtml(game.cover_url || "")}" placeholder="https://..." />
               </div>
             </div>`
      : `<input type="hidden" id="edit-custom-appid" value="${escapeHtml(game.app_id || "")}" />
             <div class="preset-field">
               <label>${de ? "Eigene Cover-Bild-URL (Optional)" : "Custom Cover Image URL (Optional)"}</label>
               <input type="text" id="edit-custom-cover" class="toolbar-select" value="${escapeHtml(game.cover_url || "")}" placeholder="https://..." />
             </div>`
    }
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="resetDisplayOverrideNow('${game.id}')">${de ? "Auf Standard zurücksetzen" : "Reset to Default"}</button>
      <button class="btn btn-accent" onclick="saveDisplayOverrideNow('${game.id}')">💾 ${de ? "Speichern" : "Save Changes"}</button>
    </div>
  `;

  backdrop.style.display = "flex";
  if (isSteamGame) {
    runSteamStoreSearch();
  }
};

window.runSteamStoreSearch = async function () {
  const input = document.getElementById("edit-steam-query");
  const container = document.getElementById("steam-search-results");
  if (!input || !container) return;
  const query = input.value.trim();
  if (!query) return;

  const de = isDe();
  container.innerHTML = `<div class="muted">${de ? "Durchsuche Steam-Store..." : "Searching Steam Store..."}</div>`;
  try {
    const results = await invoke("search_steam_store", { query });
    if (!results || results.length === 0) {
      container.innerHTML = `<div class="muted">${de
        ? `Keine Treffer im Steam-Store für „${escapeHtml(query)}“.`
        : `No Steam Store matches found for "${escapeHtml(query)}".`
        }</div>`;
      return;
    }
    container.innerHTML = results
      .map(
        (r) => `
        <div class="steam-result-card" onclick="pickSteamResult('${escapeJsStr(r.name)}', '${escapeJsStr(r.app_id)}', '${escapeJsStr(r.hero_url)}')">
          <img src="${escapeHtml(r.header_url)}" alt="${escapeHtml(r.name)}" loading="lazy" />
          <div class="sr-title" title="${escapeHtml(r.name)} (${escapeHtml(r.app_id)})">${escapeHtml(r.name)}</div>
        </div>`
      )
      .join("");
  } catch (e) {
    container.innerHTML = `<div class="muted">${de ? "Steam-Suchfehler:" : "Steam search error:"} ${escapeHtml(String(e))}</div>`;
  }
};

window.pickSteamResult = function (name, appId, heroUrl) {
  const nameEl = document.getElementById("edit-custom-name");
  const appIdEl = document.getElementById("edit-custom-appid");
  const coverEl = document.getElementById("edit-custom-cover");
  if (nameEl) nameEl.value = name;
  if (appIdEl) appIdEl.value = appId;
  if (coverEl) coverEl.value = heroUrl;
  showToast(isDe() ? `„${name}“ ausgewählt (App-ID ${appId})` : `Selected "${name}" (App ID ${appId})`);
};

window.saveDisplayOverrideNow = async function (gameId) {
  const customName = document.getElementById("edit-custom-name")?.value || null;
  const customAppId = document.getElementById("edit-custom-appid")?.value || null;
  const customCoverUrl = document.getElementById("edit-custom-cover")?.value || null;
  try {
    const payload = await invoke("save_game_display_override", {
      gameId,
      customName,
      customAppId,
      customCoverUrl,
    });
    closeSecondaryModal();
    applyPayload(payload);
    showToast(isDe() ? "✓ Anzeige- & Cover-Einstellungen gespeichert!" : "✓ Saved display & cover settings!");
  } catch (e) {
    showToast(String(e), true);
  }
};

window.resetDisplayOverrideNow = async function (gameId) {
  try {
    const payload = await invoke("reset_game_display_override", { gameId });
    closeSecondaryModal();
    applyPayload(payload);
    showToast(isDe() ? "Anzeige-Einstellungen zurückgesetzt." : "Reset display settings to default.");
  } catch (e) {
    showToast(String(e), true);
  }
};

/* ==========================================================================
   Per-Game NVIDIA DLSS Preset Settings Modal (`nvapi64.dll` DRS)
   ========================================================================== */
window.openGameDlssPresetsModal = function (gameId) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;

  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const de = isDe();

  const hasSrTech = (game.dlls || []).some((d) => d.type_label === "DLSS");
  const hasRrTech = gameHasRrTech(game);
  const hasFgTech = gameHasFgTech(game);

  const sr = hasSrTech ? game.dlss_sr_preset || "Default" : "Default";
  const rr = hasRrTech ? game.dlss_rr_preset || "Default" : "Default";
  const fg = hasFgTech ? game.dlss_fg_preset || "Default" : "Default";
  const exe = game.custom_exe_path || "";

  const srDllVer = (game.dlls || []).find((d) => d.type_label === "DLSS")?.version || "310.9.16.0";
  const rrDllVer = (game.dlls || []).find((d) => d.type_label === "DLSS-RR")?.version || "310.9.16.0";
  const fgDllVer = (game.dlls || []).find((d) => d.type_label === "DLSS-G")?.version || "310.9.16.0";

  const srOptions = getSupportedPresetsForDllVersion("sr", srDllVer);
  const rrOptions = getSupportedPresetsForDllVersion("rr", rrDllVer);
  const fgOptions = getSupportedPresetsForDllVersion("fg", fgDllVer);
  if (!srOptions.includes(sr)) srOptions.push(sr);
  if (!rrOptions.includes(rr)) rrOptions.push(rr);
  if (!fgOptions.includes(fg)) fgOptions.push(fg);
  const unusedLabel = de ? "Nicht im Spiel vorhanden" : "Not used by game";

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>🎛️ NVIDIA DLSS Preset-Einstellungen — ${escapeHtml(game.name)}</h3>
        <div class="muted" style="font-size:0.76rem">${de
      ? `Konfiguriere Treiber-Presets passend zur aktuell im Spiel installierten DLL-Version (<code>v${escapeHtml(srDllVer)}</code>) über NVIDIA DRS (<code>nvapi64.dll</code>).`
      : `Configure driver presets matching the game's currently installed DLL version (<code>v${escapeHtml(srDllVer)}</code>) via NVIDIA DRS (<code>nvapi64.dll</code>).`
    }</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>
    <div class="modal-body" style="gap:16px">
      <div class="preset-select-grid" style="grid-template-columns:1fr">
        <div class="preset-field" style="${!hasSrTech ? "opacity:0.5" : ""}">
          <label>DLSS Super Resolution (SR) Preset · <code>v${escapeHtml(srDllVer)}</code></label>
          ${hasSrTech
      ? `<select id="game-sr-preset" class="toolbar-select" onchange="document.getElementById('game-sr-desc').textContent = '💡 ' + getPresetDescription('sr', this.value, '${escapeJsStr(srDllVer)}')">
                   ${buildPresetOptionsHtml("sr", sr, "Preset K", srDllVer)}
                 </select>
                 <div id="game-sr-desc" class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("sr", sr, srDllVer))}</div>`
      : `<select id="game-sr-preset" class="toolbar-select" disabled><option value="Default">${unusedLabel}</option></select>`
    }
        </div>
        ${hasRrTech
      ? `<div class="preset-field">
                 <label>DLSS Ray Reconstruction (RR) Preset · <code>v${escapeHtml(rrDllVer)}</code></label>
                 <select id="game-rr-preset" class="toolbar-select" onchange="document.getElementById('game-rr-desc').textContent = '💡 ' + getPresetDescription('rr', this.value, '${escapeJsStr(rrDllVer)}')">
                   ${buildPresetOptionsHtml("rr", rr, "Preset F", rrDllVer)}
                 </select>
                 <div id="game-rr-desc" class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("rr", rr, rrDllVer))}</div>
               </div>`
      : `<input type="hidden" id="game-rr-preset" value="Default" />`
    }
        ${hasFgTech
      ? `<div class="preset-field">
                 <label>DLSS Frame Generation (FG) Preset · <code>v${escapeHtml(fgDllVer)}</code></label>
                 <select id="game-fg-preset" class="toolbar-select" onchange="document.getElementById('game-fg-desc').textContent = '💡 ' + getPresetDescription('fg', this.value, '${escapeJsStr(fgDllVer)}')">
                   ${buildPresetOptionsHtml("fg", fg, "Latest", fgDllVer)}
                 </select>
                 <div id="game-fg-desc" class="preset-slot-desc">💡 ${escapeHtml(getPresetDescription("fg", fg, fgDllVer))}</div>
               </div>
               <div class="preset-select-grid" style="grid-template-columns:repeat(3,1fr);gap:10px">
                 <div class="preset-field">
                   <label>${de ? "DLSS-G MFG Modus (RTX 50)" : "DLSS-G MFG Mode (RTX 50)"}</label>
                   <select id="game-fg-mode" class="toolbar-select">
                     <option value="Default" ${!game.dlss_fg_mode || game.dlss_fg_mode === "Default" ? "selected" : ""}>Default</option>
                     <option value="Fixed" ${game.dlss_fg_mode === "Fixed" ? "selected" : ""}>Fixed (2x–6x)</option>
                     <option value="Dynamic" ${game.dlss_fg_mode === "Dynamic" ? "selected" : ""}>Dynamic (Auto-FPS)</option>
                     <option value="Off" ${game.dlss_fg_mode === "Off" ? "selected" : ""}>Off</option>
                   </select>
                 </div>
                 <div class="preset-field">
                   <label>${de ? "MFG Multiplikator" : "MFG Multiplier"}</label>
                   <select id="game-fg-mult" class="toolbar-select">
                     ${[2, 3, 4, 5, 6].map((m) => `<option value="${m}" ${(game.dlss_fg_multiplier || 2) === m ? "selected" : ""}>${m}x</option>`).join("")}
                   </select>
                 </div>
                 <div class="preset-field">
                   <label>${de ? "Dynamic Ziel-FPS (0 = Auto)" : "Dynamic Target FPS (0 = Auto)"}</label>
                   <input type="number" id="game-fg-target-fps" class="toolbar-select" min="0" max="1000" value="${game.dlss_fg_target_fps || 0}" />
                 </div>
               </div>`
      : `<input type="hidden" id="game-fg-preset" value="Default" />`
    }
      </div>
      ${buildPresetGuideHtml(true, hasRrTech, hasFgTech)}
      <div class="preset-field">
        <label>${de
      ? "Eigener Spiel-Executable-Pfad (.exe) (Optional — für Spielstart & DRS-Profilbindung)"
      : "Custom Game Executable Path (Optional — used for Launch & DRS Profile)"
    }</label>
        <input type="text" id="game-custom-exe" class="toolbar-select" value="${escapeHtml(exe)}" placeholder="${de ? `Automatisch erkannt in ${escapeHtml(game.path)}` : `Auto-detected in ${escapeHtml(game.path)}`
    }" />
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="saveGameDlssPresetsNow('${game.id}', true)">${de ? "Auf Standard zurücksetzen" : "Reset to Default"
    }</button>
      <button class="btn btn-accent" onclick="saveGameDlssPresetsNow('${game.id}', false)">⚡ ${de ? "Presets anwenden" : "Apply Presets"
    }</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.saveGameDlssPresetsNow = async function (gameId, reset = false) {
  const srPreset = reset ? "Default" : document.getElementById("game-sr-preset")?.value || "Default";
  const rrPreset = reset ? "Default" : document.getElementById("game-rr-preset")?.value || "Default";
  const fgPreset = reset ? "Default" : document.getElementById("game-fg-preset")?.value || "Default";
  const fgMode = reset ? "Default" : document.getElementById("game-fg-mode")?.value || null;
  const fgMultRaw = document.getElementById("game-fg-mult")?.value;
  const fgMultiplier = reset || !fgMultRaw ? null : parseInt(fgMultRaw, 10) || 2;
  const fgFpsRaw = document.getElementById("game-fg-target-fps")?.value;
  const fgTargetFps = reset || !fgFpsRaw ? null : parseInt(fgFpsRaw, 10) || null;
  const customExePath = reset ? null : document.getElementById("game-custom-exe")?.value || null;

  try {
    const payload = await invoke("save_game_dlss_presets", {
      gameId,
      srPreset,
      rrPreset,
      fgPreset,
      fgMode,
      fgMultiplier,
      fgTargetFps,
      customExePath,
    });
    closeSecondaryModal();
    applyPayload(payload);
    showToast(
      reset
        ? isDe()
          ? "DLSS-Presets auf Standard zurückgesetzt."
          : "Reset game DLSS presets to Default."
        : isDe()
          ? "✓ NVIDIA DLSS-Presets angewendet!"
          : "✓ Applied NVIDIA DLSS presets!"
    );
  } catch (e) {
    showToast(String(e), true);
  }
};

/* ==========================================================================
   Bulk Update Scope Checklist Modal (`Update All (N)`)
   ========================================================================== */
window.openBulkUpdateScopeModal = function () {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const enabledList = appState.config?.enabled_technologies || ALL_TECH_GROUPS.map((t) => t.label);
  const de = isDe();

  // Filter to technologies supported by the user's GPU or present in games
  const presentLabels = new Set();
  for (const g of appState.games) {
    for (const d of g.dlls || []) {
      if (d.gpu_supported !== false) presentLabels.add(d.type_label);
    }
  }
  const relevantTechs = ALL_TECH_GROUPS.filter(
    (t) => presentLabels.size === 0 || presentLabels.has(t.label)
  );

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>🚀 ${de ? "Massen-Update für alle Spiele" : "Batch Update All Games"}</h3>
        <div class="muted" style="font-size:0.76rem">${de
      ? "Wähle aus, welche Upscaler-Technologien in allen nicht-ignorierten Spielen aktualisiert werden sollen."
      : "Select which upscaler technologies to update across all non-ignored games."
    }</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>
    <div class="modal-body" style="gap:12px">
      <div class="tech-scope-grid">
        ${relevantTechs
      .map(
        (t) => `
          <label class="tech-scope-item">
            <input type="checkbox" class="bulk-scope-cb" value="${escapeHtml(t.label)}"
                   ${enabledList.includes(t.label) ? "checked" : ""} />
            <span>${escapeHtml(t.label)}</span>
          </label>`
      )
      .join("")}
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="closeSecondaryModal()">${de ? "Abbrechen" : "Cancel"}</button>
      <button class="btn btn-accent" onclick="executeScopedBulkUpdate()">🚀 ${de ? "Massen-Update starten" : "Start Batch Update"
    }</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.executeScopedBulkUpdate = async function () {
  const checked = Array.from(document.querySelectorAll(".bulk-scope-cb:checked")).map(
    (cb) => cb.value
  );
  if (checked.length === 0) {
    showToast(
      isDe()
        ? "Bitte wähle mindestens eine Technologie für das Update aus."
        : "Please select at least one technology to update.",
      true
    );
    return;
  }
  closeSecondaryModal();
  try {
    showToast(
      isDe()
        ? "Aktualisiere alle Spiele mit den ausgewählten Technologien..."
        : "Updating all games with selected technologies..."
    );
    const payload = await invoke("bulk_update_selected_games", {
      gameIds: [],
      allowedTechs: checked,
    });
    applyPayload(payload);
    showToast(isDe() ? "✓ Massen-Update abgeschlossen!" : "✓ Batch update complete!");
  } catch (e) {
    showToast(String(e), true);
  }
};

/* ==========================================================================
   Application Logs Modal & Launcher Paths Modal
   ========================================================================== */
window.openLogsModal = function () {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const logs = appState.logs || [];
  const de = isDe();

  const rows =
    logs.length === 0
      ? `<div class="muted">${de
        ? "Noch keine Protokoll-Einträge in dieser Sitzung vorhanden."
        : "No log events recorded in this session."
      }</div>`
      : logs
        .slice()
        .reverse()
        .map(
          (l) => `
          <div class="log-entry-row">
            <span class="muted" style="flex-shrink:0">${escapeHtml(l.timestamp)}</span>
            <span class="log-badge ${escapeHtml(l.level)}">${escapeHtml(l.level)}</span>
            <span style="word-break:break-word">${escapeHtml(l.message)}</span>
          </div>`
        )
        .join("");

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>📋 ${de ? "Aktivitäts-Protokoll (Logs)" : "Application Activity Logs"}</h3>
        <div class="muted" style="font-size:0.76rem">${de ? `${logs.length} Einträge in der aktuellen Sitzung` : `${logs.length} events recorded in current session`
    }</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>
    <div class="modal-body" style="max-height:55vh">
      ${rows}
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="copyLogsNow()">📋 ${de ? "Alle kopieren" : "Copy All"}</button>
      <button class="btn btn-secondary" onclick="clearLogsNow()">🗑️ ${de ? "Logs leeren" : "Clear Logs"}</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.clearLogsNow = async function () {
  try {
    appState.logs = await invoke("clear_app_logs");
    openLogsModal();
  } catch (e) {
    showToast(String(e), true);
  }
};

window.copyLogsNow = function () {
  const text = (appState.logs || [])
    .map((l) => `[${l.timestamp}] [${l.level}] ${l.message}`)
    .join("\n");
  copyTextToClipboard(
    text || "No logs",
    isDe() ? "Protokoll in Zwischenablage kopiert" : "Copied application logs to clipboard"
  );
};

window.openLauncherDetailsModal = function (launcherName) {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const games = appState.games.filter((g) => g.launcher_name === launcherName);
  const de = isDe();

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>📂 ${escapeHtml(launcherName)} — ${de ? `Erkannte Spielpfade (${games.length})` : `Detected Game Paths (${games.length})`
    }</h3>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ ${de ? "Schließen" : "Close"}</button>
    </div>
    <div class="modal-body">
      ${games
      .map(
        (g) => `
        <div class="dll-row">
          <div class="dll-row-info">
            <div style="font-weight:600;font-size:0.88rem">${escapeHtml(g.name)}</div>
            <div class="dll-preset-sub" title="${escapeHtml(g.path)}"><code>${escapeHtml(g.path)}</code></div>
          </div>
          <div class="dll-row-action" style="gap:6px">
            <button class="btn btn-secondary btn-card-action" onclick="openFolderInExplorer('${escapeJsStr(g.path)}')">📂 ${de ? "Öffnen" : "Open"
          }</button>
            <button class="btn btn-secondary btn-card-action" onclick="copyTextToClipboard('${escapeJsStr(g.path)}', '${de ? "Pfad kopiert" : "Copied path"
          }')">📋 ${de ? "Kopieren" : "Copy"}</button>
          </div>
        </div>`
      )
      .join("")}
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick="closeSecondaryModal(); selectLauncherFromHub('${escapeJsStr(launcherName)}')">${de ? "In Spiele-Bibliothek anzeigen" : "View in Games Library"
    }</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

/* ==========================================================================
   Backups & Settings Helpers
   ========================================================================== */
function renderBackups() {
  const list = document.getElementById("backups-list");
  const de = isDe();
  if (appState.backups.length === 0) {
    list.innerHTML = `<div class="muted">${de
      ? "Noch keine Backups vorhanden. Backups werden automatisch vor jedem DLL-Update angelegt."
      : "No backups recorded yet. Backups are created automatically before any DLL update."
      }</div>`;
    return;
  }

  const byGame = new Map();
  for (const b of appState.backups) {
    if (!byGame.has(b.game_id)) {
      byGame.set(b.game_id, {
        gameId: b.game_id,
        gameName: b.game_name,
        gamePath: b.game_path,
        newestDate: b.created_at,
        items: [],
      });
    }
    byGame.get(b.game_id).items.push(b);
  }

  list.innerHTML = Array.from(byGame.values())
    .map(
      (grp) => `
      <div class="backup-card">
        <div class="backup-card-info">
          <div style="font-weight:600;font-size:0.96rem">${escapeHtml(grp.gameName)}</div>
          <div class="muted" style="white-space:nowrap;overflow:hidden;text-overflow:ellipsis" title="${grp.items.map((i) => escapeHtml(i.filename)).join(", ")}">
            ${grp.items.length} ${de ? "gesicherte" : "backed-up"} ${grp.items.length === 1 ? "DLL" : "DLLs"} (${grp.items.slice(0, 3).map((i) => escapeHtml(i.filename)).join(", ")}${grp.items.length > 3 ? ", …" : ""}) · ${new Date(grp.newestDate).toLocaleString()}
          </div>
          <div class="muted" style="font-size:0.74rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${escapeHtml(grp.gamePath)}</div>
        </div>
        <div class="backup-card-actions">
          <button class="btn btn-secondary" onclick="openFolderInExplorer('${escapeJsStr(grp.gamePath)}')">📂 ${de ? "Ordner" : "Folder"
        }</button>
          <button class="btn btn-primary" onclick="restoreAllGameBackups('${grp.gameId}')">⏪ ${de ? "Original wiederherstellen" : "Restore Original"
        }</button>
        </div>
      </div>`
    )
    .join("");
}

function renderCustomFolders() {
  const container = document.getElementById("custom-folders-list");
  if (!container) return;
  const folders = appState.config?.custom_folders || [];
  const de = isDe();
  container.innerHTML = folders
    .map(
      (f, idx) => `
      <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
        <code>${escapeHtml(f)}</code>
        <div style="display:flex;gap:6px">
          <button class="btn btn-secondary" style="padding:4px 8px" onclick="openFolderInExplorer('${escapeJsStr(f)}')">📂</button>
          <button class="btn btn-secondary" style="padding:4px 8px" onclick="removeCustomFolder(${idx})">${de ? "Entfernen" : "Remove"
        }</button>
        </div>
      </div>`
    )
    .join("");
}

function renderSettingsTechScope() {
  const grid = document.getElementById("settings-tech-scope-grid");
  if (!grid || !appState.config) return;
  const enabled = appState.config.enabled_technologies || ALL_TECH_GROUPS.map((t) => t.label);

  grid.innerHTML = ALL_TECH_GROUPS.map(
    (t) => `
    <label class="tech-scope-item" title="${escapeHtml(t.title)}">
      <input type="checkbox" ${enabled.includes(t.label) ? "checked" : ""}
             onchange="toggleSettingTechScope('${escapeJsStr(t.label)}', this.checked)" />
      <span>${escapeHtml(t.label)}</span>
    </label>`
  ).join("");
}

window.toggleSettingTechScope = async function (label, checked) {
  if (!appState.config) return;
  const set = new Set(appState.config.enabled_technologies || ALL_TECH_GROUPS.map((t) => t.label));
  if (checked) set.add(label);
  else set.delete(label);
  appState.config.enabled_technologies = Array.from(set);
  await saveConfig();
  const initial = await invoke("get_initial_state");
  applyPayload(initial);
};

function renderIgnoredAndBlacklistedSettings() {
  const ignoredEl = document.getElementById("ignored-games-settings-list");
  const blackEl = document.getElementById("blacklisted-games-settings-list");
  if (!ignoredEl || !blackEl || !appState.config) return;

  const de = isDe();
  const ignoredGames = appState.games.filter((g) => g.ignored);
  ignoredEl.innerHTML =
    ignoredGames.length === 0
      ? `<div class="muted" style="font-size:0.78rem">${de
        ? "Aktuell sind keine Spiele ignoriert. Über das <code>⋮</code>-Menü einer Spielkarte kannst du Spiele ignorieren."
        : "No games are currently ignored. Use a game card's <code>⋮</code> menu to ignore or unignore a game."
      }</div>`
      : ignoredGames
        .map(
          (g) => `
          <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
            <span><strong>${escapeHtml(g.name)}</strong> <span class="muted">(${escapeHtml(g.launcher_name)})</span></span>
            <button class="btn btn-secondary" style="padding:4px 8px" onclick="toggleIgnoreGameNow('${g.id}')">${de ? "Freigeben" : "Unignore"
            }</button>
          </div>`
        )
        .join("");

  const blacklisted = appState.config.blacklisted_games || [];
  blackEl.innerHTML = blacklisted
    .map(
      (b, idx) => `
      <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
        <code>${escapeHtml(b)}</code>
        <button class="btn btn-secondary" style="padding:4px 8px" onclick="removeBlacklistEntry(${idx})">${de ? "Entfernen" : "Remove"
        }</button>
      </div>`
    )
    .join("");
}

window.removeBlacklistEntry = async function (idx) {
  if (!appState.config) return;
  appState.config.blacklisted_games.splice(idx, 1);
  await saveConfig();
  const initial = await invoke("get_initial_state");
  applyPayload(initial);
};

window.updateAllGameDlls = async function (gameId, forceReinstall = false) {
  const game = appState.games.find((g) => g.id === gameId);
  const gameTitle = game ? game.name : "Game";
  const btn = document.getElementById(`btn-update-card-${gameId}`);
  if (btn && !forceReinstall) {
    btn.disabled = true;
    btn.textContent = isDe() ? "⏳ Update..." : "⏳ Updating...";
  }
  try {
    showToast(
      isDe()
        ? `${forceReinstall ? "Installiere DLLs neu für" : "Aktualisiere"} ${gameTitle}...`
        : `${forceReinstall ? "Reinstalling DLLs for" : "Updating"} ${gameTitle}...`
    );
    const payload = await invoke("update_all_for_game", {
      gameId,
      forceReinstall,
    });
    applyPayload(payload);
    showToast(
      isDe()
        ? `✓ ${gameTitle}: ${forceReinstall ? "DLLs neu installiert!" : "Auf neueste DLLs aktualisiert!"}`
        : `✓ ${gameTitle} ${forceReinstall ? "DLLs reinstalled!" : "updated to latest DLLs!"}`
    );
  } catch (e) {
    if (btn && !forceReinstall) {
      btn.disabled = false;
      btn.textContent = "⚡ Update";
    }
    showToast(String(e), true);
  }
};

window.restoreAllGameBackups = async function (gameId) {
  try {
    showToast(isDe() ? "Stelle originale Spiel-DLLs wieder her..." : "Restoring original DLLs for game...");
    const payload = await invoke("restore_all_for_game", { gameId });
    applyPayload(payload);
    showToast(isDe() ? "✓ Originale Spiel-DLLs wiederhergestellt!" : "✓ Original game DLLs restored!");
  } catch (e) {
    showToast(String(e), true);
  }
};

window.removeCustomFolder = async function (idx) {
  if (!appState.config) return;
  appState.config.custom_folders.splice(idx, 1);
  await saveConfig();
};

async function saveConfig() {
  try {
    const updated = await invoke("update_app_config", { newConfig: appState.config });
    appState.config = updated;
    renderCustomFolders();
    renderIgnoredAndBlacklistedSettings();
    showToast(isDe() ? "Einstellungen gespeichert." : "Settings saved.");
  } catch (e) {
    showToast(String(e), true);
  }
}

window.switchView = function (viewName) {
  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.view === viewName);
  });
  document.querySelectorAll(".view-section").forEach((sec) => {
    sec.classList.toggle("active", sec.id === `view-${viewName}`);
  });
};

async function triggerGameScan(switchAfter = true) {
  const scanBtn = document.getElementById("btn-scan-now");
  try {
    scanBtn.disabled = true;
    scanBtn.innerHTML = `<span data-i18n="btn_scanning">${t("btn_scanning")}</span>`;
    showToast(
      isDe()
        ? "Scanne alle Laufwerke nach Spielen mit kompatiblen Upscaler-DLLs..."
        : "Scanning drives for games with GPU-compatible upscaler DLLs..."
    );
    const payload = await invoke("scan_games");
    applyPayload(payload);
    showToast(
      isDe()
        ? `Scan abgeschlossen: ${payload.games.length} unterstützte Spiele gefunden.`
        : `Scan complete: ${payload.games.length} supported games found.`
    );
    if (switchAfter) {
      switchView("games");
    }
  } catch (e) {
    showToast(String(e), true);
  } finally {
    scanBtn.disabled = false;
    scanBtn.innerHTML = `<span data-i18n="btn_scan_games">${t("btn_scan_games")}</span>`;
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  document.addEventListener("click", () => {
    closeCardContextMenu();
  });

  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (btn.dataset.view === "games") {
        appState.selectedLauncher = "all";
        appState.statusFilter = "all";
        renderLauncherSelect();
        renderStatusChips();
        renderGamesGrid();
      }
      switchView(btn.dataset.view);
    });
  });

  document.getElementById("global-search").addEventListener("input", (e) => {
    appState.searchQuery = e.target.value;
    switchView("games");
    renderGamesGrid();
  });

  document.getElementById("select-launcher-filter").addEventListener("change", (e) => {
    appState.selectedLauncher = e.target.value;
    renderStatusChips();
    renderGamesGrid();
  });

  document.getElementById("select-sort-mode").addEventListener("change", (e) => {
    appState.sortMode = e.target.value;
    renderGamesGrid();
  });

  document.getElementById("btn-toggle-select-mode").addEventListener("click", () => {
    appState.selectMode = !appState.selectMode;
    if (!appState.selectMode) {
      appState.selectedGameIds.clear();
    }
    renderBatchSelectionBar();
    renderGamesGrid();
  });

  document.querySelectorAll(".density-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".density-btn").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      appState.gridDensity = btn.dataset.density;
      renderGamesGrid();
    });
  });

  document.getElementById("dll-modal-backdrop").addEventListener("click", (e) => {
    if (e.target.id === "dll-modal-backdrop") {
      closeGameDllModal();
    }
  });

  document.getElementById("secondary-modal-backdrop").addEventListener("click", (e) => {
    if (e.target.id === "secondary-modal-backdrop") {
      closeSecondaryModal();
    }
  });

  document.getElementById("btn-scan-now").addEventListener("click", () => {
    triggerGameScan(true);
  });

  document.getElementById("btn-bulk-update").addEventListener("click", () => {
    openBulkUpdateScopeModal();
  });

  document.getElementById("btn-open-logs").addEventListener("click", () => {
    openLogsModal();
  });

  const btnLangToggle = document.getElementById("btn-lang-toggle");
  if (btnLangToggle) {
    btnLangToggle.addEventListener("click", async () => {
      if (!appState.config) return;
      const next = getLang() === "de" ? "en" : "de";
      appState.config.language = next;
      renderAll();
      await saveConfig();
    });
  }

  const cfgLang = document.getElementById("cfg-language");
  if (cfgLang) {
    cfgLang.addEventListener("change", async (e) => {
      if (!appState.config) return;
      appState.config.language = e.target.value === "en" ? "en" : "de";
      renderAll();
      await saveConfig();
    });
  }

  document.getElementById("btn-theme-toggle").addEventListener("click", async () => {
    if (!appState.config) return;
    const next = appState.config.theme_mode === "light" ? "dark" : "light";
    appState.config.theme_mode = next;
    applyThemeMode(next);
    await saveConfig();
  });

  document.getElementById("btn-refresh-manifest").addEventListener("click", async () => {
    try {
      showToast(isDe() ? "Prüfe auf neueste DLL-Versionen..." : "Checking latest DLL versions...");
      appState.catalog = await invoke("refresh_dll_manifest");
      renderHubTechStrip();
      showToast(isDe() ? "Upscaler-Versionen sind aktuell!" : "Upscaler versions up to date!");
    } catch (e) {
      showToast(String(e), true);
    }
  });

  document.getElementById("toggle-dlss-overlay").addEventListener("change", async (e) => {
    try {
      const enabled = await invoke("toggle_dlss_indicator", { enabled: e.target.checked });
      e.target.checked = enabled;
      showToast(
        isDe()
          ? `DLSS-Overlay-Anzeige ${enabled ? "aktiviert" : "deaktiviert"}.`
          : `DLSS Overlay Indicator ${enabled ? "enabled" : "disabled"}.`
      );
    } catch (err) {
      e.target.checked = !e.target.checked;
      showToast(String(err), true);
    }
  });

  document.getElementById("btn-add-folder").addEventListener("click", async () => {
    const input = document.getElementById("input-custom-folder");
    const val = input.value.trim();
    if (!val || !appState.config) return;
    if (!appState.config.custom_folders.includes(val)) {
      appState.config.custom_folders.push(val);
      input.value = "";
      await saveConfig();
    }
  });

  document.getElementById("btn-browse-folder").addEventListener("click", async () => {
    const selected = await openFolderDialog();
    if (selected && appState.config) {
      if (!appState.config.custom_folders.includes(selected)) {
        appState.config.custom_folders.push(selected);
        await saveConfig();
      }
    }
  });

  document.getElementById("btn-add-blacklist").addEventListener("click", async () => {
    const input = document.getElementById("input-blacklist-game");
    const val = input.value.trim();
    if (!val || !appState.config) return;
    if (!appState.config.blacklisted_games.includes(val)) {
      appState.config.blacklisted_games.push(val);
      input.value = "";
      await saveConfig();
      const initial = await invoke("get_initial_state");
      applyPayload(initial);
    }
  });

  ["cfg-auto-backup", "cfg-skip-anticheat", "cfg-allow-preview", "cfg-hide-ignored", "cfg-auto-report-db"].forEach((id) => {
    const el = document.getElementById(id);
    if (!el) return;
    el.addEventListener("change", async () => {
      if (!appState.config) return;
      appState.config.auto_backup = document.getElementById("cfg-auto-backup").checked;
      appState.config.skip_anti_cheat_games = document.getElementById("cfg-skip-anticheat").checked;
      appState.config.allow_preview_dlls = document.getElementById("cfg-allow-preview").checked;
      appState.config.hide_ignored_games = document.getElementById("cfg-hide-ignored").checked;
      const autoRep = document.getElementById("cfg-auto-report-db");
      if (autoRep) appState.config.auto_report_games_to_db = autoRep.checked;
      await saveConfig();
      const initial = await invoke("get_initial_state");
      applyPayload(initial);
    });
  });

  ["cfg-global-sr-preset", "cfg-global-rr-preset", "cfg-global-fg-preset"].forEach((id) => {
    const el = document.getElementById(id);
    if (!el) return;
    el.addEventListener("change", async () => {
      if (!appState.config) return;
      appState.config.global_dlss_preset = document.getElementById("cfg-global-sr-preset").value;
      appState.config.global_dlss_rr_preset = document.getElementById("cfg-global-rr-preset").value;
      appState.config.global_dlss_fg_preset = document.getElementById("cfg-global-fg-preset").value;
      await saveConfig();
    });
  });

  async function refreshNvidiaAppWhitelistStatus() {
    const statusEl = document.getElementById("nvapp-whitelist-status");
    const btnEl = document.getElementById("btn-nvapp-whitelist");
    const hubCard = document.getElementById("hub-nvapp-whitelist-card");
    const hubStatusEl = document.getElementById("hub-nvapp-whitelist-status");
    const hubBtnEl = document.getElementById("btn-hub-nvapp-whitelist");
    const adminBanner = document.getElementById("admin-warning-banner");
    try {
      const st = await invoke("get_nvidia_app_whitelist_status");
      if (adminBanner) {
        adminBanner.style.display = st.is_admin ? "none" : "flex";
      }
      const otaSuffix = st.ngx_ota_dlss_version
        ? ` · NGX OTA: v${st.ngx_ota_dlss_version}`
        : "";
      let msg = "";
      if (!st.applicable) {
        msg = (isDe()
          ? "NVIDIA App NvBackend nicht gefunden (nicht erforderlich)."
          : "NVIDIA App NvBackend not found (not required).") + otaSuffix;
        if (btnEl) btnEl.disabled = true;
        if (hubCard) hubCard.style.display = "none";
      } else if (st.blocked_flags === 0 && (st.ops_unsupported_count || 0) === 0) {
        msg = (isDe()
          ? "✅ Alle Spiele sind in der NVIDIA App für DLSS-Overrides & UI freigeschaltet."
          : "✅ All games are unlocked for DLSS overrides & UI in NVIDIA App.") + otaSuffix;
        if (btnEl) {
          btnEl.disabled = true;
          btnEl.textContent = isDe() ? "✅ Bereits freigeschaltet" : "✅ Already Unlocked";
        }
        if (hubCard) hubCard.style.display = "flex";
        if (hubBtnEl) hubBtnEl.style.display = "none";
      } else {
        const preview = (st.blocked_apps || []).slice(0, 4).join(", ");
        const opsCount = st.ops_unsupported_count || 0;
        msg = (isDe()
          ? `⚠️ ${st.blocked_flags} Sperr-Flags & ${opsCount} Ops-Sperren in NVIDIA App aktiv (z. B. ${preview || "Star Citizen"}).`
          : `⚠️ ${st.blocked_flags} override block flags & ${opsCount} Ops locks active in NVIDIA App (e.g. ${preview || "Star Citizen"}).`) + otaSuffix;
        if (btnEl) {
          btnEl.disabled = false;
          btnEl.textContent = isDe() ? "🔓 Alle Spiele freischalten" : "🔓 Unlock All Games";
        }
        if (hubCard) hubCard.style.display = "flex";
        if (hubBtnEl) hubBtnEl.style.display = "inline-flex";
      }
      if (statusEl) statusEl.textContent = msg;
      if (hubStatusEl) hubStatusEl.textContent = msg;
    } catch (err) {
      if (statusEl) statusEl.textContent = String(err);
      if (hubStatusEl) hubStatusEl.textContent = String(err);
    }
  }

  async function runUnlockNvidiaAppWhitelist(triggerBtn) {
    try {
      if (triggerBtn) triggerBtn.disabled = true;
      await invoke("apply_nvidia_app_whitelist", { unlockOps: true });
      await refreshNvidiaAppWhitelistStatus();
      showToast(
        isDe()
          ? "NVIDIA App Whitelist & IsOpsSupported angewendet & blockierende DRS-Profilsperren bereinigt."
          : "NVIDIA App whitelist & IsOpsSupported applied & blocking DRS profile overrides cleared."
      );
    } catch (err) {
      showToast(String(err), true);
    } finally {
      if (triggerBtn) triggerBtn.disabled = false;
    }
  }

  const btnRelaunchAdmin = document.getElementById("btn-relaunch-admin");
  if (btnRelaunchAdmin) {
    btnRelaunchAdmin.addEventListener("click", async () => {
      try {
        await invoke("relaunch_as_admin_cmd");
      } catch (err) {
        showToast(String(err), true);
      }
    });
  }

  async function runRestartNvidiaServices(triggerBtn) {
    try {
      if (triggerBtn) triggerBtn.disabled = true;
      const msg = await invoke("restart_nvidia_services_cmd");
      showToast(msg);
    } catch (err) {
      showToast(String(err), true);
    } finally {
      if (triggerBtn) triggerBtn.disabled = false;
    }
  }

  const btnNvWhitelist = document.getElementById("btn-nvapp-whitelist");
  if (btnNvWhitelist) {
    btnNvWhitelist.addEventListener("click", () => runUnlockNvidiaAppWhitelist(btnNvWhitelist));
  }
  const btnHubNvWhitelist = document.getElementById("btn-hub-nvapp-whitelist");
  if (btnHubNvWhitelist) {
    btnHubNvWhitelist.addEventListener("click", () => runUnlockNvidiaAppWhitelist(btnHubNvWhitelist));
  }

  const btnRestartNv = document.getElementById("btn-restart-nv-services");
  if (btnRestartNv) {
    btnRestartNv.addEventListener("click", () => runRestartNvidiaServices(btnRestartNv));
  }
  const btnHubRestartNv = document.getElementById("btn-hub-restart-nv");
  if (btnHubRestartNv) {
    btnHubRestartNv.addEventListener("click", () => runRestartNvidiaServices(btnHubRestartNv));
  }

  refreshNvidiaAppWhitelistStatus();

  document.querySelectorAll("[data-url]").forEach((btn) => {
    btn.addEventListener("click", () => {
      openExternalUrl(btn.dataset.url);
    });
  });

  const startupOverlay = document.getElementById("startup-overlay");
  const startupText = document.getElementById("startup-status-text");
  const startupStep = document.getElementById("startup-status-step");
  const startupProgress = document.getElementById("startup-progress-fill");
  const setStartupStatus = (msgDe, msgEn, stepText, pct) => {
    if (startupText) startupText.textContent = isDe() ? msgDe : msgEn;
    if (startupStep && stepText !== undefined) startupStep.textContent = stepText;
    if (startupProgress && pct !== undefined) startupProgress.style.width = `${pct}%`;
  };

  try {
    setStartupStatus(
      "⏳ Schritt 1/3: Lade Spielebibliothek, GPU & wende NVIDIA DRS Presets an…",
      "⏳ Step 1/3: Loading games library, GPU & enforcing NVIDIA DRS presets…",
      "1 / 3",
      33
    );
    const initial = await invoke("get_initial_state");
    applyPayload(initial);
    if (!initial.games || initial.games.length === 0) {
      setStartupStatus(
        "🔍 Scanne Launcher & Spieleordner nach Upscaler-DLLs…",
        "🔍 Scanning launchers & game folders for upscaler DLLs…",
        "1 / 3",
        45
      );
      await triggerGameScan(false);
    }

    setStartupStatus(
      "🌐 Schritt 2/3: Prüfe offizielle Upstreams (NVIDIA DLSS, Streamline, DirectStorage, Intel XeSS) & Spiele-DB…",
      "🌐 Step 2/3: Checking official upstreams (NVIDIA DLSS, Streamline, DirectStorage, Intel XeSS) & Game DB…",
      "2 / 3",
      70
    );

    const manifestTask = invoke("refresh_dll_manifest")
      .then((catalog) => {
        if (catalog) {
          appState.catalog = catalog;
          renderHubTechStrip();
        }
      })
      .catch(() => { });

    const dbTask = invoke("sync_game_database", { customUrl: null })
      .then((payload) => {
        if (payload) applyPayload(payload);
      })
      .catch(() => { });

    const updateTask = invoke("check_app_update")
      .then((info) => {
        if (info && info.update_available) {
          appState.appUpdateInfo = info;
          const pill = document.getElementById("app-update-pill");
          if (pill) {
            pill.style.display = "inline-flex";
            pill.innerHTML = `<span>⬆️ v${escapeHtml(info.latest_version)} ${isDe() ? "verfügbar" : "Available"
              }</span>`;
            pill.onclick = () => openExternalUrl(info.release_url);
          }
        }
      })
      .catch(() => { });

    Promise.allSettled([manifestTask, dbTask, updateTask]).then(() => {
      setStartupStatus(
        "✅ Start-Checks abgeschlossen — Alle Manifeste, DRS-Presets & Spiele-Datenbank sind aktuell.",
        "✅ Startup checks complete — Manifests, DRS presets & game database are up to date.",
        "3 / 3",
        100
      );
      if (startupOverlay) {
        setTimeout(() => {
          startupOverlay.style.display = "none";
        }, 1200);
      }
    });
  } catch (e) {
    if (startupOverlay) startupOverlay.style.display = "none";
    showToast("Failed to load initial state: " + e, true);
  }
});
