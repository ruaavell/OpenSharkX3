//! Linux-native driver for the Attack Shark X3 (2.4 GHz receiver and USB cable).
//!
//! Transport: `/dev/hidraw*` on the configuration interface, using
//! `HIDIOCSFEATURE` for writes and `read(2)` for the status channel. The kernel
//! driver is never detached and the USB interface is never claimed.
//!
//! Writes are disabled by default. See [`crate::safety`] for the three modes
//! and the separate gate for reports whose physical effect is unverified.

use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

use super::ConnMode;
use crate::{
    hidraw::{self, HidrawHandle, HidrawInfo, PID_WIRED, PID_WIRELESS},
    protocols::{
        self,
        dpi::{build_dpi, DpiConfig, DPI_REPORT_ID},
        macros::{build_macros, MacrosConfig, BUTTON_REPORT_ID},
        polling_rate::{build_polling_rate, PollingRateConfig, POLLING_REPORT_ID},
        status::{self, EventKind, StatusEvent},
        user_prefs::{build_user_prefs, UserPrefsConfig, LIGHT_REPORT_ID},
        ConnectionMode,
    },
    safety::{self, WriteMode},
};

const ACK_TIMEOUT: Duration = Duration::from_millis(1500);
const BATTERY_WAIT: Duration = Duration::from_millis(1800);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteReceipt {
    pub report_id: u8,
    pub length: usize,
    pub label: String,
    pub mode: String,
    /// `true` when the report was built but not transmitted (dry-run mode).
    pub simulated: bool,
    /// `true` when the device emitted a `0x5010` acknowledgement for this
    /// report ID after the transfer.
    pub acknowledged: bool,
    pub ack_report: Option<u8>,
    /// Always `false` in this release: an acknowledgement is not proof that the
    /// setting took effect. Physical verification is documented per report in
    /// `docs/HARDWARE_VALIDATION.md`.
    pub verified: bool,
    pub verification: String,
    pub report_hex: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub battery_percent: Option<u8>,
    pub asleep: bool,
    pub dpi_stage: Option<u8>,
    pub event_count: u64,
    pub last_event: Option<StatusEvent>,
}

#[derive(Default)]
struct Telemetry {
    battery_percent: Option<u8>,
    asleep: bool,
    dpi_stage: Option<u8>,
    last_event: Option<StatusEvent>,
    event_count: u64,
    acks: Vec<(u8, Instant)>,
}

struct Shared {
    telemetry: Mutex<Telemetry>,
    changed: Condvar,
    stop: AtomicBool,
}

impl Shared {
    fn new() -> Self {
        Self {
            telemetry: Mutex::new(Telemetry::default()),
            changed: Condvar::new(),
            stop: AtomicBool::new(false),
        }
    }

    fn record(&self, event: &StatusEvent) {
        let Ok(mut telemetry) = self.telemetry.lock() else {
            return;
        };
        telemetry.event_count = telemetry.event_count.saturating_add(1);
        telemetry.last_event = Some(event.clone());

        match event.kind {
            EventKind::Battery | EventKind::BatteryLevel => {
                if let Some(percent) = event.battery_percent {
                    telemetry.battery_percent = Some(percent);
                }
                if let Some(asleep) = event.asleep {
                    telemetry.asleep = asleep;
                }
            }
            EventKind::DpiStage => {
                if let Some(stage) = event.dpi_stage {
                    telemetry.dpi_stage = Some(stage);
                }
            }
            EventKind::WriteAck => {
                if let Some(id) = event.acknowledged_report {
                    telemetry.acks.push((id, Instant::now()));
                    let len = telemetry.acks.len();
                    if len > 16 {
                        telemetry.acks.drain(0..len - 16);
                    }
                }
            }
            _ => {}
        }

        drop(telemetry);
        self.changed.notify_all();
    }

    fn snapshot(&self) -> TelemetrySnapshot {
        let Ok(telemetry) = self.telemetry.lock() else {
            return TelemetrySnapshot {
                battery_percent: None,
                asleep: false,
                dpi_stage: None,
                event_count: 0,
                last_event: None,
            };
        };
        TelemetrySnapshot {
            battery_percent: telemetry.battery_percent,
            asleep: telemetry.asleep,
            dpi_stage: telemetry.dpi_stage,
            event_count: telemetry.event_count,
            last_event: telemetry.last_event.clone(),
        }
    }

