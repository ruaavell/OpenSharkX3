//! Device → host status events for the Attack Shark X3.
//!
//! The configuration interface is a **write-only** channel: there is no
//! reliable `GET_FEATURE` readback. The device does volunteer a small set of
//! input reports on the same interface, though, and those are the only live
//! state the UI can honestly display.
//!
//! Frame on 2.4 GHz / USB (input report `0x03`, five bytes):
//!
//! ```text
//! 03 <code-lo> <code-hi> <value-lo> <value-hi>
//! ```
//!
//! Bluetooth FEE4 notifications carry the same four payload bytes without the
//! leading report ID.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    DpiStage,
    PollingRate,
    Battery,
    WriteAck,
    BatteryLevel,
    LightMode,
    Profile,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    pub kind: EventKind,
    pub code: u16,
    pub value: u16,
    /// 1-based DPI stage, when this is a `0x1000` event.
    pub dpi_stage: Option<u8>,
    /// Battery percentage in 10% steps, when this is a battery event.
    pub battery_percent: Option<u8>,
    /// `true` when a `0x4010` event reports the sleep flag (low byte 0).
    pub asleep: Option<bool>,
    /// Report ID acknowledged by a `0x5010` event.
    pub acknowledged_report: Option<u8>,
    pub raw: Vec<u8>,
}

impl StatusEvent {
    pub fn describe(&self) -> String {
        match self.kind {
            EventKind::DpiStage => format!(
                "DPI stage changed to {}",
                self.dpi_stage.unwrap_or_default()
            ),
            EventKind::PollingRate => {
                format!("polling-rate index changed to {}", self.value)
            }
            EventKind::Battery => {
                let level = self
                    .battery_percent
                    .map(|p| format!("{p}%"))
                    .unwrap_or_else(|| format!("invalid({})", self.value >> 8));
                let sleep = if self.asleep.unwrap_or(false) {
                    "asleep"
                } else {
                    "awake"
                };
                format!("battery {level}, {sleep}")
            }
            EventKind::WriteAck => format!(
                "write acknowledged for report 0x{:02X}",
                self.acknowledged_report.unwrap_or(0)
            ),
            EventKind::BatteryLevel => self
                .battery_percent
                .map(|p| format!("battery level {p}%"))
                .unwrap_or_else(|| format!("battery-level event outside 1..10: {}", self.value)),
            EventKind::LightMode => format!("light mode changed to {}", self.value),
            EventKind::Profile => format!("profile changed to {}", self.value),
            EventKind::Unknown => format!(
                "unknown event 0x{:04X}, value 0x{:04X}",
                self.code, self.value
            ),
        }
    }
}

fn decode(code: u16, value: u16, raw: Vec<u8>) -> StatusEvent {
    let (kind, dpi_stage, battery_percent, asleep, acknowledged_report) = match code {
        0x1000 => (EventKind::DpiStage, u8::try_from(value).ok(), None, None, None),
        0x2000 => (EventKind::PollingRate, None, None, None, None),
        0x4010 => {
            let level = (value >> 8) as u8;
            let percent = if (1..=10).contains(&level) {
                Some(level * 10)
            } else {
                None
            };
            (
                EventKind::Battery,
                None,
                percent,
                Some((value & 0xFF) == 0),
                None,
            )
        }
        0x5010 => (
            EventKind::WriteAck,
            None,
            None,
            None,
            Some((value >> 8) as u8),
        ),
        0x6000 => {
            let level = value as u8;
            let percent = if (1..=10).contains(&level) {
                Some(level * 10)
            } else {
                None
            };
            (EventKind::BatteryLevel, None, percent, None, None)
        }
        0x7000 => (EventKind::LightMode, None, None, None, None),
        0x8000 => (EventKind::Profile, None, None, None, None),
        _ => (EventKind::Unknown, None, None, None, None),
    };

    StatusEvent {
        kind,
        code,
        value,
        dpi_stage,
        battery_percent,
        asleep,
        acknowledged_report,
        raw,
    }
}

/// Parse one 2.4 GHz / USB input report. Returns `None` for unrelated input
/// reports (mouse movement, keyboard, consumer control, …).
pub fn parse(raw: &[u8]) -> Option<StatusEvent> {
    if raw.len() < 5 || raw[0] != 0x03 {
        return None;
    }
    let code = (raw[2] as u16) << 8 | raw[1] as u16;
    let value = (raw[4] as u16) << 8 | raw[3] as u16;
    Some(decode(code, value, raw.to_vec()))
}

/// Parse a Bluetooth FEE4 notification (no leading report ID).
pub fn parse_ble(payload: &[u8]) -> Option<StatusEvent> {
    if payload.len() < 4 {
        return None;
    }
    let code = (payload[1] as u16) << 8 | payload[0] as u16;
    let value = (payload[3] as u16) << 8 | payload[2] as u16;
    Some(decode(code, value, payload.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_live_battery_heartbeat() {
        // Observed on the receiver every few seconds.
        let event = parse(&[0x03, 0x10, 0x40, 0x01, 0x07]).unwrap();
        assert_eq!(event.kind, EventKind::Battery);
        assert_eq!(event.code, 0x4010);
        assert_eq!(event.value, 0x0701);
        assert_eq!(event.battery_percent, Some(70));
        assert_eq!(event.asleep, Some(false));
        assert_eq!(event.describe(), "battery 70%, awake");
    }

    #[test]
    fn parses_a_sleeping_battery_event() {
        let event = parse(&[0x03, 0x10, 0x40, 0x00, 0x05]).unwrap();
        assert_eq!(event.battery_percent, Some(50));
        assert_eq!(event.asleep, Some(true));
    }

    #[test]
    fn parses_a_dpi_stage_event() {
        let event = parse(&[0x03, 0x00, 0x10, 0x02, 0x00]).unwrap();
        assert_eq!(event.kind, EventKind::DpiStage);
        assert_eq!(event.dpi_stage, Some(2));
        assert_eq!(event.describe(), "DPI stage changed to 2");
    }

    #[test]
    fn parses_a_write_acknowledgement() {
        let event = parse(&[0x03, 0x10, 0x50, 0x00, 0x0C]).unwrap();
        assert_eq!(event.kind, EventKind::WriteAck);
        assert_eq!(event.acknowledged_report, Some(0x0C));
    }

    #[test]
    fn rejects_mouse_movement_and_short_frames() {
        assert!(parse(&[0x01, 0x02, 0x03, 0x04]).is_none());
        assert!(parse(&[0x03, 0x10]).is_none());
        assert!(parse(&[0x02, 0x00, 0x10, 0x01, 0x00]).is_none());
    }

    #[test]
    fn parses_the_same_event_over_bluetooth() {
        let event = parse_ble(&[0x10, 0x50, 0x00, 0x0C]).unwrap();
        assert_eq!(event.kind, EventKind::WriteAck);
        assert_eq!(event.acknowledged_report, Some(0x0C));
    }

    #[test]
    fn out_of_range_battery_is_not_displayed_as_a_percentage() {
        let event = parse(&[0x03, 0x10, 0x40, 0x00, 0x0F]).unwrap();
        assert_eq!(event.battery_percent, None);
        assert!(event.describe().contains("invalid"));
    }
}
