//! Polling rate — feature report `0x06`.
//!
//! Wire value is a divider against 1000 Hz: `1000 / hz`. Byte 4 is the one's
//! complement of byte 3. This is the one configuration report whose physical
//! effect has been verified on this receiver (see `docs/HARDWARE_VALIDATION.md`).

use serde::{Deserialize, Serialize};

pub const POLLING_REPORT_ID: u8 = 0x06;
pub const POLLING_REPORT_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PollingRate {
    Hz125 = 125,
    Hz250 = 250,
    Hz500 = 500,
    Hz1000 = 1000,
}

impl PollingRate {
    pub fn from_hz(hz: u32) -> Self {
        match hz {
            125 => Self::Hz125,
            250 => Self::Hz250,
            500 => Self::Hz500,
            _ => Self::Hz1000,
        }
    }

    pub fn hz(self) -> u32 {
        self as u32
    }

    pub fn divider(self) -> u8 {
        // Divide in u32: 1000 does not fit in u8, and 1000/125 == 8.
        (1000 / self.hz()) as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingRateConfig {
    pub rate: PollingRate,
}

pub fn build_polling_rate(cfg: &PollingRateConfig) -> Vec<u8> {
    let mut buf = vec![0u8; POLLING_REPORT_LEN];
    buf[0] = POLLING_REPORT_ID;
    buf[1] = POLLING_REPORT_LEN as u8;
    buf[2] = 0x01;
    buf[3] = cfg.rate.divider();
    buf[4] = !buf[3];
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divides_against_one_thousand_hertz() {
        let report = build_polling_rate(&PollingRateConfig { rate: PollingRate::Hz125 });
        assert_eq!(report, vec![0x06, 0x09, 0x01, 0x08, 0xF7, 0, 0, 0, 0]);

        let report = build_polling_rate(&PollingRateConfig { rate: PollingRate::Hz1000 });
        assert_eq!(report, vec![0x06, 0x09, 0x01, 0x01, 0xFE, 0, 0, 0, 0]);
    }

    #[test]
    fn every_rate_has_a_valid_complement() {
        for hz in [125, 250, 500, 1000] {
            let report = build_polling_rate(&PollingRateConfig {
                rate: PollingRate::from_hz(hz),
            });
            assert_eq!(report[3] ^ report[4], 0xFF);
        }
    }
}