    fn wait_for_battery(&self, timeout: Duration) -> Option<u8> {
        let deadline = Instant::now() + timeout;
        let mut telemetry = self.telemetry.lock().ok()?;
        loop {
            if telemetry.battery_percent.is_some() {
                return telemetry.battery_percent;
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            let (guard, _) = self.changed.wait_timeout(telemetry, deadline - now).ok()?;
            telemetry = guard;
        }
    }

    fn wait_for_ack(&self, report_id: u8, timeout: Duration) -> Option<u8> {
        let deadline = Instant::now() + timeout;
        let mut telemetry = self.telemetry.lock().ok()?;
        loop {
            if let Some((id, _)) = telemetry.acks.iter().rev().find(|(id, _)| *id == report_id) {
                return Some(*id);
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            let (guard, _) = self.changed.wait_timeout(telemetry, deadline - now).ok()?;
            telemetry = guard;
        }
    }

    fn clear_acks(&self, report_id: u8) {
        if let Ok(mut telemetry) = self.telemetry.lock() {
            telemetry.acks.retain(|(id, _)| *id != report_id);
        }
    }
}

pub struct AttackShark {
    info: HidrawInfo,
    conn_mode: ConnMode,
    writer: Mutex<Option<HidrawHandle>>,
    shared: Arc<Shared>,
    poll_stop: Option<oneshot::Sender<()>>,
}

impl AttackShark {
    /// Open the configuration interface and start the read-only status listener.
    ///
    /// This does **not** send any configuration report.
    pub fn open(preferred: ConnMode, app: AppHandle) -> Result<Self, String> {
        let pid = match preferred {
            ConnMode::Wired => PID_WIRED,
            ConnMode::Wireless => PID_WIRELESS,
            ConnMode::Bluetooth => return Err("BLE is handled by the Bluetooth driver".into()),
        };

        let info = hidraw::find_config(pid).ok_or_else(|| {
            let all = hidraw::find_all();
            let seen: Vec<String> = all
                .iter()
                .map(|d| {
                    format!(
                        "{} {:04x}:{:04x} if={:?} feat={}",
                        d.node,
                        d.vendor_id,
                        d.product_id,
                        d.interface,
                        d.feature_reports.len()
                    )
                })
                .collect();
            format!(
                "configuration interface for {:04x}:{:04x} not found (nodes: {})",
                hidraw::VID,
                pid,
                if seen.is_empty() {
                    "none".to_string()
                } else {
                    seen.join(", ")
                }
            )
        })?;

        if info.feature_reports.is_empty() {
            return Err(format!(
                "{} has no feature reports; refusing to open it as the configuration interface",
                info.node
            ));
        }

        let reader = HidrawHandle::open(&info, false).map_err(|e| {
            format!(
                "cannot open {} read-only: {e}. Install tools/70-attack-shark-x3-hidraw.rules \
                 in /etc/udev/rules.d/ and re-plug the receiver",
                info.node
            )
        })?;

        let shared = Arc::new(Shared::new());
        let (tx, mut rx) = oneshot::channel::<()>();

        {
            let shared = Arc::clone(&shared);
            let app = app.clone();
            let node = info.node.clone();
            std::thread::spawn(move || {
                let mut reader = reader;
                info!("[hidraw] listening on {node} for status reports");
                loop {
                    if shared.stop.load(Ordering::Acquire) || rx.try_recv().is_ok() {
                        break;
                    }
                    match reader.read_input(Duration::from_millis(250)) {
                        Ok(Some(raw)) => {
                            if let Some(event) = status::parse(&raw) {
                                debug!("[hidraw] event {:?}: {}", event.kind, event.describe());
                                shared.record(&event);
                                match event.kind {
                                    EventKind::Battery | EventKind::BatteryLevel => {
                                        if let Some(percent) = event.battery_percent {
                                            let _ = app.emit("mouse:battery", percent);
                                        }
                                        if let Some(asleep) = event.asleep {
                                            let _ = app.emit("mouse:asleep", asleep);
                                        }
                                    }
                                    EventKind::DpiStage => {
                                        if let Some(stage) = event.dpi_stage {
                                            // The UI is 0-based; the wire is 1-based.
                                            let _ = app.emit(
                                                "mouse:dpi_stage",
                                                stage.saturating_sub(1),
                                            );
                                        }
                                    }
                                    EventKind::WriteAck => {
                                        if let Some(id) = event.acknowledged_report {
                                            let _ = app.emit("mouse:write_ack", id);
                                        }
                                    }
                                    _ => {
                                        let _ = app.emit("mouse:status_event", &event);
                                    }
                                }
                            } else {
                                debug!("[hidraw] unhandled input report: {:02x?}", raw);
                            }
                        }
                        Ok(None) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(e) => {
                            warn!("[hidraw] read error on {node}: {e}");
                            let _ = app.emit("mouse:disconnected", ());
                            break;
                        }
                    }
                }
                info!("[hidraw] status listener stopped");
            });
        }

        Ok(Self {
            info,
            conn_mode: preferred,
            writer: Mutex::new(None),
            shared,
            poll_stop: Some(tx),
        })
    }

    pub fn info(&self) -> &HidrawInfo {
        &self.info
    }

    #[allow(dead_code)]
    pub fn conn_mode(&self) -> ConnMode {
        self.conn_mode
    }

    /// Stable label for the UI: `wired`, `wireless` or `bluetooth`.
    pub fn conn_mode_label(&self) -> &'static str {
        match self.conn_mode {
            ConnMode::Wired => "wired",
            ConnMode::Wireless => "wireless",
            ConnMode::Bluetooth => "bluetooth",
        }
    }

    pub fn protocol_mode(&self) -> ConnectionMode {
        match self.conn_mode {
            ConnMode::Wired => ConnectionMode::Wired,
            _ => ConnectionMode::Wireless,
        }
    }

    pub fn telemetry(&self) -> TelemetrySnapshot {
        self.shared.snapshot()
    }

    /// Battery percentage, from the device's own `0x4010` heartbeat.
    ///
    /// Returns `-1` when no battery event has arrived. It never invents a value.
    pub fn get_battery(&self) -> i32 {
        if self.conn_mode == ConnMode::Wired {
            return -1;
        }
        self.shared
            .wait_for_battery(BATTERY_WAIT)
            .map(i32::from)
            .unwrap_or(-1)
    }

    pub fn close(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(tx) = self.poll_stop.take() {
            let _ = tx.send(());
        }
        if let Ok(mut writer) = self.writer.lock() {
            writer.take();
        }
        shared_notify(&self.shared);
    }

    // ── Writes ───────────────────────────────────────────────────────────────

    fn send_report(
        &self,
        report: Vec<u8>,
        label: &str,
        experimental: bool,
    ) -> Result<WriteReceipt, String> {
        let mode = safety::require_write(experimental)?;
        let report_id = report[0];
        let report_hex = hex(&report);

        if mode == WriteMode::DryRun {
            info!("[dry-run] {label}: report 0x{report_id:02x} ({}) {report_hex}", report.len());
            return Ok(WriteReceipt {
                report_id,
                length: report.len(),
                label: label.to_string(),
                mode: mode.label().to_string(),
                simulated: true,
                acknowledged: false,
                ack_report: None,
                verified: false,
                verification: "dry-run: not transmitted".into(),
                report_hex,
            });
        }

        let mut guard = self
            .writer
            .lock()
            .map_err(|_| "writer mutex poisoned".to_string())?;
        if guard.is_none() {
            let handle = HidrawHandle::open(&self.info, true).map_err(|e| {
                format!(
                    "cannot open {} for writing: {e}. Check the udev rule and permissions",
                    self.info.node
                )
            })?;
            *guard = Some(handle);
        }
        let handle = guard.as_ref().expect("writer was just opened");

        self.shared.clear_acks(report_id);

        handle
            .set_feature(&protocols::X3_PREAMBLE)
            .map_err(|e| format!("preamble write failed: {e}"))?;
        std::thread::sleep(protocols::X3_WRITE_DELAY);
        handle
            .set_feature(&report)
            .map_err(|e| format!("report 0x{report_id:02x} write failed: {e}"))?;

        let ack = self.shared.wait_for_ack(report_id, ACK_TIMEOUT);
        if ack.is_some() {
            info!(
                "[hidraw] report 0x{report_id:02x} ({label}) acknowledged after {} bytes",
                report.len()
            );
        } else {
            warn!(
                "[hidraw] report 0x{report_id:02x} ({label}) sent but not acknowledged within {} ms",
                ACK_TIMEOUT.as_millis()
            );
        }

        Ok(WriteReceipt {
            report_id,
            length: report.len(),
            label: label.to_string(),
            mode: mode.label().to_string(),
            simulated: false,
            acknowledged: ack.is_some(),
            ack_report: ack,
            verified: false,
            verification: if ack.is_some() {
                "acknowledged by the device; physical effect not verified on this unit".into()
            } else {
                "no acknowledgement; the firmware may have ignored the report".into()
            },
            report_hex,
        })
    }

    pub fn set_dpi(&self, cfg: &DpiConfig) -> Result<WriteReceipt, String> {
        let mode = self.protocol_mode();
        let report = build_dpi(cfg, mode);
        // Configuration reports are occasionally dropped by the 2.4 GHz link
        // (no 0x5010 acknowledgement). Retry once before reporting failure,
        // matching the behaviour documented in docs/HARDWARE_VALIDATION.md.
        let mut receipt = self.send_report(report.clone(), "DPI + sensor toggles", true)?;
        if !receipt.acknowledged && safety::write_mode() == WriteMode::Hardware {
            std::thread::sleep(std::time::Duration::from_millis(200));
            receipt = self.send_report(report, "DPI + sensor toggles", true)?;
        }
        Ok(receipt)
    }

    pub fn set_user_prefs(&self, cfg: &UserPrefsConfig) -> Result<WriteReceipt, String> {
        let mode = self.protocol_mode();
        self.send_report(
            build_user_prefs(cfg, mode),
            "lighting + power + debounce",
            true,
        )
    }

    pub fn set_lighting(&self, cfg: &UserPrefsConfig) -> Result<WriteReceipt, String> {
        self.set_user_prefs(cfg)
    }

    pub fn set_polling_rate(&self, cfg: &PollingRateConfig) -> Result<WriteReceipt, String> {
        // The protocol for this report is verified; its physical interval is
        // still not measured here, so the receipt remains ack-only.
        self.send_report(build_polling_rate(cfg), "polling rate", false)
    }

    pub fn set_macros(&self, cfg: &MacrosConfig) -> Result<WriteReceipt, String> {
        if !cfg.has_left_click() {
            return Err(
                "refusing to write a button map with no left click; the mouse would be unusable"
                    .into(),
            );
        }
        self.send_report(build_macros(cfg), "button remap", true)
    }

    pub fn set_custom_macro(&self, _cfg: &crate::protocols::custom_macro::CustomMacroConfig) -> Result<WriteReceipt, String> {
        Err(
            "custom macro upload is not implemented for the X3: the report layout has not \
             been verified on this firmware, so nothing was sent"
                .into(),
        )
    }

    pub fn report_ids(&self) -> (u8, u8, u8, u8) {
        (
            DPI_REPORT_ID,
            LIGHT_REPORT_ID,
            POLLING_REPORT_ID,
            BUTTON_REPORT_ID,
        )
    }
}

fn shared_notify(shared: &Arc<Shared>) {
    shared.changed.notify_all();
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns (wireless_device_found, wired_device_found).
pub fn detect_usb_devices() -> (bool, bool) {
    let mut wireless = false;
    let mut wired = false;
    for device in hidraw::find_all() {
        if device.vendor_id != hidraw::VID {
            continue;
        }
        if device.product_id == PID_WIRELESS {
            wireless = true;
        } else if device.product_id == PID_WIRED {
            wired = true;
        }
    }
    (wireless, wired)
}
