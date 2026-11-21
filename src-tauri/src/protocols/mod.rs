pub mod custom_macro;
pub mod dpi;
pub mod macros;
pub mod polling_rate;
pub mod status;
pub mod user_prefs;

/// The mandatory preamble sent before every configuration report.
///
/// The receiver's descriptor declares report `0x0C` as six bytes, but the
/// vendor software sends ten and the device acknowledges it. hidraw passes the
/// full report to the USB stack; the firmware, not the descriptor, decides what
/// it accepts. Without this preamble a configuration write is acknowledged and
/// then silently ignored.
pub const X3_PREAMBLE: [u8; 10] = [0x0C, 0x0A, 0x01, 0xFE, 0x01, 0xFE, 0, 0, 0, 0];

/// Delay the vendor software keeps between the preamble and the report, and
/// between consecutive reports.
pub const X3_WRITE_DELAY: std::time::Duration = std::time::Duration::from_millis(120);

/// Whether the device is connected via USB-C wired or 2.4GHz wireless adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ConnectionMode {
    Wired,
    Wireless,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preamble_is_the_vendor_sequence() {
        assert_eq!(
            X3_PREAMBLE,
            [0x0C, 0x0A, 0x01, 0xFE, 0x01, 0xFE, 0, 0, 0, 0]
        );
        assert_eq!(X3_PREAMBLE[1] as usize, X3_PREAMBLE.len());
    }
}
