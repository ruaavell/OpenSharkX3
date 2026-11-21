use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FirmwareAction {
    DisableButton   = 0x01,
    LeftClick       = 0x02,
    RightClick      = 0x03,
    MiddleClick     = 0x04,
    Backward        = 0x05,
    Forward         = 0x06,
    DoubleClick     = 0x07,
    Fire            = 0x08,
    ScrollUp        = 0x09,
    ScrollDown      = 0x0a,
    DpiCycle        = 0x0d,
    DpiPlus         = 0x0e,
    DpiMinus        = 0x0f,
    EasyAim         = 0x10,
    Keyboard        = 0x11,
    CustomMacro     = 0x12,
    MediaPlayer     = 0x15,
    PreviousTrack   = 0x16,
    NextTrack       = 0x17,
    PlayPause       = 0x18,
    Stop            = 0x19,
    Mute            = 0x1a,
    VolPlus         = 0x1b,
    VolMinus        = 0x1c,
    Calculator      = 0x1d,
    Email           = 0x1e,
    BrowserForward  = 0x20,
    BrowserBackward = 0x21,
    BrowserStop     = 0x22,
    MyComputer      = 0x23,
    BrowserRefresh  = 0x24,
    BrowserHome     = 0x25,
    BrowserSearch   = 0x26,
}

pub const MOD_NONE : u8 = 0x00;
pub const MOD_CTRL : u8 = 0x01;
pub const MOD_SHIFT: u8 = 0x02;
pub const MOD_ALT  : u8 = 0x04;
pub const MOD_WIN  : u8 = 0x08;

pub type MacroTuple = (FirmwareAction, u8, u8);

