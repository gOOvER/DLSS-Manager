# 📚 DLSS Manager Wiki

Welcome to the official **DLSS Manager** Wiki (`v1.0.0+`) — the native **Rust + Tauri 2** all-in-one desktop manager for **NVIDIA DLSS 4.5 / 5**, **NVIDIA Driver DRS Presets (`nvapi64.dll`)**, **NVIDIA Streamline SDK**, **Intel XeSS 2**, and **Microsoft DirectStorage**.

> 💡 **Why DLSS Manager Was Built:** Instead of juggling 4 different tools and manual workarounds (one tool to swap DLLs, *NVIDIA Profile Inspector* to force driver presets, manual edits to `ApplicationStorage.json` / `fingerprint.db` so NVIDIA App doesn't overwrite your settings, and Registry hacks for the DLSS indicator), **DLSS Manager** combines everything into a single, 1-click application.

---

## 🧭 Quick Navigation

| Wiki Page | Contents |
| :--- | :--- |
| **[🚀 Installation & Quick Start](Installation-and-Quick-Start)** | Portable `DLSS-Manager.exe`, NSIS/MSI installers, Winget, Administrator privileges (UAC), storage paths (`%LOCALAPPDATA%\gOOvER\DLSS-Manager`) & 1-minute setup |
| **[📊 DLSS 4.5 / 5 Versions, Presets (A–M) & Technologies](DLSS-Presets-and-Versions)** | Complete DLSS version history (`v2.0` to `v310.9.16+`), differences between **Preset J, K, L, M, E, F**, DLSS-G Multi-Frame Generation (`2×–6×`) & all managed DLLs |
| **[🎛️ NVIDIA Driver DRS & NVIDIA App Whitelist](NVIDIA-Driver-DRS-and-NVIDIA-App-Whitelist)** | How `nvapi64.dll` driver presets work, why **NVIDIA App** resets overrides to `0`, how the **1-Click Whitelist Unlocker** works & live verification via the **DLSS Overlay** |
| **[🎮 Supported Launchers, Star Citizen & Anti-Cheat](Supported-Launchers-and-Anti-Cheat)** | Automatic multi-drive scanning (`A:\`–`Z:\`) across **9 launchers**, dedicated **Star Citizen** guide (EasyAntiCheat & Driver DRS), manifest verification & Authenticode checks |
| **[☁️ Online Compatibility Database (`dlssdb.goover.dev`)](Online-Compatibility-Database)** | Creating your personal API key, syncing compatibility profiles, `unverified` auto-discovery (with SteamDB/PCGamingWiki links) & 1-click community reporting |
| **[🛠️ Architecture & Building from Source](Architecture-and-Building-from-Source)** | Rust backend modules (`scanner.rs`, `gpu_nvapi.rs`, `nvapp_whitelist.rs`, `pe_version.rs`, `repository.rs`), SQLite schema & compiling on Windows/Linux |
| **[❓ Troubleshooting & FAQ](Troubleshooting-and-FAQ)** | Common questions & solutions (NVIDIA App preset resets, missing or duplicate games, 1-click backup restore, watermark builds `310.7.0`, anti-cheat rules) |

---

## ✨ Key Features at a Glance

1. **🦀 Native Rust + Tauri 2 Engine:**
   - Instant startup, low memory footprint, embedded **SQLite (WAL + FTS5)** database, and native Win32 PE `VS_FIXEDFILEINFO` version & `WinVerifyTrust` signature inspection without loading untrusted DLLs into memory.
2. **🟢 Official Upstream Sources & 74.18 MB DLSS 4.5 Build:**
   - Downloads **DLSS 4.5 (`v310.9.16.0`)** exclusively from official NVIDIA sources (`NVIDIA/DLSS` & local NVIDIA NGX OTA driver cache), preferring the full **74.18 MB production build** (containing all presets `A–M`) and automatically blocking watermarked dev builds (`v310.7.*`).
   - Official upstreams for **NVIDIA Streamline SDK** (`NVIDIA-RTX/Streamline` GitHub Releases), **Microsoft DirectStorage** (official Microsoft NuGet package `Microsoft.Direct3D.DirectStorage`), and **Intel XeSS 2**.
3. **🎛️ Direct NVIDIA Driver DRS Control (`nvapi64.dll`):**
   - Configures **Super Resolution (`Preset K / J / L / M / E`)**, **Ray Reconstruction (`Preset F / D`)**, and **Frame Generation (`Latest / A / B` + RTX 50 MFG `Fixed 2×–6×` / `Dynamic` + Target FPS)** globally and per game directly in the NVIDIA driver.
   - Reads the actual live driver status (**`Driver Live: Preset K`**) straight back from `nvapi64.dll`.
4. **🔓 1-Click NVIDIA App Override Whitelist Unlocker:**
   - Prevents **NVIDIA App** from silently resetting driver presets back to `0` on non-whitelisted games (such as **Star Citizen**) by cleaning `Disable_*_Override` locks in `ApplicationStorage.json`, `fingerprint.db`, and the game's DRS profile.
5. **🛡️ Automatic Backups & Anti-Cheat Protection:**
   - Every original game DLL is backed up automatically before replacement and can be restored per technology or for the entire game in one click.
   - Automatically detects **EasyAntiCheat**, **BattlEye**, **Vanguard**, and **Ricochet**, protecting EAC-monitored titles (like Star Citizen) with a dedicated **`Driver DRS Only`** mode.
6. **🇬🇧 / 🇩🇪 Full Bilingual UI (English & German):**
   - Seamless 1-click language toggle in the top bar with crisp inline SVG flags and fully localized descriptions.

---

## 🔗 Important Links

- **GitHub Repository:** [https://github.com/gOOvER/DLSS-Manager](https://github.com/gOOvER/DLSS-Manager)
- **Latest Releases (Download):** [https://github.com/gOOvER/DLSS-Manager/releases/latest](https://github.com/gOOvER/DLSS-Manager/releases/latest)
- **Online Compatibility Database:** [https://dlssdb.goover.dev](https://dlssdb.goover.dev)
- **Report Issues / Feature Requests:** [https://github.com/gOOvER/DLSS-Manager/issues](https://github.com/gOOvER/DLSS-Manager/issues)
- **Support & Donations:** [https://donate.goover.dev](https://donate.goover.dev)
