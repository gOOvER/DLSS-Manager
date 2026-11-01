use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherType {
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    Battlenet,
    Xbox,
    Rsi,
    Custom,
}

impl LauncherType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Epic => "Epic Games",
            Self::Gog => "GOG Galaxy",
            Self::Ubisoft => "Ubisoft Connect",
            Self::Ea => "EA App",
            Self::Battlenet => "Battle.net",
            Self::Xbox => "Xbox",
            Self::Rsi => "Star Citizen",
            Self::Custom => "Custom Folder",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Steam => "steam.png",
            Self::Epic => "epic.png",
            Self::Gog => "gog.png",
            Self::Ubisoft => "ubisoft.png",
            Self::Ea => "ea.png",
            Self::Battlenet => "battlenet.png",
            Self::Xbox => "xbox.png",
            Self::Rsi => "starcitizen.png",
            Self::Custom => "dlss_manager.png",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DllType {
    Dlss,
    DlssG,
    DlssD,
    DlssNr,
    Fsr,
    Fsr3,
    FsrVk,
    Xess,
    XessFg,
    Xell,
    DirectStorage,
    Streamline,
}

impl DllType {
    pub fn from_filename(filename: &str) -> Option<Self> {
        match filename.to_ascii_lowercase().as_str() {
            "nvngx_dlss.dll" => Some(Self::Dlss),
            "nvngx_dlssg.dll" => Some(Self::DlssG),
            "nvngx_dlssd.dll" => Some(Self::DlssD),
            "nvngx_dlssnr.dll" => Some(Self::DlssNr),
            "amd_fidelityfx_dx12.dll"
            | "amd_fidelityfx_loader_dx12.dll"
            | "amd_fidelityfx_upscaler_dx12.dll"
            | "amd_fidelityfx_framegeneration_dx12.dll"
            | "amd_fidelityfx_denoiser_dx12.dll"
            | "amd_fidelityfx_radiancecache_dx12.dll" => Some(Self::Fsr3),
            "amd_fidelityfx_vk.dll"
            | "amd_fidelityfx_loader_vk.dll"
            | "amd_fidelityfx_upscaler_vk.dll"
            | "amd_fidelityfx_framegeneration_vk.dll" => Some(Self::FsrVk),
            "ffx_fsr2_api_x64.dll"
            | "ffx_fsr2_api_dx12_x64.dll"
            | "ffx_fsr2_api_vk_x64.dll" => Some(Self::Fsr),
            "libxess.dll" | "libxess_dx11.dll" => Some(Self::Xess),
            "libxess_fg.dll" => Some(Self::XessFg),
            "libxell.dll" => Some(Self::Xell),
            "dstorage.dll" | "dstoragecore.dll" => Some(Self::DirectStorage),
            "sl.interposer.dll"
            | "sl.common.dll"
            | "sl.dlss.dll"
            | "sl.dlss_g.dll"
            | "sl.dlss_d.dll"
            | "sl.reflex.dll"
            | "sl.pcl.dll"
            | "sl.directsr.dll"
            | "sl.nis.dll" => Some(Self::Streamline),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dlss => "DLSS Super Resolution",
            Self::DlssG => "DLSS Frame Generation",
            Self::DlssD => "DLSS Ray Reconstruction",
            Self::DlssNr => "DLSS 5 Neural Rendering",
            Self::Fsr => "AMD FSR 2",
            Self::Fsr3 => "AMD FSR 3 / 4 (DX12)",
            Self::FsrVk => "AMD FSR 3 / 4 (Vulkan)",
            Self::Xess => "Intel XeSS",
            Self::XessFg => "Intel XeSS Frame Gen",
            Self::Xell => "Intel XeLL",
            Self::DirectStorage => "DirectStorage",
            Self::Streamline => "NVIDIA Streamline",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Dlss => "DLSS",
            Self::DlssG => "DLSS-G",
            Self::DlssD => "DLSS-RR",
            Self::DlssNr => "DLSS 5",
            Self::Fsr => "FSR 2",
            Self::Fsr3 => "FSR 3/4",
            Self::FsrVk => "FSR VK",
            Self::Xess => "XeSS",
            Self::XessFg => "XeSS-FG",
            Self::Xell => "XeLL",
            Self::DirectStorage => "DStorage",
            Self::Streamline => "Streamline",
        }
    }

