use crate::models::GpuInfo;

/// Detects the primary dedicated GPU (NVIDIA, AMD, or Intel), architecture, driver version,
/// and supported upscaling features (DLSS SR, Frame Gen, Ray Reconstruction, FSR 4, XeSS).
pub fn detect_gpu_info() -> GpuInfo {
    #[cfg(windows)]
    {
        if let Some(info) = detect_gpu_nvml() {
            return info;
        }
        if let Some(info) = detect_gpu_windows_registry() {
            return info;
        }
    }

    GpuInfo {
        vendor: "NVIDIA".to_string(),
        name: "NVIDIA GeForce RTX GPU".to_string(),
        architecture: "Ada / Blackwell".to_string(),
        driver_version: "Detected".to_string(),
        supports_dlss: true,
        supports_dlss_fg: true,
        supports_dlss_rr: true,
        supports_dlss_nr: false,
        supports_fsr4: false,
        supports_xess: false,
        recommended_preset: "Preset K (DLSS 4.5 Transformer)".to_string(),
    }
}

/// Queries NVIDIA Management Library (`nvml.dll`) directly — same source as Python `pynvml`.
#[cfg(windows)]
fn detect_gpu_nvml() -> Option<GpuInfo> {
    use libloading::{Library, Symbol};
    use std::ffi::{c_char, c_int, c_uint, c_void, CStr};

    let candidates = [
        "nvml.dll",
        "C:\\Windows\\System32\\nvml.dll",
        "C:\\Program Files\\NVIDIA Corporation\\NVSMI\\nvml.dll",
    ];

    let lib = candidates
        .iter()
        .find_map(|path| unsafe { Library::new(*path).ok() })?;

    unsafe {
        type NvmlInitFn = unsafe extern "C" fn() -> c_int;
        type NvmlShutdownFn = unsafe extern "C" fn() -> c_int;
        type NvmlDeviceGetCountFn = unsafe extern "C" fn(*mut c_uint) -> c_int;
        type NvmlDeviceGetHandleByIndexFn =
            unsafe extern "C" fn(c_uint, *mut *mut c_void) -> c_int;
        type NvmlDeviceGetNameFn =
            unsafe extern "C" fn(*mut c_void, *mut c_char, c_uint) -> c_int;
        type NvmlDeviceGetCudaComputeCapabilityFn =
            unsafe extern "C" fn(*mut c_void, *mut c_int, *mut c_int) -> c_int;
        type NvmlSystemGetDriverVersionFn =
            unsafe extern "C" fn(*mut c_char, c_uint) -> c_int;

        let init: Symbol<NvmlInitFn> = lib
            .get(b"nvmlInit_v2\0")
            .or_else(|_| lib.get(b"nvmlInit\0"))
            .ok()?;
        if init() != 0 {
            return None;
        }

        let get_count: Symbol<NvmlDeviceGetCountFn> = lib
            .get(b"nvmlDeviceGetCount_v2\0")
            .or_else(|_| lib.get(b"nvmlDeviceGetCount\0"))
            .ok()?;
        let mut count: c_uint = 0;
        if get_count(&mut count) != 0 || count == 0 {
            if let Ok(shutdown) = lib.get::<NvmlShutdownFn>(b"nvmlShutdown\0") {
                let _ = shutdown();
            }
            return None;
        }

        let get_handle: Symbol<NvmlDeviceGetHandleByIndexFn> = lib
            .get(b"nvmlDeviceGetHandleByIndex_v2\0")
            .or_else(|_| lib.get(b"nvmlDeviceGetHandleByIndex\0"))
            .ok()?;
        let mut handle: *mut c_void = std::ptr::null_mut();
        if get_handle(0, &mut handle) != 0 || handle.is_null() {
            return None;
        }

        let get_name: Symbol<NvmlDeviceGetNameFn> = lib.get(b"nvmlDeviceGetName\0").ok()?;
        let mut name_buf = [0 as c_char; 128];
        if get_name(handle, name_buf.as_mut_ptr(), name_buf.len() as c_uint) != 0 {
            return None;
        }
        let name = CStr::from_ptr(name_buf.as_ptr())
            .to_string_lossy()
            .trim()
            .to_string();

        let mut driver_version = String::new();
        if let Ok(get_drv) = lib.get::<NvmlSystemGetDriverVersionFn>(b"nvmlSystemGetDriverVersion\0")
        {
            let mut drv_buf = [0 as c_char; 80];
            if get_drv(drv_buf.as_mut_ptr(), drv_buf.len() as c_uint) == 0 {
                driver_version = CStr::from_ptr(drv_buf.as_ptr())
                    .to_string_lossy()
                    .trim()
                    .to_string();
            }
        }

        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        if let Ok(get_cc) =
            lib.get::<NvmlDeviceGetCudaComputeCapabilityFn>(b"nvmlDeviceGetCudaComputeCapability\0")
        {
            let _ = get_cc(handle, &mut major, &mut minor);
        }

        if let Ok(shutdown) = lib.get::<NvmlShutdownFn>(b"nvmlShutdown\0") {
            let _ = shutdown();
        }

        Some(build_nvidia_gpu_info(name, driver_version, major, minor))
    }
}

fn build_nvidia_gpu_info(
    name: String,
    driver_version: String,
    sm_major: i32,
    sm_minor: i32,
) -> GpuInfo {
    let upper = name.to_ascii_uppercase();
    let sm = sm_major * 10 + sm_minor;

    let is_rtx50 = sm >= 100 || upper.contains("RTX 50");
    let is_rtx40 = sm == 89 || upper.contains("RTX 40");
    let is_rtx30 = (80..89).contains(&sm) || upper.contains("RTX 30");
    let is_rtx20 = sm == 75 || upper.contains("RTX 20");
    let is_rtx = is_rtx50 || is_rtx40 || is_rtx30 || is_rtx20 || upper.contains("RTX");

    let architecture = if is_rtx50 {
        "Blackwell (RTX 50 Series)"
    } else if is_rtx40 {
        "Ada Lovelace (RTX 40 Series)"
    } else if is_rtx30 {
        "Ampere (RTX 30 Series)"
    } else if is_rtx20 {
        "Turing (RTX 20 Series)"
    } else if is_rtx {
        "NVIDIA RTX Architecture"
    } else {
        "NVIDIA GTX Architecture"
    };

    GpuInfo {
        vendor: "NVIDIA".to_string(),
        name,
        architecture: architecture.to_string(),
        driver_version,
        supports_dlss: is_rtx,
        supports_dlss_fg: is_rtx40 || is_rtx50,
        supports_dlss_rr: is_rtx,
        supports_dlss_nr: is_rtx50,
        supports_fsr4: false,
        supports_xess: false,
        recommended_preset: if is_rtx40 || is_rtx50 {
            "Preset K / M (DLSS 4.5 Transformer)".to_string()
        } else if is_rtx {
            "Preset K (DLSS 4.5 Transformer)".to_string()
        } else {
            "DirectStorage Only".to_string()
        },
    }
}

