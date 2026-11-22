# 🚀 Installation & Erste Schritte (Quick Start)

Diese Seite erklärt die Installation von **DLSS Manager**, warum Administrator-Rechte unter Windows benötigt werden, wo alle Daten gespeichert werden und wie du in einer Minute deine Spiele optimierst.

---

## 📥 1. Installation unter Windows

Du kannst **DLSS Manager** auf drei Arten nutzen:

### Option A: Portable Einzeldatei (`DLSS-Manager.exe`)
- Lade `DLSS-Manager.exe` von der [**GitHub Releases**](https://github.com/gOOvER/DLSS-Manager/releases/latest)-Seite herunter oder nutze `dist\DLSS-Manager.exe`.
- Es ist **keine Installation** erforderlich – die Datei enthält das gesamte Rust-Backend und die Benutzeroberfläche.

### Option B: Windows-Installer (`.exe` NSIS / `.msi`)
- Lade das Setup (`DLSS.Manager_x.y.z_x64-setup.exe` oder `.msi`) von [**GitHub Releases**](https://github.com/gOOvER/DLSS-Manager/releases/latest) herunter.
- Erstellt automatisch Startmenü- und Desktop-Verknüpfungen sowie einen Deinstallations-Eintrag.

### Option C: Winget (Windows Package Manager)
```powershell
winget install "DLSS Manager"
```

> **Voraussetzung unter Windows 10 / 11:** Die **Microsoft Edge WebView2 Runtime** (auf aktuellen Windows-10/11-Systemen bereits standardmäßig vorinstalliert).

---

## 🛡️ 2. Warum Administrator-Rechte (UAC) wichtig sind

Seit `v1.0.0-beta.2` besitzt `DLSS-Manager.exe` ein eingebettetes Windows-UAC-Manifest (`requireAdministrator`). Falls die App ohne Admin-Rechte gestartet wird, erscheint auf dem **Hub** ein gelbes Warnbanner mit dem Button **„🛡️ Als Administrator neu starten“**.

Administrator-Rechte werden für vier Kernfunktionen benötigt:
1. **NVIDIA Treiber-DRS (`nvapi64.dll`):** Das Schreiben und Löschen von globalen und spielspezifischen DLSS-Profilen in der NVIDIA-Treiberdatenbank (`C:\ProgramData\NVIDIA Corporation\Drs\nvdrsdb0.bin` / `nvdrsdb1.bin`).
2. **NVIDIA App Override-Whitelist:** Das Patchen von `C:\ProgramData\NVIDIA Corporation\NVIDIA app\UpdateFramework\ota-artifacts\grd\post-processing\*\ApplicationStorage.json` und `%LOCALAPPDATA%\NVIDIA Corporation\NVIDIA app\CefCache\Default\fingerprint.db`.
3. **NVIDIA Dienst-Neustart:** Das Neustarten von `NVDisplay.ContainerLocalSystem`, damit die NVIDIA App die freigeschaltete Whitelist neu einliest.
4. **DLSS On-Screen Overlay (Registry):** Das Setzen von `ShowDlssIndicator` unter `HKEY_LOCAL_MACHINE\SOFTWARE\NVIDIA Corporation\Global\NGXCore`.

---

## 📂 3. Speicherorte & Automatische Migration (`AppData`)

Alle lokalen Einstellungen, Backups und gecachten DLLs liegen zentral unter:

```text
%LOCALAPPDATA%\gOOvER\DLSS-Manager\
```
*(z. B. `C:\Users\<DeinName>\AppData\Local\gOOvER\DLSS-Manager\`)*

| Datei / Ordner | Beschreibung |
| :--- | :--- |
| `config.json` | App-Konfiguration (Sprache `de`/`en`, Theme, globale DLSS-Presets, eigene Spieleordner, Blacklist, Steam- & Online-DB-API-Keys) |
| `dlss_manager_v2.db` | Lokale SQLite-Datenbank (WAL + FTS5) mit allen erkannten Spielen, benutzerdefinierten Presets, Cover-URLs und Backup-Metadaten |
| `backups\<game_id>\` | Automatischer Backup-Tresor für alle originalen Spiel-DLLs vor dem Überschreiben |
| `dll_cache\` | Heruntergeladene und Authenticode-geprüfte Upscaler-DLLs (`nvngx_dlss.dll`, `sl.*.dll`, `libxess.dll`, `dstorage.dll` usw.) |
| `artwork_cache\` | Lokal zwischengespeicherte Steam-Hero- und Cover-Bilder |
| `game_compat_db_cache.json` | Offline-Cache der zuletzt von `dlssdb.goover.dev` synchronisierten Kompatibilitätsdatenbank |

> 💡 **Automatische Migration:** Falls auf deinem System noch ein alter Ordner `%LOCALAPPDATA%\Recol\DLSS-Updater` existiert, migriert **DLSS Manager** beim ersten Start automatisch alle Einstellungen, Datenbanken, Backups und Caches nach `%LOCALAPPDATA%\gOOvER\DLSS-Manager`.

Über **ℹ️ Info → System- & Speicher-Diagnose** kannst du den AppData-Ordner, den DLL-Cache und den Backup-Tresor jederzeit mit einem Klick im Windows Explorer öffnen.

---

## ⚡ 4. Schnellstart in 4 Schritten

1. **Spiele scannen (passiert beim Start automatisch):**
   - Beim Start durchsucht DLSS Manager automatisch alle Laufwerke (`A:\` bis `Z:\`) nach Spielen aus **9 Launchern** (Steam, Star Citizen, Battle.net, Epic Games, GOG, EA App, Ubisoft Connect, Xbox Game Pass & eigene Ordner).
2. **NVIDIA App Whitelist freischalten (auf dem Hub):**
   - Falls auf dem **Hub** unter **„🛡️ NVIDIA App Override-Status“** angezeigt wird, dass Spiele durch die NVIDIA App blockiert werden, klicke auf **„🔓 Alle Spiele freischalten“**. Dadurch überschreibt die NVIDIA App deine DLSS-Presets nicht mehr.
3. **DLLs aktualisieren & Optimales Preset aktivieren:**
   - Öffne die **🎮 Spiele**-Ansicht und klicke auf ein Spiel (oder direkt auf **⚡ Update** auf der Spielkarte).
   - Im **Spieldetails-Modal** siehst du alle erkannten Technologien (`DLSS`, `DLSS-RR`, `DLSS-G`, `Streamline`, `XeSS`, `DirectStorage`) sowie die Box **„🎛️ Angewendete NVIDIA DLSS Presets (Treiber-DRS)“**.
   - Klicke auf **„✨ Optimales Preset anwenden (Preset K)“** oder wähle dein Wunsch-Preset im Dropdown aus – es wird sofort live in den NVIDIA-Treiber geschrieben (`Treiber-Live: Preset K`).
4. **Im Spiel überprüfen (Optional):**
   - Schalte oben in der Topbar den Schalter **„DLSS-Overlay“** ein. Beim nächsten Spielstart zeigt NVIDIA unten links im Spiel exakt an, welche DLSS-Version und welches Preset (`Preset K`, `Transformer Gen 2`) aktiv rendert.