/// All macro templates, matching macroTemplates in MacrosBuilder.ts
pub static MACRO_TEMPLATES: std::sync::LazyLock<HashMap<&'static str, MacroTuple>> =
    std::sync::LazyLock::new(|| {
        use FirmwareAction::*;
        let mut m: HashMap<&'static str, MacroTuple> = HashMap::new();
        m.insert("global-disable-button",     (DisableButton,   MOD_NONE,              0x00));
        m.insert("global-left-click",          (LeftClick,       MOD_NONE,              0x00));
        m.insert("global-right-click",         (RightClick,      MOD_NONE,              0x00));
        m.insert("global-middle",              (MiddleClick,     MOD_NONE,              0x00));
        m.insert("global-backward",            (Backward,        MOD_NONE,              0x00));
        m.insert("global-forward",             (Forward,         MOD_NONE,              0x00));
        m.insert("global-double-click",        (DoubleClick,     MOD_NONE,              0x00));
        m.insert("global-fire-button",         (Fire,            MOD_NONE,              0x00));
        m.insert("global-scroll-up",           (ScrollUp,        MOD_NONE,              0x00));
        m.insert("global-scroll-down",         (ScrollDown,      MOD_NONE,              0x00));
        m.insert("global-easy-aim",            (EasyAim,         MOD_NONE,              0x03));
        m.insert("global-dpi-cycle",           (DpiCycle,        MOD_NONE,              0x00));
        m.insert("global-dpi-+",               (DpiPlus,         MOD_NONE,              0x00));
        m.insert("global-dpi--",               (DpiMinus,        MOD_NONE,              0x00));
        m.insert("multimedia-media-player",    (MediaPlayer,     MOD_NONE,              0x00));
        m.insert("multimedia-play-pause",      (PlayPause,       MOD_NONE,              0x00));
        m.insert("multimedia-stop-music",      (Stop,            MOD_NONE,              0x00));
        m.insert("multimedia-previous-track",  (PreviousTrack,   MOD_NONE,              0x00));
        m.insert("multimedia-next-track",      (NextTrack,       MOD_NONE,              0x00));
        m.insert("multimedia-volume-+",        (VolPlus,         MOD_NONE,              0x00));
        m.insert("multimedia-volume--",        (VolMinus,        MOD_NONE,              0x00));
        m.insert("multimedia-mute",            (Mute,            MOD_NONE,              0x00));
        m.insert("browser-home",               (BrowserHome,     MOD_NONE,              0x00));
        m.insert("browser-favorites",          (Keyboard,        MOD_CTRL | MOD_SHIFT,  0x12)); // Ctrl+Shift+O
        m.insert("browser-forward",            (BrowserForward,  MOD_NONE,              0x00));
        m.insert("browser-backward",           (BrowserBackward, MOD_NONE,              0x00));
        m.insert("browser-stop",               (BrowserStop,     MOD_NONE,              0x00));
        m.insert("browser-refresh",            (BrowserRefresh,  MOD_NONE,              0x00));
        m.insert("browser-search",             (BrowserSearch,   MOD_NONE,              0x00));
        m.insert("browser-email",              (Email,           MOD_NONE,              0x00));
        m.insert("browser-calculator",         (Calculator,      MOD_NONE,              0x00));
        m.insert("browser-my-computer",        (MyComputer,      MOD_NONE,              0x00));
        m.insert("shortcut-cut",               (Keyboard,        MOD_CTRL,              0x1b)); // X
        m.insert("shortcut-copy",              (Keyboard,        MOD_CTRL,              0x06)); // C
        m.insert("shortcut-paste",             (Keyboard,        MOD_CTRL,              0x19)); // V
        m.insert("shortcut-open",              (Keyboard,        MOD_CTRL,              0x12)); // O
        m.insert("shortcut-save",              (Keyboard,        MOD_CTRL,              0x16)); // S
        m.insert("shortcut-find",              (Keyboard,        MOD_CTRL,              0x09)); // F
        m.insert("shortcut-undo",              (Keyboard,        MOD_CTRL,              0x1d)); // Z
        m.insert("shortcut-redo",              (Keyboard,        MOD_CTRL,              0x1c)); // Y
        m.insert("shortcut-select-all",        (Keyboard,        MOD_CTRL,              0x04)); // A
        m.insert("shortcut-print",             (Keyboard,        MOD_CTRL,              0x13)); // P
        m.insert("shortcut-close-window",      (Keyboard,        MOD_ALT,               0x3d)); // F4
        m.insert("shortcut-swap-window",       (Keyboard,        MOD_ALT,               0x2b)); // Tab
        m.insert("shortcut-show-desktop",      (Keyboard,        MOD_WIN,               0x07)); // D
        m.insert("shortcut-run-command",       (Keyboard,        MOD_WIN,               0x15)); // R
        m.insert("shortcut-lock-pc",           (Keyboard,        MOD_WIN,               0x0f)); // L
        m.insert("shortcut-screen-capture",    (Keyboard,        MOD_WIN | MOD_SHIFT,   0x16)); // S
        m.insert("custom-macro-left-button",   (CustomMacro,     MOD_NONE,              0x01));
        m.insert("custom-macro-right-button",  (CustomMacro,     MOD_NONE,              0x02));
        m.insert("custom-macro-middle-button", (CustomMacro,     MOD_NONE,              0x03));
        m.insert("custom-macro-extra-button_4",(CustomMacro,     MOD_NONE,              0x07));
        m.insert("custom-macro-extra-button_5",(CustomMacro,     MOD_NONE,              0x08));
        m
    });

/// UI button index → the vendor report's fixed entry index.
///
/// The X3 report has **18** entries. Only eight are driveable by this shell:
/// left, right, middle, DPI, forward and backward, plus the two scroll
/// directions. Entry 4 is the Bluetooth mode switch and is preserved from the
/// factory table rather than exposed in the UI.
const BUTTON_ENTRIES: [(usize, usize); 8] = [
    (0, 0),   // left
    (1, 1),   // right
    (2, 2),   // middle
    (3, 6),   // forward
    (4, 7),   // backward
    (5, 3),   // dpi
    (6, 17),  // scroll_up
    (7, 16),  // scroll_down
];

pub const BUTTON_REPORT_ID: u8 = 0x08;
pub const BUTTON_REPORT_LEN: usize = 59;
pub const BUTTON_ENTRIES_TOTAL: usize = 18;

/// Handlers that are never remapped by the UI but must survive an apply.
const FACTORY_MODE_SWITCH: u8 = 0x3C;
const SCROLL_DOWN: u8 = 0x0A;
const SCROLL_UP: u8 = 0x09;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEntry {
    pub action:   u8,
    pub modifier: u8,
    pub key_code: u8,
}

impl MacroEntry {
    pub fn from_template(name: &str) -> Self {
        let (a, m, k) = MACRO_TEMPLATES.get(name).copied()
            .unwrap_or((FirmwareAction::DisableButton, MOD_NONE, 0x00));
        Self { action: a as u8, modifier: m, key_code: k }
    }