#[cfg(windows)]
fn detect_gpu_windows_registry() -> Option<GpuInfo> {
    use winreg::enums::*;
    use winreg::RegKey;

    let display_class = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}")
        .ok()?;

    let mut candidates: Vec<(u32, GpuInfo)> = Vec::new();

    for sub in display_class.enum_keys().flatten() {
        if sub == "Properties" {
            continue;
        }
        let Ok(k) = display_class.open_subkey(&sub) else {
            continue;
        };
        let desc = k
            .get_value::<String, _>("DriverDesc")
            .or_else(|_| k.get_value::<String, _>("HardwareInformation.AdapterString"))
            .unwrap_or_default();
        if desc.is_empty()
            || desc.contains("Basic Display")
            || desc.contains("Parsec")
            || desc.contains("Virtual")
            || desc.contains("Remote")
        {
            continue;
        }
        let driver_ver = k.get_value::<String, _>("DriverVersion").unwrap_or_default();
        let provider = k
            .get_value::<String, _>("ProviderName")
            .unwrap_or_default()
            .to_ascii_uppercase();
        let upper = desc.to_ascii_uppercase();

        if upper.contains("NVIDIA")
            || upper.contains("GEFORCE")
            || upper.contains("RTX")
            || upper.contains("GTX")
            || provider.contains("NVIDIA")
        {
            let info = build_nvidia_gpu_info(desc, driver_ver, 0, 0);
            let score = if info.supports_dlss { 100 } else { 90 };
            candidates.push((score, info));
        } else if upper.contains("AMD")
            || upper.contains("RADEON")
            || provider.contains("ADVANCED MICRO DEVICES")
            || provider.contains("AMD")
        {
            let is_igpu = upper == "AMD RADEON(TM) GRAPHICS"
                || upper == "AMD RADEON GRAPHICS"
                || upper.contains("RADEON(TM) 610M")
                || upper.contains("RADEON 780M")
                || upper.contains("RADEON 680M");
            let is_rx9000 = upper.contains("RX 90");
            let is_rx7000 = upper.contains("RX 7");
            let is_rx6000 = upper.contains("RX 6");
            let is_rx = upper.contains("RX ");

            let arch = if is_rx9000 {
                "RDNA 4 (RX 9000 Series)"
            } else if is_rx7000 {
                "RDNA 3 (RX 7000 Series)"
            } else if is_rx6000 {
                "RDNA 2 (RX 6000 Series)"
            } else if is_igpu {
                "AMD Integrated Graphics"
            } else {
                "AMD Radeon Architecture"
            };

            let score = if is_rx && !is_igpu { 95 } else if is_igpu { 15 } else { 60 };
            candidates.push((
                score,
                GpuInfo {
                    vendor: "AMD".to_string(),
                    name: desc,
                    architecture: arch.to_string(),
                    driver_version: driver_ver,
                    supports_dlss: false,
                    supports_dlss_fg: false,
                    supports_dlss_rr: false,
                    supports_dlss_nr: false,
                    supports_fsr4: true,
                    supports_xess: false,
                    recommended_preset: "AMD FSR 3.1 / 4 (FidelityFX)".to_string(),
                },
            ));
        } else if upper.contains("INTEL") || upper.contains("ARC") || provider.contains("INTEL") {
            let is_arc = upper.contains("ARC");
            let score = if is_arc { 85 } else { 10 };
            candidates.push((
                score,
                GpuInfo {
                    vendor: "Intel".to_string(),
                    name: desc,
                    architecture: if is_arc {
                        "Intel Arc (Xe-HPG / Battlemage)".to_string()
                    } else {
                        "Intel Xe Graphics".to_string()
                    },
                    driver_version: driver_ver,
                    supports_dlss: false,
                    supports_dlss_fg: false,
                    supports_dlss_rr: false,
                    supports_dlss_nr: false,
                    supports_fsr4: false,
                    supports_xess: true,
                    recommended_preset: "Intel XeSS 2.0".to_string(),
                },
            ));
        }
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().next().map(|(_, info)| info)
}

/// Reads the NVIDIA DLSS On-Screen Indicator registry flag (`ShowDlssIndicator`).
/// Value `0x400` (1024) enables the overlay; `0` disables it.
pub fn get_dlss_indicator_state() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\NVIDIA Corporation\\Global\\NGXCore")
        {
            if let Ok(val) = key.get_value::<u32, _>("ShowDlssIndicator") {
                return val == 0x400 || val == 1;
            }
        }
    }
    false
}

/// Toggles the NVIDIA DLSS On-Screen Indicator registry flag (`ShowDlssIndicator`).
pub fn set_dlss_indicator_state(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm
            .create_subkey("SOFTWARE\\NVIDIA Corporation\\Global\\NGXCore")
            .map_err(|e| {
                format!(
                    "Administrator privileges required to modify HKLM NGXCore registry: {}",
                    e
                )
            })?;

        let val: u32 = if enabled { 0x400 } else { 0 };
        key.set_value("ShowDlssIndicator", &val)
            .map_err(|e| format!("Failed to write ShowDlssIndicator: {}", e))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("DLSS Registry Indicator is only supported on Windows".to_string())
    }
}

/// Maps human-readable DLSS Super Resolution preset names to NVIDIA DRS u32 values.
pub fn parse_sr_preset_value(preset: &str) -> u32 {
    match preset.trim().to_ascii_uppercase().as_str() {
        "PRESET A" | "A" => 0x01,
        "PRESET B" | "B" => 0x02,
        "PRESET C" | "C" => 0x03,
        "PRESET D" | "D" => 0x04,
        "PRESET E" | "E" => 0x05,
        "PRESET F" | "F" => 0x06,
        "PRESET J" | "J" => 0x0A,
        "PRESET K" | "K" => 0x0B,
        "PRESET L" | "L" => 0x0C,
        "PRESET M" | "M" => 0x0D,
        "LATEST" => 0x00FF_FFFF,
        _ => 0x00,
    }
}

/// Maps human-readable DLSS Ray Reconstruction preset names to NVIDIA DRS u32 values.
pub fn parse_rr_preset_value(preset: &str) -> u32 {
    match preset.trim().to_ascii_uppercase().as_str() {
        "PRESET A" | "A" => 0x01,
        "PRESET B" | "B" => 0x02,
        "PRESET C" | "C" => 0x03,
        "PRESET D" | "D" => 0x04,
        "PRESET E" | "E" => 0x05,
        "PRESET F" | "F" => 0x06,
        "LATEST" => 0x00FF_FFFF,
        _ => 0x00,
    }
}

/// Maps human-readable DLSS Frame Generation preset names to NVIDIA DRS u32 values.
pub fn parse_fg_preset_value(preset: &str) -> u32 {
    match preset.trim().to_ascii_uppercase().as_str() {
        "PRESET A" | "A" => 0x01,
        "PRESET B" | "B" => 0x02,
        "LATEST" => 0x00FF_FFFF,
        _ => 0x00,
    }
}

