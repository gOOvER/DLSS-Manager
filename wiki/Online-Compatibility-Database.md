# ☁️ Online Compatibility & Settings Database (`dlssdb.goover.dev`)

**DLSS Manager** integrates directly with the official cloud database at **[`https://dlssdb.goover.dev`](https://dlssdb.goover.dev)**, providing verified anti-cheat rules, optimal DLSS presets (`Preset K / J / L / M / F`), unsupported technology explanations, and recommended in-game graphics settings.

---

## 🔑 1. Creating & Connecting Your Personal API Key

1. Open **[`https://dlssdb.goover.dev`](https://dlssdb.goover.dev)** in your browser (or click **🌐 Open Web Portal** in **DLSS Manager → ⚙️ Settings**).
2. Register a free account or sign in, and generate your personal API key in the dashboard (prefixed with `dlss_live_...`).
3. Paste your API key into **DLSS Manager → ⚙️ Settings → ☁️ Game Compatibility & Settings Database → Personal DLSS-DB API Key**.
4. Click **🔄 Save Key & Sync DB**.

> 💡 **Offline Seed Fallback:** Even offline or without an API key, DLSS Manager includes an embedded seed database (`src-tauri/resources/games_db_seed.json`) for major titles (*Star Citizen*, *Cyberpunk 2077*, *Alan Wake 2*, *Indiana Jones*, *Black Myth: Wukong*, *Diablo IV*, *Overwatch 2*) and caches every online sync locally in `%LOCALAPPDATA%\gOOvER\DLSS-Manager\game_compat_db_cache.json`.

---

## 📤 2. Reporting a Game to the Online-DB (Community Report)

You can share your compatibility test results and optimal settings for any game directly from DLSS Manager in one click:

1. Click a game card's **`⋮`** menu → **📤 Report Game to Online-DB** (or click the **`📤`** icon in the top-right header of the Game Details modal).
2. The dialog automatically populates the game title, launcher, detected anti-cheat, and all upscaler technologies found in the game directory (`DLSS`, `DLSS-RR`, `DLSS-G`, `Streamline`, `XeSS`, `DirectStorage`).
3. **Required (1 Click):** Select whether the game works with DLSS / upscaler updates:
   - **`✅ Works`**
   - **`❌ Doesn't Work (Broken / Blocked)`**
4. **Optional Fields:**
   - **Recommended DLSS Preset:** e.g. `Preset K (DLSS 4.5 Transformer Gen 2)`
   - **Recommended In-Game Settings:** One setting per line (e.g. `Graphics Renderer: Vulkan`)
   - **Notes / Issue Description:** Short note (e.g. *"Works great with Preset K"* or *"EasyAntiCheat blocks DLL swap, Driver DRS works"*)
5. Click **📤 Submit Report to Database**.

---

## 🔍 3. Automatic Library Discovery (`Auto-Discovery`)

When **Automatically report scanned games & detected technologies to `dlssdb.goover.dev`** is enabled under **⚙️ Settings** (or when clicking **📤 Send Scanned Games to DB**), your library helps expand the database with newly discovered games:

- **No Unverified "Works" Claims:** Automatic library scans always submit **`works_status: "unverified"`** and **`anticheat_status: "unverified"`**. A game is never automatically marked as `works` / `safe` until a user or administrator explicitly tests and verifies it.
- **Automatic Reference Links for Verification:**
  - For **Steam games** (with a known Steam `app_id`), an automatic **SteamDB link** (`https://steamdb.info/app/<app_id>/`) is attached.
  - For **non-Steam games**, an automatic **PCGamingWiki search link** (`https://www.pcgamingwiki.com/w/index.php?search=<Title>`) is attached.
- Inside the admin portal on `dlssdb.goover.dev`, newly submitted games can be filtered and reviewed in bulk using multi-select checkboxes (`Shift+Click` range selection, search filter, **Bulk Approve / Bulk Delete**).
