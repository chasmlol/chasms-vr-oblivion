//! The input bridge: the runtime's actions reach Oblivion as its own keyboard and mouse.
//!
//! The adapter is Oblivion's `dinput8.dll` ([`crate::proxy`]), so the DirectInput devices Oblivion
//! makes are hooked ([`on_direct_input`]): the keyboard (found by its type) reads the keys the
//! runtime's actions hold, the mouse its buttons. While VR is on (and always in the background test
//! mode) the real keyboard and mouse don't reach the game: the player is in the headset, and a
//! desk keyboard or mouse being used for something else mustn't walk the player off.
//!
//! Which key is which action is Oblivion's own setting (`Oblivion.ini` `[Controls]`, each
//! `00KKMMJJ`: keyboard key, mouse button, joystick button; 0xFF none), so a player's rebinding
//! still works.

use rustvr_adapter::actions::{Presses, held, pressed};
use rustvr_protocol::{Action, Intents};
use std::collections::HashMap;

/// A control's keyboard key (DirectInput code) and mouse button, from `Oblivion.ini`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Binding {
    pub key: Option<u8>,
    pub mouse: Option<u8>,
}

/// `[Controls]` of an `Oblivion.ini`: control name -> binding.
pub fn parse_controls(ini: &str) -> HashMap<String, Binding> {
    let mut out = HashMap::new();
    let mut section = false;
    for line in ini.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            section = t.eq_ignore_ascii_case("[Controls]");
            continue;
        }
        let Some((k, v)) = t.split_once('=') else { continue };
        if !section || v.len() != 8 || !v.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let byte = |i: usize| u8::from_str_radix(&v[i..i + 2], 16).ok().filter(|&b| b != 0xFF);
        out.insert(k.trim().to_owned(), Binding { key: byte(2), mouse: byte(4) });
    }
    out
}

/// Oblivion's defaults (its `Oblivion_default.ini`), for controls the player's file doesn't have.
pub fn default_controls() -> HashMap<String, Binding> {
    parse_controls(
        "[Controls]\nForward=0011FFFF\nBack=001FFFFF\nSlide Left=001EFFFF\nSlide Right=0020FFFF\nUse=00FF00FF\n\
         Activate=0039FFFF\nBlock=003801FF\nCast=002EFFFF\nReady Item=0021FFFF\nCrouch/Sneak=001DFFFF\nRun=002AFFFF\n\
         Always Run=003AFFFF\nAuto Move=0010FFFF\nJump=0012FFFF\nToggle POV=001302FF\nMenu Mode=000FFFFF\nRest=0014FFFF\n\
         Quick Menu=003BFFFF\nGrab=002CFFFF\n",
    )
}

pub const DIK_ESCAPE: u8 = 0x01;

/// What the game's keyboard and mouse read this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Virtual {
    pub keys: Vec<u8>,
    /// Mouse buttons held (bit per button).
    pub buttons: u8,
}

/// The stick pushed past this (of its travel) presses a direction's key.
const STICK_KEY: f32 = 0.35;

/// Turns the runtime's actions into Oblivion's keys and buttons.
pub struct Bridge {
    controls: HashMap<String, Binding>,
    presses: Presses,
    /// Keys held a little longer after a press (a game that polls mustn't miss a tap).
    pulses: Vec<(Binding, f64)>,
}

/// A press holds its key this long.
const PULSE: f64 = 0.12;

impl Bridge {
    pub fn new(controls: HashMap<String, Binding>) -> Bridge {
        let mut c = default_controls();
        c.extend(controls);
        Bridge { controls: c, presses: Presses::default(), pulses: Vec::new() }
    }

    fn binding(&self, name: &str) -> Binding {
        self.controls.get(name).copied().unwrap_or_default()
    }

    /// Holds the control `name` (as the player's file binds it) until `until` (seconds, the link's
    /// clock).
    pub fn hold(&mut self, name: &str, until: f64) {
        self.pulses.push((self.binding(name), until));
    }

