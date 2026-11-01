use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::models::DllType;

/// Fast, zero-dependency PE version parser that reads `VS_FIXEDFILEINFO` directly from
/// a Windows DLL's `.rsrc` / version resource without loading the DLL into memory.
pub fn extract_dll_version(path: &Path) -> Option<String> {
    #[cfg(windows)]
    {
        if let Some(ver) = extract_version_win32(path) {
            return Some(ver);
        }
    }
    extract_version_binary_scan(path)
}

#[cfg(windows)]
fn extract_version_win32(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };

    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut handle: u32 = 0;
        let size = GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle);
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr() as *mut _) == 0 {
            return None;
        }

        let sub_block: Vec<u16> = "\\\0".encode_utf16().collect();
        let mut lp_buffer: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut len: u32 = 0;

        if VerQueryValueW(
            buffer.as_ptr() as *const _,
            sub_block.as_ptr(),
            &mut lp_buffer,
            &mut len,
        ) == 0
            || lp_buffer.is_null()
            || (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>()
        {
            return None;
        }

        let info = &*(lp_buffer as *const VS_FIXEDFILEINFO);
        if info.dwSignature != 0xFEEF04BD {
            return None;
        }

        let major = (info.dwFileVersionMS >> 16) & 0xFFFF;
        let minor = info.dwFileVersionMS & 0xFFFF;
        let build = (info.dwFileVersionLS >> 16) & 0xFFFF;
        let rev = info.dwFileVersionLS & 0xFFFF;

        if major == 0 && minor == 0 && build == 0 && rev == 0 {
            return None;
        }

        Some(normalize_version_numbers(major, minor, build, rev))
    }
}

/// Cross-platform fallback that scans the binary for the `VS_FIXEDFILEINFO` magic signature `0xFEEF04BD`.
fn extract_version_binary_scan(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;

    // Look for VS_FIXEDFILEINFO signature 0xFEEF04BD (little-endian: BD 04 EF FE)
    let sig = [0xBDu8, 0x04, 0xEF, 0xFE];
    for i in 0..buf.len().saturating_sub(52) {
        if buf[i..i + 4] == sig {
            let struc_ver = u32::from_le_bytes(buf[i + 4..i + 8].try_into().ok()?);
            if struc_ver != 0x00010000 {
                continue;
            }
            let file_ver_ms = u32::from_le_bytes(buf[i + 8..i + 12].try_into().ok()?);
            let file_ver_ls = u32::from_le_bytes(buf[i + 12..i + 16].try_into().ok()?);
            let major = (file_ver_ms >> 16) & 0xFFFF;
            let minor = file_ver_ms & 0xFFFF;
            let build = (file_ver_ls >> 16) & 0xFFFF;
            let rev = file_ver_ls & 0xFFFF;
            if major > 0 || minor > 0 {
                return Some(normalize_version_numbers(major, minor, build, rev));
            }
        }
    }
    None
}

fn normalize_version_numbers(major: u32, minor: u32, build: u32, rev: u32) -> String {
    if rev == 0 {
        format!("{}.{}.{}", major, minor, build)
    } else {
        format!("{}.{}.{}.{}", major, minor, build, rev)
    }
}

/// Returns the human-readable DLSS Preset / Architecture hint for a given DLL type and version.
pub fn preset_hint_for_dll(dll_type: DllType, version: &str) -> Option<String> {
    match dll_type {
        DllType::Dlss | DllType::DlssD | DllType::DlssG => {
            let parts: Vec<u32> = version
                .split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
            let major = *parts.first().unwrap_or(&0);
            let minor = *parts.get(1).unwrap_or(&0);
            let patch = *parts.get(2).unwrap_or(&0);

            if major >= 310 {
                if minor >= 3 {
                    Some("DLSS 4.5 (Transformer Gen 2 · Preset J/K)".to_string())
                } else if minor >= 2 {
                    Some("DLSS 4 (Transformer · Preset J/K)".to_string())
                } else {
                    Some("DLSS 4 (Transformer · Preset J)".to_string())
                }
            } else if major == 3 && minor >= 7 {
                if patch >= 20 {
                    Some("DLSS 3.7+ (CNN · Preset E)".to_string())
                } else {
                    Some("DLSS 3.7 (CNN · Preset E)".to_string())
                }
            } else if major == 3 {
                Some("DLSS 3.x (CNN · Preset C/D)".to_string())
            } else if major == 2 {
                Some("DLSS 2.x (CNN)".to_string())
            } else {
                None
            }
        }
        DllType::Fsr3 | DllType::FsrVk => {
            if version.starts_with("4.") || version.starts_with("2.4") {
                Some("FSR 4 / FidelityFX SDK 2.4".to_string())
            } else if version.starts_with("3.") || version.starts_with("1.") {
                Some("FSR 3.1".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Compare two dotted version strings numerically.
pub fn is_version_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };
    let a = parse(candidate);
    let b = parse(current);
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        if av != bv {
            return av > bv;
        }
    }
    false
}

