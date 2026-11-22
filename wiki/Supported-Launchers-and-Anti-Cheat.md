# 🎮 Supported Launchers, Star Citizen & Anti-Cheat Protection

**DLSS Manager** automatically scans all connected drives (`A:\` through `Z:\`) for installed games across **9 launchers** while protecting against anti-cheat conflicts and leftover uninstalled folders.

---

## 1. All 9 Supported Game Launchers

| Launcher | Detection & Verification Mechanism | Key Features |
| :--- | :--- | :--- |
| **Steam** | Windows Registry (`HKCU`/`HKLM`) + `libraryfolders.vdf` + `appmanifest_*.acf` + `loginusers.vdf` | Filters uninstalled leftover folders in `steamapps\common` against active `.acf` manifests; auto-detects your `Steam64 ID` and resolves 2025/2026 SHA-1 hashed cover & hero artwork |
| **Star Citizen (RSI Launcher)** | `%APPDATA%\rsilauncher\logs\log.log` + multi-drive scan (`StarCitizen\LIVE`, `PTU`, `EPTU`, `TECH-PREVIEW`, `HOTFIX`) | Dedicated card per channel (`LIVE`, `PTU`, etc.), automatic binding to the `"Squadron 42 - Star Citizen"` NVIDIA DRS profile, built-in cover artwork & EAC-safe DRS mode |
| **Battle.net** | `C:\ProgramData\Battle.net\Agent\product.db` (Protobuf parser) + Windows Uninstall Registry | Verifies active flavor subdirectories (`_retail_`, `_classic_`, `_ptr_`, `_beta_`) for real game executables and filters uninstalled ghost folders |
| **Epic Games** | `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` + `LauncherInstalled.dat` | Verifies every `.egstore` directory against active Epic manifests (`bIsIncompleteInstall != true`) so leftover folders from uninstalled games are ignored |
| **GOG Galaxy** | `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\*` + multi-drive scan (`GOG Games`) | Resolves install paths, titles, and primary executables directly from the GOG registry |
| **EA App / Origin** | `C:\ProgramData\EA Desktop\InstallData` + Registry + multi-drive scan (`EA Games`) | Automatic discovery of EA Desktop libraries across all drives |
| **Ubisoft Connect** | `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\*` + multi-drive scan (`Ubisoft Game Launcher\games`) | Detects all Ubisoft Connect installations via registry and drive libraries |
| **Xbox / PC Game Pass** | `.GamingRoot` + `MicrosoftGame.config` across all drives (`XboxGames`) | Resolves `Content\` subdirectories, display names, and local splash/logo artwork from `MicrosoftGame.config` |
| **Custom Folders** | User-configured directories in **⚙️ Settings → Custom Game Folders** + drive scan (`Games`, `Spiele`, `PC Games`) | Accepts both single game folders and multi-game parent libraries; automatically drops standalone duplicates if a game is already installed under an official launcher |

---

## 2. 🚀 Special Guide: Star Citizen & EasyAntiCheat (EAC)

**Star Citizen** benefits tremendously from DLSS 4 / 4.5 Transformer upscaling (much sharper cockpit MFD text, cleaner HUD elements, and crisp distant ship hulls), but has two important technical characteristics:

### Why Must `Bin64\nvngx_dlss.dll` Not Be Replaced on Disk in Star Citizen?
1. Star Citizen uses **EasyAntiCheat (EAC)**, which verifies the file hashes of all files inside `LIVE\Bin64\` against CIG's official build manifest at launch.
2. If `Bin64\nvngx_dlss.dll` is replaced on disk with a newer DLL, EasyAntiCheat blocks the game from launching with a file integrity violation (even though the newer DLL carries an official NVIDIA signature, because its hash differs from CIG's patch manifest).

### How to Enable DLSS 4.5 / Preset K Safely in Star Citizen
- Star Citizen already ships with **`nvngx_dlss.dll v310.2.1.0`** (DLSS 4.0) out of the box.
- DLSS Manager therefore places Star Citizen automatically into **`🛡️ Driver DRS Only (Stock DLL EAC Protected)`** mode:
  - The on-disk `Bin64\nvngx_dlss.dll` file remains completely untouched (100% EAC-safe!).
  - Clicking **✨ Apply Optimal Preset (Preset K)** writes the override enable flag (`0x10E41DF3 = 1`) and **`Preset K` (`0x10E41DF7 = 0x0B`)** directly into the NVIDIA driver profile **`"Squadron 42 - Star Citizen"`** (`starcitizen.exe`).
  - Combined with the **NVIDIA App Whitelist Unlocker** (`🔓 Unlock All Games`), the NVIDIA driver overrides Star Citizen's default CNN fallback (`r.TSR.UpscalingPreset = 0` / `Preset E`) and runs **Preset K (Transformer)** with the latest NGX OTA model weights.

### Recommended In-Game Settings for Star Citizen
- **Graphics Renderer:** `Vulkan` *(or DirectX 11)*
- **Upscaling:** `DLSS` (`Quality` or `Balanced` at 1440p / 4K)
- **NVIDIA Driver DRS Preset:** `Preset K` *(Transformer Gen 2)*

---

## 3. 🛡️ Anti-Cheat Tiers & Authenticode Signature Verification

### The 4 Anti-Cheat Categories in DLSS Manager

| Badge | Status Code | Meaning & Behavior in DLSS Manager |
| :--- | :--- | :--- |
| **🟢 Safe (Full DLL Swap & DRS)** | `safe` | No restrictive anti-cheat present (e.g. *Cyberpunk 2077*). Both on-disk DLL updates and NVIDIA Driver DRS presets work without restriction. |
| **🟡 Signed DLLs & Driver DRS Only** | `signed_only` | Game uses anti-cheat (e.g. *Diablo IV*, *Overwatch 2*, *Call of Duty*), but accepts official digitally signed NVIDIA/Intel/Microsoft binaries and Driver DRS presets. Do not inject unsigned third-party wrappers! |
| **🟠 Driver DRS Override Recommended** | `driver_override_only` / `drs_only` | EasyAntiCheat checks the exact file hash of the bundled DLL (e.g. *Star Citizen*). DLSS Manager locks the on-disk DLL update button (`🛡️ Driver DRS Only`) to keep the original DLL intact and configures the game safely via NVIDIA Driver DRS. |
| **🔴 Blocked by Anti-Cheat** | `blocked` | Strict kernel anti-cheat blocks on-disk DLL modifications. Use global or per-game NVIDIA Driver DRS presets only. |

### Automatic PE & Authenticode Verification (`WinVerifyTrust`)
Before copying any downloaded DLL into a game folder, DLSS Manager performs two security checks in [`pe_version.rs`](../src-tauri/src/pe_version.rs):
1. **Win32 PE Header & `VS_FIXEDFILEINFO` Validation:** Confirms the file is a valid 64-bit Windows PE binary (`MZ` / `PE\0\0`) with a readable version resource (preventing corrupted downloads or HTML error pages).
2. **Cryptographic Authenticode Signature Verification (`wintrust.dll!WinVerifyTrust`):** Every NVIDIA DLSS and Streamline binary is verified against the Windows certificate trust chain. Unsigned or tampered binaries are strictly rejected.
