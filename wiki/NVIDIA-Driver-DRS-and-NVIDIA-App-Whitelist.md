# 🎛️ NVIDIA Driver DRS (`nvapi64.dll`) & NVIDIA App Whitelist

One of **DLSS Manager**'s most powerful capabilities is direct **NVIDIA Driver Settings (DRS)** configuration via `nvapi64.dll`, combined with the **1-Click NVIDIA App Override Whitelist Unlocker**.

---

## 1. How NVIDIA Driver DRS (`nvapi64.dll`) Works

Instead of injecting unsigned third-party wrapper DLLs into game directories (which triggers anti-cheat blocks in EasyAntiCheat and BattlEye), DLSS Manager communicates directly with NVIDIA's official driver interface `nvapi64.dll`.

### NVAPI DRS Setting IDs Used

| Function | NVAPI Setting ID (Hex) | Values |
| :--- | :--- | :--- |
| **DLSS-SR Override Enable** | `0x10E41DF3` | `1` = Enabled, *deleted* on `Default` (`0` means "Explicitly blocked by NVIDIA App"!) |
| **DLSS-SR Render Preset** | `0x10E41DF7` | `0x01`–`0x06` (`Preset A–F`), `0x0A` (`Preset J`), `0x0B` (`Preset K`), `0x0C` (`Preset L`), `0x0D` (`Preset M`), `0x00FFFFFF` (`Latest`) |
| **DLSS-RR Override Enable** | `0x10E41DF5` | `1` = Enabled, *deleted* on `Default` |
| **DLSS-RR Render Preset** | `0x10E41DF8` | `0x04` (`Preset D`), `0x06` (`Preset F`), `0x00FFFFFF` (`Latest`) |
| **DLSS-FG Override Enable** | `0x10E41DF4` | `1` = Enabled, *deleted* on `Default` |
| **DLSS-FG Render Preset** | `0x10E41DF6` | `0x01` (`Preset A`), `0x02` (`Preset B`), `0x00FFFFFF` (`Latest`) |
| **Streamline Override Gate** | `0x10E41E06` | Used by NVIDIA App as a blocking gate (`0`); automatically cleaned by DLSS Manager |
| **DLSS-G Multi-Frame Gen (MFG)** | `0x10308298`, `0x104D6667`, `0x10562D0F`, `0x10CF4125` | Controls MFG Mode (`Default`, `Fixed`, `Dynamic`, `Off`), Multiplier (`2×–6×`), and Dynamic Target FPS |

### Crucial Detail: Why `DeleteProfileSetting` Instead of Writing `0` on "Default"?
In the NVIDIA driver, setting `0x10E41DF3` to `0` does **not** mean "Inherit Default" — it means **"Explicitly forbid DLSS overrides for this application"**!  
Whenever you reset a game preset to `Default` in DLSS Manager, it invokes `NvAPI_DRS_DeleteProfileSetting` to remove the setting cleanly from the game profile so global presets or in-game defaults work normally.

---

## 2. Why Does NVIDIA App Reset Presets Back to `0`?

When **NVIDIA App** is installed on your PC, its background service (`NVDisplay.ContainerLocalSystem`) periodically evaluates an internal application whitelist:

1. **`ApplicationStorage.json`** located at:
   ```text
   C:\ProgramData\NVIDIA Corporation\NVIDIA app\UpdateFramework\ota-artifacts\grd\post-processing\*\ApplicationStorage.json
   ```
