//! Lighting + power + debounce — feature report `0x05`.
//!
//! One report carries three unrelated settings because that is how the vendor
//! firmware groups them: LED mode/colour, sleep timers and the button debounce
//! window. Writing lighting therefore also rewrites the timers and debounce,
//! so all three are always taken from the current configuration state.
//!
//! Layout (the receiver declares 13 bytes; the vendor's logical buffer is 15
//! with two trailing zero bytes):
//!
//! ```text
//! 0      0x05
//! 1      declared length (13 on FA60)
//! 2      0x01
//! 3      mode << 4
//! 4      (deep-sleep high nibble) | (9 - speed)
//! 5      (deep-sleep low nibble) << 4 | brightness
//! 6..8   R, G, B
//! 9      sleep minutes (1..60)
//! 10     debounce in 2 ms units (2..25)
//! 11..12 checksum, big-endian, sum of bytes 3..=10
//! ```
//!
//! `speed` on the wire is 4..8 and is inverted (`9 - speed`), so a UI speed of
//! 1 (slowest) maps to wire 8 and a UI speed of 5 (fastest) maps to wire 4.

use super::ConnectionMode;
use serde::{Deserialize, Serialize};

pub const LIGHT_REPORT_ID: u8 = 0x05;
pub const LIGHT_REPORT_LEN: usize = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LightMode {
    Off = 0x00,
    Static = 0x10,
    Breathing = 0x20,
    Neon = 0x30,
    ColorBreathing = 0x40,
    StaticDpi = 0x50,
    BreathingDpi = 0x60,
}

impl LightMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x10 => Self::Static,
            0x20 => Self::Breathing,
            0x30 => Self::Neon,
            0x40 => Self::ColorBreathing,
            0x50 => Self::StaticDpi,
            0x60 => Self::BreathingDpi,
            _ => Self::Off,
        }
    }

    pub fn uses_brightness(self) -> bool {
        matches!(self, Self::Static | Self::StaticDpi)
    }
}

pub use super::dpi::RgbColor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPrefsConfig {
    pub light_mode: LightMode,
    pub rgb: RgbColor,
    /// UI speed, 1 (slow) ..= 5 (fast).
    pub led_speed: u8,
    /// Sleep timer in minutes (1..60).
    pub sleep_time: f32,
    /// Deep-sleep timer in minutes (1..60).
    pub deep_sleep_time: u32,
    /// Button debounce in effective milliseconds (4..50).
    pub key_response: u8,
    /// LED brightness, 1..=8.
    pub brightness: u8,
}

impl Default for UserPrefsConfig {
    fn default() -> Self {
        Self {
            light_mode: LightMode::Off,
            rgb: RgbColor { r: 0x00, g: 0xFF, b: 0x00 },
            led_speed: 3,
            sleep_time: 10.0,
            deep_sleep_time: 10,
            key_response: 8,
            brightness: 8,
        }
    }
}

pub fn build_user_prefs(cfg: &UserPrefsConfig, _mode: ConnectionMode) -> Vec<u8> {
    let mut buf = vec![0u8; LIGHT_REPORT_LEN];
    buf[0] = LIGHT_REPORT_ID;
    buf[1] = LIGHT_REPORT_LEN as u8;
    buf[2] = 0x01;

    buf[3] = cfg.light_mode as u8;

    let deep = cfg.deep_sleep_time.clamp(1, 60) as u8;
    let speed = cfg.led_speed.clamp(1, 5) + 3; // 1..5 -> wire 4..8
    buf[4] = (deep & 0xF0) | (9 - speed);

    let brightness = if cfg.light_mode.uses_brightness() {
        cfg.brightness.clamp(1, 8)
    } else {
        8
    };
    buf[5] = ((deep & 0x0F) << 4) | brightness;

    buf[6] = cfg.rgb.r;
    buf[7] = cfg.rgb.g;
    buf[8] = cfg.rgb.b;

    buf[9] = cfg.sleep_time.round().clamp(1.0, 60.0) as u8;

    // The wire counts 2 ms units; the UI exposes effective milliseconds.
    buf[10] = (cfg.key_response.clamp(4, 50) / 2).clamp(2, 25);

    let checksum: u16 = buf[3..=10].iter().map(|&b| b as u16).sum();
    buf[11] = (checksum >> 8) as u8;
    buf[12] = (checksum & 0xFF) as u8;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_the_descriptor_length() {
        let report = build_user_prefs(&UserPrefsConfig::default(), ConnectionMode::Wireless);
        assert_eq!(report.len(), 13);
        assert_eq!(report[0], 0x05);
        assert_eq!(report[1], 13);
        assert_eq!(report[2], 0x01);
    }

    #[test]
    fn mode_is_shifted_into_the_high_nibble() {
        let mut cfg = UserPrefsConfig::default();
        cfg.light_mode = LightMode::BreathingDpi;
        assert_eq!(build_user_prefs(&cfg, ConnectionMode::Wireless)[3], 0x60);
    }

    #[test]
    fn speed_is_inverted_and_offset_to_four_through_eight() {
        let mut cfg = UserPrefsConfig::default();
        cfg.led_speed = 1;
        assert_eq!(build_user_prefs(&cfg, ConnectionMode::Wireless)[4] & 0x0F, 5);
        cfg.led_speed = 5;
        assert_eq!(build_user_prefs(&cfg, ConnectionMode::Wireless)[4] & 0x0F, 1);
    }

    #[test]
    fn deep_sleep_is_split_across_two_nibbles() {
        let mut cfg = UserPrefsConfig::default();
        cfg.deep_sleep_time = 25; // 0x19
        let report = build_user_prefs(&cfg, ConnectionMode::Wireless);
        assert_eq!(report[4] >> 4, 0x1);
        assert_eq!(report[5] >> 4, 0x9);
    }

    #[test]
    fn debounce_is_stored_in_two_millisecond_units() {
        let mut cfg = UserPrefsConfig::default();
        cfg.key_response = 10;
        assert_eq!(build_user_prefs(&cfg, ConnectionMode::Wireless)[10], 5);
        cfg.key_response = 50;
        assert_eq!(build_user_prefs(&cfg, ConnectionMode::Wireless)[10], 25);
    }

    #[test]
    fn checksum_covers_bytes_three_to_ten() {
        let report = build_user_prefs(&UserPrefsConfig::default(), ConnectionMode::Wireless);
        let expected: u16 = report[3..=10].iter().map(|&b| b as u16).sum();
        assert_eq!(report[11], (expected >> 8) as u8);
        assert_eq!(report[12], (expected & 0xFF) as u8);
    }
}
