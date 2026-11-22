# 📊 DLSS 4.5 / 5 Versionen, Presets (A–M) & Unterstützte Technologien

Diese Seite bietet eine vollständige technische Referenz aller **NVIDIA DLSS-Generationen**, **Super Resolution / Ray Reconstruction / Frame Generation Presets** sowie aller weiteren von **DLSS Manager** unterstützten Upscaler- und Streaming-Technologien.

---

## 1. NVIDIA DLSS Versionshistorie & Unterstützte Presets

Nicht jede `nvngx_dlss.dll`-Version enthält jedes Preset! Wenn du per NVIDIA Treiber-DRS ein Preset erzwingst, das die im Spielordner liegende DLL (bzw. das NVIDIA NGX OTA-Treiberupdate) noch nicht kennt, fällt DLSS automatisch auf das Standard-Preset zurück.

| DLL-Versionsbereich | Generation & Architektur | Super Resolution (SR) Presets | Ray Reconstruction (RR) | Frame Generation (FG) | Hinweise & Hardware-Unterstützung |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`v310.9.0 – 310.9.16+`** | **★ DLSS 4.5 (Transformer Gen 2)** | **Preset J, K ★, L, M** *(+ CNN A–E im 74,18 MB Voll-Build)* | **Preset D, F ★** | **Preset A, B** *(Dynamic MFG bis 6×)* | Vollständiges DLSS 4.5 Release mit neuen Profilen **Preset L** (4K Performance/Ultra-Perf) und **Preset M** (DLAA/Max Quality). SR/RR: ab RTX 20, FG: ab RTX 40, MFG 3×–6×: RTX 50. |
| **`v310.4.0 – 310.8.0`** | **DLSS 4.5 (Early Gen 2)** | **Preset J, K ★** *(Kein Preset L / M)* | **Preset D, F ★** | **Preset A, B** *(MFG 3× / 4×)* | Führte verbesserte Transformer Gen 2 Gewichte & RR Preset F ein. *(⚠️ `v310.7.x` ist ein fehlerhafter Dev-Build mit Wasserzeichen und wird blockiert).* |
| **`v310.2.1 – 310.3.0`** | **DLSS 4.0 (Transformer Gen 1)** | **Preset J, K** + **CNN A–E** *(Kein DLSS 4.5 · Kein L/M)* | **Preset D, E** *(Kein Preset F)* | **Preset A, B** *(MFG 3× / 4×)* | Erste DLSS 4.0 Releases (z. B. die Standard-DLL `v310.2.1.0` in Star Citizen). Unterstützt Transformer Gen 1 (`J`/`K`) & CNN (`E`), aber **ohne** NGX-OTA kein `L`/`M`. |
| **`v310.1.0`** | **DLSS 4.0 (Launch-Build)** | **Preset J** + **CNN A–E** *(Kein Preset K / L / M)* | **Preset D** | **Preset A** | Allererste DLSS 4.0 DLL: Enthält nur das erste Transformer-Modell (`Preset J`), noch **kein** `Preset K`. |
| **`v3.7.0 – 3.8.10`** | **DLSS 3.7 / 3.8 (CNN Final)** | **Preset A, B, C, D, E ★, F** *(Kein Transformer J–M)* | **CNN (A–C)** | **DLSS 3.x FG** *(nur 2×)* | Letzte reine CNN-Generation. Führte **Preset E** (bestes CNN-Profil) ein. Unterstützt keine Transformer-Presets (`J–M`). |
| **`v3.1.1 – 3.6.0`** | **DLSS 3.1 – 3.6 (CNN)** | **Preset A, B, C, D, F** *(Kein Preset E oder J–M)* | **CNN (ab 3.5+)** | **DLSS 3.x FG** *(Optical Flow)* | Einführung des Preset-Systems (`A–F`) ab `v3.1.1` und Ray Reconstruction ab `v3.5.0`. |
| **`v2.0.0 – 2.5.1`** | **DLSS 2.x (Legacy CNN)** | *Festes CNN-Modell (keine Presets)* | — | — | Klassisches DLSS 2 Super Resolution auf RTX 20/30/40/50. Kann durch DLL-Update direkt auf `v310.9.16.0` (DLSS 4.5) aktualisiert werden. |

