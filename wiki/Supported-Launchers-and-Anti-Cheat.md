# 🎮 Unterstützte Launcher, Star Citizen & Anti-Cheat-Schutz

**DLSS Manager** durchsucht automatisch alle angeschlossenen Festplatten und SSDs (`A:\` bis `Z:\`) nach installierten Spielen aus **9 Launchern** und schützt dich zuverlässig vor Anti-Cheat-Konflikten sowie verwaisten Deinstallations-Ordnern.

---

## 1. Alle 9 unterstützten Spiele-Launcher im Überblick

| Launcher | Erkennungs- & Verifizierungs-Mechanismus | Besonderheiten |
| :--- | :--- | :--- |
| **Steam** | Windows Registry (`HKCU`/`HKLM`) + `libraryfolders.vdf` + `appmanifest_*.acf` + `loginusers.vdf` | Filtert deinstallierte Restordner in `steamapps\common` anhand vorhandener `.acf`-Manifeste; erkennt automatisch deine `Steam64 ID` und löst 2025/2026 SHA-1-gehashte Cover- & Hero-Bilder auf |
| **Star Citizen (RSI Launcher)** | `%APPDATA%\rsilauncher\logs\log.log` + Laufwerks-Scan (`StarCitizen\LIVE`, `PTU`, `EPTU`, `TECH-PREVIEW`, `HOTFIX`) | Eigene Spielkarte pro Kanal (`LIVE`, `PTU` usw.), automatische Bindung an das NVIDIA-Treiberprofil `"Squadron 42 - Star Citizen"`, offizielles Star-Citizen-Cover & EAC-Schutzmodus |
| **Battle.net** | `C:\ProgramData\Battle.net\Agent\product.db` (Protobuf-Parser) + Windows Uninstall Registry | Prüft aktive Flavor-Unterordner (`_retail_`, `_classic_`, `_ptr_`, `_beta_`) auf echte Spiel-Executables und ignoriert leere/deinstallierte Geisterordner |
| **Epic Games** | `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` + `LauncherInstalled.dat` | Verifiziert jeden Ordner mit `.egstore` gegen echte Epic-Manifeste (`bIsIncompleteInstall != true`), damit alte Restordner nicht fälschlich als installiert angezeigt werden |
| **GOG Galaxy** | `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\*` + Laufwerks-Scan (`GOG Games`) | Erkennt Installationspfad, Titel und primäre `.exe` direkt aus der GOG-Registry |
| **EA App / Origin** | `C:\ProgramData\EA Desktop\InstallData` + Registry + Laufwerks-Scan (`EA Games`) | Automatischer Scan aller EA-Desktop-Bibliotheken auf allen Laufwerken |
| **Ubisoft Connect** | `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\*` + Laufwerks-Scan (`Ubisoft Game Launcher\games`) | Erkennt alle Ubisoft-Connect-Installationen über Registry und Laufwerksbibliotheken |
| **Xbox / PC Game Pass** | `.GamingRoot` + `MicrosoftGame.config` auf allen Laufwerken (`XboxGames`) | Löst `Content\`-Unterordner, Anzeigenamen und lokale Splash-/Logo-Grafiken aus `MicrosoftGame.config` auf |
| **Eigene Ordner (Custom)** | Frei konfigurierbar unter **⚙️ Einstellungen → Eigene Spieleordner** + Laufwerks-Scan (`Games`, `Spiele`, `PC Games`) | Akzeptiert sowohl einzelne Spielordner als auch übergeordnete Bibliotheksordner mit vielen Spielen; verwirft automatisch Duplikate, falls ein Spiel bereits unter einem offiziellen Launcher erkannt wurde |

---

## 2. 🚀 Spezial-Guide: Star Citizen & EasyAntiCheat (EAC)

**Star Citizen** ist eines der beliebtesten Spiele für DLSS 4 / 4.5 Transformer-Upscaling (deutlich schärfere MFD-Displays im Cockpit, klarere HUD-Schriften und ruhigere Schiffskanten in der Distanz), besitzt aber zwei technische Besonderheiten:

### Warum darf `Bin64\nvngx_dlss.dll` in Star Citizen nicht auf der Festplatte ersetzt werden?
1. Star Citizen wird durch **EasyAntiCheat (EAC)** geschützt. Beim Spielstart prüft EAC die Datei-Hashes sämtlicher Dateien im Ordner `LIVE\Bin64\` gegen das offizielle CIG-Manifest.
2. Wird `Bin64\nvngx_dlss.dll` manuell durch eine neuere DLL überschrieben, verweigert EasyAntiCheat den Spielstart mit einer File-Integrity-Warnung (obwohl die neue DLL offiziell von NVIDIA signiert ist, weil ihr Hash nicht mit dem CIG-Patch-Manifest übereinstimmt).

### Wie aktiviert man trotzdem DLSS 4.5 / Preset K in Star Citizen?
- Star Citizen liefert ab Werk bereits **`nvngx_dlss.dll v310.2.1.0`** (DLSS 4.0) aus.
- Deshalb schaltet DLSS Manager Star Citizen automatisch in den geschützten Modus **„🛡️ Nur Treiber-DRS (Original-DLL EAC-geschützt)“**:
  - Die Datei `Bin64\nvngx_dlss.dll` auf der Festplatte bleibt unangetastet (100 % EAC-sicher!).
  - Über **„✨ Optimales Preset anwenden (Preset K)“** schreibt DLSS Manager das Override-Flag (`0x10E41DF3 = 1`) und **`Preset K` (`0x10E41DF7 = 0x0B`)** direkt in das NVIDIA-Treiberprofil **`"Squadron 42 - Star Citizen"`** (`starcitizen.exe`).
  - Gleichzeitig hebt der **NVIDIA App Whitelist-Unlocker** (`🔓 Alle Spiele freischalten`) NVIDIAs Sperre für Star Citizen auf und nutzt das aktuelle NGX-OTA-Modell aus dem NVIDIA-Treiber.

### Empfohlene In-Game Einstellungen für Star Citizen
- **Graphics Renderer:** `Vulkan` *(oder DirectX 11)*
- **Upscaling:** `DLSS` (`Quality` oder `Balanced` bei 1440p / 4K)
- **NVIDIA Treiber-DRS Preset:** `Preset K` *(Transformer Gen 2)*
- **Hinweis zu `r.TSR.UpscalingPreset`:** Ohne aktives NVIDIA-Treiber-DRS-Override fordert Star Citizen intern `Preset E` (CNN) an (`r.TSR.UpscalingPreset = 0`). Erst durch das Treiber-DRS-Override in DLSS Manager wird im Treiber **Preset K (Transformer)** erzwungen.

---

## 3. 🛡️ Anti-Cheat-Stufen & Authenticode-Signaturprüfung

### Die 4 Anti-Cheat-Kategorien in DLSS Manager

| Badge | Status-Code | Bedeutung & Verhalten in DLSS Manager |
| :--- | :--- | :--- |
| **🟢 Sicher (Voller DLL-Tausch & DRS)** | `safe` | Kein restriktives Anti-Cheat vorhanden (z. B. *Cyberpunk 2077*). Sowohl der direkte Austausch der DLL-Dateien als auch NVIDIA Treiber-DRS-Presets sind uneingeschränkt nutzbar. |
| **🟡 Nur signierte DLLs & Treiber-DRS** | `signed_only` | Das Spiel nutzt Anti-Cheat (z. B. *Diablo IV*, *Overwatch 2*, *Call of Duty*), akzeptiert aber offiziell von NVIDIA/Intel/Microsoft digital signierte Original-DLLs sowie Treiber-DRS-Presets. Keine unsignierten Drittanbieter-Wrapper injizieren! |
| **🟠 Nur Treiber-DRS empfohlen** | `driver_override_only` / `drs_only` | EasyAntiCheat prüft den exakten Datei-Hash der mitgelieferten DLL (z. B. *Star Citizen*). DLSS Manager sperrt den direkten DLL-Button im Spiel-Modal (`🛡️ Nur Treiber-DRS`), damit die Original-DLL intakt bleibt, und konfiguriert das Spiel sicher über das NVIDIA Treiber-DRS-Preset. |
| **🔴 Durch Anti-Cheat blockiert** | `blocked` | Strenges Kernel-Anti-Cheat blockiert jegliche DLL-Modifikation auf der Festplatte. Nutze ausschließlich das globale oder spielspezifische NVIDIA Treiber-DRS-Preset. |

### Automatische PE- & Authenticode-Prüfung (`WinVerifyTrust`)
Bevor DLSS Manager eine heruntergeladene DLL in einen Spielordner kopiert, durchläuft jede Datei zwei Sicherheitsprüfungen in [`pe_version.rs`](../src-tauri/src/pe_version.rs):
1. **Win32 PE-Header & `VS_FIXEDFILEINFO`-Prüfung:** Stellt sicher, dass es sich um eine gültige 64-Bit-Windows-DLL (`MZ` / `PE\0\0`) mit auslesbarer Versionsressource handelt (verhindert beschädigte Downloads oder HTML-Fehlerseiten).
2. **Kryptografische Authenticode-Signaturprüfung (`wintrust.dll!WinVerifyTrust`):** Jede NVIDIA DLSS- und Streamline-DLL wird gegen die Windows-Zertifikatskette auf eine gültige, unveränderte digitale Signatur geprüft. Unsignierte oder manipulierte Binärdateien werden strikt abgelehnt.