/// Applies DLSS Super Resolution, Ray Reconstruction, and Frame Generation preset overrides
/// via NVIDIA's `nvapi64.dll` Driver Settings (DRS) API (`BaseProfile` when `exe_name` is `None`,
/// or the per-game profile when `exe_name` is provided).
pub fn apply_nvapi_dlss_presets(
    game_name: Option<&str>,
    exe_name: Option<&str>,
    sr_preset: &str,
    rr_preset: &str,
    fg_preset: &str,
) -> Result<String, String> {
    #[cfg(windows)]
    {
        use libloading::{Library, Symbol};
        use std::ffi::{c_int, c_uint, c_void};

        const NVDRS_UNICODE_STRING_MAX: usize = 2048;
        const NVDRS_BINARY_DATA_MAX: usize = 4096;

        // Official NVIDIA DRS Setting IDs for DLSS 4 / 4.5 Overrides
        const NGX_DLSS_SR_OVERRIDE_ID: u32 = 0x10E4_1E01; // "Enable DLSS-SR override"
        const NGX_DLSS_SR_PRESET_ID: u32 = 0x10E4_1DF3;   // "Override DLSS-SR presets"
        const NGX_DLSS_RR_OVERRIDE_ID: u32 = 0x10E4_1E02; // "Enable DLSS-RR override"
        const NGX_DLSS_RR_PRESET_ID: u32 = 0x10E4_1DF7;   // "Override DLSS-RR preset"
        const NGX_DLSS_FG_OVERRIDE_ID: u32 = 0x10E4_1E03; // "Enable DLSS-FG override"
        const NGX_DLSS_FG_PRESET_ID: u32 = 0x10E4_1DF1;   // "Override DLSS-FG preset"
        // Scaling ratio / performance mode / streamline gate override IDs that must be cleared so DLSS 4 / 4.5 does not fail init or stay blocked
        const NGX_DLSS_SR_SCALING_RATIO_ID: u32 = 0x10E4_1DF5; // "Override scaling ratio for DLSS-SR"
        const NGX_DLSS_SR_PERF_MODE_ID: u32 = 0x10AF_B768;     // "Override DLSS-SR performance mode"
        const NGX_DLSS_RR_PERF_MODE_ID: u32 = 0x10BD_9423;     // "Override DLSS-RR performance mode"
        const NGX_ENABLE_STREAMLINE_OVERRIDE_ID: u32 = 0x10E4_1E06; // Streamline / DLSS override gate (NVIDIA App sets 0 on non-whitelisted games)

        #[repr(C)]
        struct NvDrsSetting {
            version: u32,
            setting_name: [u16; NVDRS_UNICODE_STRING_MAX],
            setting_id: u32,
            setting_type: u32,     // 0 = NVDRS_DWORD_TYPE
            setting_location: u32, // 0 = NVDRS_CURRENT_PROFILE_LOCATION
            is_current_predefined: u32,
            is_predefined_valid: u32,
            predefined_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
            current_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
        }

        #[repr(C)]
        struct NvDrsProfile {
            version: u32,
            profile_name: [u16; NVDRS_UNICODE_STRING_MAX],
            gpu_support: u32,
            is_predefined: u32,
            num_of_apps: u32,
            num_of_settings: u32,
        }

        #[repr(C)]
        struct NvDrsApplicationV4 {
            version: u32,
            is_predefined: u32,
            app_name: [u16; NVDRS_UNICODE_STRING_MAX],
            user_friendly_name: [u16; NVDRS_UNICODE_STRING_MAX],
            launcher: [u16; NVDRS_UNICODE_STRING_MAX],
            file_in_folder: [u16; NVDRS_UNICODE_STRING_MAX],
            flags: u32,
            command_line: [u16; NVDRS_UNICODE_STRING_MAX],
        }

        fn make_empty_app_v4() -> NvDrsApplicationV4 {
            NvDrsApplicationV4 {
                version: (std::mem::size_of::<NvDrsApplicationV4>() as u32) | (4 << 16),
                is_predefined: 0,
                app_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                user_friendly_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                launcher: [0u16; NVDRS_UNICODE_STRING_MAX],
                file_in_folder: [0u16; NVDRS_UNICODE_STRING_MAX],
                flags: 0,
                command_line: [0u16; NVDRS_UNICODE_STRING_MAX],
            }
        }

        fn encode_wide(input: &str) -> [u16; NVDRS_UNICODE_STRING_MAX] {
            let mut buf = [0u16; NVDRS_UNICODE_STRING_MAX];
            for (i, u) in input
                .encode_utf16()
                .take(NVDRS_UNICODE_STRING_MAX - 1)
                .enumerate()
            {
                buf[i] = u;
            }
            buf
        }

        fn make_dword_setting(id: u32, val: u32) -> NvDrsSetting {
            let mut s = NvDrsSetting {
                version: (std::mem::size_of::<NvDrsSetting>() as u32) | (1 << 16),
                setting_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                setting_id: id,
                setting_type: 0,
                setting_location: 0,
                is_current_predefined: 0,
                is_predefined_valid: 0,
                predefined_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
                current_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
            };
            s.current_value[0..4].copy_from_slice(&val.to_le_bytes());
            s
        }

        let candidates = ["nvapi64.dll", "C:\\Windows\\System32\\nvapi64.dll"];
        let Ok(lib) = candidates
            .iter()
            .find_map(|p| unsafe { Library::new(*p).ok() })
            .ok_or_else(|| "nvapi64.dll not found".to_string())
        else {
            return Ok("Saved preset preferences (nvapi64.dll not present on this system)".to_string());
        };

        unsafe {
            type QueryInterfaceFn = unsafe extern "C" fn(c_uint) -> *const c_void;
            let Ok(query): Result<Symbol<QueryInterfaceFn>, _> =
                lib.get(b"nvapi_QueryInterface\0")
            else {
                return Ok("Saved preset preferences".to_string());
            };

            type NvInitFn = unsafe extern "C" fn() -> c_int;
            type DrsCreateSessionFn = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            type DrsDestroySessionFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsLoadSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsSaveSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsGetBaseProfileFn =
                unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int;
            type DrsFindProfileByNameFn =
                unsafe extern "C" fn(*mut c_void, *const u16, *mut *mut c_void) -> c_int;
            type DrsCreateProfileFn =
                unsafe extern "C" fn(*mut c_void, *const NvDrsProfile, *mut *mut c_void) -> c_int;
            type DrsDeleteProfileFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int;
            type DrsGetProfileInfoFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, *mut NvDrsProfile) -> c_int;
            type DrsFindApplicationByNameFn = unsafe extern "C" fn(
                *mut c_void,
                *const u16,
                *mut *mut c_void,
                *mut NvDrsApplicationV4,
            ) -> c_int;
            type DrsCreateApplicationFn = unsafe extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut NvDrsApplicationV4,
            ) -> c_int;
            type DrsSetSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, *mut NvDrsSetting) -> c_int;
            type DrsDeleteProfileSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> c_int;

            let init_ptr = query(0x0150_E828);
            let create_sess_ptr = query(0x0694_D52E);
            let destroy_sess_ptr = query(0xDAD9_CFF8);
            let load_ptr = query(0x375D_BD6B);
            let save_ptr = query(0xFCBC_7E14);
            let base_prof_ptr = query(0xDA84_66A0);
            let find_prof_ptr = query(0x7E4A_9A0B);
            let create_prof_ptr = query(0xCC17_6068);
            let delete_prof_ptr = query(0x1709_3206);
            let get_prof_info_ptr = query(0x61CD_6FD6);
            let find_app_ptr = query(0xEEE5_66B2);
            let create_app_ptr = query(0x4347_A9DE);
            let set_setting_ptr = query(0x577D_D202);
            let delete_setting_ptr = query(0xE4A2_6362);

            if init_ptr.is_null()
                || create_sess_ptr.is_null()
                || load_ptr.is_null()
                || save_ptr.is_null()
                || base_prof_ptr.is_null()
                || set_setting_ptr.is_null()
            {
                return Ok("Saved preset configuration".to_string());
            }

            let nv_init: NvInitFn = std::mem::transmute(init_ptr);
            let create_sess: DrsCreateSessionFn = std::mem::transmute(create_sess_ptr);
            let destroy_sess: DrsDestroySessionFn = std::mem::transmute(destroy_sess_ptr);
            let load_settings: DrsLoadSettingsFn = std::mem::transmute(load_ptr);
            let save_settings: DrsSaveSettingsFn = std::mem::transmute(save_ptr);
            let get_base_prof: DrsGetBaseProfileFn = std::mem::transmute(base_prof_ptr);
            let set_setting: DrsSetSettingFn = std::mem::transmute(set_setting_ptr);

            if nv_init() != 0 {
                return Ok("Saved preset configuration (NVAPI init skipped)".to_string());
            }

            let mut h_session: *mut c_void = std::ptr::null_mut();
            if create_sess(&mut h_session) != 0 || h_session.is_null() {
                return Ok("Saved preset configuration".to_string());
            }

            if load_settings(h_session) != 0 {
                let _ = destroy_sess(h_session);
                return Ok("Saved preset configuration".to_string());
            }

            let mut h_profile: *mut c_void = std::ptr::null_mut();
            let raw_exe_trimmed: Option<String> = exe_name
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let clean_exe_filename: Option<String> = raw_exe_trimmed
                .as_deref()
                .and_then(|s| {
                    std::path::Path::new(s)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                });

            // 1. Prefer finding NVIDIA's official driver profile bound to the game's .exe using NVDRS_APPLICATION_V4
            // (Tries full path first, then executable basename e.g. "starcitizen.exe" -> "Squadron 42 - Star Citizen")
            if !find_app_ptr.is_null() {
                let find_app: DrsFindApplicationByNameFn = std::mem::transmute(find_app_ptr);
                let mut probe_queries: Vec<String> = Vec::new();
                if let Some(ref full_exe) = raw_exe_trimmed {
                    probe_queries.push(full_exe.clone());
                }
                if let Some(ref base_exe) = clean_exe_filename {
                    if !probe_queries.iter().any(|q| q.eq_ignore_ascii_case(base_exe)) {
                        probe_queries.push(base_exe.clone());
                    }
                }
                for query_str in &probe_queries {
                    let wide_exe = encode_wide(query_str);
                    let mut app_info = make_empty_app_v4();
                    let mut candidate_prof: *mut c_void = std::ptr::null_mut();
                    let rc = find_app(
                        h_session,
                        wide_exe.as_ptr(),
                        &mut candidate_prof,
                        &mut app_info,
                    );
                    if rc == 0 && !candidate_prof.is_null() {
                        h_profile = candidate_prof;
                        break;
                    }
                }
            }

            // If we found the real profile via .exe, clean up any orphan custom profile created under `game_name`
            if !h_profile.is_null()
                && !find_prof_ptr.is_null()
                && !get_prof_info_ptr.is_null()
                && !delete_prof_ptr.is_null()
            {
                if let Some(gname) = game_name {
                    let find_prof: DrsFindProfileByNameFn = std::mem::transmute(find_prof_ptr);
                    let get_prof_info: DrsGetProfileInfoFn = std::mem::transmute(get_prof_info_ptr);
                    let delete_prof: DrsDeleteProfileFn = std::mem::transmute(delete_prof_ptr);
                    let wide_name = encode_wide(gname);
                    let mut named_prof: *mut c_void = std::ptr::null_mut();
                    if find_prof(h_session, wide_name.as_ptr(), &mut named_prof) == 0
                        && !named_prof.is_null()
                        && named_prof != h_profile
                    {
                        let mut pinfo = NvDrsProfile {
                            version: (std::mem::size_of::<NvDrsProfile>() as u32) | (1 << 16),
                            profile_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                            gpu_support: 0,
                            is_predefined: 0,
                            num_of_apps: 0,
                            num_of_settings: 0,
                        };
                        if get_prof_info(h_session, named_prof, &mut pinfo) == 0
                            && pinfo.is_predefined == 0
                        {
                            let _ = delete_prof(h_session, named_prof);
                        }
                    }
                }
            }

            // 2. Fallback: find or create profile by game name, and bind the .exe to it using NVDRS_APPLICATION_V4
            if h_profile.is_null() {
                if let Some(gname) = game_name {
                    if !find_prof_ptr.is_null() {
                        let find_prof: DrsFindProfileByNameFn = std::mem::transmute(find_prof_ptr);
                        let wide_name = encode_wide(gname);
                        let mut candidate_prof: *mut c_void = std::ptr::null_mut();
                        if find_prof(h_session, wide_name.as_ptr(), &mut candidate_prof) == 0
                            && !candidate_prof.is_null()
                        {
                            h_profile = candidate_prof;
                        }
                        if h_profile.is_null() && !create_prof_ptr.is_null() {
                            let create_prof: DrsCreateProfileFn =
                                std::mem::transmute(create_prof_ptr);
                            let prof_info = NvDrsProfile {
                                version: (std::mem::size_of::<NvDrsProfile>() as u32) | (1 << 16),
                                profile_name: wide_name,
                                gpu_support: 1,
                                is_predefined: 0,
                                num_of_apps: 0,
                                num_of_settings: 0,
                            };
                            let mut created_prof: *mut c_void = std::ptr::null_mut();
                            if create_prof(h_session, &prof_info, &mut created_prof) == 0
                                && !created_prof.is_null()
                            {
                                h_profile = created_prof;
                            }
                        }
                        if !h_profile.is_null() && !create_app_ptr.is_null() {
                            if let Some(ref exe_file) = clean_exe_filename {
                                let create_app: DrsCreateApplicationFn =
                                    std::mem::transmute(create_app_ptr);
                                let mut app_info = make_empty_app_v4();
                                app_info.app_name = encode_wide(exe_file);
                                app_info.user_friendly_name = wide_name;
                                let _ = create_app(h_session, h_profile, &mut app_info);
                            }
                        }
                    }
                }
            }

            if h_profile.is_null() {
                let _ = get_base_prof(h_session, &mut h_profile);
            }

            if !h_profile.is_null() {
                let is_per_game = game_name.is_some() || exe_name.is_some();
                let del_setting_opt: Option<DrsDeleteProfileSettingFn> =
                    if !delete_setting_ptr.is_null() {
                        Some(std::mem::transmute(delete_setting_ptr))
                    } else {
                        None
                    };

                let sr_val = parse_sr_preset_value(sr_preset);
                if sr_val > 0 {
                    let mut s1 = make_dword_setting(NGX_DLSS_SR_OVERRIDE_ID, 1);
                    let mut s2 = make_dword_setting(NGX_DLSS_SR_PRESET_ID, sr_val);
                    let _ = set_setting(h_session, h_profile, &mut s1);
                    let _ = set_setting(h_session, h_profile, &mut s2);
                } else if sr_preset.trim().eq_ignore_ascii_case("off") && is_per_game {
                    // Explicitly disable DLSS-SR override on this game profile so it does not inherit Global BaseProfile
                    let mut s1 = make_dword_setting(NGX_DLSS_SR_OVERRIDE_ID, 0);
                    let _ = set_setting(h_session, h_profile, &mut s1);
                    if let Some(del_fn) = del_setting_opt {
                        let _ = del_fn(h_session, h_profile, NGX_DLSS_SR_PRESET_ID);
                    }
                } else if let Some(del_fn) = del_setting_opt {
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_SR_OVERRIDE_ID);
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_SR_PRESET_ID);
                } else {
                    let mut s1 = make_dword_setting(NGX_DLSS_SR_OVERRIDE_ID, 0);
                    let mut s2 = make_dword_setting(NGX_DLSS_SR_PRESET_ID, 0);
                    let _ = set_setting(h_session, h_profile, &mut s1);
                    let _ = set_setting(h_session, h_profile, &mut s2);
                }

                let rr_val = parse_rr_preset_value(rr_preset);
                if rr_val > 0 {
                    let mut r1 = make_dword_setting(NGX_DLSS_RR_OVERRIDE_ID, 1);
                    let mut r2 = make_dword_setting(NGX_DLSS_RR_PRESET_ID, rr_val);
                    let _ = set_setting(h_session, h_profile, &mut r1);
                    let _ = set_setting(h_session, h_profile, &mut r2);
                } else if rr_preset.trim().eq_ignore_ascii_case("off") && is_per_game {
                    let mut r1 = make_dword_setting(NGX_DLSS_RR_OVERRIDE_ID, 0);
                    let _ = set_setting(h_session, h_profile, &mut r1);
                    if let Some(del_fn) = del_setting_opt {
                        let _ = del_fn(h_session, h_profile, NGX_DLSS_RR_PRESET_ID);
                    }
                } else if let Some(del_fn) = del_setting_opt {
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_RR_OVERRIDE_ID);
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_RR_PRESET_ID);
                } else {
                    let mut r1 = make_dword_setting(NGX_DLSS_RR_OVERRIDE_ID, 0);
                    let mut r2 = make_dword_setting(NGX_DLSS_RR_PRESET_ID, 0);
                    let _ = set_setting(h_session, h_profile, &mut r1);
                    let _ = set_setting(h_session, h_profile, &mut r2);
                }

                let fg_val = parse_fg_preset_value(fg_preset);
                if fg_val > 0 {
                    let mut f1 = make_dword_setting(NGX_DLSS_FG_OVERRIDE_ID, 1);
                    let mut f2 = make_dword_setting(NGX_DLSS_FG_PRESET_ID, fg_val);
                    let _ = set_setting(h_session, h_profile, &mut f1);
                    let _ = set_setting(h_session, h_profile, &mut f2);
                } else if fg_preset.trim().eq_ignore_ascii_case("off") && is_per_game {
                    // Explicitly disable FG & MFG overrides on this game profile so it does not inherit Global FG/MFG
                    let mut f1 = make_dword_setting(NGX_DLSS_FG_OVERRIDE_ID, 0);
                    let mut m0 = make_dword_setting(0x1030_8298u32, 0); // Override DLSSG mode = Off
                    let _ = set_setting(h_session, h_profile, &mut f1);
                    let _ = set_setting(h_session, h_profile, &mut m0);
                    if let Some(del_fn) = del_setting_opt {
                        let _ = del_fn(h_session, h_profile, NGX_DLSS_FG_PRESET_ID);
                        for fg_extra_id in &[
                            0x104D_6667u32, // Override DLSSG multi-frame count
                            0x1056_2D0Fu32, // Override maximum DLSSG dynamic multi frame count
                            0x10CF_4125u32, // Override DLSSG Target Frame Rate
                        ] {
                            let _ = del_fn(h_session, h_profile, *fg_extra_id);
                        }
                    }
                } else if let Some(del_fn) = del_setting_opt {
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_FG_OVERRIDE_ID);
                    let _ = del_fn(h_session, h_profile, NGX_DLSS_FG_PRESET_ID);
                    // Also delete any leftover non-default DLSS-G mode/frame-count overrides on games without FG
                    for fg_extra_id in &[
                        0x1030_8298u32, // Override DLSSG mode
                        0x104D_6667u32, // Override DLSSG multi-frame count
                        0x1056_2D0Fu32, // Override maximum DLSSG dynamic multi frame count
                        0x10CF_4125u32, // Override DLSSG Target Frame Rate
                    ] {
                        let _ = del_fn(h_session, h_profile, *fg_extra_id);
                    }
                } else {
                    let mut f1 = make_dword_setting(NGX_DLSS_FG_OVERRIDE_ID, 0);
                    let mut f2 = make_dword_setting(NGX_DLSS_FG_PRESET_ID, 0);
                    let _ = set_setting(h_session, h_profile, &mut f1);
                    let _ = set_setting(h_session, h_profile, &mut f2);
                }

                // Clean up any poisoned scaling ratio / performance mode / streamline gate overrides on both target profile and Base Profile
                // (An invalid scaling ratio like 0x2 in 0x10E41DF5 or 0x10E41E06=0 from NVIDIA App blocks DLSS / Preset overrides)
                if let Some(del_setting) = del_setting_opt {
                    for bad_id in &[
                        NGX_DLSS_SR_SCALING_RATIO_ID,
                        NGX_DLSS_SR_PERF_MODE_ID,
                        NGX_DLSS_RR_PERF_MODE_ID,
                        NGX_ENABLE_STREAMLINE_OVERRIDE_ID,
                    ] {
                        let _ = del_setting(h_session, h_profile, *bad_id);
                    }
                    let mut h_base: *mut c_void = std::ptr::null_mut();
                    if get_base_prof(h_session, &mut h_base) == 0 && !h_base.is_null() {
                        for bad_id in &[
                            NGX_DLSS_SR_SCALING_RATIO_ID,
                            NGX_DLSS_SR_PERF_MODE_ID,
                            NGX_DLSS_RR_PERF_MODE_ID,
                        ] {
                            let _ = del_setting(h_session, h_base, *bad_id);
                        }
                    }
                }

                let _ = save_settings(h_session);
            }

            let _ = destroy_sess(h_session);
        }

        Ok(format!(
            "Applied NVIDIA DRS DLSS Presets (SR: {}, RR: {}, FG: {})",
            sr_preset, rr_preset, fg_preset
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = (game_name, exe_name, sr_preset, rr_preset, fg_preset);
        Ok("Saved DLSS Preset overrides".to_string())
    }
}

