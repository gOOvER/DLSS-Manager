# 📊 DLSS 4.5 / 5 Versions, Presets (A–M) & Supported Technologies

This page provides a complete technical reference for all **NVIDIA DLSS generations**, **Super Resolution / Ray Reconstruction / Frame Generation presets**, and all other upscaler and streaming technologies managed by **DLSS Manager**.

---

## 1. NVIDIA DLSS Version History & Supported Presets

Not every `nvngx_dlss.dll` binary contains every preset! If you force a preset via NVIDIA Driver DRS that is not supported by the game's installed DLL (or the NVIDIA NGX OTA driver model cache), DLSS falls back to the game's default preset.

| DLL Version Range | Generation & Architecture | Super Resolution (SR) Presets | Ray Reconstruction (RR) | Frame Generation (FG) | Notes & Hardware Support |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`v310.9.0 – 310.9.16+`** | **★ DLSS 4.5 (Transformer Gen 2)** | **Preset J, K ★, L, M** *(+ CNN A–E in 74.18 MB Full Build)* | **Preset D, F ★** | **Preset A, B** *(Dynamic MFG up to 6×)* | Full DLSS 4.5 release introducing **Preset L** (4K Performance/Ultra-Perf) and **Preset M** (DLAA/Max Quality). SR/RR: RTX 20+, FG: RTX 40+, MFG 3×–6×: RTX 50. |
| **`v310.4.0 – 310.8.0`** | **DLSS 4.5 (Early Gen 2)** | **Preset J, K ★** *(No Preset L / M)* | **Preset D, F ★** | **Preset A, B** *(MFG 3× / 4×)* | Introduced refined Transformer Gen 2 weights & RR Preset F. *(⚠️ `v310.7.x` is a watermarked dev build and is automatically blocked).* |
| **`v310.2.1 – 310.3.0`** | **DLSS 4.0 (Transformer Gen 1)** | **Preset J, K** + **CNN A–E** *(No DLSS 4.5 · No L/M)* | **Preset D, E** *(No Preset F)* | **Preset A, B** *(MFG 3× / 4×)* | Initial DLSS 4.0 releases (e.g. stock `v310.2.1.0` in Star Citizen). Supports Transformer Gen 1 (`J`/`K`) & CNN (`E`), but **not** `L`/`M` without NGX OTA. |
| **`v310.1.0`** | **DLSS 4.0 (Launch Build)** | **Preset J** + **CNN A–E** *(No Preset K / L / M)* | **Preset D** | **Preset A** | First DLSS 4.0 binary: includes only the initial Transformer model (`Preset J`), no `Preset K` yet. |
| **`v3.7.0 – 3.8.10`** | **DLSS 3.7 / 3.8 (CNN Final)** | **Preset A, B, C, D, E ★, F** *(No Transformer J–M)* | **CNN (A–C)** | **DLSS 3.x FG** *(2× only)* | Final pure CNN generation. Introduced **Preset E** (best CNN profile). Does not support Transformer presets (`J–M`). |
| **`v3.1.1 – 3.6.0`** | **DLSS 3.1 – 3.6 (CNN)** | **Preset A, B, C, D, F** *(No Preset E or J–M)* | **CNN (from 3.5+)** | **DLSS 3.x FG** *(Optical Flow)* | Introduced the DLSS preset system (`A–F`) in `v3.1.1` and Ray Reconstruction in `v3.5.0`. |
| **`v2.0.0 – 2.5.1`** | **DLSS 2.x (Legacy CNN)** | *Fixed CNN model (no presets)* | — | — | Classic DLSS 2 Super Resolution on RTX 20/30/40/50. Can be upgraded directly to `v310.9.16.0` (DLSS 4.5). |

### 💡 Why Does DLSS Manager Prefer the 74.18 MB Build of `v310.9.16.0`?
NVIDIA distributes both slimmed-down DLL variants (which strip out legacy CNN presets `A–E`) and the full **74.18 MB production build**, which includes **all new Transformer Gen 2 presets (`J, K, L, M`) as well as all classic CNN presets (`A–E`)**. DLSS Manager always prefers the full production build to guarantee maximum compatibility across all games.

