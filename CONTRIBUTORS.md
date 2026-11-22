# Contributors

Thank you to everyone who has contributed to **DLSS Manager** and its upstream foundation!

---

## 👨‍💻 Core Team & Maintainers

| Contributor | Role | Contributions |
| :--- | :--- | :--- |
| **[gOOvER](https://github.com/gOOvER)** | **Lead Developer & Maintainer** (`DLSS Manager` `v1.0.0-beta.1+`) | Complete **Rust + Tauri 2** rewrite, Direct `nvapi64.dll` NVIDIA Driver DRS Preset Engine (Global & Per-Game SR/RR/FG overrides), Official NVIDIA Upstream & Local NGX Model Cache integration, Star Citizen RSI Launcher (`LIVE`/`PTU`/`EPTU`) & Battle.net `product.db` scanners, Steam Web API `IStoreBrowseService` SHA-1 artwork resolver, Cloudflare Pages + D1 Online Compatibility Database ([`dlssdb.goover.dev`](https://dlssdb.goover.dev)), Bilingual UI (`🇩🇪 DE` / `🇬🇧 EN`), Tabular Version & Preset Reference Matrix, CI/CD & Packaging. |

---

## 🌱 Upstream Foundation & Original Author

| Contributor | Role | Contributions |
| :--- | :--- | :--- |
| **[Recol (Deco)](https://github.com/Recol)** | **Original Author** ([`DLSS Updater`](https://github.com/Recol/DLSS-Updater) `<= v5.2.0`) | Created and maintained the original Python / Flet *DLSS Updater* application, multi-launcher concept, and initial Linux / Proton support upon which *DLSS Manager* was forked and re-architected. |

---

## 🤝 Community & Upstream Contributors

Special thanks to the open-source projects and community efforts that make **DLSS Manager** possible:

- **[NVIDIA](https://github.com/NVIDIA/DLSS)** & **[NVIDIA Streamline](https://github.com/NVIDIAGameWorks/Streamline)** — Official DLSS Super Resolution, Ray Reconstruction, Frame Generation, and Streamline SDK binaries.
- **[AMD GPUOpen](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK)** — Official FidelityFX Super Resolution (FSR 3.1 / FSR 4) SDK binaries.
- **[Intel](https://github.com/intel/xess)** — Official Intel XeSS 2, XeSS Frame Generation, and Xe Low Latency (XeLL) binaries.
- **[Microsoft](https://www.nuget.org/packages/Microsoft.Direct3D.DirectStorage)** — Official DirectStorage (`dstorage.dll` & `dstoragecore.dll`) runtime binaries.
- **[Brad Moore (`beeradmoore`)](https://github.com/beeradmoore/dlss-swapper-manifest-builder)** — Official `dlss-swapper-manifest-builder` stable manifest feed.
- **All Community Reporters on [`dlssdb.goover.dev`](https://dlssdb.goover.dev)** — For submitting game compatibility reports, anti-cheat behavior notes, and optimal DLSS preset recommendations.

---

## 💡 How to Contribute

We welcome bug reports, game compatibility reports, feature requests, and pull requests!

1. **Report Game Compatibility:** Use the built-in **📤 Report Game to Online-DB** button directly inside **DLSS Manager** (or visit [`https://dlssdb.goover.dev`](https://dlssdb.goover.dev)).
2. **Report Bugs & Feature Requests:** Open an issue on [GitHub Issues](https://github.com/gOOvER/DLSS-Manager/issues).
3. **Submit Pull Requests:**
   - Fork the repository and create a feature/fix branch from `main`.
   - Document your changes under `## [Unreleased]` in [`CHANGELOG.md`](CHANGELOG.md) following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
   - Keep [`README.md`](README.md) in sync with any new features or options.