    pub fn update(&mut self, it: &Intents, now: f64) -> Virtual {
        let counts = self.presses.update(it);
        let mut v = Virtual::default();
        let hold = |b: Binding, v: &mut Virtual| {
            if let Some(k) = b.key
                && !v.keys.contains(&k)
            {
                v.keys.push(k);
            }
            if let Some(m) = b.mouse {
                v.buttons |= 1 << m.min(7);
            }
        };
        // Walking: the stick as Oblivion's direction keys (the player faces where the head looks).
        let (x, y) = (it.move_xy[0], it.move_xy[1]);
        for (on, name) in [(y > STICK_KEY, "Forward"), (y < -STICK_KEY, "Back"), (x < -STICK_KEY, "Slide Left"), (x > STICK_KEY, "Slide Right")] {
            if on {
                hold(self.binding(name), &mut v);
            }
        }
        // Held while the action is.
        for (a, name) in [(Action::Crouch, "Crouch/Sneak"), (Action::Sprint, "Run")] {
            if held(it, a) {
                hold(self.binding(name), &mut v);
            }
        }
        // Pressed: a moment's tap.
        for (a, name) in [(Action::Jump, "Jump"), (Action::Use, "Activate"), (Action::Inventory, "Menu Mode"), (Action::UseHold, "Rest")] {
            if pressed(&counts, a) {
                self.pulses.push((self.binding(name), now + PULSE));
            }
        }
        // The laser's trigger on the game's menus: the left mouse button.
        if it.flags & rustvr_protocol::INT_CURSOR_DOWN != 0 {
            v.buttons |= 1;
        }
        if pressed(&counts, Action::Menu) {
            self.pulses.push((Binding { key: Some(DIK_ESCAPE), mouse: None }, now + PULSE));
        }
        self.pulses.retain(|&(_, until)| until > now);
        for &(b, _) in &self.pulses {
            hold(b, &mut v);
        }
        v
    }
}

