# ❓ Troubleshooting & Frequently Asked Questions (FAQ)

Here are direct answers and solutions to the most common questions about **DLSS Manager**, **NVIDIA Driver DRS Presets**, **NVIDIA App**, and the **Game Scanner**.

---

### 1. Does NVIDIA App overwrite my DLSS presets?
**Cause:** Yes — by default, **NVIDIA App**'s background service (`NVDisplay.ContainerLocalSystem`) checks `ApplicationStorage.json` and `fingerprint.db` on startup or update. For any game flagged with `"Disable_SR_Override": true` (such as **Star Citizen**), NVIDIA App actively writes `0` (Blocked) into the game's NVIDIA driver profile.  
**Solution in DLSS Manager:**
1. On the **Hub** (or under **⚙️ Settings → Global NVIDIA DLSS Presets**), click **🔓 Unlock All Games** inside the **🛡️ NVIDIA App Override Status** card.
2. Click **🔄 Restart NVIDIA Service**.
3. In addition, DLSS Manager automatically cleans any `0`-blocks in your game DRS profiles and re-enforces your saved presets on every app launch.

---

### 2. How can I verify which DLSS preset is actually active in-game?
You have two ways to verify:
1. **Inside DLSS Manager (Live Driver Readback):** Click any game card to open the **Game Details Modal**. Inside the **🎛️ Applied NVIDIA DLSS Presets (Driver DRS)** panel, DLSS Manager reads the live setting directly from `nvapi64.dll` and displays **`🟢 Driver Live: Preset K`**.
2. **In-Game (DLSS On-Screen Overlay):** Enable the **DLSS Overlay** toggle in the top bar of DLSS Manager and launch your game. In the bottom-left corner of the game screen, NVIDIA's official NGX indicator displays the exact DLSS version and active preset (`Preset K`, `Transformer`).

---

### 3. A game (e.g. from Ubisoft Connect, EA App, or on another drive) is missing — why?
- **Are all drives scanned?** Yes, DLSS Manager automatically scans all drive letters from `A:\` to `Z:\` for Steam, Star Citizen, Battle.net, Epic Games, GOG, EA App, Ubisoft Connect, Xbox Game Pass, and generic game folders (`Games`, `Spiele`, `PC Games`).
- **Important:** A game is only listed if its installation directory contains at least one supported **64-bit upscaler or streaming DLL** (`nvngx_dlss*.dll`, `sl.*.dll`, `libxess*.dll`, or `dstorage*.dll`).
- **Note:** Older titles released before DLSS/XeSS/DirectStorage existed (such as *Tom Clancy's The Division 1* from 2016) do not ship with any of these DLLs and therefore do not appear as upscaler games.
- If a game is installed in a non-standard directory, add the folder under **⚙️ Settings → Custom Game Folders** and click **🔍 Scan Games** in the sidebar.

---

### 4. Why did a game appear twice (e.g. under Steam and Epic Games) even though it was uninstalled on Epic?
- Some uninstallers leave behind old folders containing a `.egstore` subdirectory (Epic) or leftover files in `steamapps\common\` or Battle.net.
- **Solution (automatic):** DLSS Manager verifies every Epic Games folder against official Epic manifests (`C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` with `bIsIncompleteInstall != true`), checks Steam folders against `appmanifest_*.acf`, verifies Battle.net games against active flavor executables (`_retail_`, `_classic_`), and drops generic standalone duplicates when the same game is already installed under an official launcher.

---

### 5. Do I need to update the DLL file AND apply the preset in Star Citizen?
- **Yes — you need both for true DLSS 4.5!**
  1. **Update the DLL (`⚡ Update` to `v310.9.16.0`):** Star Citizen ships with an older DLSS 4.0 DLL (`v310.2.1.0`). Replacing it via DLSS Manager's **⚡ Update** button installs the official NVIDIA-signed **DLSS 4.5 (`v310.9.16.0`, 74.18 MB)** binary containing the new Transformer Gen 2 weights (`Preset K`, `Preset L`, `Preset M`). Because DLSS Manager only installs official NVIDIA Authenticode-signed DLLs (`WinVerifyTrust`), **EasyAntiCheat allows this update safely**.
  2. **Apply the Driver DRS Preset (`✨ Apply Optimal Preset (Preset K)`):** Without a driver preset override, Star Citizen's engine still defaults to `Preset E` (old CNN model) even when `v310.9.16.0` is installed. Applying **Preset K** in DLSS Manager forces the NVIDIA driver to use the new DLSS 4.5 Transformer Gen 2 model from the updated DLL!

---

### 6. A game crashes or shows visual artifacts after a DLL update — how do I restore the original DLL?
DLSS Manager automatically backs up every original game DLL to `%LOCALAPPDATA%\gOOvER\DLSS-Manager\backups\` before replacing it:
- **Restore a single technology:** Open the game in DLSS Manager and click **⏪ Restore** next to the specific technology row (e.g. `DLSS` or `Streamline`).
- **Restore the entire game:** Click **⏪ Restore All Backups** in the Game Details modal footer, or switch to the **🛡️ Backups** tab in the sidebar and click **⏪ Restore Original**.

---

### 7. Are AMD FSR DLLs (`amd_fidelityfx_*.dll`) managed as well?
No. Starting with `v1.0.0-beta.2`, AMD FSR was removed from the scanner and update catalog to focus exclusively on **NVIDIA DLSS 4.5 / 5**, **NVIDIA Streamline**, **Intel XeSS 2**, and **Microsoft DirectStorage** from official vendor upstreams.
