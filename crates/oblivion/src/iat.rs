//! Import Address Table patching: one of Oblivion's imported functions redirected to ours. It only
//! touches the game's own calls. The exe is 32-bit (PE32): 4-byte thunks, the import directory at
//! optional header + 104.

use crate::mem;

/// Patches `Oblivion.exe`'s import `dll!func` to `hook`; returns the original.
#[cfg(windows)]
pub fn patch(dll: &str, func: &str, hook: usize) -> Option<usize> {
    use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, PAGE_READWRITE, VirtualProtect};
    let base = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None) }.ok()?.0 as usize;
    let nt = base + mem::at::<u32>(base + 0x3C)? as usize;
    let optional = nt + 24;
    let ptr = std::mem::size_of::<usize>();
    let import_rva = mem::at::<u32>(optional + if ptr == 4 { 104 } else { 120 })? as usize;
    if import_rva == 0 {
        return None;
    }
    let mut desc = base + import_rva;
    loop {
        let names_rva = mem::at::<u32>(desc)? as usize;
        let name_rva = mem::at::<u32>(desc + 12)? as usize;
        let first_thunk = mem::at::<u32>(desc + 16)? as usize;
        if name_rva == 0 && first_thunk == 0 {
            return None;
        }
        if read_cstr(base + name_rva).eq_ignore_ascii_case(dll) {
            let names = base + if names_rva != 0 { names_rva } else { first_thunk };
            let thunks = base + first_thunk;
            for i in 0.. {
                let entry = mem::at::<usize>(names + i * ptr)?;
                if entry == 0 {
                    return None;
                }
                let by_ordinal = entry & (1usize << (ptr * 8 - 1)) != 0;
                if !by_ordinal && read_cstr(base + entry + 2) == func {
                    let slot = (thunks + i * ptr) as *mut usize;
                    let mut old = PAGE_PROTECTION_FLAGS(0);
                    unsafe { VirtualProtect(slot as *const _, ptr, PAGE_READWRITE, &mut old) }.ok()?;
                    let orig = unsafe { slot.read() };
                    unsafe { slot.write(hook) };
                    let _ = unsafe { VirtualProtect(slot as *const _, ptr, old, &mut old) };
                    return Some(orig);
                }
            }
        }
        desc += 20;
    }
}

#[cfg(windows)]
fn read_cstr(at: usize) -> String {
    let mut out = Vec::new();
    for i in 0..256 {
        match mem::at::<u8>(at + i) {
            Some(0) | None => break,
            Some(b) => out.push(b),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