    pub fn canonical_filename(&self) -> &'static str {
        match self {
            Self::Dlss => "nvngx_dlss.dll",
            Self::DlssG => "nvngx_dlssg.dll",
            Self::DlssD => "nvngx_dlssd.dll",
            Self::DlssNr => "nvngx_dlssnr.dll",
            Self::Fsr => "ffx_fsr2_api_x64.dll",
            Self::Fsr3 => "amd_fidelityfx_dx12.dll",
            Self::FsrVk => "amd_fidelityfx_vk.dll",
            Self::Xess => "libxess.dll",
            Self::XessFg => "libxess_fg.dll",
            Self::Xell => "libxell.dll",
            Self::DirectStorage => "dstorage.dll",
            Self::Streamline => "sl.interposer.dll",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedDll {
    pub path: String,
    pub filename: String,
    pub dll_type: DllType,
    pub type_label: String,
    pub version: String,
    pub file_size: u64,
    pub preset_label: Option<String>,
    pub update_available: bool,
    pub latest_version: Option<String>,
    #[serde(default = "default_true")]
    pub gpu_supported: bool,
    #[serde(default = "default_true")]
    pub technology_enabled: bool,
    #[serde(default)]
    pub bad_build_warning: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_preset() -> String {
    "Default".to_string()
}

pub fn default_enabled_techs() -> Vec<String> {
    vec![
        "DLSS".to_string(),
        "DLSS-G".to_string(),
        "DLSS-RR".to_string(),
        "DLSS 5".to_string(),
        "Streamline".to_string(),
        "DStorage".to_string(),
        "FSR 3/4".to_string(),
        "FSR VK".to_string(),
        "FSR 2".to_string(),
        "XeSS".to_string(),
        "XeSS-FG".to_string(),
        "XeLL".to_string(),
    ]
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameOverride {
    pub game_id: String,
    pub custom_name: Option<String>,
    pub custom_app_id: Option<String>,
    pub custom_cover_url: Option<String>,
    pub custom_exe_path: Option<String>,
    pub dlss_sr_preset: Option<String>,
    pub dlss_rr_preset: Option<String>,
    pub dlss_fg_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedTechNote {
    pub tech: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfileEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub steam_app_ids: Vec<String>,
    #[serde(default)]
    pub match_names: Vec<String>,
    #[serde(default)]
    pub match_executables: Vec<String>,
    /// "safe" | "signed_only" | "driver_override_only" | "blocked"
    #[serde(default = "default_safe_policy")]
    pub anticheat_status: String,
    #[serde(default)]
    pub supported_techs: Vec<String>,
    #[serde(default)]
    pub unsupported_techs: Vec<UnsupportedTechNote>,
    #[serde(default)]
    pub warning_banner: Option<String>,
    #[serde(default)]
    pub recommended_sr_preset: Option<String>,
    #[serde(default)]
    pub recommended_rr_preset: Option<String>,
    #[serde(default)]
    pub recommended_fg_preset: Option<String>,
    #[serde(default)]
    pub recommended_settings: Vec<String>,
}

fn default_safe_policy() -> String {
    "safe".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDbManifest {
    pub schema_version: u32,
    pub updated_at: String,
    pub games: Vec<GameProfileEntry>,
}

pub fn default_game_db_url() -> String {
    "https://dlssdb.goover.dev/api/v1/games.json".to_string()
}

pub fn default_language() -> String {
    "de".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub path: String,
    pub launcher: LauncherType,
    pub launcher_name: String,
    pub launcher_icon: String,
    pub app_id: Option<String>,
    pub cover_url: Option<String>,
    pub dlls: Vec<DetectedDll>,
    pub anti_cheat: Option<String>,
    pub ignored: bool,
    pub blacklisted: bool,
    pub has_backups: bool,
    pub last_scanned: String,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub custom_exe_path: Option<String>,
    #[serde(default)]
    pub dlss_sr_preset: Option<String>,
    #[serde(default)]
    pub dlss_rr_preset: Option<String>,
    #[serde(default)]
    pub dlss_fg_preset: Option<String>,
    #[serde(default)]
    pub db_profile: Option<GameProfileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDllVersion {
    pub dll_type: DllType,
    pub type_label: String,
    pub filename: String,
    pub version: String,
    pub download_url: String,
    pub sha256: Option<String>,
    pub file_size: Option<u64>,
    pub release_date: Option<String>,
    pub is_preview: bool,
    pub is_cached: bool,
    pub preset_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: String,
    pub game_id: String,
    pub game_name: String,
    pub game_path: String,
    pub dll_type: DllType,
    pub filename: String,
    pub original_version: String,
    pub replaced_with_version: String,
    pub backup_file_path: String,
    pub original_dll_path: String,
    pub created_at: String,
    pub file_size: u64,
    #[serde(default)]
    pub added_by_updater: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub name: String,
    pub architecture: String,
    pub driver_version: String,
    pub supports_dlss: bool,
    pub supports_dlss_fg: bool,
    pub supports_dlss_rr: bool,
    #[serde(default)]
    pub supports_dlss_nr: bool,
    pub supports_fsr4: bool,
    pub supports_xess: bool,
    pub recommended_preset: String,
}

impl GpuInfo {
    /// Returns true if this GPU vendor and architecture supports updating the given `DllType`.
    /// Prevents offering incompatible updates (e.g. DLSS/Streamline on AMD/Intel, or FSR/XeSS on NVIDIA RTX,
    /// DLSS Frame Generation on pre-RTX 40 GPUs, or DLSS 5 Neural Rendering on pre-RTX 50 GPUs).
    pub fn supports_dll_type(&self, dll_type: DllType) -> bool {
        let vendor_upper = self.vendor.to_ascii_uppercase();
        match dll_type {
            DllType::Dlss => self.supports_dlss,
            DllType::DlssG => self.supports_dlss_fg,
            DllType::DlssD => self.supports_dlss_rr,
            DllType::DlssNr => self.supports_dlss_nr,
            DllType::Streamline => vendor_upper == "NVIDIA" && self.supports_dlss,
            DllType::Fsr | DllType::Fsr3 | DllType::FsrVk => {
                vendor_upper == "AMD" || (!self.supports_dlss && vendor_upper != "INTEL")
            }
            DllType::Xess | DllType::XessFg | DllType::Xell => {
                vendor_upper == "INTEL" || (!self.supports_dlss && vendor_upper != "AMD")
            }
            DllType::DirectStorage => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub custom_folders: Vec<String>,
    pub ignored_games: Vec<String>,
    pub blacklisted_games: Vec<String>,
    pub allow_preview_dlls: bool,
    pub auto_backup: bool,
    pub skip_anti_cheat_games: bool,
    pub theme_mode: String,
    pub grid_density: String,
    pub steam_api_key: String,
    #[serde(default)]
    pub steam_id: String,
    pub dlss_indicator_enabled: bool,
    pub global_dlss_preset: String,
    #[serde(default = "default_preset")]
    pub global_dlss_rr_preset: String,
    #[serde(default = "default_preset")]
    pub global_dlss_fg_preset: String,
    #[serde(default)]
    pub hide_ignored_games: bool,
    #[serde(default = "default_enabled_techs")]
    pub enabled_technologies: Vec<String>,
    #[serde(default = "default_true")]
    pub check_app_updates_on_startup: bool,
    #[serde(default = "default_game_db_url")]
    pub game_db_url: String,
    #[serde(default)]
    pub game_db_api_key: String,
    #[serde(default = "default_true")]
    pub auto_report_games_to_db: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_folders: Vec::new(),
            ignored_games: Vec::new(),
            blacklisted_games: Vec::new(),
            allow_preview_dlls: false,
            auto_backup: true,
            skip_anti_cheat_games: false,
            theme_mode: "dark".to_string(),
            grid_density: "comfortable".to_string(),
            steam_api_key: String::new(),
            steam_id: String::new(),
            dlss_indicator_enabled: false,
            global_dlss_preset: "Default".to_string(),
            global_dlss_rr_preset: "Default".to_string(),
            global_dlss_fg_preset: "Default".to_string(),
            hide_ignored_games: false,
            enabled_technologies: default_enabled_techs(),
            check_app_updates_on_startup: true,
            game_db_url: default_game_db_url(),
            game_db_api_key: String::new(),
            auto_report_games_to_db: true,
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_games: usize,
    pub games_with_dlls: usize,
    pub updates_available: usize,
    pub ignored_count: usize,
    pub total_backups: usize,
    pub cached_dlls: usize,
    pub cache_size_mb: f64,
    pub backup_size_mb: f64,
    pub last_scan_time: Option<String>,
    pub app_data_dir: String,
    #[serde(default)]
    pub game_db_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamValidationResult {
    pub valid: bool,
    pub steam_id: Option<String>,
    pub owned_games_count: usize,
    pub games_to_improve: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamSearchResult {
    pub app_id: String,
    pub name: String,
    pub hero_url: String,
    pub header_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}
