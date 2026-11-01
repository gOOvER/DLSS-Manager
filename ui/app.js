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
  { label: "FSR 3/4", title: "AMD FSR 3 / 4 (DX12)" },
  { label: "FSR VK", title: "AMD FSR 3 / 4 (Vulkan)" },
  { label: "XeSS", title: "Intel XeSS" },
  { label: "XeSS-FG", title: "Intel XeSS Frame Gen" },
  { label: "XeLL", title: "Intel XeLL" },
];

let appState = {
  version: "1.0.0-beta.1",
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
    document.getElementById("gpu-vendor-badge").textContent = `DETECTED ${vendor} GPU`;
    document.getElementById("gpu-name").textContent = appState.gpu.name;
    const drv = appState.gpu.driver_version ? ` · Driver ${appState.gpu.driver_version}` : "";
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

  const k1 = document.getElementById("input-steam-api-key");
  const s1 = document.getElementById("input-steam-id");
  const k2 = document.getElementById("settings-steam-api-key");
  const s2 = document.getElementById("settings-steam-id");
  if (k1 && document.activeElement !== k1) k1.value = key;
  if (s1 && document.activeElement !== s1) s1.value = sid;
  if (k2 && document.activeElement !== k2) k2.value = key;
  if (s2 && document.activeElement !== s2) s2.value = sid;

  const pill1 = document.getElementById("steam-api-status-pill");
  const pill2 = document.getElementById("settings-steam-status-pill");
  const label = hasKey
    ? `● Connected${sid ? ` · ID ${sid}` : ""}`
    : "○ Not configured";
  if (pill1) {
    pill1.textContent = label;
    pill1.classList.toggle("connected", hasKey);
  }
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
      ? `● API Key Active · ${cnt} Profiles Loaded`
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
    showToast("Syncing Game Compatibility & Settings Database...");
    const payload = await invoke("sync_game_database", { customUrl, apiKey });
    applyPayload(payload);
    showToast(`✓ Game Database synced (${payload.stats?.game_db_count || 0} profiles active)!`);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.reportGamesToDbNow = async function (gameId = null) {
  try {
    showToast(
      gameId
        ? "Reporting game to dlssdb.goover.dev..."
        : "Sending all scanned games & detected technologies to dlssdb.goover.dev..."
    );
    const msg = await invoke("report_games_to_db", { gameId });
    showToast(`✓ ${msg}`);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.applyRecommendedProfileNow = async function (gameId) {
  try {
    const payload = await invoke("apply_recommended_game_profile", { gameId });
    applyPayload(payload);
    showToast("✨ Applied recommended DLSS presets from Game Database!");
  } catch (e) {
    showToast(String(e), true);
  }
};

window.toggleRevealSteamApiKey = function () {
  const el = document.getElementById("input-steam-api-key");
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
  if (fb) fb.textContent = "Validating Steam Web API key...";
  try {
    const res = await invoke("validate_and_save_steam_api", {
      apiKey,
      steamId,
    });
    const initial = await invoke("get_initial_state");
    applyPayload(initial);
    if (fb) fb.textContent = res.message || "";
    showToast(res.message || "Steam API updated.", !res.valid && apiKey.trim().length > 0);
  } catch (e) {
    if (fb) fb.textContent = String(e);
    showToast(String(e), true);
  }
}

window.reresolveGamesViaSteamNow = async function () {
  const fb = document.getElementById("steam-api-feedback");
  if (fb) fb.textContent = "Re-resolving game App IDs & artwork via Steam API...";
  try {
    showToast("Re-resolving games via Steam Web API...");
    const payload = await invoke("reresolve_games_via_steam_api");
    applyPayload(payload);
    if (fb) fb.textContent = "Re-resolution complete! Artwork and App IDs updated.";
    showToast("✓ Steam API re-resolution complete!");
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
    const items = [
      { label: "Detected GPU", val: gpu.name || "Unknown" },
      { label: "GPU Architecture", val: gpu.architecture || "—" },
      { label: "Graphics Driver", val: gpu.driver_version || "—" },
      { label: "Recommended Preset", val: gpu.recommended_preset || "—" },
      { label: "Cached Repository DLLs", val: `${stats.cached_dlls || 0} files (${(stats.cache_size_mb || 0).toFixed(1)} MB)` },
      { label: "Active Game Backups", val: `${stats.total_backups || 0} files (${(stats.backup_size_mb || 0).toFixed(1)} MB)` },
      { label: "Game Settings DB", val: `${stats.game_db_count || 0} Profiles (Cloudflare Pages + Local)` },
      { label: "Steam64 ID", val: appState.config?.steam_id || "Auto-detected on scan" },
      { label: "AppData Storage Path", val: stats.app_data_dir || "%LOCALAPPDATA%\\Recol\\DLSS-Updater" },
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
    showToast("Checking GitHub for newer DLSS Manager releases...");
    const info = await invoke("check_app_update");
    if (info && info.update_available) {
      showToast(`New release v${info.latest_version} available! Opening release page...`);
      openExternalUrl(info.release_url);
    } else {
      showToast(`✓ You are running the latest version (v${appState.version}).`);
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
        <div class="tech-sub">${escapeHtml(c.preset_hint || "Latest official release")}</div>
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

  const entries = Object.entries(counts);
  if (entries.length === 0) {
    grid.innerHTML = `<div class="muted">No games with compatible upscaler DLLs detected yet. Click <strong>Scan Games</strong> on the left.</div>`;
    return;
  }

  grid.innerHTML = entries
    .map(
      ([name, info]) => `
      <div class="launcher-card" onclick="selectLauncherFromHub('${escapeJsStr(name)}')">
        <img src="assets/icons/${escapeHtml(info.icon)}" alt="${escapeHtml(name)}" />
        <div style="flex:1;min-width:0">
          <div style="font-weight:600">${escapeHtml(name)}</div>
          <div class="muted">${info.count} Games · ${
        info.outdatedCount > 0
          ? `<span style="color:#fbbf24">${info.outdatedCount} need update</span>`
          : `<span style="color:#a3e635">All up to date</span>`
      }</div>
        </div>
        <button class="btn btn-secondary btn-card-action" style="padding:5px 8px"
                onclick="event.stopPropagation(); openLauncherDetailsModal('${escapeJsStr(name)}')"
                title="View detected ${escapeHtml(name)} folders">
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

  const chips = [
    { id: "all", label: `All (${visibleTotal})` },
    { id: "needs_update", label: `● Needs Update (${needsUpdate})` },
    { id: "up_to_date", label: `● Up to Date (${upToDate})` },
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
              title="Toggle visibility of ignored games">
        👁️ ${hiding ? `Show Ignored (${ignoredCount})` : `Ignored Shown (${ignoredCount})`}
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

function renderLauncherSelect() {
  const sel = document.getElementById("select-launcher-filter");
  if (!sel) return;
  const launchers = ["all", ...new Set(appState.games.map((g) => g.launcher_name))];
  sel.innerHTML = launchers
    .map(
      (l) => `
      <option value="${escapeHtml(l)}" ${appState.selectedLauncher === l ? "selected" : ""}>
        ${l === "all" ? "All Launchers" : escapeHtml(l)}
      </option>`
    )
    .join("");
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

  const count = appState.selectedGameIds.size;
  if (btnToggle) {
    btnToggle.classList.toggle("btn-primary", appState.selectMode);
    btnToggle.classList.toggle("btn-secondary", !appState.selectMode);
    btnToggle.textContent = appState.selectMode ? "✕ Exit Select" : "☑ Select";
  }

  if (appState.selectMode || count > 0) {
    bar.style.display = "flex";
    document.getElementById("batch-selected-label").textContent = `${count} ${
      count === 1 ? "game" : "games"
    } selected`;
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
    showToast(`Updating ${ids.length} selected game(s)...`);
    const payload = await invoke("bulk_update_selected_games", {
      gameIds: ids,
      allowedTechs: null,
    });
    appState.selectedGameIds.clear();
    appState.selectMode = false;
    applyPayload(payload);
    showToast(`✓ Updated ${ids.length} selected game(s)!`);
  } catch (e) {
    showToast(String(e), true);
  }
};

function renderGamesGrid() {
  const grid = document.getElementById("games-grid");
  grid.className = `games-hero-grid density-${appState.gridDensity}`;

  const filtered = getFilteredGames();

  if (filtered.length === 0) {
    grid.innerHTML = `<div class="muted">No matching games found for the active filter.</div>`;
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
        ? "Ignored"
        : hasUpdate
        ? "Needs update"
        : "Up to date";
      const statusDotClass = g.ignored ? "" : hasUpdate ? "outdated" : "current";
      const statusColor = g.ignored
        ? "#94a3b8"
        : hasUpdate
        ? "#fbbf24"
        : "#a3e635";

      const badgeText = g.ignored
        ? `👁️ Ignored`
        : hasUpdate
        ? `⚡ ${counts.outdated}/${counts.total} outdated`
        : `✓ Up to date`;
      const badgeClass = g.ignored
        ? ""
        : hasUpdate
        ? "badge-outdated"
        : "badge-current";

      const hasCustomPreset =
        (g.dlss_sr_preset && g.dlss_sr_preset !== "Default") ||
        (g.dlss_rr_preset && g.dlss_rr_preset !== "Default") ||
        (g.dlss_fg_preset && g.dlss_fg_preset !== "Default");

      return `
      <div class="hero-game-card ${hasUpdate ? "needs-update" : ""} ${
        g.ignored ? "is-ignored" : ""
      } ${isSelected ? "is-selected" : ""}"
           oncontextmenu="openCardContextMenu(event, '${g.id}')">
        <!-- Hero Banner -->
        <div class="hero-card-banner" onclick="${
          appState.selectMode
            ? `toggleSelectGame(event, '${g.id}')`
            : `openGameDllModal('${g.id}')`
        }" title="${escapeHtml(g.name)}\n${escapeHtml(g.path)}">
          <div class="hero-card-fallback-icon">
            <img src="assets/icons/${escapeHtml(g.launcher_icon)}" alt="" />
          </div>
          ${artTag}
          <div class="hero-top-badges">
            <div style="display:flex;align-items:center;gap:6px;pointer-events:auto">
              ${
                appState.selectMode || isSelected
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
              ${
                hasCustomPreset
                  ? `<span class="anticheat-badge" style="background:rgba(118,185,0,0.22);color:#a3e635;border-color:rgba(118,185,0,0.45)">🎛️ ${escapeHtml(
                      g.dlss_sr_preset || "Preset"
                    )}</span>`
                  : ""
              }
              ${
                g.anti_cheat
                  ? `<span class="anticheat-badge">🛡️ ${escapeHtml(g.anti_cheat)}</span>`
                  : ""
              }
              <button class="card-menu-btn" onclick="openCardContextMenu(event, '${g.id}')" title="More actions">⋮</button>
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
            <div class="hero-tech-line">${escapeHtml(techLine)}</div>
          </div>
        </div>

        <!-- Fixed 52px Footer -->
        <div class="hero-card-footer">
          <button class="footer-dll-badge ${badgeClass}" onclick="openGameDllModal('${g.id}')" title="Click to inspect technology groups">
            <span>${badgeText}</span>
          </button>
          <div class="footer-actions">
            ${
              g.has_backups
                ? `<button class="btn btn-secondary btn-card-action" onclick="restoreAllGameBackups('${g.id}')" title="Restore original DLLs for ${escapeHtml(g.name)}">
                     ⏪ Restore
                   </button>`
                : ""
            }
            ${
              hasUpdate
                ? `<button class="btn btn-primary btn-card-action" id="btn-update-card-${g.id}" onclick="updateAllGameDlls('${g.id}')" title="Update all outdated technologies in ${escapeHtml(g.name)}">
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

  menu.innerHTML = `
    ${
      counts.outdated > 0
        ? `<button class="context-menu-item" onclick="closeCardContextMenu(); updateAllGameDlls('${game.id}')">
             <span>⚡</span><span>Update Outdated DLLs (${counts.outdated})</span>
           </button>`
        : ""
    }
    <button class="context-menu-item" onclick="closeCardContextMenu(); openGameDllModal('${game.id}')">
      <span>🔍</span><span>Inspect Technologies & Backups</span>
    </button>
    ${
      isNvidia
        ? `<button class="context-menu-item" onclick="closeCardContextMenu(); openGameDlssPresetsModal('${game.id}')">
             <span>🎛️</span><span>DLSS Preset Settings (DRS)</span>
           </button>`
        : ""
    }
    <button class="context-menu-item" onclick="closeCardContextMenu(); openEditDisplayModal('${game.id}')">
      <span>✏️</span><span>Edit Name & Cover Art</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); reportGamesToDbNow('${game.id}')">
      <span>📤</span><span>Report Game to Online-DB</span>
    </button>
    <div class="context-menu-divider"></div>
    <button class="context-menu-item" onclick="closeCardContextMenu(); launchGameNow('${game.id}')">
      <span>🚀</span><span>Launch Game</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); openFolderInExplorer('${escapeJsStr(game.path)}')">
      <span>📂</span><span>Open Game Folder</span>
    </button>
    <button class="context-menu-item" onclick="closeCardContextMenu(); copyTextToClipboard('${escapeJsStr(game.path)}', 'Copied game path to clipboard')">
      <span>📋</span><span>Copy Install Path</span>
    </button>
    <div class="context-menu-divider"></div>
    <button class="context-menu-item" onclick="closeCardContextMenu(); toggleIgnoreGameNow('${game.id}')">
      <span>👁️</span><span>${game.ignored ? "Unignore Game" : "Ignore Game"}</span>
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

window.copyTextToClipboard = async function (text, toastMsg = "Copied to clipboard") {
  try {
    await navigator.clipboard.writeText(text);
    showToast(toastMsg);
  } catch (_) {
    showToast("Copied: " + text);
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
    showToast("Updated Ignore List.");
  } catch (e) {
    showToast(String(e), true);
  }
};

/* ==========================================================================
   Game Details Modal (Per-Tech Update & Per-Tech Restore)
   ========================================================================== */
window.openGameDllModal = function (gameId) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;
  appState.activeModalGameId = gameId;

  const backdrop = document.getElementById("dll-modal-backdrop");
  const modal = document.getElementById("dll-modal-content");
  const counts = getGameUpdateCounts(game);
  const isNvidia = (appState.gpu?.vendor || "").toUpperCase() === "NVIDIA";

  const groupRows = counts.groups
    .map((grp) => {
      const fullFileList = grp.dlls.map((d) => d.filename).join(", ");
      const shortFileList =
        grp.dlls.length <= 2
          ? fullFileList
          : `${grp.dlls.length} files (${grp.dlls
              .slice(0, 2)
              .map((d) => d.filename)
              .join(", ")}, …)`;
      const encodedType = escapeJsStr(grp.typeLabel);

      // Check if backups exist for this specific technology group in this game
      const techBackupsCount = appState.backups.filter(
        (b) =>
          b.game_id === game.id &&
          grp.dlls.some((d) => d.filename.toLowerCase() === b.filename.toLowerCase())
      ).length;

      return `
      <div class="dll-row" style="${!grp.technologyEnabled ? "opacity:0.58" : ""}">
        <div class="dll-row-info">
          <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
            <span class="dll-badge">${escapeHtml(grp.typeLabel)}</span>
            <span class="dll-ver-text">v${escapeHtml(grp.version)}</span>
            ${
              grp.updateAvailable
                ? `<span style="color:#76b900;font-weight:600;font-size:0.82rem">→ v${escapeHtml(grp.latestVersion)}</span>`
                : !grp.technologyEnabled
                ? `<span class="muted" style="font-size:0.74rem;margin-left:4px">(Updates disabled in Settings)</span>`
                : `<span class="muted" style="font-size:0.75rem;margin-left:4px">✓ Up to date</span>`
            }
            ${
              grp.badBuildWarning
                ? `<span class="anticheat-badge" title="${escapeHtml(grp.badBuildWarning)}">⚠️ Watermark Build</span>`
                : ""
            }
          </div>
          <div class="dll-preset-sub" title="${escapeHtml(fullFileList)}">
            ${grp.presetLabel ? escapeHtml(grp.presetLabel) + " · " : ""}<code>${escapeHtml(shortFileList)}</code>
          </div>
        </div>
        <div class="dll-row-action" style="gap:6px">
          ${
            techBackupsCount > 0
              ? `<button class="btn btn-secondary btn-card-action" onclick="restoreTechGroupForGame('${game.id}', '${encodedType}')" title="Restore original ${escapeHtml(grp.typeLabel)} files">
                   ⏪ Restore
                 </button>`
              : ""
          }
          ${
            grp.updateAvailable
              ? `<button class="btn btn-primary btn-card-action" onclick="updateTechGroupForGame('${game.id}', '${encodedType}')">
                   ⚡ Update
                 </button>`
              : `<span class="muted" style="font-size:0.76rem">Current</span>`
          }
        </div>
      </div>`;
    })
    .join("");

  const dbProf = game.db_profile;
  let dbCompatibilityPanel = "";
  if (dbProf || game.anti_cheat) {
    const status = (dbProf?.anticheat_status || "signed_only").toLowerCase();
    const statusMeta =
      status === "safe"
        ? { label: "🟢 Safe — Full DLL Swap & Driver DRS Supported", cls: "compat-safe" }
        : status === "driver_override_only"
        ? { label: "🟠 Driver DRS Override Recommended (Avoid DLL File Swap)", cls: "compat-warn" }
        : status === "blocked"
        ? { label: "🔴 Blocked by Anti-Cheat — Do Not Modify Game DLLs", cls: "compat-danger" }
        : { label: "🟡 Official Signed DLLs & Driver DRS Presets Only", cls: "compat-info" };

    const warnBanner = dbProf?.warning_banner
      ? `<div class="compat-warning-banner">⚠️ ${escapeHtml(dbProf.warning_banner)}</div>`
      : game.anti_cheat
      ? `<div class="compat-warning-banner">🛡️ <strong>${escapeHtml(
          game.anti_cheat
        )} detected:</strong> Only use official signed upscaler DLLs or NVIDIA Driver DRS Presets.</div>`
      : "";

    const unsupportedRows = (dbProf?.unsupported_techs || [])
      .map(
        (u) => `
        <div class="compat-unsupported-item">
          <span class="compat-unsupported-tag">🚫 ${escapeHtml(u.tech)}</span>
          <span class="muted" style="font-size:0.76rem">${escapeHtml(u.reason)}</span>
        </div>`
      )
      .join("");

    const recPresetBadges = [];
    if (dbProf?.recommended_sr_preset)
      recPresetBadges.push(`DLSS SR: ${dbProf.recommended_sr_preset}`);
    if (dbProf?.recommended_rr_preset)
      recPresetBadges.push(`DLSS RR: ${dbProf.recommended_rr_preset}`);
    if (dbProf?.recommended_fg_preset)
      recPresetBadges.push(`DLSS FG: ${dbProf.recommended_fg_preset}`);

    const recSettingsList = (dbProf?.recommended_settings || [])
      .map((s) => `<li>${escapeHtml(s)}</li>`)
      .join("");

    dbCompatibilityPanel = `
      <div class="compat-db-box">
        <div class="compat-db-header">
          <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
            <strong>🎯 Compatibility &amp; Optimal Settings</strong>
            <span class="compat-status-pill ${statusMeta.cls}">${statusMeta.label}</span>
          </div>
          ${
            isNvidia && recPresetBadges.length > 0
              ? `<button class="btn btn-accent btn-card-action" onclick="applyRecommendedProfileNow('${game.id}')" title="Apply ${escapeHtml(recPresetBadges.join(" · "))}">
                   ✨ Apply Optimal Preset (${escapeHtml(dbProf.recommended_sr_preset || "Preset K")})
                 </button>`
              : ""
          }
        </div>
        ${warnBanner}
        ${
          unsupportedRows
            ? `<div class="compat-section-title">Why some technologies are not listed / supported:</div>
               <div class="compat-unsupported-list">${unsupportedRows}</div>`
            : ""
        }
        ${
          recSettingsList
            ? `<div class="compat-section-title">Recommended Graphics &amp; Upscaler Settings:</div>
               <ul class="compat-settings-list">${recSettingsList}</ul>`
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
        <button class="btn btn-secondary btn-card-action" onclick="reportGamesToDbNow('${game.id}')" title="Report Game & Detected Techs to dlssdb.goover.dev">📤</button>
        <button class="btn btn-secondary btn-card-action" onclick="openFolderInExplorer('${escapeJsStr(game.path)}')" title="Open Game Folder">📂</button>
        <button class="btn btn-secondary btn-card-action" onclick="openEditDisplayModal('${game.id}')" title="Edit Name & Cover">✏️</button>
        ${
          isNvidia
            ? `<button class="btn btn-secondary btn-card-action" onclick="openGameDlssPresetsModal('${game.id}')" title="DLSS Preset Overrides">🎛️</button>`
            : ""
        }
        <button class="btn btn-secondary btn-card-action" onclick="launchGameNow('${game.id}')" title="Launch Game">🚀</button>
        <button class="btn btn-secondary btn-card-action" onclick="closeGameDllModal()">✕ Close</button>
      </div>
    </div>
    <div class="modal-body">
      ${dbCompatibilityPanel}
      ${groupRows}
    </div>
    <div class="modal-footer">
      ${
        game.has_backups
          ? `<button class="btn btn-secondary" onclick="restoreAllGameBackups('${game.id}')">
               ⏪ Restore All Backups
             </button>`
          : ""
      }
      ${
        counts.outdated > 0
          ? `<button class="btn btn-accent" onclick="updateAllGameDlls('${game.id}')">
               ⚡ Update All (${counts.outdated})
             </button>`
          : ""
      }
    </div>
  `;

  backdrop.style.display = "flex";
};

window.updateTechGroupForGame = async function (gameId, typeLabel) {
  const game = appState.games.find((g) => g.id === gameId);
  if (!game) return;
  const targetDlls = (game.dlls || []).filter(
    (d) => d.type_label === typeLabel && d.update_available
  );
  if (targetDlls.length === 0) return;

  try {
    showToast(`Updating ${typeLabel} in ${game.name}...`);
    let latestPayload = null;
    for (const d of targetDlls) {
      latestPayload = await invoke("update_single_game_dll", {
        gameId,
        dllPath: d.path,
      });
    }
    if (latestPayload) applyPayload(latestPayload);
    showToast(`✓ ${typeLabel} updated!`);
  } catch (e) {
    showToast(String(e), true);
  }
};

window.restoreTechGroupForGame = async function (gameId, techLabel) {
  try {
    showToast(`Restoring ${techLabel} backups...`);
    const payload = await invoke("restore_tech_group_for_game", {
      gameId,
      techLabel,
    });
    applyPayload(payload);
    showToast(`✓ Restored original ${techLabel} files!`);
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

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>✏️ Edit Display & Artwork — ${escapeHtml(game.name)}</h3>
        <div class="muted" style="font-size:0.76rem">Search Steam for official cover art or set a custom display title.</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ Close</button>
    </div>
    <div class="modal-body" style="gap:14px">
      <div class="preset-field">
        <label>Search Steam Store for Official Cover & App ID</label>
        <div style="display:flex;gap:8px">
          <input type="text" id="edit-steam-query" class="toolbar-select" style="flex:1"
                 value="${escapeHtml(game.name)}" placeholder="e.g. Overwatch 2, Diablo IV, Cyberpunk 2077" />
          <button class="btn btn-primary" onclick="runSteamStoreSearch()">🔎 Search Steam</button>
        </div>
      </div>
      <div id="steam-search-results" class="steam-search-grid"></div>

      <div class="preset-field">
        <label>Display Name</label>
        <input type="text" id="edit-custom-name" class="toolbar-select" value="${escapeHtml(game.name)}" />
      </div>
      <div class="preset-select-grid">
        <div class="preset-field">
          <label>Steam App ID (for Hero & Portrait Artwork)</label>
          <input type="text" id="edit-custom-appid" class="toolbar-select" value="${escapeHtml(game.app_id || "")}" placeholder="e.g. 2357570" />
        </div>
        <div class="preset-field">
          <label>Custom Cover Image URL (Optional)</label>
          <input type="text" id="edit-custom-cover" class="toolbar-select" value="${escapeHtml(game.cover_url || "")}" placeholder="https://..." />
        </div>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="resetDisplayOverrideNow('${game.id}')">Reset to Default</button>
      <button class="btn btn-accent" onclick="saveDisplayOverrideNow('${game.id}')">💾 Save Changes</button>
    </div>
  `;

  backdrop.style.display = "flex";
  runSteamStoreSearch();
};

window.runSteamStoreSearch = async function () {
  const input = document.getElementById("edit-steam-query");
  const container = document.getElementById("steam-search-results");
  if (!input || !container) return;
  const query = input.value.trim();
  if (!query) return;

  container.innerHTML = `<div class="muted">Searching Steam Store...</div>`;
  try {
    const results = await invoke("search_steam_store", { query });
    if (!results || results.length === 0) {
      container.innerHTML = `<div class="muted">No Steam Store matches found for "${escapeHtml(query)}".</div>`;
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
    container.innerHTML = `<div class="muted">Steam search error: ${escapeHtml(String(e))}</div>`;
  }
};

window.pickSteamResult = function (name, appId, heroUrl) {
  const nameEl = document.getElementById("edit-custom-name");
  const appIdEl = document.getElementById("edit-custom-appid");
  const coverEl = document.getElementById("edit-custom-cover");
  if (nameEl) nameEl.value = name;
  if (appIdEl) appIdEl.value = appId;
  if (coverEl) coverEl.value = heroUrl;
  showToast(`Selected "${name}" (App ID ${appId})`);
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
    showToast("✓ Saved display & cover settings!");
  } catch (e) {
    showToast(String(e), true);
  }
};

window.resetDisplayOverrideNow = async function (gameId) {
  try {
    const payload = await invoke("reset_game_display_override", { gameId });
    closeSecondaryModal();
    applyPayload(payload);
    showToast("Reset display settings to default.");
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

  const sr = game.dlss_sr_preset || "Default";
  const rr = game.dlss_rr_preset || "Default";
  const fg = game.dlss_fg_preset || "Default";
  const exe = game.custom_exe_path || "";

  const srOptions = ["Default", "Latest", "Preset J", "Preset K", "Preset L", "Preset M"];
  const rrOptions = ["Default", "Latest", "Preset D", "Preset F"];
  const fgOptions = ["Default", "Latest", "Preset A", "Preset B"];

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>🎛️ NVIDIA DLSS Preset Overrides — ${escapeHtml(game.name)}</h3>
        <div class="muted" style="font-size:0.76rem">Configure driver-level DLSS 4 / 4.5 presets via NVIDIA DRS (<code>nvapi64.dll</code>).</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ Close</button>
    </div>
    <div class="modal-body" style="gap:14px">
      <div class="preset-select-grid">
        <div class="preset-field">
          <label>DLSS Super Resolution Preset</label>
          <select id="game-sr-preset" class="toolbar-select">
            ${srOptions
              .map((o) => `<option value="${o}" ${sr === o ? "selected" : ""}>${o}</option>`)
              .join("")}
          </select>
        </div>
        <div class="preset-field">
          <label>DLSS Ray Reconstruction Preset</label>
          <select id="game-rr-preset" class="toolbar-select">
            ${rrOptions
              .map((o) => `<option value="${o}" ${rr === o ? "selected" : ""}>${o}</option>`)
              .join("")}
          </select>
        </div>
        <div class="preset-field">
          <label>DLSS Frame Generation Preset</label>
          <select id="game-fg-preset" class="toolbar-select">
            ${fgOptions
              .map((o) => `<option value="${o}" ${fg === o ? "selected" : ""}>${o}</option>`)
              .join("")}
          </select>
        </div>
      </div>
      <div class="preset-field">
        <label>Custom Game Executable Path (Optional — used for Launch & DRS Profile)</label>
        <input type="text" id="game-custom-exe" class="toolbar-select" value="${escapeHtml(exe)}" placeholder="Auto-detected in ${escapeHtml(game.path)}" />
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="saveGameDlssPresetsNow('${game.id}', true)">Reset to Default</button>
      <button class="btn btn-accent" onclick="saveGameDlssPresetsNow('${game.id}', false)">⚡ Apply Presets</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.saveGameDlssPresetsNow = async function (gameId, reset = false) {
  const srPreset = reset ? "Default" : document.getElementById("game-sr-preset")?.value || "Default";
  const rrPreset = reset ? "Default" : document.getElementById("game-rr-preset")?.value || "Default";
  const fgPreset = reset ? "Default" : document.getElementById("game-fg-preset")?.value || "Default";
  const customExePath = reset ? null : document.getElementById("game-custom-exe")?.value || null;

  try {
    const payload = await invoke("save_game_dlss_presets", {
      gameId,
      srPreset,
      rrPreset,
      fgPreset,
      customExePath,
    });
    closeSecondaryModal();
    applyPayload(payload);
    showToast(reset ? "Reset game DLSS presets to Default." : "✓ Applied NVIDIA DLSS presets!");
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
        <h3>🚀 Batch Update All Games</h3>
        <div class="muted" style="font-size:0.76rem">Select which upscaler technologies to update across all non-ignored games.</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ Close</button>
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
      <button class="btn btn-secondary" onclick="closeSecondaryModal()">Cancel</button>
      <button class="btn btn-accent" onclick="executeScopedBulkUpdate()">🚀 Start Batch Update</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

window.executeScopedBulkUpdate = async function () {
  const checked = Array.from(document.querySelectorAll(".bulk-scope-cb:checked")).map(
    (cb) => cb.value
  );
  if (checked.length === 0) {
    showToast("Please select at least one technology to update.", true);
    return;
  }
  closeSecondaryModal();
  try {
    showToast("Updating all games with selected technologies...");
    const payload = await invoke("bulk_update_selected_games", {
      gameIds: [],
      allowedTechs: checked,
    });
    applyPayload(payload);
    showToast("✓ Batch update complete!");
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

  const rows =
    logs.length === 0
      ? `<div class="muted">No log events recorded in this session.</div>`
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
        <h3>📋 Application Activity Logs</h3>
        <div class="muted" style="font-size:0.76rem">${logs.length} events recorded in current session</div>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ Close</button>
    </div>
    <div class="modal-body" style="max-height:55vh">
      ${rows}
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick="copyLogsNow()">📋 Copy All</button>
      <button class="btn btn-secondary" onclick="clearLogsNow()">🗑️ Clear Logs</button>
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
  copyTextToClipboard(text || "No logs", "Copied application logs to clipboard");
};

window.openLauncherDetailsModal = function (launcherName) {
  const backdrop = document.getElementById("secondary-modal-backdrop");
  const modal = document.getElementById("secondary-modal-content");
  const games = appState.games.filter((g) => g.launcher_name === launcherName);

  modal.innerHTML = `
    <div class="modal-header">
      <div class="modal-header-info">
        <h3>📂 ${escapeHtml(launcherName)} — Detected Game Paths (${games.length})</h3>
      </div>
      <button class="btn btn-secondary btn-card-action" onclick="closeSecondaryModal()">✕ Close</button>
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
            <button class="btn btn-secondary btn-card-action" onclick="openFolderInExplorer('${escapeJsStr(g.path)}')">📂 Open</button>
            <button class="btn btn-secondary btn-card-action" onclick="copyTextToClipboard('${escapeJsStr(g.path)}', 'Copied path')">📋 Copy</button>
          </div>
        </div>`
        )
        .join("")}
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick="closeSecondaryModal(); selectLauncherFromHub('${escapeJsStr(launcherName)}')">View in Games Library</button>
    </div>
  `;

  backdrop.style.display = "flex";
};

/* ==========================================================================
   Backups & Settings Helpers
   ========================================================================== */
function renderBackups() {
  const list = document.getElementById("backups-list");
  if (appState.backups.length === 0) {
    list.innerHTML = `<div class="muted">No backups recorded yet. Backups are created automatically before any DLL update.</div>`;
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
            ${grp.items.length} backed-up ${grp.items.length === 1 ? "DLL" : "DLLs"} (${grp.items.slice(0, 3).map((i) => escapeHtml(i.filename)).join(", ")}${grp.items.length > 3 ? ", …" : ""}) · ${new Date(grp.newestDate).toLocaleString()}
          </div>
          <div class="muted" style="font-size:0.74rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${escapeHtml(grp.gamePath)}</div>
        </div>
        <div class="backup-card-actions">
          <button class="btn btn-secondary" onclick="openFolderInExplorer('${escapeJsStr(grp.gamePath)}')">📂 Folder</button>
          <button class="btn btn-primary" onclick="restoreAllGameBackups('${grp.gameId}')">⏪ Restore Original</button>
        </div>
      </div>`
    )
    .join("");
}

function renderCustomFolders() {
  const container = document.getElementById("custom-folders-list");
  if (!container) return;
  const folders = appState.config?.custom_folders || [];
  container.innerHTML = folders
    .map(
      (f, idx) => `
      <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
        <code>${escapeHtml(f)}</code>
        <div style="display:flex;gap:6px">
          <button class="btn btn-secondary" style="padding:4px 8px" onclick="openFolderInExplorer('${escapeJsStr(f)}')">📂</button>
          <button class="btn btn-secondary" style="padding:4px 8px" onclick="removeCustomFolder(${idx})">Remove</button>
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

  const ignoredGames = appState.games.filter((g) => g.ignored);
  ignoredEl.innerHTML =
    ignoredGames.length === 0
      ? `<div class="muted" style="font-size:0.78rem">No games are currently ignored. Use a game card's <code>⋮</code> menu to ignore or unignore a game.</div>`
      : ignoredGames
          .map(
            (g) => `
          <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
            <span><strong>${escapeHtml(g.name)}</strong> <span class="muted">(${escapeHtml(g.launcher_name)})</span></span>
            <button class="btn btn-secondary" style="padding:4px 8px" onclick="toggleIgnoreGameNow('${g.id}')">Unignore</button>
          </div>`
          )
          .join("");

  const blacklisted = appState.config.blacklisted_games || [];
  blackEl.innerHTML = blacklisted
    .map(
      (b, idx) => `
      <div style="display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
        <code>${escapeHtml(b)}</code>
        <button class="btn btn-secondary" style="padding:4px 8px" onclick="removeBlacklistEntry(${idx})">Remove</button>
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

window.updateAllGameDlls = async function (gameId) {
  const game = appState.games.find((g) => g.id === gameId);
  const gameTitle = game ? game.name : "Game";
  const btn = document.getElementById(`btn-update-card-${gameId}`);
  if (btn) {
    btn.disabled = true;
    btn.textContent = "⏳ Updating...";
  }
  try {
    showToast(`Updating ${gameTitle}...`);
    const payload = await invoke("update_all_for_game", { gameId });
    applyPayload(payload);
    showToast(`✓ ${gameTitle} updated to latest DLLs!`);
  } catch (e) {
    if (btn) {
      btn.disabled = false;
      btn.textContent = "⚡ Update";
    }
    showToast(String(e), true);
  }
};

window.restoreAllGameBackups = async function (gameId) {
  try {
    showToast("Restoring original DLLs for game...");
    const payload = await invoke("restore_all_for_game", { gameId });
    applyPayload(payload);
    showToast("✓ Original game DLLs restored!");
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
    showToast("Settings saved.");
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
  const origHtml = `<span>🔍 Scan Games</span>`;
  try {
    scanBtn.disabled = true;
    scanBtn.innerHTML = `<span>⏳ Scanning...</span>`;
    showToast("Scanning drives for games with GPU-compatible upscaler DLLs...");
    const payload = await invoke("scan_games");
    applyPayload(payload);
    showToast(`Scan complete: ${payload.games.length} supported games found.`);
    if (switchAfter) {
      switchView("games");
    }
  } catch (e) {
    showToast(String(e), true);
  } finally {
    scanBtn.disabled = false;
    scanBtn.innerHTML = origHtml;
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  document.addEventListener("click", () => {
    closeCardContextMenu();
  });

  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.addEventListener("click", () => switchView(btn.dataset.view));
  });

  document.getElementById("global-search").addEventListener("input", (e) => {
    appState.searchQuery = e.target.value;
    switchView("games");
    renderGamesGrid();
  });

  document.getElementById("select-launcher-filter").addEventListener("change", (e) => {
    appState.selectedLauncher = e.target.value;
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

  document.getElementById("btn-theme-toggle").addEventListener("click", async () => {
    if (!appState.config) return;
    const next = appState.config.theme_mode === "light" ? "dark" : "light";
    appState.config.theme_mode = next;
    applyThemeMode(next);
    await saveConfig();
  });

  document.getElementById("btn-refresh-manifest").addEventListener("click", async () => {
    try {
      showToast("Checking latest DLL versions...");
      appState.catalog = await invoke("refresh_dll_manifest");
      renderHubTechStrip();
      showToast("Upscaler versions up to date!");
    } catch (e) {
      showToast(String(e), true);
    }
  });

  document.getElementById("toggle-dlss-overlay").addEventListener("change", async (e) => {
    try {
      const enabled = await invoke("toggle_dlss_indicator", { enabled: e.target.checked });
      e.target.checked = enabled;
      showToast(`DLSS Overlay Indicator ${enabled ? "enabled" : "disabled"}.`);
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

  document.querySelectorAll("[data-url]").forEach((btn) => {
    btn.addEventListener("click", () => {
      openExternalUrl(btn.dataset.url);
    });
  });

  try {
    const initial = await invoke("get_initial_state");
    applyPayload(initial);
    if (!initial.games || initial.games.length === 0) {
      triggerGameScan(false);
    }
    invoke("refresh_dll_manifest")
      .then((catalog) => {
        if (catalog) {
          appState.catalog = catalog;
          renderHubTechStrip();
        }
      })
      .catch(() => {});

    invoke("sync_game_database", { customUrl: null })
      .then((payload) => {
        if (payload) applyPayload(payload);
      })
      .catch(() => {});

    invoke("check_app_update")
      .then((info) => {
        if (info && info.update_available) {
          appState.appUpdateInfo = info;
          const pill = document.getElementById("app-update-pill");
          if (pill) {
            pill.style.display = "inline-flex";
            pill.innerHTML = `<span>⬆️ v${escapeHtml(info.latest_version)} Available</span>`;
            pill.onclick = () => openExternalUrl(info.release_url);
          }
        }
      })
      .catch(() => {});
  } catch (e) {
    showToast("Failed to load initial state: " + e, true);
  }
});