#[cfg(windows)]
mod win {
    use crate::hooks::{guard, install};
    use std::ffi::c_void;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::*};

    const DEVTYPE_MOUSE: u32 = 0x12;
    const DEVTYPE_KEYBOARD: u32 = 0x13;
    const DI_OK: i32 = 0;
    const DISCL_NONEXCLUSIVE: u32 = 0x2;
    const DISCL_BACKGROUND: u32 = 0x8;
    /// `DIMOFS_BUTTON0`: the first mouse button's offset in its state and events.
    const MOUSE_BUTTON0: u32 = 12;

    static REAL_CREATE: AtomicUsize = AtomicUsize::new(0);
    static REAL_STATE: AtomicUsize = AtomicUsize::new(0);
    static REAL_DATA: AtomicUsize = AtomicUsize::new(0);
    static REAL_COOP: AtomicUsize = AtomicUsize::new(0);
    static REAL_ACQUIRE: AtomicUsize = AtomicUsize::new(0);
    static KEYBOARD: AtomicUsize = AtomicUsize::new(0);
    static MOUSE: AtomicUsize = AtomicUsize::new(0);
    /// The real devices don't reach the game (VR on, or background tests).
    static OURS_ONLY: AtomicBool = AtomicBool::new(false);

    struct State {
        v: super::Virtual,
        /// Keys and buttons the game's events have been told of.
        told_keys: Vec<u8>,
        told_buttons: u8,
        sequence: u32,
    }
    static STATE: Mutex<State> = Mutex::new(State { v: super::Virtual { keys: Vec::new(), buttons: 0 }, told_keys: Vec::new(), told_buttons: 0, sequence: 0x4000_0000 });

    pub fn set(v: super::Virtual, ours_only: bool) {
        OURS_ONLY.store(ours_only, Release);
        let mut s = STATE.lock().unwrap();
        if s.v != v {
            tracing::info!("input: keys {:02x?}, mouse buttons {:#x}", v.keys, v.buttons);
            s.v = v;
        }
    }

    type CreateDeviceFn = unsafe extern "system" fn(*mut c_void, *const [u8; 16], *mut *mut c_void, *mut c_void) -> i32;
    type GetCapsFn = unsafe extern "system" fn(*mut c_void, *mut u32) -> i32;
    type GetStateFn = unsafe extern "system" fn(*mut c_void, u32, *mut u8) -> i32;
    type GetDataFn = unsafe extern "system" fn(*mut c_void, u32, *mut u8, *mut u32, u32) -> i32;
    type CoopFn = unsafe extern "system" fn(*mut c_void, *mut c_void, u32) -> i32;
    type AcquireFn = unsafe extern "system" fn(*mut c_void) -> i32;

    fn method(object: *mut c_void, slot: usize) -> *const () {
        unsafe { *(*(object as *const *const *const ())).add(slot) }
    }

    fn real<T>(slot: &AtomicUsize) -> T {
        unsafe { std::mem::transmute_copy(&slot.load(Acquire)) }
    }

    fn ours(dev: *mut c_void) -> bool {
        let d = dev as usize;
        d != 0 && (d == KEYBOARD.load(Acquire) || d == MOUSE.load(Acquire))
    }

    /// DirectInput made (`IDirectInput8`): its `CreateDevice` hooked, once.
    pub fn on_direct_input(di: *mut c_void) {
        if di.is_null() || REAL_CREATE.load(Acquire) != 0 {
            return;
        }
        if let Some(o) = install(method(di, 3), create_device as *const (), "IDirectInput8::CreateDevice") {
            REAL_CREATE.store(o as usize, Release);
        }
    }

    unsafe extern "system" fn create_device(di: *mut c_void, guid: *const [u8; 16], out: *mut *mut c_void, outer: *mut c_void) -> i32 {
        let hr = unsafe { real::<CreateDeviceFn>(&REAL_CREATE)(di, guid, out, outer) };
        guard("CreateDevice", (), || {
            if hr < 0 || out.is_null() || unsafe { (*out).is_null() } {
                return;
            }
            let dev = unsafe { *out };
            // `DIDEVCAPS`: 11 words, the type the third.
            let mut caps = [0u32; 11];
            caps[0] = 44;
            let get_caps: GetCapsFn = unsafe { std::mem::transmute(method(dev, 3)) };
            if unsafe { get_caps(dev, caps.as_mut_ptr()) } < 0 {
                return;
            }
            let slot = match caps[2] & 0xFF {
                DEVTYPE_KEYBOARD => &KEYBOARD,
                DEVTYPE_MOUSE => &MOUSE,
                _ => return,
            };
            slot.store(dev as usize, Release);
            tracing::info!("input: the game's {} {dev:p}", if caps[2] & 0xFF == DEVTYPE_KEYBOARD { "keyboard" } else { "mouse" });
            // (The keyboard and the mouse share one implementation: each method is hooked once.)
            for (slot, hook, real, name) in [
                (9, get_state as *const (), &REAL_STATE, "IDirectInputDevice8::GetDeviceState"),
                (10, get_data as *const (), &REAL_DATA, "IDirectInputDevice8::GetDeviceData"),
                (13, set_coop as *const (), &REAL_COOP, "IDirectInputDevice8::SetCooperativeLevel"),
                (7, acquire as *const (), &REAL_ACQUIRE, "IDirectInputDevice8::Acquire"),
            ] {
                if real.load(Acquire) == 0
                    && let Some(o) = install(method(dev, slot), hook, name)
                {
                    real.store(o as usize, Release);
                }
            }
        });
        hr
    }

    /// Ours work without the window in front (and never take the devices for themselves).
    unsafe extern "system" fn set_coop(dev: *mut c_void, wnd: *mut c_void, flags: u32) -> i32 {
        let flags = if ours(dev) { DISCL_NONEXCLUSIVE | DISCL_BACKGROUND } else { flags };
        unsafe { real::<CoopFn>(&REAL_COOP)(dev, wnd, flags) }
    }

    unsafe extern "system" fn acquire(dev: *mut c_void) -> i32 {
        let hr = unsafe { real::<AcquireFn>(&REAL_ACQUIRE)(dev) };
        if ours(dev) && hr < 0 { DI_OK } else { hr }
    }

    unsafe extern "system" fn get_state(dev: *mut c_void, size: u32, data: *mut u8) -> i32 {
        let hr = unsafe { real::<GetStateFn>(&REAL_STATE)(dev, size, data) };
        guard("GetDeviceState", hr, || {
            if !ours(dev) || data.is_null() {
                return hr;
            }
            let only = OURS_ONLY.load(Acquire);
            if hr < 0 || only {
                unsafe { std::ptr::write_bytes(data, 0, size as usize) };
            }
            let s = STATE.lock().unwrap();
            if dev as usize == KEYBOARD.load(Acquire) && size >= 256 {
                for &k in &s.v.keys {
                    unsafe { *data.add(k as usize) = 0x80 };
                }
            } else if dev as usize == MOUSE.load(Acquire) && size >= 16 {
                for b in 0..((size - MOUSE_BUTTON0) as usize).min(8) {
                    if s.v.buttons & (1 << b) != 0 {
                        unsafe { *data.add(MOUSE_BUTTON0 as usize + b) = 0x80 };
                    }
                }
            }
            DI_OK
        })
    }

    unsafe extern "system" fn get_data(dev: *mut c_void, stride: u32, data: *mut u8, count: *mut u32, flags: u32) -> i32 {
        let room = if count.is_null() { 0 } else { unsafe { *count } };
        let hr = unsafe { real::<GetDataFn>(&REAL_DATA)(dev, stride, data, count, flags) };
        guard("GetDeviceData", hr, || {
            const DIGDD_PEEK: u32 = 1;
            if !ours(dev) || count.is_null() || data.is_null() || stride < 16 || flags & DIGDD_PEEK != 0 {
                return hr;
            }
            let mut n = if hr < 0 || OURS_ONLY.load(Acquire) { 0 } else { unsafe { *count } };
            let mut s = STATE.lock().unwrap();
            let s = &mut *s;
            // The changes since the game was last told: (offset, down).
            let mut changes: Vec<(u32, bool)> = Vec::new();
            if dev as usize == KEYBOARD.load(Acquire) {
                changes.extend(s.v.keys.iter().filter(|k| !s.told_keys.contains(k)).map(|&k| (k as u32, true)));
                changes.extend(s.told_keys.iter().filter(|k| !s.v.keys.contains(k)).map(|&k| (k as u32, false)));
                s.told_keys = s.v.keys.clone();
            } else {
                let diff = s.v.buttons ^ s.told_buttons;
                for b in 0..8 {
                    if diff & (1 << b) != 0 {
                        changes.push((MOUSE_BUTTON0 + b, s.v.buttons & (1 << b) != 0));
                    }
                }
                s.told_buttons = s.v.buttons;
            }
            let time = unsafe { windows::Win32::System::SystemInformation::GetTickCount() };
            for (offset, down) in changes {
                if n >= room {
                    break;
                }
                let at = unsafe { data.add((n * stride) as usize) };
                s.sequence = s.sequence.wrapping_add(1);
                unsafe {
                    std::ptr::write_bytes(at, 0, stride as usize);
                    (at as *mut u32).write_unaligned(offset);
                    (at.add(4) as *mut u32).write_unaligned(if down { 0x80 } else { 0 });
                    (at.add(8) as *mut u32).write_unaligned(time);
                    (at.add(12) as *mut u32).write_unaligned(s.sequence);
                }
                n += 1;
            }
            unsafe { *count = n };
            DI_OK
        })
    }
}

