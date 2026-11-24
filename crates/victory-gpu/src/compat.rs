//! Host restrictions that affect the shared CPU/GPU memory implementation.

/// Whether CPU writes may be tracked by making pages read-only and handling
/// their first write. Wine's write watch and our mapped-view guards both use
/// that mechanism. Under Rosetta, an AVX store crossing a protected host-page
/// boundary can abort in EmulateForward before Windows exception dispatch.
/// On Wine/macOS use the existing full-copy/hash fallback instead.
pub fn fault_based_write_tracking_supported() -> bool {
    static SUPPORTED: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
        let supported = !wine_on_macos();
        if !supported {
            tracing::warn!(
                "Wine/macOS: fault-based memory write tracking disabled for Rosetta compatibility"
            );
        }
        supported
    });
    *SUPPORTED
}

#[cfg(windows)]
fn wine_on_macos() -> bool {
    use std::ffi::{CStr, c_char};
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    type GetHostVersion = unsafe extern "C" fn(*mut *const c_char, *mut *const c_char);
    // SAFETY: ntdll is loaded for the process lifetime. Native Windows has no
    // such export; Wine documents its cdecl signature and returns static strings.
    unsafe {
        let ntdll = GetModuleHandleA(c"ntdll.dll".as_ptr().cast());
        if ntdll.is_null() {
            return false;
        }
        let Some(export) = GetProcAddress(ntdll, c"wine_get_host_version".as_ptr().cast()) else {
            return false;
        };
        let get_host: GetHostVersion = std::mem::transmute(export);
        let mut system = std::ptr::null();
        get_host(&mut system, std::ptr::null_mut());
        !system.is_null() && CStr::from_ptr(system).to_bytes() == b"Darwin"
    }
}

#[cfg(not(windows))]
fn wine_on_macos() -> bool {
    false
}