### 💡 Warum bevorzugt DLSS Manager den 74,18 MB Build von `v310.9.16.0`?
NVIDIA veröffentlicht teilweise schlanke („Slim“) DLL-Varianten, in denen ältere CNN-Presets (`A–E`) entfernt wurden, sowie den vollständigen **74,18 MB Production-Build**, der **sowohl alle neuen Transformer-Gen-2-Presets (`J, K, L, M`) als auch alle klassischen CNN-Presets (`A–E`)** enthält. DLSS Manager bevorzugt immer den vollständigen Production-Build, um maximale Kompatibilität mit allen Spielen zu garantieren.

### ⚠️ Warum wird `v310.7.0` als „Wasserzeichen-Build“ gewarnt/blockiert?
Die geleakte Entwickler-Version `v310.7.0` blendet im Vollbild dauerhaft ein nicht abschaltbares NVIDIA-Confidential-Wasserzeichen ein. DLSS Manager erkennt diese Version automatisch, warnt mit einem `⚠️ Wasserzeichen-Build`-Badge und überspringt sie bei Updates zugunsten von `v310.9.16.0`.

---

## 2. Alle NVIDIA DLSS Presets im Detail erklärt

### A. DLSS Super Resolution (`nvngx_dlss.dll`)

| Preset | Architektur | Min. DLL-Version | Optimaler Modus | Empfehlung & Wirkung |
| :--- | :--- | :--- | :--- | :--- |
| **`Preset K ★`** | **Transformer Gen 2** | `v310.2.1+` *(Gen 2 ab `310.4+`)* | **Quality / Balanced / DLAA** | **★ Standard-Empfehlung für fast alle Spiele:** Schärfste feine Details (HUD, Schrift, entfernte Objekte, Schiffskanten) bei höchster temporaler Bildruhe und minimalem Schimmern. |
| **`Preset J`** | **Transformer Gen 1** | `v310.1.0+` | **Quality / Balanced** | Erstes Transformer-Modell aus DLSS 4.0: Extrem hohe Bewegungsschärfe und sehr wenig Ghosting, neigt auf feinen Gittern oder Zäunen aber etwas eher zu leichtem Schimmern als Preset K. |
| **`Preset L`** | **Transformer Gen 2 (DLSS 4.5)** | `v310.9.0+` | **Performance / Ultra-Perf. (4K)** | Speziell in DLSS 4.5 für starke Skalierungsfaktoren (`2×` / `3×` im Performance- & Ultra-Performance-Modus bei 4K/UHD) trainiertes Transformer-Profil. |
| **`Preset M`** | **Transformer Gen 2 (DLSS 4.5)** | `v310.9.0+` | **DLAA / Quality (RTX 40/50)** | Maximale Bildqualität und feinste Kantenglättung für DLAA & Quality in DLSS 4.5 (benötigt etwas mehr Tensor-Rechenleistung als Preset K). |
| **`Latest`** | **Vom Treiber gewählt** | `v310.1.0+` | **Automatisch** | Überlässt dem aktuellen NVIDIA-Grafiktreiber die automatische Wahl des neuesten empfohlenen Presets (aktuell meist identisch mit `Preset K`). |
| **`Preset E`** | **CNN (Final)** | `v3.7.0+` *(im 74 MB `310.9.16`)* | **Quality / Balanced / Perf.** | Bestes klassisches Convolutional-Neural-Network (CNN) Profil. Geringerer GPU-Aufwand auf älteren RTX-20-Karten und Standard-Fallback vieler Spiele ohne Override. |
| **`Preset A–D / F`** | **CNN (Legacy)** | `v3.1.1+` | **Legacy / Ultra-Perf. (F)** | Ältere CNN-Profile aus DLSS 3.1–3.6 (`C` = schnelle Bewegung, `D` = Bildruhe, `F` = Ultra-Performance). Heute durch `Preset K` und `Preset L` abgelöst. |

### B. DLSS Ray Reconstruction (`nvngx_dlssd.dll` / `sl.dlss_d.dll`)

| Preset | Architektur | Min. DLL-Version | Empfehlung & Wirkung |
| :--- | :--- | :--- | :--- |
| **`Preset F ★`** | **Transformer Gen 2** | `v310.4.0+` | **★ Empfohlen für alle Spiele mit Raytracing / Pathtracing** (z. B. *Cyberpunk 2077*, *Alan Wake 2*, *Indiana Jones*): Deutlich klarere Reflexionen, stabilere Schatten und viel weniger Ghosting an Drähten/Zäunen als Preset D. |
| **`Preset D`** | **Transformer Gen 1** | `v310.1.0+` | Erstes Transformer-Denoiser-Modell für Ray Reconstruction aus DLSS 4.0. |