    pub fn is_left_click(&self) -> bool {
        self.action == FirmwareAction::LeftClick as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacrosConfig {
    /// 8 entries: [left, right, middle, forward, backward, dpi, scroll_up, scroll_down]
    pub buttons: [MacroEntry; 8],
}

impl Default for MacrosConfig {
    fn default() -> Self {
        Self {
            buttons: [
                MacroEntry::from_template("global-left-click"),
                MacroEntry::from_template("global-right-click"),
                MacroEntry::from_template("global-middle"),
                MacroEntry::from_template("global-forward"),
                MacroEntry::from_template("global-backward"),
                MacroEntry::from_template("global-dpi-cycle"),
                MacroEntry::from_template("global-scroll-up"),
                MacroEntry::from_template("global-scroll-down"),
            ],
        }
    }
}

impl MacrosConfig {
    /// Refuse a mapping that removes every primary click. The vendor software
    /// enforces the same rule: without a left click the mouse is unusable.
    pub fn has_left_click(&self) -> bool {
        self.buttons.iter().any(MacroEntry::is_left_click)
    }
}

/// Build the 59-byte button report.
pub fn build_macros(cfg: &MacrosConfig) -> Vec<u8> {
    let mut buf = vec![0u8; BUTTON_REPORT_LEN];
    buf[0] = BUTTON_REPORT_ID;
    buf[1] = BUTTON_REPORT_LEN as u8;
    buf[2] = 0x01;

    // Every unassigned entry is "button off" (0x01 00 00), never zeros: a zero
    // action is accepted by the firmware but produces an undefined button.
    for entry in 0..BUTTON_ENTRIES_TOTAL {
        buf[3 + entry * 3] = FirmwareAction::DisableButton as u8;
    }

    // Factory-only entries that are not exposed by the UI.
    buf[3 + 4 * 3] = FACTORY_MODE_SWITCH; // Bluetooth mode switch
    buf[3 + 16 * 3] = SCROLL_DOWN;
    buf[3 + 17 * 3] = SCROLL_UP;

    for (ui_index, entry_index) in BUTTON_ENTRIES {
        let entry = &cfg.buttons[ui_index];
        let offset = 3 + entry_index * 3;
        buf[offset] = entry.action;
        buf[offset + 1] = entry.modifier;
        buf[offset + 2] = entry.key_code;
    }

    let checksum: u16 = buf[3..=56].iter().map(|&b| b as u16).sum();
    buf[57] = (checksum >> 8) as u8;
    buf[58] = (checksum & 0xFF) as u8;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_the_descriptor_length() {
        let report = build_macros(&MacrosConfig::default());
        assert_eq!(report.len(), 59);
        assert_eq!(report[0], 0x08);
        assert_eq!(report[1], 59);
        assert_eq!(report[2], 0x01);
    }

    #[test]
    fn default_mapping_keeps_every_physical_button() {
        let report = build_macros(&MacrosConfig::default());
        // entry 0 left, 1 right, 2 middle, 3 DPI, 4 mode, 6 forward, 7 backward,
        // 16 scroll down, 17 scroll up.
        assert_eq!(report[3], 0x02);
        assert_eq!(report[6], 0x03);
        assert_eq!(report[9], 0x04);
        assert_eq!(report[12], 0x0D);
        assert_eq!(report[15], 0x3C);
        assert_eq!(report[21], 0x06);
        assert_eq!(report[24], 0x05);
        assert_eq!(report[51], 0x0A);
        assert_eq!(report[54], 0x09);
    }

    #[test]
    fn checksum_covers_bytes_three_to_fifty_six() {
        let report = build_macros(&MacrosConfig::default());
        let expected: u16 = report[3..=56].iter().map(|&b| b as u16).sum();
        assert_eq!(report[57], (expected >> 8) as u8);
        assert_eq!(report[58], (expected & 0xFF) as u8);
    }

    #[test]
    fn detects_a_mapping_without_left_click() {
        let mut cfg = MacrosConfig::default();
        for button in &mut cfg.buttons {
            button.action = FirmwareAction::DisableButton as u8;
        }
        assert!(!cfg.has_left_click());
        cfg.buttons[0] = MacroEntry::from_template("global-left-click");
        assert!(cfg.has_left_click());
    }
}