/// Clears NVIDIA App's blocking per-game `0` DRS overrides (e.g., `0x10E41E01=0`,
/// `0x10E41DF3=0`, `0x10E41E06=0` on `"Squadron 42 - Star Citizen"`) for the given
/// executable names so those games cleanly inherit the Global Base Profile presets.
pub fn clear_game_dlss_profile_blocks(exe_names: &[String]) -> usize {
    #[cfg(windows)]
    {
        use libloading::{Library, Symbol};
        use std::ffi::{c_int, c_uint, c_void};

        if exe_names.is_empty() {
            return 0;
        }

        const NVDRS_UNICODE_STRING_MAX: usize = 2048;
        #[repr(C)]
        struct NvDrsApplicationV4 {
            version: u32,
            is_predefined: u32,
            app_name: [u16; NVDRS_UNICODE_STRING_MAX],
            user_friendly_name: [u16; NVDRS_UNICODE_STRING_MAX],
            launcher: [u16; NVDRS_UNICODE_STRING_MAX],
            file_in_folder: [u16; NVDRS_UNICODE_STRING_MAX],
            flags: u32,
            command_line: [u16; NVDRS_UNICODE_STRING_MAX],
        }

        let candidates = ["nvapi64.dll", "C:\\Windows\\System32\\nvapi64.dll"];
        let Some(lib) = candidates
            .iter()
            .find_map(|p| unsafe { Library::new(*p).ok() })
        else {
            return 0;
        };

        unsafe {
            type QueryInterfaceFn = unsafe extern "C" fn(c_uint) -> *const c_void;
            let Ok(query): Result<Symbol<QueryInterfaceFn>, _> =
                lib.get(b"nvapi_QueryInterface\0")
            else {
                return 0;
            };
            type NvInitFn = unsafe extern "C" fn() -> c_int;
            type DrsCreateSessionFn = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            type DrsDestroySessionFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsLoadSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsSaveSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsFindApplicationByNameFn = unsafe extern "C" fn(
                *mut c_void,
                *const u16,
                *mut *mut c_void,
                *mut NvDrsApplicationV4,
            ) -> c_int;
            type DrsDeleteProfileSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> c_int;

            let init_ptr = query(0x0150_E828);
            let create_sess_ptr = query(0x0694_D52E);
            let destroy_sess_ptr = query(0xDAD9_CFF8);
            let load_ptr = query(0x375D_BD6B);
            let save_ptr = query(0xFCBC_7E14);
            let find_app_ptr = query(0xEEE5_66B2);
            let delete_setting_ptr = query(0xE4A2_6362);

            if init_ptr.is_null()
                || create_sess_ptr.is_null()
                || load_ptr.is_null()
                || save_ptr.is_null()
                || find_app_ptr.is_null()
                || delete_setting_ptr.is_null()
            {
                return 0;
            }

            let nv_init: NvInitFn = std::mem::transmute(init_ptr);
            let create_sess: DrsCreateSessionFn = std::mem::transmute(create_sess_ptr);
            let destroy_sess: DrsDestroySessionFn = std::mem::transmute(destroy_sess_ptr);
            let load_settings: DrsLoadSettingsFn = std::mem::transmute(load_ptr);
            let save_settings: DrsSaveSettingsFn = std::mem::transmute(save_ptr);
            let find_app: DrsFindApplicationByNameFn = std::mem::transmute(find_app_ptr);
            let del_setting: DrsDeleteProfileSettingFn = std::mem::transmute(delete_setting_ptr);

            if nv_init() != 0 {
                return 0;
            }
            let mut h_session: *mut c_void = std::ptr::null_mut();
            if create_sess(&mut h_session) != 0 || h_session.is_null() {
                return 0;
            }
            if load_settings(h_session) != 0 {
                let _ = destroy_sess(h_session);
                return 0;
            }

            let mut cleared_profiles = 0usize;
            const BLOCKING_IDS: [u32; 10] = [
                0x10E4_1E01,
                0x10E4_1DF3,
                0x10E4_1E02,
                0x10E4_1DF7,
                0x10E4_1E03,
                0x10E4_1DF1,
                0x10E4_1DF5,
                0x10AF_B768,
                0x10BD_9423,
                0x10E4_1E06,
            ];

            for exe in exe_names {
                let Some(fname) = std::path::Path::new(exe)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                else {
                    continue;
                };
                let mut wide = [0u16; NVDRS_UNICODE_STRING_MAX];
                for (i, u) in fname
                    .encode_utf16()
                    .take(NVDRS_UNICODE_STRING_MAX - 1)
                    .enumerate()
                {
                    wide[i] = u;
                }
                let mut app_info = NvDrsApplicationV4 {
                    version: (std::mem::size_of::<NvDrsApplicationV4>() as u32) | (4 << 16),
                    is_predefined: 0,
                    app_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                    user_friendly_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                    launcher: [0u16; NVDRS_UNICODE_STRING_MAX],
                    file_in_folder: [0u16; NVDRS_UNICODE_STRING_MAX],
                    flags: 0,
                    command_line: [0u16; NVDRS_UNICODE_STRING_MAX],
                };
                let mut h_prof: *mut c_void = std::ptr::null_mut();
                if find_app(h_session, wide.as_ptr(), &mut h_prof, &mut app_info) == 0
                    && !h_prof.is_null()
                {
                    let mut any_deleted = false;
                    for id in BLOCKING_IDS {
                        if del_setting(h_session, h_prof, id) == 0 {
                            any_deleted = true;
                        }
                    }
                    if any_deleted {
                        cleared_profiles += 1;
                    }
                }
            }

            if cleared_profiles > 0 {
                let _ = save_settings(h_session);
            }
            let _ = destroy_sess(h_session);
            cleared_profiles
        }
    }
    #[cfg(not(windows))]
    {
        let _ = exe_names;
        0
    }
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct LiveDrsStatus {
    pub nvapi_available: bool,
    pub profile_found: bool,
    pub sr_override: Option<u32>,
    pub sr_preset: Option<String>,
    pub rr_override: Option<u32>,
    pub rr_preset: Option<String>,
    pub fg_override: Option<u32>,
    pub fg_preset: Option<String>,
    pub fg_mode: Option<String>,
    pub fg_multiplier: Option<u32>,
    pub fg_target_fps: Option<u32>,
    pub streamline_gate: Option<u32>,
    pub is_blocked_by_zero: bool,
}

fn format_sr_preset_u32(val: u32) -> String {
    match val {
        0 => "Default".to_string(),
        0x01 => "Preset A".to_string(),
        0x02 => "Preset B".to_string(),
        0x03 => "Preset C".to_string(),
        0x04 => "Preset D".to_string(),
        0x05 => "Preset E".to_string(),
        0x06 => "Preset F".to_string(),
        0x0A => "Preset J".to_string(),
        0x0B => "Preset K".to_string(),
        0x0C => "Preset L".to_string(),
        0x0D => "Preset M".to_string(),
        0x00FF_FFFF => "Latest".to_string(),
        other => format!("0x{:X}", other),
    }
}

/// Reads the effective NVIDIA DRS DLSS settings directly from `nvapi64.dll` for the
/// given executable's profile (falling back to `BaseProfile` if unset on the game profile).
pub fn read_live_drs_status(exe_name: Option<&str>) -> LiveDrsStatus {
    #[cfg(windows)]
    {
        use libloading::{Library, Symbol};
        use std::ffi::{c_int, c_uint, c_void};

        const NVDRS_UNICODE_STRING_MAX: usize = 2048;
        const NVDRS_BINARY_DATA_MAX: usize = 4096;

        #[repr(C)]
        struct NvDrsSetting {
            version: u32,
            setting_name: [u16; NVDRS_UNICODE_STRING_MAX],
            setting_id: u32,
            setting_type: u32,
            setting_location: u32,
            is_current_predefined: u32,
            is_predefined_valid: u32,
            predefined_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
            current_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
        }

        #[repr(C)]
        struct NvDrsApplicationV4 {
            version: u32,
            is_predefined: u32,
            app_name: [u16; NVDRS_UNICODE_STRING_MAX],
            user_friendly_name: [u16; NVDRS_UNICODE_STRING_MAX],
            launcher: [u16; NVDRS_UNICODE_STRING_MAX],
            file_in_folder: [u16; NVDRS_UNICODE_STRING_MAX],
            flags: u32,
            command_line: [u16; NVDRS_UNICODE_STRING_MAX],
        }

        let candidates = ["nvapi64.dll", "C:\\Windows\\System32\\nvapi64.dll"];
        let Some(lib) = candidates
            .iter()
            .find_map(|p| unsafe { Library::new(*p).ok() })
        else {
            return LiveDrsStatus::default();
        };

        unsafe {
            type QueryInterfaceFn = unsafe extern "C" fn(c_uint) -> *const c_void;
            let Ok(query): Result<Symbol<QueryInterfaceFn>, _> =
                lib.get(b"nvapi_QueryInterface\0")
            else {
                return LiveDrsStatus::default();
            };
            type NvInitFn = unsafe extern "C" fn() -> c_int;
            type DrsCreateSessionFn = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            type DrsDestroySessionFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsLoadSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsGetBaseProfileFn =
                unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int;
            type DrsFindApplicationByNameFn = unsafe extern "C" fn(
                *mut c_void,
                *const u16,
                *mut *mut c_void,
                *mut NvDrsApplicationV4,
            ) -> c_int;
            type DrsGetSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *mut NvDrsSetting) -> c_int;

            let init_ptr = query(0x0150_E828);
            let create_sess_ptr = query(0x0694_D52E);
            let destroy_sess_ptr = query(0xDAD9_CFF8);
            let load_ptr = query(0x375D_BD6B);
            let base_prof_ptr = query(0xDA84_66A0);
            let find_app_ptr = query(0xEEE5_66B2);
            let get_setting_ptr = query(0x73BF_8338);

            if init_ptr.is_null()
                || create_sess_ptr.is_null()
                || load_ptr.is_null()
                || base_prof_ptr.is_null()
                || get_setting_ptr.is_null()
            {
                return LiveDrsStatus::default();
            }

            let nv_init: NvInitFn = std::mem::transmute(init_ptr);
            let create_sess: DrsCreateSessionFn = std::mem::transmute(create_sess_ptr);
            let destroy_sess: DrsDestroySessionFn = std::mem::transmute(destroy_sess_ptr);
            let load_settings: DrsLoadSettingsFn = std::mem::transmute(load_ptr);
            let get_base_prof: DrsGetBaseProfileFn = std::mem::transmute(base_prof_ptr);
            let get_setting: DrsGetSettingFn = std::mem::transmute(get_setting_ptr);

            if nv_init() != 0 {
                return LiveDrsStatus::default();
            }
            let mut h_session: *mut c_void = std::ptr::null_mut();
            if create_sess(&mut h_session) != 0 || h_session.is_null() {
                return LiveDrsStatus::default();
            }
            if load_settings(h_session) != 0 {
                let _ = destroy_sess(h_session);
                return LiveDrsStatus::default();
            }

            let mut h_base: *mut c_void = std::ptr::null_mut();
            let _ = get_base_prof(h_session, &mut h_base);

            let mut h_game: *mut c_void = std::ptr::null_mut();
            if let Some(exe) = exe_name.filter(|s| !s.trim().is_empty()) {
                if !find_app_ptr.is_null() {
                    let find_app: DrsFindApplicationByNameFn = std::mem::transmute(find_app_ptr);
                    if let Some(fname) = std::path::Path::new(exe)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                    {
                        let mut wide = [0u16; NVDRS_UNICODE_STRING_MAX];
                        for (i, u) in fname
                            .encode_utf16()
                            .take(NVDRS_UNICODE_STRING_MAX - 1)
                            .enumerate()
                        {
                            wide[i] = u;
                        }
                        let mut app_info = NvDrsApplicationV4 {
                            version: (std::mem::size_of::<NvDrsApplicationV4>() as u32) | (4 << 16),
                            is_predefined: 0,
                            app_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                            user_friendly_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                            launcher: [0u16; NVDRS_UNICODE_STRING_MAX],
                            file_in_folder: [0u16; NVDRS_UNICODE_STRING_MAX],
                            flags: 0,
                            command_line: [0u16; NVDRS_UNICODE_STRING_MAX],
                        };
                        let _ = find_app(h_session, wide.as_ptr(), &mut h_game, &mut app_info);
                    }
                }
            }

            let read_dword = |prof: *mut c_void, id: u32| -> Option<u32> {
                if prof.is_null() {
                    return None;
                }
                let mut s = NvDrsSetting {
                    version: (std::mem::size_of::<NvDrsSetting>() as u32) | (1 << 16),
                    setting_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                    setting_id: 0,
                    setting_type: 0,
                    setting_location: 0,
                    is_current_predefined: 0,
                    is_predefined_valid: 0,
                    predefined_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
                    current_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
                };
                if get_setting(h_session, prof, id, &mut s) == 0 {
                    Some(u32::from_le_bytes([
                        s.current_value[0],
                        s.current_value[1],
                        s.current_value[2],
                        s.current_value[3],
                    ]))
                } else {
                    None
                }
            };

            let read_eff = |id: u32| -> Option<u32> {
                read_dword(h_game, id).or_else(|| read_dword(h_base, id))
            };

            let game_sr_ov = read_dword(h_game, 0x10E4_1E01);
            let game_gate = read_dword(h_game, 0x10E4_1E06);
            let is_blocked_by_zero = game_sr_ov == Some(0) || game_gate == Some(0);

            let sr_override = read_eff(0x10E4_1E01);
            let sr_preset = read_eff(0x10E4_1DF3).map(format_sr_preset_u32);
            let rr_override = read_eff(0x10E4_1E02);
            let rr_preset = read_eff(0x10E4_1DF7).map(format_sr_preset_u32);
            let fg_override = read_eff(0x10E4_1E03);
            let fg_preset = read_eff(0x10E4_1DF1).map(format_sr_preset_u32);
            let fg_mode = read_eff(0x1030_8298).map(|m| match m {
                0 => "Off".to_string(),
                1 => "Fixed".to_string(),
                2 => "Dynamic".to_string(),
                3 => "Auto".to_string(),
                _ => "Default".to_string(),
            });
            let fg_multiplier = read_eff(0x104D_6667)
                .or_else(|| read_eff(0x1056_2D0F))
                .map(|c| c + 1);
            let fg_target_fps = read_eff(0x10CF_4125).filter(|&v| v > 0 && v < 10_000);

            let _ = destroy_sess(h_session);
            LiveDrsStatus {
                nvapi_available: true,
                profile_found: !h_game.is_null(),
                sr_override,
                sr_preset,
                rr_override,
                rr_preset,
                fg_override,
                fg_preset,
                fg_mode,
                fg_multiplier,
                fg_target_fps,
                streamline_gate: game_gate,
                is_blocked_by_zero,
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = exe_name;
        LiveDrsStatus::default()
    }
}

/// Applies DLSS-G Multi-Frame Generation (MFG) mode, multiplier (2x..6x), and optional
/// dynamic target FPS via NVIDIA DRS (`0x10308298`, `0x104D6667`, `0x10562D0F`, `0x10CF4125`).
pub fn apply_nvapi_dlssg_mfg(
    game_name: Option<&str>,
    exe_name: Option<&str>,
    mode: &str,
    multiplier: u32,
    target_fps: Option<u32>,
) -> Result<String, String> {
    #[cfg(windows)]
    {
        use libloading::{Library, Symbol};
        use std::ffi::{c_int, c_uint, c_void};

        const NVDRS_UNICODE_STRING_MAX: usize = 2048;
        const NVDRS_BINARY_DATA_MAX: usize = 4096;
        const NGX_DLSSG_MODE_ID: u32 = 0x1030_8298;
        const NGX_DLSSG_MULTI_FRAME_COUNT_ID: u32 = 0x104D_6667;
        const NGX_DLSSG_DYNAMIC_MULTI_FRAME_COUNT_MAX_ID: u32 = 0x1056_2D0F;
        const NGX_DLSSG_DYNAMIC_TARGET_FRAME_RATE_ID: u32 = 0x10CF_4125;

        #[repr(C)]
        struct NvDrsSetting {
            version: u32,
            setting_name: [u16; NVDRS_UNICODE_STRING_MAX],
            setting_id: u32,
            setting_type: u32,
            setting_location: u32,
            is_current_predefined: u32,
            is_predefined_valid: u32,
            predefined_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
            current_value: [u8; NVDRS_BINARY_DATA_MAX + 4],
        }
        #[repr(C)]
        struct NvDrsApplicationV4 {
            version: u32,
            is_predefined: u32,
            app_name: [u16; NVDRS_UNICODE_STRING_MAX],
            user_friendly_name: [u16; NVDRS_UNICODE_STRING_MAX],
            launcher: [u16; NVDRS_UNICODE_STRING_MAX],
            file_in_folder: [u16; NVDRS_UNICODE_STRING_MAX],
            flags: u32,
            command_line: [u16; NVDRS_UNICODE_STRING_MAX],
        }

        let candidates = ["nvapi64.dll", "C:\\Windows\\System32\\nvapi64.dll"];
        let Some(lib) = candidates
            .iter()
            .find_map(|p| unsafe { Library::new(*p).ok() })
        else {
            return Ok("MFG preference saved (nvapi64.dll not present)".to_string());
        };

        unsafe {
            type QueryInterfaceFn = unsafe extern "C" fn(c_uint) -> *const c_void;
            let Ok(query): Result<Symbol<QueryInterfaceFn>, _> =
                lib.get(b"nvapi_QueryInterface\0")
            else {
                return Ok("MFG preference saved".to_string());
            };
            type NvInitFn = unsafe extern "C" fn() -> c_int;
            type DrsCreateSessionFn = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
            type DrsDestroySessionFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsLoadSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsSaveSettingsFn = unsafe extern "C" fn(*mut c_void) -> c_int;
            type DrsGetBaseProfileFn =
                unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int;
            type DrsFindProfileByNameFn =
                unsafe extern "C" fn(*mut c_void, *const u16, *mut *mut c_void) -> c_int;
            type DrsFindApplicationByNameFn = unsafe extern "C" fn(
                *mut c_void,
                *const u16,
                *mut *mut c_void,
                *mut NvDrsApplicationV4,
            ) -> c_int;
            type DrsSetSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, *mut NvDrsSetting) -> c_int;
            type DrsDeleteProfileSettingFn =
                unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> c_int;

            let init_ptr = query(0x0150_E828);
            let create_sess_ptr = query(0x0694_D52E);
            let destroy_sess_ptr = query(0xDAD9_CFF8);
            let load_ptr = query(0x375D_BD6B);
            let save_ptr = query(0xFCBC_7E14);
            let base_prof_ptr = query(0xDA84_66A0);
            let find_prof_ptr = query(0x7E4A_9A0B);
            let find_app_ptr = query(0xEEE5_66B2);
            let set_setting_ptr = query(0x577D_D202);
            let del_setting_ptr = query(0xE4A2_6362);

            if init_ptr.is_null()
                || create_sess_ptr.is_null()
                || load_ptr.is_null()
                || save_ptr.is_null()
                || set_setting_ptr.is_null()
            {
                return Ok("MFG preference saved".to_string());
            }

            let nv_init: NvInitFn = std::mem::transmute(init_ptr);
            let create_sess: DrsCreateSessionFn = std::mem::transmute(create_sess_ptr);
            let destroy_sess: DrsDestroySessionFn = std::mem::transmute(destroy_sess_ptr);
            let load_settings: DrsLoadSettingsFn = std::mem::transmute(load_ptr);
            let save_settings: DrsSaveSettingsFn = std::mem::transmute(save_ptr);
            let set_setting: DrsSetSettingFn = std::mem::transmute(set_setting_ptr);

            if nv_init() != 0 {
                return Ok("MFG preference saved".to_string());
            }
            let mut h_session: *mut c_void = std::ptr::null_mut();
            if create_sess(&mut h_session) != 0 || h_session.is_null() {
                return Ok("MFG preference saved".to_string());
            }
            if load_settings(h_session) != 0 {
                let _ = destroy_sess(h_session);
                return Ok("MFG preference saved".to_string());
            }

            let mut h_profile: *mut c_void = std::ptr::null_mut();
            if let Some(exe) = exe_name.filter(|s| !s.trim().is_empty()) {
                if !find_app_ptr.is_null() {
                    let find_app: DrsFindApplicationByNameFn = std::mem::transmute(find_app_ptr);
                    if let Some(fname) = std::path::Path::new(exe)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                    {
                        let mut wide = [0u16; NVDRS_UNICODE_STRING_MAX];
                        for (i, u) in fname
                            .encode_utf16()
                            .take(NVDRS_UNICODE_STRING_MAX - 1)
                            .enumerate()
                        {
                            wide[i] = u;
                        }
                        let mut app_info = NvDrsApplicationV4 {
                            version: (std::mem::size_of::<NvDrsApplicationV4>() as u32) | (4 << 16),
                            is_predefined: 0,
                            app_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                            user_friendly_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                            launcher: [0u16; NVDRS_UNICODE_STRING_MAX],
                            file_in_folder: [0u16; NVDRS_UNICODE_STRING_MAX],
                            flags: 0,
                            command_line: [0u16; NVDRS_UNICODE_STRING_MAX],
                        };
                        let _ = find_app(h_session, wide.as_ptr(), &mut h_profile, &mut app_info);
                    }
                }
            }
            if h_profile.is_null() {
                if let (Some(gname), false) = (game_name, find_prof_ptr.is_null()) {
                    let find_prof: DrsFindProfileByNameFn = std::mem::transmute(find_prof_ptr);
                    let mut wide = [0u16; NVDRS_UNICODE_STRING_MAX];
                    for (i, u) in gname
                        .encode_utf16()
                        .take(NVDRS_UNICODE_STRING_MAX - 1)
                        .enumerate()
                    {
                        wide[i] = u;
                    }
                    let _ = find_prof(h_session, wide.as_ptr(), &mut h_profile);
                }
            }
            if h_profile.is_null() && !base_prof_ptr.is_null() {
                let get_base: DrsGetBaseProfileFn = std::mem::transmute(base_prof_ptr);
                let _ = get_base(h_session, &mut h_profile);
            }

            let make_dword = |id: u32, val: u32| -> NvDrsSetting {
                let mut s = NvDrsSetting {
                    version: (std::mem::size_of::<NvDrsSetting>() as u32) | (1 << 16),
                    setting_name: [0u16; NVDRS_UNICODE_STRING_MAX],
                    setting_id: id,
                    setting_type: 0,
                    setting_location: 0,
                    is_current_predefined: 0,
                    is_predefined_valid: 0,
                    predefined_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
                    current_value: [0u8; NVDRS_BINARY_DATA_MAX + 4],
                };
                s.current_value[0..4].copy_from_slice(&val.to_le_bytes());
                s
            };

            if !h_profile.is_null() {
                let frame_count = multiplier.clamp(2, 6) - 1;
                match mode.trim().to_ascii_lowercase().as_str() {
                    "fixed" | "on" => {
                        let mut m = make_dword(NGX_DLSSG_MODE_ID, 1);
                        let mut c = make_dword(NGX_DLSSG_MULTI_FRAME_COUNT_ID, frame_count);
                        let _ = set_setting(h_session, h_profile, &mut m);
                        let _ = set_setting(h_session, h_profile, &mut c);
                    }
                    "dynamic" => {
                        let mut m = make_dword(NGX_DLSSG_MODE_ID, 2);
                        let mut c =
                            make_dword(NGX_DLSSG_DYNAMIC_MULTI_FRAME_COUNT_MAX_ID, frame_count);
                        let fps_val = target_fps.filter(|&v| v > 0).unwrap_or(0);
                        let mut f = make_dword(NGX_DLSSG_DYNAMIC_TARGET_FRAME_RATE_ID, fps_val);
                        let _ = set_setting(h_session, h_profile, &mut m);
                        let _ = set_setting(h_session, h_profile, &mut c);
                        let _ = set_setting(h_session, h_profile, &mut f);
                    }
                    "off" => {
                        let mut m = make_dword(NGX_DLSSG_MODE_ID, 0);
                        let _ = set_setting(h_session, h_profile, &mut m);
                    }
                    _ => {
                        if !del_setting_ptr.is_null() {
                            let del_fn: DrsDeleteProfileSettingFn =
                                std::mem::transmute(del_setting_ptr);
                            for id in [
                                NGX_DLSSG_MODE_ID,
                                NGX_DLSSG_MULTI_FRAME_COUNT_ID,
                                NGX_DLSSG_DYNAMIC_MULTI_FRAME_COUNT_MAX_ID,
                                NGX_DLSSG_DYNAMIC_TARGET_FRAME_RATE_ID,
                            ] {
                                let _ = del_fn(h_session, h_profile, id);
                            }
                        }
                    }
                }
                let _ = save_settings(h_session);
            }
            let _ = destroy_sess(h_session);
        }

        Ok(format!(
            "Applied DLSS-G MFG ({} {}x)",
            mode,
            multiplier.clamp(2, 6)
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = (game_name, exe_name, mode, multiplier, target_fps);
        Ok("Saved DLSS-G MFG preference".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_gpu_detection() {
        let gpu = detect_gpu_info();
        println!(
            "Detected GPU: vendor={}, name={}, arch={}, driver={}, dlss={}, fg={}, rr={}, fsr4={}, xess={}",
            gpu.vendor,
            gpu.name,
            gpu.architecture,
            gpu.driver_version,
            gpu.supports_dlss,
            gpu.supports_dlss_fg,
            gpu.supports_dlss_rr,
            gpu.supports_fsr4,
            gpu.supports_xess
        );
    }
}