### ⚠️ Why Is `v310.7.0` Flagged/Blocked as a "Watermark Build"?
The leaked developer build `v310.7.0` displays a permanent "NVIDIA Confidential" watermark on screen during gameplay. DLSS Manager detects this build automatically, warns with a `⚠️ Watermark Build` badge, and skips it in favor of `v310.9.16.0`.

---

## 2. Detailed Breakdown of All NVIDIA DLSS Presets

### A. DLSS Super Resolution (`nvngx_dlss.dll`)

| Preset | Architecture | Min. DLL Version | Optimal Mode | Recommendation & Visual Characteristics |
| :--- | :--- | :--- | :--- | :--- |
| **`Preset K ★`** | **Transformer Gen 2** | `v310.2.1+` *(Gen 2 in `310.4+`)* | **Quality / Balanced / DLAA** | **★ Recommended default for almost all games:** Sharpest fine details (HUD, text, distant geometry, ship hulls) with highest temporal stability and minimal shimmering. |
| **`Preset J`** | **Transformer Gen 1** | `v310.1.0+` | **Quality / Balanced** | First Transformer model from DLSS 4.0: Very high motion clarity and minimal ghosting, but slightly more prone to shimmering on fine meshes/fences than Preset K. |
| **`Preset L`** | **Transformer Gen 2 (DLSS 4.5)** | `v310.9.0+` | **4K Performance / Ultra-Perf.** | Specifically trained in DLSS 4.5 for aggressive scaling ratios (`2×` / `3×` in 4K/UHD Performance & Ultra-Performance modes). |
| **`Preset M`** | **Transformer Gen 2 (DLSS 4.5)** | `v310.9.0+` | **DLAA / Quality (RTX 40/50)** | Maximum image quality and anti-aliasing stability for DLAA & Quality modes in DLSS 4.5 (slightly higher Tensor compute cost than Preset K). |
| **`Latest`** | **Driver Controlled** | `v310.1.0+` | **Automatic** | Lets the current NVIDIA graphics driver automatically select the newest recommended preset (typically resolves to `Preset K`). |
| **`Preset E`** | **CNN (Final)** | `v3.7.0+` *(included in 74 MB `310.9.16`)* | **Quality / Balanced / Perf.** | Best classic Convolutional Neural Network (CNN) profile. Lower GPU overhead on older RTX 20 cards and default fallback for many games without an override. |
| **`Preset A–D / F`** | **CNN (Legacy)** | `v3.1.1+` | **Legacy / Ultra-Perf. (F)** | Legacy CNN profiles from DLSS 3.1–3.6 (`C` = fast motion, `D` = stability, `F` = Ultra-Performance). Superseded today by Presets `K` and `L`. |

### B. DLSS Ray Reconstruction (`nvngx_dlssd.dll` / `sl.dlss_d.dll`)

| Preset | Architecture | Min. DLL Version | Recommendation & Visual Characteristics |
| :--- | :--- | :--- | :--- |
| **`Preset F ★`** | **Transformer Gen 2** | `v310.4.0+` | **★ Recommended for all Ray Tracing / Path Tracing games** (e.g. *Cyberpunk 2077*, *Alan Wake 2*, *Indiana Jones*): Much cleaner reflections, sharper shadows, and significantly reduced wire/fence ghosting compared to Preset D. |
| **`Preset D`** | **Transformer Gen 1** | `v310.1.0+` | First-generation Transformer denoiser model for Ray Reconstruction from DLSS 4.0. |

### C. DLSS Frame Generation & Multi-Frame Generation (`nvngx_dlssg.dll` / `sl.dlss_g.dll`)

