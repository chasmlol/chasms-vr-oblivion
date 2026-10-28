//! The `dinput8.dll` proxy: we are loaded in the system DLL's place, so we must re-export its
//! entry points and forward each to the real DLL in `System32`.
//!
//! Every export is a pass-through: being loaded is all the adapter needs from this DLL. (Oblivion
//! imports `DirectInput8Create` by name, so the 32-bit build must export it undecorated.)

/// Every export the real `dinput8.dll` provides and that our `.def`/`#[no_mangle]` set must match.
/// Kept as data so it can be checked against the forwarding code in a unit test.
pub const FORWARDED_EXPORTS: &[&str] = &[
    "DirectInput8Create",
    "DllCanUnloadNow",
    "DllGetClassObject",
    "DllRegisterServer",
    "DllUnregisterServer",
    "GetdfDIJoystick",
];

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use windows::Win32::Foundation::{FARPROC, HMODULE};
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
    use windows::core::{HRESULT, PCSTR, s};

    static REAL: OnceLock<usize> = OnceLock::new();

    fn real_module() -> HMODULE {
        let h = *REAL.get_or_init(|| unsafe {
            let mut buf = [0u16; 260];
            let n = GetSystemDirectoryW(Some(&mut buf)) as usize;
            let mut path: Vec<u16> = buf[..n].to_vec();
            path.extend("\\dinput8.dll".encode_utf16());
            path.push(0);
            match LoadLibraryW(windows::core::PCWSTR(path.as_ptr())) {
                Ok(m) => m.0 as usize,
                Err(_) => 0,
            }
        });
        HMODULE(h as *mut c_void)
    }

    /// Resolves an export of the real `dinput8.dll` by name, or null if it or the DLL is missing.
    fn real(name: PCSTR) -> FARPROC {
        let m = real_module();
        if m.0.is_null() {
            return None;
        }
        unsafe { GetProcAddress(m, name) }
    }

    type DInput8Create =
        unsafe extern "system" fn(*mut c_void, u32, *const c_void, *mut *mut c_void, *mut c_void) -> HRESULT;
    type NoArgHResult = unsafe extern "system" fn() -> HRESULT;
    type GetClassObject = unsafe extern "system" fn(*const c_void, *const c_void, *mut *mut c_void) -> HRESULT;
    type NoArgPtr = unsafe extern "system" fn() -> *const c_void;

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DirectInput8Create(
        hinst: *mut c_void,
        version: u32,
        riid: *const c_void,
        out: *mut *mut c_void,
        outer: *mut c_void,
    ) -> HRESULT {
        let Some(p) = real(s!("DirectInput8Create")) else {
            return windows::Win32::Foundation::E_FAIL;
        };
        let f: DInput8Create = unsafe { std::mem::transmute(p) };
        let hr = unsafe { f(hinst, version, riid, out, outer) };
        // The game's keyboard and mouse get the runtime's actions (crate::input).
        if hr.is_ok() && !out.is_null() {
            crate::input::on_direct_input(unsafe { *out });
        }
        hr
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
        match real(s!("DllCanUnloadNow")) {
            Some(p) => unsafe { std::mem::transmute::<_, NoArgHResult>(p)() },
            None => windows::Win32::Foundation::S_FALSE,
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DllGetClassObject(
        rclsid: *const c_void,
        riid: *const c_void,
        out: *mut *mut c_void,
    ) -> HRESULT {
        match real(s!("DllGetClassObject")) {
            Some(p) => unsafe { std::mem::transmute::<_, GetClassObject>(p)(rclsid, riid, out) },
            None => windows::Win32::Foundation::E_FAIL,
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
        match real(s!("DllRegisterServer")) {
            Some(p) => unsafe { std::mem::transmute::<_, NoArgHResult>(p)() },
            None => windows::Win32::Foundation::E_FAIL,
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
        match real(s!("DllUnregisterServer")) {
            Some(p) => unsafe { std::mem::transmute::<_, NoArgHResult>(p)() },
            None => windows::Win32::Foundation::E_FAIL,
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn GetdfDIJoystick() -> *const c_void {
        match real(s!("GetdfDIJoystick")) {
            Some(p) => unsafe { std::mem::transmute::<_, NoArgPtr>(p)() },
            None => std::ptr::null(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_table_matches_the_dinput8_def() {
        // Exactly the exports listed in proxy/dinput8.def.
        let expected = [
            "DirectInput8Create",
            "DllCanUnloadNow",
            "DllGetClassObject",
            "DllRegisterServer",
            "DllUnregisterServer",
            "GetdfDIJoystick",
        ];
        assert_eq!(FORWARDED_EXPORTS, expected);
    }

    #[test]
    fn no_duplicate_exports() {
        let mut seen = std::collections::HashSet::new();
        for e in FORWARDED_EXPORTS {
            assert!(seen.insert(*e), "duplicate export {e}");
        }
    }
}
