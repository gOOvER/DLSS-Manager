# 🛠️ Architecture & Building from Source

**DLSS Manager** (`v1.0.0+`) is built from the ground up in **Rust + Tauri 2** with a dependency-free **Vanilla ES2022 + CSS Glassmorphism** frontend.

---

## 🏗️ 1. Project Structure & Rust Backend Modules

```text
DLSS-Manager/
├── src-tauri/
│   ├── build.rs                 # Embeds Windows UAC manifest (requireAdministrator) & app icon
│   ├── Cargo.toml               # Rust dependencies (tauri 2, rusqlite, reqwest, windows-sys, sha1)
│   ├── tauri.conf.json          # Tauri 2 window, bundle, mainBinaryName ("DLSS-Manager") & CSP config
│   ├── resources/
│   │   └── games_db_seed.json   # Embedded offline seed database for game compatibility & presets
│   └── src/
│       ├── main.rs              # Windows/Linux binary entry point
│       ├── lib.rs               # Tauri IPC commands, startup pipeline & Steam Store API
│       ├── models.rs            # Data structures (Game, DetectedDll, DllType, AppConfig, LiveDrsStatus)
│       ├── scanner.rs           # Multithreaded multi-drive launcher scanner (9 launchers, A:\–Z:\)
│       ├── repository.rs        # Async HTTP manifest engine (NVIDIA/DLSS, Streamline, DirectStorage, XeSS)
│       ├── gpu_nvapi.rs         # Dynamic FFI client for nvapi64.dll (NVIDIA Driver DRS Read/Write)
│       ├── nvapp_whitelist.rs   # NVIDIA App Whitelist Unlocker (ApplicationStorage.json & fingerprint.db)
│       ├── pe_version.rs        # Win32 PE VS_FIXEDFILEINFO parser & WinVerifyTrust Authenticode check
│       ├── database.rs          # SQLite (WAL + FTS5) persistence, backup vault & AppData migration
│       └── game_db.rs           # Client for dlssdb.goover.dev (sync, auto-discovery & community reports)
├── ui/
│   ├── index.html               # Main layout (Sidebar, Topbar, Hub, Games, Backups, Settings, Info)
│   ├── app.js                   # Frontend logic, DE/EN localization, modals & reference tables
│   ├── styles.css               # Dark/Light theme, hero game cards, preset grid & table styling
│   └── assets/                  # Launcher icons (SVG) & built-in covers (e.g. Star Citizen)
└── wiki/                        # Official project documentation & GitHub Wiki
```

---

## ⚙️ 2. Build Prerequisites

### Windows 10 / 11 (x64)
1. **Rust Toolchain (`1.85+`, `stable-x86_64-pc-windows-msvc`)** via [rustup.rs](https://rustup.rs/)
2. **Visual Studio 2022 Build Tools** with the *"Desktop development with C++"* workload (including Windows 10/11 SDK)
3. **Microsoft Edge WebView2 Runtime** (pre-installed on Windows 10/11)
4. Optional for installer bundles (`.msi` / NSIS `.exe`):
   ```powershell
   cargo install tauri-cli --version "^2.0.0"
   ```

---

## 🚀 3. Build Commands

### Clone the Repository
```powershell
git clone https://github.com/gOOvER/DLSS-Manager.git
cd DLSS-Manager
```

### Build Portable Executable (`DLSS-Manager.exe`)
```powershell
cd src-tauri
cargo build --release --bin DLSS-Manager
Copy-Item target\release\DLSS-Manager.exe ..\dist\DLSS-Manager.exe -Force
```

### Run Unit & Scanner Tests
```powershell
cd src-tauri
cargo test --release
```

### Build Windows Installers (`.msi` & NSIS `.exe`)
```powershell
.\build_rust_installer.ps1
```
*(or manually via `cd src-tauri && cargo tauri build`)*

---

## 💡 4. Windows 11 Smart App Control Tip for Local Builds

If Windows 11 **Smart App Control** is enabled on your development machine and `rustc` reports `error[E0463]: can't find crate for serde` (Win32 Error `4551`) when loading previously compiled proc-macro DLLs (`serde_derive-*.dll` or `tauri_macros-*.dll`) after a reboot, remove the cached `.dll` files in `target\release\deps` so `link.exe` relinks them with a fresh kernel trust attribute:

```powershell
cd src-tauri
Remove-Item "target\release\deps\*.dll" -Force
Remove-Item "target\release\.fingerprint\serde_derive-*", "target\release\.fingerprint\tauri-macros-*" -Recurse -Force
cargo build --release --bin DLSS-Manager
```
