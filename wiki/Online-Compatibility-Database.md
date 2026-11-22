# ☁️ Online-Kompatibilitäts- & Einstellungs-Datenbank (`dlssdb.goover.dev`)

**DLSS Manager** ist direkt mit der offiziellen Cloud-Datenbank **[`https://dlssdb.goover.dev`](https://dlssdb.goover.dev)** verbunden. Dort werden für Hunderte Spiele geprüfte Anti-Cheat-Hinweise, optimale DLSS-Presets (`Preset K / J / L / M / F`), nicht unterstützte Technologien und empfohlene In-Game-Grafikeinstellungen gepflegt.

---

## 🔑 1. Persönlichen API-Key erstellen & verbinden

1. Öffne **[`https://dlssdb.goover.dev`](https://dlssdb.goover.dev)** im Browser (oder klicke in DLSS Manager unter **⚙️ Einstellungen** auf **„🌐 Web-Portal öffnen“**).
2. Registriere ein kostenloses Konto bzw. melde dich an und erstelle im Dashboard deinen persönlichen API-Key (beginnt mit `dlss_live_...`).
3. Kopiere den Key und füge ihn in **DLSS Manager → ⚙️ Einstellungen → ☁️ Spiele-Kompatibilitäts- & Einstellungs-Datenbank** in das Feld **„Persönlicher DLSS-DB API-Key“** ein.
4. Klicke auf **„🔄 Key speichern & DB synchronisieren“**.

> 💡 **Offline-Fallback:** Selbst ohne Internetverbindung oder ohne API-Key enthält DLSS Manager bereits eine eingebettete Seed-Datenbank (`src-tauri/resources/games_db_seed.json`) für wichtige Titel wie *Star Citizen*, *Cyberpunk 2077*, *Alan Wake 2*, *Indiana Jones*, *Black Myth: Wukong*, *Diablo IV* und *Overwatch 2* und speichert jede erfolgreiche Online-Synchronisation lokal in `%LOCALAPPDATA%\gOOvER\DLSS-Manager\game_compat_db_cache.json`.

---

## 📤 2. Ein Spiel an die Online-DB melden (Community-Report)

Du kannst deine Erfahrungen mit einem Spiel direkt aus DLSS Manager heraus mit einem Klick an `dlssdb.goover.dev` melden:

1. Klicke auf einer Spielkarte auf das **`⋮`**-Menü → **„📤 Spiel an Online-DB melden“** (oder im Spieldetails-Modal oben rechts auf den **`📤`**-Button).
2. Im Melde-Dialog siehst du automatisch den Spielnamen, den Launcher, erkanntes Anti-Cheat und alle im Spielordner gefundenen Upscaler-DLLs (`DLSS`, `DLSS-RR`, `DLSS-G`, `Streamline`, `XeSS`, `DirectStorage`).
3. **Pflichtangabe (1 Klick):** Wähle aus, ob das Spiel mit DLSS-/Upscaler-Update funktioniert:
   - **`✅ Geht (Funktioniert)`**
   - **`❌ Geht nicht (Probleme / Blockiert)`**
4. **Optionale Angaben:**
   - **Empfohlenes DLSS Preset:** z. B. `Preset K (DLSS 4.5 Transformer Gen 2)`
   - **Empfohlene In-Game Einstellungen:** Eine Einstellung pro Zeile (z. B. `Graphics Renderer: Vulkan`)
   - **Hinweise / Fehlerbeschreibung:** Kurze Notiz (z. B. *„Läuft perfekt mit Preset K“* oder *„EasyAntiCheat blockiert DLL-Tausch, Treiber-DRS funktioniert“*)
5. Klicke auf **„📤 Jetzt an Datenbank melden“**.

---

## 🔍 3. Automatische Bibliotheks-Übermittlung (`Auto-Discovery`)

Wenn unter **⚙️ Einstellungen** die Option **„Gescannte Spiele & erkannte Technologien automatisch an `dlssdb.goover.dev` senden“** aktiviert ist (oder du auf **„📤 Gescannte Spiele an DB senden“** klickst), hilft deine Bibliothek dabei, neue Spiele mit DLSS/XeSS/Streamline/DirectStorage in die Datenbank aufzunehmen:

- **Keine ungeprüften „Funktioniert“-Behauptungen:** Automatische Scans werden grundsätzlich mit dem Status **`works_status: "unverified"`** und **`anticheat_status: "unverified"`** übermittelt. Ein Spiel wird also niemals automatisch als „Geht / Sicher“ markiert, solange kein Nutzer oder Administrator es aktiv getestet und bestätigt hat.
- **Automatische Referenz-Links zur Verifizierung:**
  - Für **Steam-Spiele** (mit bekannter Steam `app_id`) wird automatisch der passende **SteamDB-Link** (`https://steamdb.info/app/<app_id>/`) mitgesendet.
  - Für **Nicht-Steam-Spiele** wird automatisch ein **PCGamingWiki-Suchlink** (`https://www.pcgamingwiki.com/w/index.php?search=<Titel>`) hinterlegt.
- Im Admin-Panel auf `dlssdb.goover.dev` können neue Einreichungen über Mehrfachauswahl-Checkboxen (`Shift+Klick`-Bereichsauswahl, Suchfilter, **Alle geprüften freigeben / löschen**) komfortabel geprüft und veröffentlicht werden.
