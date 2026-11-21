//! Write policy for the Attack Shark X3.
//!
//! The default is **read-only**. Reading descriptors and status events needs no
//! write access at all, and that is the only mode that is safe on an unknown
//! firmware.
//!
//! Three modes are available through the environment:
//!
//! | `OPENSHARKX3_WRITE_MODE` | Behaviour |
//! |---|---|
//! | unset / `readonly` | every write is refused |
//! | `dryrun` | reports are built and logged, never sent |
//! | `hardware` | verified reports are sent |
//!
//! Reports whose physical effect has not been demonstrated on this receiver
//! (DPI, lighting, button remap) additionally require
//! `OPENSHARKX3_EXPERIMENTAL=1`. A transfer that is acknowledged is still not
//! proof that the setting took effect; receipts record `acknowledged` and
//! `verified` separately.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    ReadOnly,
    DryRun,
    Hardware,
}

impl WriteMode {
    pub fn from_env() -> Self {
        match std::env::var("OPENSHARKX3_WRITE_MODE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "dryrun" | "dry-run" | "simulate" => Self::DryRun,
            "hardware" | "write" | "rw" => Self::Hardware,
            _ => Self::ReadOnly,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::DryRun => "dry-run",
            Self::Hardware => "hardware",
        }
    }

    pub fn can_write(self) -> bool {
        matches!(self, Self::Hardware)
    }
}

pub fn write_mode() -> WriteMode {
    WriteMode::from_env()
}

/// Whether a report whose effect is not physically verified may be sent.
pub fn experimental_enabled() -> bool {
    matches!(
        std::env::var("OPENSHARKX3_EXPERIMENTAL").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// Backwards-compatible helper used by the Bluetooth transport.
pub fn writes_blocked() -> bool {
    !write_mode().can_write()
}

/// Gate a write and return the active mode.
///
/// `experimental` must be `true` for any report whose physical effect has not
/// been verified. This is deliberately a second, independent gate: enabling
/// hardware mode does not silently enable every unverified report.
pub fn require_write(experimental: bool) -> Result<WriteMode, String> {
    let mode = write_mode();
    match mode {
        WriteMode::ReadOnly => Err(
            "writes are disabled (read-only mode); start with \
             OPENSHARKX3_WRITE_MODE=dryrun to simulate or \
             OPENSHARKX3_WRITE_MODE=hardware to send"
                .into(),
        ),
        WriteMode::DryRun => Ok(mode),
        WriteMode::Hardware => {
            if experimental && !experimental_enabled() {
                Err(
                    "this report is experimental on the X3 firmware; set \
                     OPENSHARKX3_EXPERIMENTAL=1 only after reading \
                     docs/HARDWARE_VALIDATION.md"
                        .into(),
                )
            } else {
                Ok(mode)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_read_only() {
        // The test process has no OPENSHARKX3_WRITE_MODE set.
        std::env::remove_var("OPENSHARKX3_WRITE_MODE");
        assert_eq!(write_mode(), WriteMode::ReadOnly);
        assert!(writes_blocked());
    }

    #[test]
    fn rejects_experimental_reports_without_the_second_gate() {
        // This only inspects the pure policy, not the process environment.
        std::env::remove_var("OPENSHARKX3_EXPERIMENTAL");
        assert!(!experimental_enabled());
    }

    #[test]
    fn mode_labels_are_stable_for_the_ui() {
        assert_eq!(WriteMode::ReadOnly.label(), "read-only");
        assert_eq!(WriteMode::DryRun.label(), "dry-run");
        assert_eq!(WriteMode::Hardware.label(), "hardware");
    }
}