| Setting | Supported GPUs | Description |
| :--- | :--- | :--- |
| **`Latest ★` / `Preset A`** | **RTX 40 & RTX 50** | Newest AI Frame Generation model with hardware flip metering (~30% lower VRAM usage and smoother frame pacing than DLSS 3.x Optical Flow FG). |
| **`Preset B`** | **RTX 40 & RTX 50** | Alternative Frame Generation profile if `Preset A` exhibits minor HUD or crosshair artifacts in specific titles. |
| **MFG Mode (`Default` / `Fixed` / `Dynamic` / `Off`)** | **RTX 50 (Blackwell)** *(2× also RTX 40)* | Controls Multi-Frame Generation directly in the driver: **`Fixed`** enforces a constant multiplier (`2×`, `3×`, `4×`, `5×`, `6×`), while **`Dynamic`** adapts the multiplier dynamically to maintain your configured **Target FPS**. |

---

## 3. All Supported Upscaler & Streaming Technologies

DLSS Manager manages **8 technology groups** from official vendor upstreams:

| Technology Group | Latest Version | DLL Files Managed in Game Folder | Official Upstream Source | Supported GPU Hardware |
| :--- | :--- | :--- | :--- | :--- |
| **DLSS Super Resolution (SR)** | `v310.9.16.0` | `nvngx_dlss.dll` | `NVIDIA/DLSS` + local NVIDIA NGX OTA cache | NVIDIA GeForce **RTX 20 / 30 / 40 / 50 Series** |
| **DLSS Ray Reconstruction (RR)** | `v310.9.16.0` | `nvngx_dlssd.dll` | `NVIDIA/DLSS` + Manifest | NVIDIA GeForce **RTX 20 / 30 / 40 / 50 Series** |
| **DLSS Frame Generation (FG / MFG)** | `v310.9.16.0` | `nvngx_dlssg.dll` | `NVIDIA/DLSS` + Manifest | **RTX 40 / 50** (2× FG) · **RTX 50 Blackwell** (3×–6× Multi Frame Gen) |
| **DLSS 5 Neural Rendering (NR)** | `v310.8.0.0` | `nvngx_dlssnr.dll` | Official Manifest | NVIDIA GeForce **RTX 50 Series (Blackwell)** |
| **NVIDIA Streamline SDK** | `v2.14.16.0` | `sl.interposer.dll`, `sl.common.dll`, `sl.dlss.dll`, `sl.dlss_g.dll`, `sl.dlss_d.dll`, `sl.reflex.dll`, `sl.pcl.dll`, `sl.directsr.dll`, `sl.nis.dll` | Official `NVIDIA-RTX/Streamline` GitHub Releases | NVIDIA GeForce **RTX 20 / 30 / 40 / 50 Series** (Reflex/PCL also GTX 900+) |
| **Intel XeSS 2 (SR / DX11)** | `v2.0.2.68` | `libxess.dll`, `libxess_dx11.dll` | Intel XeSS SDK / Manifest | **All GPUs** (XMX on Intel Arc, Shader DP4a on NVIDIA & AMD) |
| **Intel XeSS-FG & XeLL** | `v2.0.2.68` | `libxess_fg.dll`, `libxell.dll` | Intel XeSS SDK / Manifest | Intel Arc GPUs (XeLL Low Latency) |
| **Microsoft DirectStorage** | `v1.2.2504+` | `dstorage.dll`, `dstoragecore.dll` | Official Microsoft NuGet (`Microsoft.Direct3D.DirectStorage`) | **All DirectX 12 GPUs** (NVIDIA, AMD, Intel) + NVMe SSD |

> **Note on Streamline Games (`sl.dlss_g.dll` / `sl.dlss_d.dll`):** Many modern games integrate DLSS Frame Generation and Ray Reconstruction via the NVIDIA Streamline SDK. DLSS Manager automatically detects `sl.dlss_g.dll` and `sl.dlss_d.dll` and unlocks the per-game **DLSS-G (FG / MFG)** and **DLSS-RR** preset selectors in the Game Details modal!