### C. DLSS Frame Generation & Multi-Frame Generation (`nvngx_dlssg.dll` / `sl.dlss_g.dll`)

| Einstellung | Unterstützte GPUs | Beschreibung |
| :--- | :--- | :--- |
| **`Latest ★` / `Preset A`** | **RTX 40 & RTX 50** | Neuestes KI-Frame-Generation-Modell mit Hardware-Flip-Metering (~30 % geringerer VRAM-Bedarf und höhere Framerate als DLSS 3.x Optical-Flow-FG). |
| **`Preset B`** | **RTX 40 & RTX 50** | Alternatives Frame-Generation-Profil, falls bei `Preset A` in bestimmten Spielen leichte Artefakte an HUD-Elementen oder Fadenkreuzen auftreten. |
| **MFG-Modus (`Default` / `Fixed` / `Dynamic` / `Off`)** | **RTX 50 (Blackwell)** *(2× auch RTX 40)* | Steuert Multi-Frame Generation direkt im Treiber: **`Fixed`** erzwingt einen festen Multiplikator (`2×`, `3×`, `4×`, `5×`, `6×`), **`Dynamic`** passt den Multiplikator dynamisch an die eingestellten **Ziel-FPS** an. |

---

## 3. Alle unterstützten Upscaler- & Streaming-Technologien

DLSS Manager verwaltet **8 Technologiegruppen** aus offiziellen Upstream-Quellen:

| Technologie | Aktuelle Version | Verwaltete DLL-Dateien im Spielordner | Offizielle Upstream-Quelle | Unterstützte Grafikkarten |
| :--- | :--- | :--- | :--- | :--- |
| **DLSS Super Resolution (SR)** | `v310.9.16.0` | `nvngx_dlss.dll` | `NVIDIA/DLSS` + lokaler NVIDIA NGX OTA-Cache | NVIDIA GeForce **RTX 20 / 30 / 40 / 50** |
| **DLSS Ray Reconstruction (RR)** | `v310.9.16.0` | `nvngx_dlssd.dll` | `NVIDIA/DLSS` + Manifest | NVIDIA GeForce **RTX 20 / 30 / 40 / 50** |
| **DLSS Frame Generation (FG / MFG)** | `v310.9.16.0` | `nvngx_dlssg.dll` | `NVIDIA/DLSS` + Manifest | **RTX 40 & RTX 50** (2× FG) · **RTX 50** (3×–6× MFG) |
| **DLSS 5 Neural Rendering (NR)** | `v310.8.0.0` | `nvngx_dlssnr.dll` | Offizielles Manifest | NVIDIA GeForce **RTX 50 Serie (Blackwell)** |
| **NVIDIA Streamline SDK** | `v2.14.16.0` | `sl.interposer.dll`, `sl.common.dll`, `sl.dlss.dll`, `sl.dlss_g.dll`, `sl.dlss_d.dll`, `sl.reflex.dll`, `sl.pcl.dll`, `sl.directsr.dll`, `sl.nis.dll` | Offizielle `NVIDIA-RTX/Streamline` GitHub Releases | NVIDIA GeForce **RTX 20 / 30 / 40 / 50** (Reflex & PCL auch ab GTX 900) |
| **Intel XeSS 2 (SR / DX11)** | `v2.0.2.68` | `libxess.dll`, `libxess_dx11.dll` | Intel XeSS SDK / Manifest | **Alle GPUs** (XMX auf Intel Arc, Shader-DP4a auf NVIDIA & AMD) |
| **Intel XeSS-FG & XeLL** | `v2.0.2.68` | `libxess_fg.dll`, `libxell.dll` | Intel XeSS SDK / Manifest | Intel Arc GPUs (XeLL Low Latency) |
| **Microsoft DirectStorage** | `v1.2.2504+` | `dstorage.dll`, `dstoragecore.dll` | Offizielles Microsoft NuGet (`Microsoft.Direct3D.DirectStorage`) | **Alle DirectX-12-fähigen GPUs** (NVIDIA, AMD, Intel) + NVMe SSD |

> **Hinweis zu Spielen mit Streamline (`sl.dlss_g.dll` / `sl.dlss_d.dll`):** Viele moderne Spiele binden DLSS Frame Generation und Ray Reconstruction über das NVIDIA Streamline SDK ein. DLSS Manager erkennt `sl.dlss_g.dll` und `sl.dlss_d.dll` automatisch und schaltet für diese Spiele ebenfalls die **DLSS-G (FG / MFG)**- und **DLSS-RR**-Preset-Auswahl im Spieldetails-Modal frei!