2. **`fingerprint.db`** (NVIDIA App's CEF cache) located at:
   ```text
   %LOCALAPPDATA%\NVIDIA Corporation\NVIDIA app\CefCache\Default\fingerprint.db
   ```

For games that NVIDIA has not yet officially whitelisted in the NVIDIA App UI (such as **Star Citizen**), these files contain blocking flags by default:
- `"Disable_SR_Override": true`
- `"Disable_RR_Override": true`
- `"Disable_FG_Override": true`
- `"Disable_SR_Model_Override": true`
- `"Disable_RR_Model_Override": true`

Whenever NVIDIA App starts or syncs in the background, it reads those `true` flags and actively writes **`0x10E41DF3 = 0`** (Blocked!) into the game's NVIDIA driver profile!

---

## 3. The Solution: 1-Click NVIDIA App Override Whitelist Unlocker

DLSS Manager solves this on three levels:

### Level 1: 1-Click Whitelist Unlocker (`🔓 Unlock All Games`)
On the **Hub** (and under **⚙️ Settings → Global NVIDIA DLSS Presets**), DLSS Manager automatically checks your NVIDIA App whitelist status on startup:
- Clicking **🔓 Unlock All Games** creates automatic backups (`ApplicationStorage.json.dlss_manager.bak` & `fingerprint.db.dlss_manager.bak`).
- It flips all `Disable_FG_Override`, `Disable_RR_Override`, `Disable_SR_Override`, `Disable_RR_Model_Override`, and `Disable_SR_Model_Override` flags in `ApplicationStorage.json` and `fingerprint.db` to `false` (`0`), enables `IsOpsSupported` in `ApplicationStorage.json`, and flips `<IsAutomatable>0</IsAutomatable>` and `<DLSS_Override_No_OPS>0</DLSS_Override_No_OPS>` to `1` in `fingerprint.db` so `IsOpsSupported` persists across Windows reboots.
- Clicking **🔄 Restart NVIDIA Service** reloads `NVDisplay.ContainerLocalSystem` so NVIDIA App stops reverting your overrides.

### Level 2: Automatic Cleanup of `0`-Blocks in Driver Profiles
Even if a game is left on `Default` (inheriting your Global preset), DLSS Manager inspects its per-game DRS profile on startup (`clear_game_dlss_profile_blocks`) and automatically deletes any stale `0`-blocks placed by NVIDIA App (e.g. on `"Squadron 42 - Star Citizen"`).

### Level 3: Live DRS Readback in Game Details (`Driver Live`)
Whenever you open a game's details modal, DLSS Manager queries `NvAPI_DRS_GetSetting` in real time and displays a live verification badge:
- **`🟢 Driver Live: Preset K`** → The preset is active and verified in the live NVIDIA driver.
- **`🟠 ⚠️ Driver DRS: Blocked by NVIDIA App (0)`** → NVIDIA App wrote a `0`-block (clicking **🔓 Unlock All Games** or re-selecting your preset clears it immediately).

---

## 4. Local NVIDIA NGX OTA Model Cache

Modern NVIDIA drivers automatically download updated DLSS 4.5 model weights via NVIDIA's **NGX Over-The-Air (OTA)** system and store them under:
```text
C:\ProgramData\NVIDIA\NGX\models\dlss\versions\
```
DLSS Manager displays the highest locally installed **NGX OTA version** (e.g. `v310.5.3` / `v310.9.16`) directly in the NVIDIA App status bar on the Hub. Thanks to this NGX cache, the NVIDIA driver can inject newer Transformer Gen 2 model weights (**Preset K**) at runtime even in games whose on-disk DLL is an older DLSS 4.0 build (like Star Citizen's `v310.2.1.0`)!

---

## 5. In-Game Verification: The DLSS On-Screen Overlay

Want to verify in-game that **Preset K** and **DLSS 4.5** are actively rendering?
1. Enable the **DLSS Overlay** toggle in the top bar of DLSS Manager (writes `HKLM\SOFTWARE\NVIDIA Corporation\Global\NGXCore\ShowDlssIndicator = 0x400`).
2. Launch your game with DLSS enabled.
3. Look at the bottom-left corner of the screen: NVIDIA's official NGX overlay displays the active **DLSS DLL version**, **internal render resolution**, and **active preset (`Preset K`, `Transformer`)** in real time.
4. Toggle **DLSS Overlay** back off when finished.
