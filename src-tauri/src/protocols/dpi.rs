//! DPI + sensor toggles — feature report `0x04`.
//!
//! The report is **atomic**: it carries eight fixed DPI slots, each slot's LED
//! colour, the active slot and the four sensor toggles. There is no readback,
//! so every write necessarily replaces the whole block.
//!
//! Layout recovered from the vendor software and from the upstream `asctl`
//! project, then checked against the receiver's own descriptor:
//!
//! ```text
//! offset  bytes  meaning
//! 0       1      report ID 0x04
//! 1       1      declared length (52 on FA60; upstream vendor builds 56)
//! 2       1      command/profile, always 0x01
//! 3       1      lift-off distance: 0 = 1 mm, 1 = 2 mm
//! 4       1      ripple control
//! 5       1      enabled-slot bitmask
//! 6       1      angle snap
//! 7       1      motion sync
//! 8..15   8      DPI low bytes, slots 0..7
//! 16..23  8      DPI high bytes, slots 0..7
//! 24      1      active slot + 1
//! 25..48  24     8 × RGB triplets
//! 49      1      constant 0x01
//! 50..51  2      checksum, big-endian, sum of bytes 3..=49
//! 52..55  —      zero padding in the vendor's 56-byte logical buffer
//! ```
//!
//! The receiver's HID descriptor declares report `0x04` as 52 bytes: the
//! vendor's trailing four zero bytes are not part of the USB report. We send
//! 52 and set the length byte to 52.
//!
//! DPI encoding is `wire = dpi / 50 - 1`, stored as a little-endian 16-bit
//! value split across two byte planes. It is **not** the X11 lookup table.

use super::ConnectionMode;
use serde::{Deserialize, Serialize};

