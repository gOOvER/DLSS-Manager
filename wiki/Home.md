# 📚 DLSS Manager Wiki

Welcome to the official **DLSS Manager** Wiki!  
Willkommen im offiziellen Wiki von **DLSS Manager** (`v1.0.0-beta.2+`) — dem nativen **Rust + Tauri 2** Desktop-Manager für **NVIDIA DLSS 4.5 / 5**, **NVIDIA Driver DRS Presets (`nvapi64.dll`)**, **NVIDIA Streamline SDK**, **Intel XeSS 2** und **Microsoft DirectStorage**.

---

## 🧭 Schnellnavigation / Quick Navigation

| Kapitel / Page | Inhalt / Contents |
| :--- | :--- |
| **[🚀 Installation & Erste Schritte](Installation-and-Quick-Start)** | Portable `.exe`, Installer, Winget, Administrator-Rechte (UAC), Speicherorte (`%LOCALAPPDATA%\gOOvER\DLSS-Manager`) & 1-Minuten-Schnellstart |
| **[📊 DLSS 4.5 / 5 Versionen, Presets A–M & Technologien](DLSS-Presets-and-Versions)** | Vollständige DLSS-Versionshistorie (`v2.0` bis `v310.9.16+`), Unterschied zwischen **Preset J, K, L, M, E, F**, DLSS-G Multi-Frame Generation (`2×–6×`) & alle unterstützten DLLs |
| **[🎛️ NVIDIA Treiber-DRS & NVIDIA App Whitelist](NVIDIA-Driver-DRS-and-NVIDIA-App-Whitelist)** | Wie `nvapi64.dll` Treiber-Presets setzt, warum die **NVIDIA App** Presets auf `0` zurücksetzt, wie der **1-Klick Whitelist-Unlocker** funktioniert & Live-Überprüfung per **DLSS-Overlay** |
| **[🎮 Unterstützte Launcher, Star Citizen & Anti-Cheat](Supported-Launchers-and-Anti-Cheat)** | Automatischer Laufwerks-Scan (`A:\`–`Z:\`) für **9 Launcher**, Spezial-Guide für **Star Citizen** (EasyAntiCheat & Treiber-DRS), Manifest-Verifizierung & Authenticode-Signaturprüfung |
| **[☁️ Online-Datenbank (`dlssdb.goover.dev`)](Online-Compatibility-Database)** | API-Key erstellen, Kompatibilitätsprofile synchronisieren, Auto-Discovery (`unverified` + SteamDB-Referenz) & Spiele mit 1 Klick an die Community-Datenbank melden |
| **[🛠️ Architektur & Build aus dem Quellcode](Architecture-and-Building-from-Source)** | Rust-Backend-Module (`scanner.rs`, `gpu_nvapi.rs`, `nvapp_whitelist.rs`, `pe_version.rs`, `repository.rs`), SQLite-Schema & Kompilieren unter Windows/Linux |
| **[❓ Fehlerbehebung & FAQ](Troubleshooting-and-FAQ)** | Häufige Fragen & Lösungen (Preset-Reset durch NVIDIA App, fehlende oder doppelte Spiele, Backup-Wiederherstellung, Wasserzeichen-Builds `310.7.0`, Anti-Cheat) |

---

## ✨ Wichtigste Funktionen auf einen Blick / Key Highlights

1. **🦀 Native Rust + Tauri 2 Engine:**
   - Startet sofort, benötigt minimalen RAM und liest Dateiversionen (`VS_FIXEDFILEINFO`) sowie digitale Signaturen (`WinVerifyTrust`) direkt aus dem Win32-PE-Header, ohne fremde DLLs in den Speicher zu laden.
2. **🟢 Offizielle Upstream-Quellen & 74,18 MB DLSS 4.5 Build:**
   - Lädt **DLSS 4.5 (`v310.9.16.0`)** ausschließlich aus offiziellen NVIDIA-Quellen (`NVIDIA/DLSS` & lokaler NVIDIA NGX OTA-Treiber-Cache), bevorzugt den vollständigen **74,18 MB Production-Build** (alle Presets A–M enthalten) und blockiert fehlerhafte Dev-Builds (`v310.7.*` mit Wasserzeichen).
   - Offizielle Upstreams für **NVIDIA Streamline SDK** (`NVIDIA-RTX/Streamline` GitHub Releases), **Microsoft DirectStorage** (offizielles Microsoft NuGet-Paket `Microsoft.Direct3D.DirectStorage`) und **Intel XeSS 2**.
3. **🎛️ Direkte NVIDIA Treiber-DRS-Steuerung (`nvapi64.dll`):**
   - Setzt **Super Resolution (`Preset K / J / L / M / E`)**, **Ray Reconstruction (`Preset F / D`)** und **Frame Generation (`Latest / A / B` + RTX 50 MFG `Fixed 2×–6×` / `Dynamic` + Ziel-FPS)** global und pro Spiel direkt im NVIDIA-Treiber.
   - Liest den echten Live-Treiberstatus (**`Treiber-Live: Preset K`**) direkt aus dem NVIDIA-Treiber zurück.
4. **🔓 1-Klick NVIDIA App Override-Whitelist:**
   - Verhindert, dass die **NVIDIA App** Treiber-Presets bei nicht-gewhitelisteten Spielen (z. B. **Star Citizen**) heimlich wieder auf `0` zurücksetzt, indem `Disable_*_Override`-Sperren in `ApplicationStorage.json`, `fingerprint.db` und dem Spiel-DRS-Profil automatisch bereinigt werden.
5. **🛡️ Automatische Backups & Anti-Cheat-Schutz:**
   - Vor jedem DLL-Tausch wird die Originaldatei im lokalen Backup-Tresor gesichert und kann jederzeit pro Technologie oder für das gesamte Spiel wiederhergestellt werden.
   - Erkennt **EasyAntiCheat**, **BattlEye**, **Vanguard** und **Ricochet** automatisch und schützt EAC-überwachte Spiele (wie Star Citizen) durch den **`Nur Treiber-DRS`**-Modus.
6. **🇩🇪 / 🇬🇧 Komplett zweisprachig (Deutsch & Englisch):**
   - Nahtloser 1-Klick-Sprachwechsel in der oberen Leiste mit gestochen scharfen SVG-Flaggen und vollständig übersetzten Beschreibungen.

---

## 🔗 Wichtige Links / Important Links

- **GitHub Repository:** [https://github.com/gOOvER/DLSS-Manager](https://github.com/gOOvER/DLSS-Manager)
- **Neueste Releases (Download):** [https://github.com/gOOvER/DLSS-Manager/releases/latest](https://github.com/gOOvER/DLSS-Manager/releases/latest)
- **Online-Kompatibilitäts-Datenbank:** [https://dlssdb.goover.dev](https://dlssdb.goover.dev)
- **Fehler melden / Feature Request:** [https://github.com/gOOvER/DLSS-Manager/issues](https://github.com/gOOvER/DLSS-Manager/issues)
- **Support & Spenden:** [https://donate.goover.dev](https://donate.goover.dev)
