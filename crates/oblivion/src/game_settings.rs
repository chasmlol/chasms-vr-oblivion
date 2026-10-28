//! Oblivion's own settings (`My Games\Oblivion\Oblivion.ini`) while it's in VR: it renders each eye
//! at the size the headset wants (`iSize W`/`iSize H`) in a window. The player's own values are
//! kept in `rustvr-oblivion-ini.txt` beside the DLL and put back when the game closes (or, after a
//! crash, the next time it starts, before anything else).

use std::path::{Path, PathBuf};

const BACKUP: &str = "rustvr-oblivion-ini.txt";

/// `Oblivion.ini`'s path.
#[cfg(windows)]
pub fn ini_path() -> Option<PathBuf> {
    use windows::Win32::UI::Shell::{CSIDL_PERSONAL, SHGetFolderPathW};
    let mut buf = [0u16; 260];
    unsafe { SHGetFolderPathW(None, CSIDL_PERSONAL as i32, None, 0, &mut buf) }.ok()?;
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..n])).join("My Games").join("Oblivion").join("Oblivion.ini"))
}

#[cfg(not(windows))]
pub fn ini_path() -> Option<PathBuf> {
    None
}

/// Sets `section`'s `key` in INI text (adding it if missing); returns the old value.
pub fn set_key(text: &mut String, section: &str, key: &str, value: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut in_section = false;
    let mut old = None;
    let mut done = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            if in_section && !done {
                out.push(format!("{key}={value}"));
                done = true;
            }
            in_section = t.eq_ignore_ascii_case(&format!("[{section}]"));
        } else if in_section
            && !done
            && let Some((k, v)) = t.split_once('=')
            && k.trim().eq_ignore_ascii_case(key)
        {
            old = Some(v.trim().to_owned());
            out.push(format!("{key}={value}"));
            done = true;
            continue;
        }
        out.push(line.to_owned());
    }
    if !done {
        if !in_section {
            out.push(format!("[{section}]"));
        }
        out.push(format!("{key}={value}"));
    }
    *text = out.join("\r\n") + "\r\n";
    old
}

/// Puts the game's values back if a backup is there (a crashed session's, or this one's at exit).
pub fn restore(dir: &Path) {
    let Ok(backup) = std::fs::read_to_string(dir.join(BACKUP)) else { return };
    let Some(ini) = ini_path() else { return };
    let Ok(mut text) = std::fs::read_to_string(&ini) else { return };
    for line in backup.lines() {
        let mut parts = line.splitn(3, '\t');
        if let (Some(s), Some(k), Some(v)) = (parts.next(), parts.next(), parts.next()) {
            set_key(&mut text, s, k, v);
        }
    }
    if std::fs::write(&ini, text).is_ok() {
        let _ = std::fs::remove_file(dir.join(BACKUP));
        tracing::info!("game settings: Oblivion.ini is the player's again");
    }
}

/// VR's values into `Oblivion.ini` (the player's kept), before the game reads it.
pub fn apply(dir: &Path, eye: Option<(u32, u32)>) {
    let Some(ini) = ini_path() else { return };
    let mut text = match std::fs::read_to_string(&ini) {
        Ok(t) => t,
        // Not made yet (the game's first start): its defaults, as the game would.
        Err(_) => match std::fs::read_to_string(dir.join("Oblivion_default.ini")) {
            Ok(t) => t,
            Err(_) => return,
        },
    };
    let mut backup = String::new();
    let mut set = |s: &str, k: &str, v: String| {
        let old = set_key(&mut text, s, k, &v);
        if let Some(old) = old
            && old != v
        {
            backup.push_str(&format!("{s}\t{k}\t{old}\n"));
        }
    };
    set("Display", "bFull Screen", "0".into());
    // Straight to the main menu: the intro films only play flat.
    set("General", "SIntroSequence", String::new());
    if let Some((w, h)) = eye {
        set("Display", "iSize W", w.to_string());
        set("Display", "iSize H", h.to_string());
    }
    if !backup.is_empty() && !dir.join(BACKUP).exists() {
        let _ = std::fs::write(dir.join(BACKUP), &backup);
    }
    if let Some(parent) = ini.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&ini, text);
    tracing::info!("game settings: Oblivion.ini windowed{}", eye.map(|(w, h)| format!(", {w}x{h}")).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_and_adds_keys() {
        let mut t = "[General]\r\nfoo=1\r\n[Display]\r\niSize W=640\r\nbFull Screen=1\r\n".to_owned();
        assert_eq!(set_key(&mut t, "Display", "iSize W", "2064").as_deref(), Some("640"));
        assert_eq!(set_key(&mut t, "display", "bFull Screen", "0").as_deref(), Some("1"));
        assert_eq!(set_key(&mut t, "Display", "iSize H", "2208"), None);
        assert_eq!(set_key(&mut t, "General", "foo", "2").as_deref(), Some("1"));
        assert!(t.contains("iSize W=2064") && t.contains("bFull Screen=0") && t.contains("iSize H=2208") && t.contains("foo=2"));
        assert!(t.find("iSize H").unwrap() > t.find("[Display]").unwrap());
    }
}