pub const DPI_REPORT_ID: u8 = 0x04;
pub const DPI_REPORT_LEN: usize = 52;
pub const MAX_STAGES: usize = 6;
pub const DPI_MAX: u32 = 26_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LedMode {
    Off = 0x00,
    Blink = 0x01,
    Flash = 0x02,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpiConfig {
    pub dpi_values: [u32; MAX_STAGES],
    /// 1..=6, as written to byte 24.
    pub active_stage: u8,
    pub angle_snap: bool,
    pub ripple_control: bool,
    #[serde(default)]
    pub lod_2mm: bool,
    #[serde(default)]
    pub motion_sync: bool,
    pub stage_colors: [RgbColor; MAX_STAGES],
    pub led_mode: LedMode,
}

impl Default for DpiConfig {
    fn default() -> Self {
        Self {
            dpi_values: [800, 1600, 2400, 3200, 5000, 26_000],
            active_stage: 2,
            angle_snap: false,
            ripple_control: true,
            lod_2mm: false,
            motion_sync: false,
            stage_colors: [
                RgbColor { r: 0xFF, g: 0x00, b: 0x00 },
                RgbColor { r: 0x00, g: 0xFF, b: 0x00 },
                RgbColor { r: 0x00, g: 0x00, b: 0xFF },
                RgbColor { r: 0xFF, g: 0xFF, b: 0x00 },
                RgbColor { r: 0x00, g: 0xFF, b: 0xFF },
                RgbColor { r: 0xFF, g: 0x00, b: 0xFF },
            ],
            led_mode: LedMode::Off,
        }
    }
}

/// Encode a DPI value as the 16-bit wire quantity.
///
/// 50 DPI is the smallest representable step; 26,000 DPI is the largest the
/// sensor family advertises. Values are rounded down to a 50 DPI step, exactly
/// like the vendor UI.
pub fn encode_dpi_wire(dpi: u32) -> u16 {
    let steps = dpi / 50;
    steps.saturating_sub(1) as u16
}

/// Decode a wire quantity back to DPI (used by diagnostics and tests).
pub fn decode_dpi_wire(wire: u16) -> u32 {
    (wire as u32 + 1) * 50
}

pub fn build_dpi(cfg: &DpiConfig, _mode: ConnectionMode) -> Vec<u8> {
    let mut buf = vec![0u8; DPI_REPORT_LEN];
    buf[0] = DPI_REPORT_ID;
    buf[1] = DPI_REPORT_LEN as u8;
    buf[2] = 0x01;

    buf[3] = u8::from(cfg.lod_2mm);
    buf[4] = u8::from(cfg.ripple_control);
    buf[6] = u8::from(cfg.angle_snap);
    buf[7] = u8::from(cfg.motion_sync);

    // All six slots exposed by the UI are enabled.
    buf[5] = (1u8 << MAX_STAGES) - 1;

    for (index, dpi) in cfg.dpi_values.iter().enumerate() {
        let wire = encode_dpi_wire((*dpi).min(DPI_MAX));
        buf[8 + index] = (wire & 0xFF) as u8;
        buf[16 + index] = (wire >> 8) as u8;
    }

    buf[24] = cfg.active_stage.clamp(1, MAX_STAGES as u8);

    for (index, colour) in cfg.stage_colors.iter().enumerate() {
        let offset = 25 + index * 3;
        buf[offset] = colour.r;
        buf[offset + 1] = colour.g;
        buf[offset + 2] = colour.b;
    }

    buf[49] = 0x01;

    let checksum: u16 = buf[3..=49].iter().map(|&b| b as u16).sum();
    buf[50] = (checksum >> 8) as u8;
    buf[51] = (checksum & 0xFF) as u8;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_the_descriptor_length_and_length_byte() {
        let report = build_dpi(&DpiConfig::default(), ConnectionMode::Wireless);
        assert_eq!(report.len(), 52);
        assert_eq!(report[0], 0x04);
        assert_eq!(report[1], 52);
        assert_eq!(report[2], 0x01);
    }

    #[test]
    fn dpi_uses_the_fifty_step_minus_one_encoding() {
        assert_eq!(encode_dpi_wire(50), 0);
        assert_eq!(encode_dpi_wire(400), 7);
        assert_eq!(encode_dpi_wire(800), 15);
        assert_eq!(encode_dpi_wire(3200), 63);
        assert_eq!(encode_dpi_wire(26_000), 519);
        assert_eq!(decode_dpi_wire(519), 26_000);
    }

    #[test]
    fn dpi_is_split_across_two_byte_planes() {
        let mut cfg = DpiConfig::default();
        cfg.dpi_values = [400, 800, 3200, 26_000, 1600, 2400];
        let report = build_dpi(&cfg, ConnectionMode::Wireless);
        assert_eq!(report[8], 7); // 400 DPI
        assert_eq!(report[9], 15); // 800 DPI
        assert_eq!(report[10], 63); // 3200 DPI
        assert_eq!(report[11], (519u16 & 0xFF) as u8); // 26000 DPI low
        assert_eq!(report[16], 0);
        assert_eq!(report[19], (519 >> 8) as u8); // 26000 DPI high
    }

    #[test]
    fn active_stage_is_one_based() {
        let mut cfg = DpiConfig::default();
        cfg.active_stage = 1;
        assert_eq!(build_dpi(&cfg, ConnectionMode::Wireless)[24], 1);
        cfg.active_stage = 6;
        assert_eq!(build_dpi(&cfg, ConnectionMode::Wireless)[24], 6);
    }

    #[test]
    fn enabled_mask_covers_six_slots() {
        let report = build_dpi(&DpiConfig::default(), ConnectionMode::Wireless);
        assert_eq!(report[5], 0b0011_1111);
    }

    #[test]
    fn checksum_covers_bytes_three_to_forty_nine() {
        let report = build_dpi(&DpiConfig::default(), ConnectionMode::Wireless);
        let expected: u16 = report[3..=49].iter().map(|&b| b as u16).sum();
        assert_eq!(report[50], (expected >> 8) as u8);
        assert_eq!(report[51], (expected & 0xFF) as u8);
    }

    #[test]
    fn toggles_land_on_the_documented_offsets() {
        let mut cfg = DpiConfig::default();
        cfg.lod_2mm = true;
        cfg.ripple_control = false;
        cfg.angle_snap = true;
        cfg.motion_sync = true;
        let report = build_dpi(&cfg, ConnectionMode::Wireless);
        assert_eq!(report[3], 1);
        assert_eq!(report[4], 0);
        assert_eq!(report[6], 1);
        assert_eq!(report[7], 1);
    }
}