#[cfg(windows)]
pub use win::{on_direct_input, set};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_oblivions_bindings() {
        let c = parse_controls("[General]\nForward=00AAFFFF\n[Controls]\nForward=0011FFFF\nUse=00FF00FF\nBlock=003801FF\nbInvertYValues=0\n");
        assert_eq!(c["Forward"], Binding { key: Some(0x11), mouse: None });
        assert_eq!(c["Use"], Binding { key: None, mouse: Some(0) });
        assert_eq!(c["Block"], Binding { key: Some(0x38), mouse: Some(1) });
        assert!(!c.contains_key("bInvertYValues"));
        let d = default_controls();
        assert_eq!(d["Activate"].key, Some(0x39));
        assert_eq!(d["Jump"].key, Some(0x12));
    }

    #[test]
    fn the_stick_walks_and_a_press_taps() {
        let mut b = Bridge::new(HashMap::new());
        let mut it: Intents = bytemuck::Zeroable::zeroed();
        it.move_xy = [0.0, 0.8];
        assert_eq!(b.update(&it, 0.0).keys, vec![0x11]);
        it.move_xy = [0.0, 0.0];
        it.presses[Action::Jump as usize] = 1;
        let v = b.update(&it, 0.1);
        assert_eq!(v.keys, vec![0x12]);
        assert!(b.update(&it, 0.3).keys.is_empty());
    }
}
