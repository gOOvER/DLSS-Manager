# 🚀 Installation & Quick Start

This page explains how to install **DLSS Manager**, why Administrator privileges are required on Windows, where local files and backups are stored, and how to optimize your games in under a minute.

---

## 📥 1. Installation on Windows

You can run **DLSS Manager** in three ways:

### Option A: Standalone Portable Executable (`DLSS-Manager.exe`)
- Download `DLSS-Manager.exe` from the [**GitHub Releases**](https://github.com/gOOvER/DLSS-Manager/releases/latest) page or run `dist\DLSS-Manager.exe`.
- **Zero installation required** — the single executable embeds the entire Rust backend and UI.

### Option B: Windows Installer (`.exe` NSIS / `.msi`)
- Download the installer (`DLSS.Manager_x.y.z_x64-setup.exe` or `.msi`) from [**GitHub Releases**](https://github.com/gOOvER/DLSS-Manager/releases/latest).
- Automatically creates Start Menu and Desktop shortcuts as well as an uninstaller entry.

### Option C: Winget (Windows Package Manager)
```powershell
winget install "DLSS Manager"
```

> **Windows 10 / 11 Requirement:** **Microsoft Edge WebView2 Runtime** (pre-installed by default on modern Windows 10 and Windows 11 systems).

---

## 🛡️ 2. Why Administrator Privileges (UAC) Are Required

`DLSS-Manager.exe` includes an embedded Windows UAC application manifest (`requireAdministrator`). If launched without elevation, a warning banner appears on the **Hub** with a **🛡️ Relaunch as Administrator** button.

Administrator rights are required for four core operations:
1. **NVIDIA Driver DRS (`nvapi64.dll`):** Writing and deleting global and per-game DLSS profiles inside the NVIDIA driver database (`C:\ProgramData\NVIDIA Corporation\Drs\nvdrsdb0.bin` / `nvdrsdb1.bin`).
2. **NVIDIA App Override Whitelist:** Patching `C:\ProgramData\NVIDIA Corporation\NVIDIA app\UpdateFramework\ota-artifacts\grd\post-processing\*\ApplicationStorage.json` and `%LOCALAPPDATA%\NVIDIA Corporation\NVIDIA app\CefCache\Default\fingerprint.db`.
3. **NVIDIA Service Restart:** Restarting `NVDisplay.ContainerLocalSystem` so NVIDIA App reloads the unlocked whitelist.
4. **DLSS On-Screen Overlay (Registry):** Toggling `ShowDlssIndicator` under `HKEY_LOCAL_MACHINE\SOFTWARE\NVIDIA Corporation\Global\NGXCore`.

---

## 📂 3. Storage Locations & Automatic Migration (`AppData`)

All configuration files, backups, and cached binaries are stored under:

```text
%LOCALAPPDATA%\gOOvER\DLSS-Manager\
```
*(e.g. `C:\Users\<YourName>\AppData\Local\gOOvER\DLSS-Manager\`)*

| File / Folder | Description |
| :--- | :--- |
| `config.json` | App configuration (language `de`/`en`, theme, global DLSS presets, custom game folders, blacklist, Steam & Online-DB API keys) |
| `dlss_manager_v2.db` | Local SQLite database (WAL + FTS5) storing discovered games, per-game preset overrides, cover URLs, and backup metadata |
| `backups\<game_id>\` | Automatic backup vault for all original game DLLs prior to replacement |
| `dll_cache\` | Downloaded and Authenticode-verified upscaler binaries (`nvngx_dlss.dll`, `sl.*.dll`, `libxess.dll`, `dstorage.dll`, etc.) |
| `artwork_cache\` | Locally cached Steam hero and portrait cover images |
| `game_compat_db_cache.json` | Offline cache of the game compatibility database synced from `dlssdb.goover.dev` |

> 💡 **Automatic Legacy Migration:** If a legacy `%LOCALAPPDATA%\Recol\DLSS-Updater` folder exists on your PC, **DLSS Manager** automatically migrates your configuration, SQLite database, backups, and caches to `%LOCALAPPDATA%\gOOvER\DLSS-Manager` on first launch.

You can open the AppData directory, DLL cache, and Backups vault at any time from **ℹ️ Info → System & Storage Diagnostics**.

---

## ⚡ 4. Quick Start in 4 Steps

1. **Scan Games (runs automatically on startup):**
   - On launch, DLSS Manager scans all connected drives (`A:\` to `Z:\`) for installed games across **9 launchers** (Steam, Star Citizen, Battle.net, Epic Games, GOG, EA App, Ubisoft Connect, Xbox Game Pass & Custom Folders).
2. **Unlock NVIDIA App Override Whitelist (on the Hub):**
   - If the **🛡️ NVIDIA App Override Status** card on the **Hub** reports that games are locked by NVIDIA App, click **🔓 Unlock All Games**. This prevents NVIDIA App from resetting your DLSS presets back to `0`.
3. **Update DLLs & Apply Optimal Preset:**
   - Open the **🎮 Games** view and click any game card (or click **⚡ Update** directly on the card footer).
   - Inside the **Game Details Modal**, inspect all detected technologies (`DLSS`, `DLSS-RR`, `DLSS-G`, `Streamline`, `XeSS`, `DirectStorage`) and the **🎛️ Applied NVIDIA DLSS Presets (Driver DRS)** panel.
   - Click **✨ Apply Optimal Preset (Preset K)** or select any preset from the dropdown — it is written immediately to the NVIDIA driver (`Driver Live: Preset K`).
4. **Verify In-Game (Optional):**
   - Toggle **DLSS Overlay** on in the top bar. Next time you launch a game with DLSS enabled, NVIDIA displays an on-screen indicator in the bottom-left corner showing the exact DLSS version and active preset (`Preset K`, `Transformer Gen 2`).
