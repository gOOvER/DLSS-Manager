# 🛠️ Architektur & Kompilieren aus dem Quellcode

**DLSS Manager** (`v1.0.0-beta.1+`) wurde von Grund auf in **Rust + Tauri 2** mit einer modernen, Abhängigkeits-freien **Vanilla ES2022 + CSS Glassmorphism** Oberfläche entwickelt.

---

## 🏗️ 1. Projektstruktur & Rust-Backend-Module

```text
DLSS-Manager/
├── src-tauri/
│   ├── build.rs                 # Betet das Windows UAC-Manifest (requireAdministrator) & App-Icon ein
│   ├── Cargo.toml               # Rust-Abhängigkeiten (tauri 2, rusqlite, reqwest, windows-sys, sha1)
│   ├── tauri.conf.json          # Tauri 2 Fenster-, Bundle- & CSP-Konfiguration
│   ├── resources/
│   │   └── games_db_seed.json   # Eingebettete Offline-Seed-Datenbank für Spielkompatibilität & Presets
│   └── src/
│       ├── main.rs              # Haupteinstiegspunkt der Windows/Linux-Binärdatei
│       ├── lib.rs               # Tauri-Befehle (IPC Commands), App-Initialisierung & Steam Store API
│       ├── models.rs            # Datenstrukturen (Game, DetectedDll, DllType, AppConfig, LiveDrsStatus)
│       ├── scanner.rs           # Multithreaded Laufwerks- & Launcher-Scanner (9 Launcher, A:\–Z:\)
│       ├── repository.rs        # Async HTTP-Manifest-Engine (NVIDIA/DLSS, Streamline, DirectStorage, XeSS)
│       ├── gpu_nvapi.rs         # Dynamischer FFI-Client für nvapi64.dll (NVIDIA Driver DRS Read/Write)
│       ├── nvapp_whitelist.rs   # NVIDIA App Whitelist-Unlocker (ApplicationStorage.json & fingerprint.db)
│       ├── pe_version.rs        # Win32 PE VS_FIXEDFILEINFO Parser & WinVerifyTrust Authenticode-Prüfung
│       ├── database.rs          # SQLite (WAL + FTS5) Persistenz, Backup-Tresor & AppData-Migration
│       └── game_db.rs           # Client für dlssdb.goover.dev (Sync, Auto-Discovery & Community-Reports)
├── ui/
│   ├── index.html               # Hauptlayout (Sidebar, Topbar, Hub, Spiele, Backups, Einstellungen, Info)
│   ├── app.js                   # Komplette Frontend-Logik, DE/EN-Lokalisierung, Modals & Referenztabellen
│   ├── styles.css               # Dark/Light Theme, Hero-Game-Cards, Presets-Grid & Tabellen-Styling
│   └── assets/                  # Launcher-Icons (SVG) & Standard-Cover (z. B. Star Citizen)
└── wiki/                        # Offizielle Projektdokumentation & GitHub Wiki
```

---

## ⚙️ 2. Voraussetzungen zum Kompilieren

### Windows 10 / 11 (x64)
1. **Rust Toolchain (`1.85+`, `stable-x86_64-pc-windows-msvc`)** über [rustup.rs](https://rustup.rs/)
2. **Visual Studio 2022 Build Tools** mit dem Workload *„Desktopentwicklung mit C++“* (inkl. Windows 10/11 SDK)
3. **Microsoft Edge WebView2 Runtime** (auf Windows 10/11 vorinstalliert)
4. Optional für Installer-Bundles (`.msi` / NSIS `.exe`):
   ```powershell
   cargo install tauri-cli --version "^2.0.0"
   ```

---

## 🚀 3. Build-Befehle

### Repository klonen
```powershell
git clone https://github.com/gOOvER/DLSS-Manager.git
cd DLSS-Manager
```

### Schneller Release-Build (Portable `DLSS_Manager.exe`)
```powershell
cd src-tauri
cargo build --release --bin dlss-manager
Copy-Item target\release\dlss-manager.exe ..\dist\DLSS_Manager.exe -Force
```

### Tests ausführen
```powershell
cd src-tauri
cargo test --release
```

### Windows-Installer (`.msi` & NSIS `.exe`) erstellen
```powershell
.\build_rust_installer.ps1
```
*(oder manuell via `cd src-tauri && cargo tauri build`)*

---

## 💡 4. Hinweis zu Windows 11 Smart App Control beim lokalen Kompilieren

Falls unter Windows 11 **Smart App Control** aktiv ist und `rustc` beim erneuten Kompilieren nach einem Systemneustart meldet, dass eine zuvor gebaute Proc-Macro-DLL (`serde_derive-*.dll` oder `tauri_macros-*.dll`) nicht geladen werden kann (`error[E0463]: can't find crate for serde` / Win32 Error `4551`), lösche einfach die alten `.dll`-Artefakte im `target\release\deps`-Ordner, damit `link.exe` sie frisch mit vertrauenswürdigem Kernel-Attribut verlinkt:

```powershell
cd src-tauri
Remove-Item "target\release\deps\*.dll" -Force
Remove-Item "target\release\.fingerprint\serde_derive-*", "target\release\.fingerprint\tauri-macros-*" -Recurse -Force
cargo build --release --bin dlss-manager
```
