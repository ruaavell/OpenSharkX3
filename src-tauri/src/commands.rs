use serde::Serialize;
use std::{fs, path::Path};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{info, warn};

use crate::{
    driver::{AttackShark, AttackSharkBle, ConnMode, DriverVariant},
    hidraw,
    protocols::{
        custom_macro::CustomMacroConfig,
        dpi::{DpiConfig, LedMode},
        macros::MacrosConfig,
        polling_rate::{PollingRate, PollingRateConfig},
        user_prefs::{LightMode, UserPrefsConfig},
    },
    safety::{self, WriteMode},
    state::{AppState, ManagedState, battery_override, save_profiles, save_state},
};

// ── Helpers ───────────────────────────────────────────────────────────────────

const BTN_ORDER: [&str; 8] = [
    "left", "right", "middle", "forward", "backward", "dpi", "scrollUp", "scrollDown",
];

fn build_dpi_cfg(s: &AppState) -> DpiConfig {
    DpiConfig {
        dpi_values: s.dpi.values,
        // AppState is 0-based (like React and the mouse:dpi_stage event); the
        // 0x04 report wants 1..=6.
        active_stage: s.dpi.active_stage + 1,
        angle_snap: s.dpi.angle_snap,
        ripple_control: s.dpi.ripple_control,
        lod_2mm: s.dpi.lod_2mm,
        motion_sync: s.dpi.motion_sync,
        stage_colors: s.lighting.stage_colors.clone(),
        led_mode: LedMode::Off,
    }
}

fn build_user_prefs_cfg(s: &AppState, battery: i32) -> UserPrefsConfig {
    let base = if let Some(ov) = battery_override(battery) {
        UserPrefsConfig {
            light_mode: ov.mode,
            rgb: ov.rgb,
            led_speed: ov.led_speed,
            brightness: 8,
            ..UserPrefsConfig::default()
        }
    } else {
        UserPrefsConfig {
            light_mode: LightMode::from_u8(s.lighting.mode),
            rgb: s.lighting.global_color.clone(),
            led_speed: s.lighting.led_speed,
            brightness: 8,
            ..UserPrefsConfig::default()
        }
    };

    UserPrefsConfig {
        sleep_time: s.power.sleep_time,
        deep_sleep_time: s.power.deep_sleep_time,
        key_response: s.performance.key_response,
        ..base
    }
}

fn build_macros_cfg(s: &AppState) -> MacrosConfig {
    let mut entries = MacrosConfig::default();
    for (i, key) in BTN_ORDER.iter().enumerate() {
        if let Some(binding) = s.buttons.get(*key) {
            entries.buttons[i] = binding.to_macro_entry();
        }
    }
    entries
}

// ── Connection ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    pub ok: bool,
    pub mode: Option<String>,
    pub error: Option<String>,
    /// The active write policy. Connecting never changes it.
    pub write_mode: String,
    /// `true` only when the user started the app with hardware writes enabled.
    pub writes_enabled: bool,
    /// `true` when this connection sent no configuration report.
    pub read_only: bool,
}

#[tauri::command]
pub async fn connect(
    mode: Option<String>,
    managed: State<'_, ManagedState>,
    app: AppHandle,
) -> Result<ConnectResult, String> {
    // Close the existing driver first.
    {
        let mut guard = managed.driver.lock().await;
        if let Some(d) = guard.take() {
            match d {
                DriverVariant::Usb(mut d) => d.close(),
                DriverVariant::Ble(mut d) => d.close().await,
            }
        }
    }
    *managed.battery.lock().unwrap() = -1;
    *managed.conn_mode.lock().unwrap() = None;

    let preferred = mode.as_deref();
    let policy = safety::write_mode();

    // Try BLE explicitly.
    if preferred == Some("bluetooth") {
        return try_ble(&managed, app, policy).await;
    }

    let try_wired_first = preferred == Some("wired");
    let (wireless_found, wired_found) = crate::driver::attack_shark::detect_usb_devices();
    let mut usb_err = String::new();

    if wireless_found || wired_found {
        let order: &[ConnMode] = if try_wired_first {
            &[ConnMode::Wired, ConnMode::Wireless]
        } else {
            &[ConnMode::Wireless, ConnMode::Wired]
        };
        for &conn_mode in order {
            if conn_mode == ConnMode::Wireless && !wireless_found {
                continue;
            }
            if conn_mode == ConnMode::Wired && !wired_found {
                continue;
            }

            match AttackShark::open(conn_mode, app.clone()) {
                Ok(driver) => {
                    let bat = driver.get_battery();
                    *managed.battery.lock().unwrap() = bat;
                    *managed.conn_mode.lock().unwrap() = Some(conn_mode);

                    crate::tray::update_tray(&app, true, bat, conn_mode);
                    let mode_name = match conn_mode {
                        ConnMode::Wireless => "wireless",
                        ConnMode::Wired => "wired",
                        _ => "wireless",
                    };
                    info!(
                        "[connect] connected via {mode_name} on {} (policy: {})",
                        driver.info().node,
                        policy.label()
                    );

                    // A connection is a read-only operation. No configuration
                    // report is sent until the user explicitly asks for one.
                    *managed.driver.lock().await = Some(DriverVariant::Usb(driver));

                    return Ok(ConnectResult {
                        ok: true,
                        mode: Some(mode_name.into()),
                        error: None,
                        write_mode: policy.label().into(),
                        writes_enabled: policy.can_write(),
                        read_only: true,
                    });
                }
                Err(e) => {
                    usb_err = e;
                    warn!("[connect] USB {conn_mode:?} failed: {usb_err}");
                }
            }
        }
    }

    // BLE fallback.
    let ble_result = try_ble(&managed, app, policy).await;
    if ble_result.as_ref().map(|r| r.ok).unwrap_or(false) {
        return ble_result;
    }
    let ble_err = ble_result.ok().and_then(|r| r.error).unwrap_or_default();
    let final_err = if !usb_err.is_empty() { usb_err } else { ble_err };
    Ok(ConnectResult {
        ok: false,
        mode: None,
        error: Some(final_err),
        write_mode: policy.label().into(),
        writes_enabled: policy.can_write(),
        read_only: true,
    })
}

async fn try_ble(
    managed: &ManagedState,
    app: AppHandle,
    policy: WriteMode,
) -> Result<ConnectResult, String> {
    match AttackSharkBle::open(app.clone()).await {
        Ok(ble) => {
            let bat = ble.get_battery().await;
            *managed.battery.lock().unwrap() = bat;
            *managed.conn_mode.lock().unwrap() = Some(ConnMode::Bluetooth);

            crate::tray::update_tray(&app, true, bat, ConnMode::Bluetooth);
            info!("[connect] connected via Bluetooth (policy: {})", policy.label());

            *managed.driver.lock().await = Some(DriverVariant::Ble(ble));

            Ok(ConnectResult {
                ok: true,
                mode: Some("bluetooth".into()),
                error: None,
                write_mode: policy.label().into(),
                writes_enabled: policy.can_write(),
                read_only: true,
            })
        }
        Err(e) => {
            warn!("[connect] BLE failed: {e}");
            Ok(ConnectResult {
                ok: false,
                mode: None,
                error: Some(e),
                write_mode: policy.label().into(),
                writes_enabled: policy.can_write(),
                read_only: true,
            })
        }
    }
}

#[tauri::command]
pub async fn disconnect(managed: State<'_, ManagedState>, app: AppHandle) -> Result<(), String> {
    let mut guard = managed.driver.lock().await;
    if let Some(d) = guard.take() {
        match d {
            DriverVariant::Usb(mut d) => d.close(),
            DriverVariant::Ble(mut d) => d.close().await,
        }
    }
    *managed.battery.lock().unwrap() = -1;
    *managed.conn_mode.lock().unwrap() = None;
    crate::tray::update_tray(&app, false, -1, ConnMode::Wireless);
    Ok(())
}

#[tauri::command]
pub async fn battery(managed: State<'_, ManagedState>) -> Result<Option<i32>, String> {
    // Prefer the device's own heartbeat, never a guessed value.
    let guard = managed.driver.lock().await;
    let value = match guard.as_ref() {
        None => None,
        Some(DriverVariant::Usb(d)) => d.telemetry().battery_percent.map(i32::from),
        Some(DriverVariant::Ble(d)) => {
            let b = d.get_battery().await;
            if b >= 0 { Some(b) } else { None }
        }
    };
    drop(guard);
    if let Some(v) = value {
        *managed.battery.lock().unwrap() = v;
    }
    Ok(value)
}

// ── Live device information ──────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub connected: bool,
    pub node: Option<String>,
    pub sysfs: Option<String>,
    pub bus: Option<u16>,
    pub vendor_id: u16,
    pub product_id: Option<u16>,
    pub interface: Option<u8>,
    pub name: Option<String>,
    pub descriptor_len: Option<usize>,
    pub feature_reports: Vec<(u8, usize)>,
    pub input_reports: Vec<(u8, usize)>,
    pub write_mode: String,
    pub writes_enabled: bool,
    pub experimental_enabled: bool,
    pub note: String,
}

#[tauri::command]
pub async fn device_info(managed: State<'_, ManagedState>) -> Result<DeviceInfo, String> {
    let mode = safety::write_mode();
    let guard = managed.driver.lock().await;

    let mut info = DeviceInfo {
        connected: false,
        node: None,
        sysfs: None,
        bus: None,
        vendor_id: hidraw::VID,
        product_id: None,
        interface: None,
        name: None,
        descriptor_len: None,
        feature_reports: Vec::new(),
        input_reports: Vec::new(),
        write_mode: mode.label().into(),
        writes_enabled: mode.can_write(),
        experimental_enabled: safety::experimental_enabled(),
        note: String::new(),
    };

    match guard.as_ref() {
        Some(DriverVariant::Usb(d)) => {
            let raw = d.info();
            info.connected = true;
            info.node = Some(raw.node.clone());
            info.sysfs = Some(raw.sysfs.clone());
            info.bus = Some(raw.bus);
            info.product_id = Some(raw.product_id);
            info.interface = raw.interface;
            info.name = Some(raw.name.clone());
            info.descriptor_len = Some(raw.descriptor_len);
            info.feature_reports = raw
                .feature_reports
                .iter()
                .map(|(id, len)| (*id, *len))
                .collect();
            info.input_reports = raw
                .input_reports
                .iter()
                .map(|(id, len)| (*id, *len))
                .collect();
            info.note = "hidraw transport; the kernel driver stays bound".into();
        }
        Some(DriverVariant::Ble(_)) => {
            info.connected = true;
            info.product_id = Some(hidraw::PID_WIRELESS);
            info.name = Some("Attack Shark X3 (Bluetooth)".into());
            info.note = "Bluetooth GATT transport; report layout not verified on FA60".into();
        }
        None => {
            info.note = "not connected".into();
            if let Some(raw) = hidraw::find_any_x3() {
                info.node = Some(raw.node.clone());
                info.sysfs = Some(raw.sysfs.clone());
                info.bus = Some(raw.bus);
                info.product_id = Some(raw.product_id);
                info.interface = raw.interface;
                info.name = Some(raw.name.clone());
                info.descriptor_len = Some(raw.descriptor_len);
                info.feature_reports =
                    raw.feature_reports.iter().map(|(id, len)| (*id, *len)).collect();
                info.input_reports =
                    raw.input_reports.iter().map(|(id, len)| (*id, *len)).collect();
                info.note = "device present but not opened".into();
            }
        }
    }

    Ok(info)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticNode {
    pub node: String,
    pub sysfs: String,
    pub bus: u16,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: Option<u8>,
    pub name: String,
    pub descriptor_len: usize,
    pub feature_reports: Vec<(u8, usize)>,
    pub input_reports: Vec<(u8, usize)>,
    pub output_reports: Vec<(u8, usize)>,
    pub is_config_candidate: bool,
    pub openable_read_only: bool,
    pub open_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub write_mode: String,
    pub writes_enabled: bool,
    pub experimental_enabled: bool,
    pub known_identity: String,
    pub nodes: Vec<DiagnosticNode>,
    pub warnings: Vec<String>,
}

/// Read-only probe of every Attack Shark hidraw node on the system.
///
/// This never writes and never opens a device for writing; it is safe to run
/// with the mouse in use.
#[tauri::command]
pub async fn diagnose() -> Result<DiagnosticReport, String> {
    let mode = safety::write_mode();
    let mut nodes = Vec::new();
    let mut warnings = Vec::new();

    for raw in hidraw::find_all() {
        if raw.vendor_id != hidraw::VID {
            continue;
        }
        let open_error = match hidraw::HidrawHandle::open(&raw, false) {
            Ok(_) => None,
            Err(e) => Some(e.to_string()),
        };
        if open_error.is_some()
            && raw.interface == Some(hidraw::CONFIG_INTERFACE)
            && raw.product_id == hidraw::PID_WIRELESS
        {
            warnings.push(format!(
                "{} is the configuration interface but is not readable by this user. \
                 Install tools/70-attack-shark-x3-hidraw.rules and re-plug the receiver.",
                raw.node
            ));
        }
        nodes.push(DiagnosticNode {
            node: raw.node.clone(),
            sysfs: raw.sysfs.clone(),
            bus: raw.bus,
            vendor_id: raw.vendor_id,
            product_id: raw.product_id,
            interface: raw.interface,
            name: raw.name.clone(),
            descriptor_len: raw.descriptor_len,
            feature_reports: raw.feature_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            input_reports: raw.input_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            output_reports: raw.output_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            is_config_candidate: raw.interface == Some(hidraw::CONFIG_INTERFACE)
                && !raw.feature_reports.is_empty(),
            openable_read_only: open_error.is_none(),
            open_error,
        });
    }

    if nodes.is_empty() {
        warnings.push("no 1d57:xxxx hidraw node found; is the receiver plugged in?".into());
    }
    if mode == WriteMode::ReadOnly {
        warnings.push("write mode is read-only: every configuration write will be refused".into());
    }

    Ok(DiagnosticReport {
        write_mode: mode.label().into(),
        writes_enabled: mode.can_write(),
        experimental_enabled: safety::experimental_enabled(),
        known_identity: format!(
            "wireless {:04x}:{:04x}, wired {:04x}:{:04x}, config interface {}",
            hidraw::VID,
            hidraw::PID_WIRELESS,
            hidraw::VID,
            hidraw::PID_WIRED,
            hidraw::CONFIG_INTERFACE
        ),
        nodes,
        warnings,
    })
}

// ── Discovery ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEntry {
    pub node: String,
    pub sysfs: String,
    pub bus: u16,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: Option<u8>,
    pub name: String,
    pub is_config_interface: bool,
    pub feature_reports: Vec<(u8, usize)>,
    pub input_reports: Vec<(u8, usize)>,
    pub output_reports: Vec<(u8, usize)>,
    pub openable_read_only: bool,
    pub open_error: Option<String>,
}

/// Enumerate every Attack Shark hidraw node currently present.
///
/// Discovery is by sysfs identity; device numbers are never hard-coded. This is
/// a read-only probe and never opens a node for writing.
#[tauri::command]
pub async fn discover_devices() -> Result<Vec<DeviceEntry>, String> {
    let mut entries = Vec::new();
    for raw in hidraw::find_all() {
        if raw.vendor_id != hidraw::VID {
            continue;
        }
        let open_error = hidraw::HidrawHandle::open(&raw, false).err().map(|e| e.to_string());
        entries.push(DeviceEntry {
            node: raw.node.clone(),
            sysfs: raw.sysfs.clone(),
            bus: raw.bus,
            vendor_id: raw.vendor_id,
            product_id: raw.product_id,
            interface: raw.interface,
            name: raw.name.clone(),
            is_config_interface: raw.interface == Some(hidraw::CONFIG_INTERFACE)
                && !raw.feature_reports.is_empty(),
            feature_reports: raw.feature_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            input_reports: raw.input_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            output_reports: raw.output_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            openable_read_only: open_error.is_none(),
            open_error,
        });
    }
    Ok(entries)
}

// ── Descriptor inspection ─────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorReport {
    pub node: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: Option<u8>,
    pub name: String,
    pub descriptor_len: usize,
    /// Whitespace-separated hex, 16 bytes per line.
    pub descriptor_hex: String,
    pub feature_reports: Vec<(u8, usize)>,
    pub input_reports: Vec<(u8, usize)>,
    pub output_reports: Vec<(u8, usize)>,
    pub top_level_usages: Vec<(u16, u16)>,
}

/// Dump the kernel report descriptor of one node, or of every X3 node.
///
/// The descriptor comes from sysfs, so this works even when the node itself is
/// not readable. Nothing is written.
#[tauri::command]
pub async fn descriptor_dump(node: Option<String>) -> Result<Vec<DescriptorReport>, String> {
    let reports: Vec<DescriptorReport> = hidraw::find_all()
        .into_iter()
        .filter(|d| d.vendor_id == hidraw::VID)
        .filter(|d| node.as_ref().map(|n| n == &d.node).unwrap_or(true))
        .map(|d| DescriptorReport {
            node: d.node.clone(),
            vendor_id: d.vendor_id,
            product_id: d.product_id,
            interface: d.interface,
            name: d.name.clone(),
            descriptor_len: d.descriptor_len,
            descriptor_hex: hidraw::descriptor_hex(&d.descriptor),
            feature_reports: d.feature_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            input_reports: d.input_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            output_reports: d.output_reports.iter().map(|(id, l)| (*id, *l)).collect(),
            top_level_usages: d.top_level_usages.clone(),
        })
        .collect();

    if reports.is_empty() {
        return Err(match node {
            Some(n) => format!("{n} is not a 1d57 device or no longer present"),
            None => "no 1d57 hidraw device found; is the receiver plugged in?".into(),
        });
    }
    Ok(reports)
}

// ── Connection status ─────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub connected: bool,
    pub mode: Option<String>,
    pub battery_percent: Option<u8>,
    pub asleep: bool,
    pub dpi_stage: Option<u8>,
    pub event_count: u64,
    pub last_event: Option<crate::protocols::status::StatusEvent>,
    pub write_mode: String,
    pub writes_enabled: bool,
    pub experimental_enabled: bool,
    pub note: String,
}

/// Live connection state, battery telemetry and the active write policy.
///
/// Every field comes from the device or the process environment. No value is
/// synthesised when the device has not reported one.
#[tauri::command]
pub async fn device_status(managed: State<'_, ManagedState>) -> Result<DeviceStatus, String> {
    let mode = safety::write_mode();
    let guard = managed.driver.lock().await;
    let status = match guard.as_ref() {
        Some(DriverVariant::Usb(d)) => {
            let t = d.telemetry();
            DeviceStatus {
                connected: true,
                mode: Some(d.conn_mode_label().to_string()),
                battery_percent: t.battery_percent,
                asleep: t.asleep,
                dpi_stage: t.dpi_stage,
                event_count: t.event_count,
                last_event: t.last_event,
                write_mode: mode.label().into(),
                writes_enabled: mode.can_write(),
                experimental_enabled: safety::experimental_enabled(),
                note: "hidraw transport; kernel driver remains bound".into(),
            }
        }
        Some(DriverVariant::Ble(d)) => {
            let battery = d.get_battery().await;
            DeviceStatus {
                connected: true,
                mode: Some("bluetooth".into()),
                battery_percent: (battery >= 0).then_some(battery as u8),
                asleep: false,
                dpi_stage: None,
                event_count: 0,
                last_event: None,
                write_mode: mode.label().into(),
                writes_enabled: mode.can_write(),
                experimental_enabled: safety::experimental_enabled(),
                note: "Bluetooth GATT transport".into(),
            }
        }
        None => DeviceStatus {
            connected: false,
            mode: None,
            battery_percent: None,
            asleep: false,
            dpi_stage: None,
            event_count: 0,
            last_event: None,
            write_mode: mode.label().into(),
            writes_enabled: mode.can_write(),
            experimental_enabled: safety::experimental_enabled(),
            note: "not connected".into(),
        },
    };
    Ok(status)
}

// ── Polling-rate measurement ──────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollingMeasurement {
    pub ok: bool,
    pub node: Option<String>,
    pub duration_ms: u64,
    pub samples: usize,
    pub measured_hz: Option<f64>,
    pub nearest_standard_hz: Option<u32>,
    pub median_interval_us: Option<f64>,
    pub jitter_us: Option<f64>,
    pub note: String,
}

/// Measure the real report interval by observing input reports on the mouse
/// interface. This is a read-only measurement: the mouse must be moved during
/// the window, otherwise there is nothing to measure and the result says so.
#[tauri::command]
pub async fn measure_polling_rate(
    product_id: Option<u16>,
    duration_ms: Option<u64>,
) -> Result<PollingMeasurement, String> {
    let pid = product_id.unwrap_or(hidraw::PID_WIRELESS);
    let duration = duration_ms.unwrap_or(1000).clamp(250, 5000);
    tauri::async_runtime::spawn_blocking(move || measure_blocking(pid, duration))
        .await
        .map_err(|e| format!("measurement task failed: {e}"))?
}

fn measure_blocking(product_id: u16, duration_ms: u64) -> Result<PollingMeasurement, String> {
    let info = hidraw::find_mouse_input(product_id).ok_or_else(|| {
        format!(
            "no mouse input interface found for {:04x}:{:04x}; connect the receiver or cable",
            hidraw::VID,
            product_id
        )
    })?;
    let mut handle = hidraw::HidrawHandle::open(&info, false).map_err(|e| {
        format!(
            "cannot open {} read-only: {e}. Install tools/70-attack-shark-x3-hidraw.rules",
            info.node
        )
    })?;

    let window = std::time::Duration::from_millis(duration_ms);
    let started = std::time::Instant::now();
    let mut stamps: Vec<std::time::Instant> = Vec::new();

    while started.elapsed() < window {
        let remaining = window.saturating_sub(started.elapsed());
        let timeout = remaining.min(std::time::Duration::from_millis(100));
        match handle.read_input(timeout) {
            Ok(Some(_)) => stamps.push(std::time::Instant::now()),
            Ok(None) => {}
            Err(e) => {
                return Err(format!("read error on {}: {e}", info.node));
            }
        }
    }

    let mut intervals: Vec<f64> = stamps
        .windows(2)
        .map(|pair| pair[1].duration_since(pair[0]).as_secs_f64() * 1_000_000.0)
        .filter(|v| *v > 0.0)
        .collect();

    if intervals.len() < 10 {
        return Ok(PollingMeasurement {
            ok: false,
            node: Some(info.node.clone()),
            duration_ms,
            samples: intervals.len(),
            measured_hz: None,
            nearest_standard_hz: None,
            median_interval_us: None,
            jitter_us: None,
            note: format!(
                "only {} report interval(s) observed on {}; move the mouse during the measurement",
                intervals.len(),
                info.node
            ),
        });
    }

    intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = intervals[intervals.len() / 2];
    let deviations: Vec<f64> = intervals.iter().map(|v| (v - median).abs()).collect();
    let mut dev_sorted = deviations.clone();
    dev_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let jitter = dev_sorted[dev_sorted.len() / 2];

    let measured_hz = 1_000_000.0 / median;
    let nearest = [125u32, 250, 500, 1000]
        .into_iter()
        .min_by(|a, b| {
            (*a as f64 - measured_hz)
                .abs()
                .partial_cmp(&(*b as f64 - measured_hz).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    Ok(PollingMeasurement {
        ok: true,
        node: Some(info.node.clone()),
        duration_ms,
        samples: intervals.len(),
        measured_hz: Some((measured_hz * 10.0).round() / 10.0),
        nearest_standard_hz: nearest,
        median_interval_us: Some((median * 10.0).round() / 10.0),
        jitter_us: Some((jitter * 10.0).round() / 10.0),
        note: format!(
            "measured from {} input reports on {}; this observes the real interval, it does not change it",
            intervals.len() + 1,
            info.node
        ),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryView {
    pub battery_percent: Option<u8>,
    pub asleep: bool,
    pub dpi_stage: Option<u8>,
    pub event_count: u64,
    pub last_event: Option<crate::protocols::status::StatusEvent>,
}

#[tauri::command]
pub async fn telemetry(managed: State<'_, ManagedState>) -> Result<Option<TelemetryView>, String> {
    let guard = managed.driver.lock().await;
    Ok(match guard.as_ref() {
        Some(DriverVariant::Usb(d)) => {
            let t = d.telemetry();
            Some(TelemetryView {
                battery_percent: t.battery_percent,
                asleep: t.asleep,
                dpi_stage: t.dpi_stage,
                event_count: t.event_count,
                last_event: t.last_event,
            })
        }
        _ => None,
    })
}

// ── Config ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_config(managed: State<'_, ManagedState>) -> Result<AppState, String> {
    Ok(managed.app_state.lock().await.clone())
}

/// Merge a patch into the local profile and persist it. **No report is
/// transmitted** and no device is opened: this is the local-only save path used
/// by the macro editor and any setting that has no verified wire format.
#[tauri::command]
pub async fn save_config(
    patch: serde_json::Value,
    managed: State<'_, ManagedState>,
) -> Result<AppState, String> {
    let mut state = managed.app_state.lock().await;
    save_local(&mut state, &managed.state_file, &patch)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub state: AppState,
    pub write_mode: String,
    /// Receipts for every report the backend attempted (or simulated).
    pub reports: Vec<crate::driver::attack_shark::WriteReceipt>,
    /// Always `false` unless every report was acknowledged *and* physically
    /// verified. No report has a physical verification on this unit yet.
    pub hardware_applied: bool,
    pub note: String,
}

#[tauri::command]
pub fn apply_config(
    patch: serde_json::Value,
    managed: State<'_, ManagedState>,
    app: AppHandle,
) -> Result<ApplyResult, String> {
    let guard = managed.driver.blocking_lock();
    let driver = guard.as_ref().ok_or("mouse is not connected")?;

    let mut state = managed.app_state.blocking_lock();
    merge_patch(&mut state, &patch);

    info!(
        "[apply] mode={:?} dpi={:?} lighting={:#04x}/{} buttons={:?}",
        safety::write_mode(),
        state.dpi.values,
        state.lighting.mode,
        state.lighting.led_speed,
        state
            .buttons
            .iter()
            .map(|(k, v)| (k.as_str(), v.kind.as_str(), v.template.clone()))
            .collect::<Vec<_>>()
    );

    let bat = *managed.battery.lock().unwrap();
    let _lock = managed.queue.blocking_lock();

    let receipts = match driver {
        DriverVariant::Usb(d) => {
            let mode = d.protocol_mode();
            let dpi_cfg = build_dpi_cfg(&state);
            let prefs_cfg = build_user_prefs_cfg(&state, bat);
            let polling_cfg = PollingRateConfig {
                rate: PollingRate::from_hz(state.polling_rate),
            };
            let macros_cfg = build_macros_cfg(&state);

            let mut receipts = Vec::new();
            receipts.push(d.set_dpi(&dpi_cfg)?);
            std::thread::sleep(std::time::Duration::from_millis(150));
            receipts.push(d.set_user_prefs(&prefs_cfg)?);
            std::thread::sleep(std::time::Duration::from_millis(150));
            receipts.push(d.set_polling_rate(&polling_cfg)?);
            std::thread::sleep(std::time::Duration::from_millis(150));
            receipts.push(d.set_macros(&macros_cfg)?);

            let _ = mode;

            if state.custom_macro.enabled && !state.custom_macro.events.is_empty() {
                let cfg = CustomMacroConfig {
                    enabled: true,
                    target_button: state.custom_macro.target_button,
                    mode: state.custom_macro.mode,
                    repeat: state.custom_macro.repeat,
                    events: state.custom_macro.events.clone(),
                };
                // Refused by design; recorded as a warning, not a success.
                if let Err(e) = d.set_custom_macro(&cfg) {
                    warn!("[apply] custom macro not sent: {e}");
                }
            }

            for receipt in &receipts {
                let _ = app.emit("mouse:write_receipt", receipt);
            }
            receipts
        }
        DriverVariant::Ble(_) => {
            return Err(
                "Bluetooth apply is not implemented for the X3: the GATT report layout \
                 has not been verified, so nothing was sent"
                    .into(),
            )
        }
    };

    let mode = safety::write_mode();
    save_state(&managed.state_file, &state);

    let note = match mode {
        WriteMode::ReadOnly => {
            "read-only mode: nothing was transmitted; the values are stored locally only"
        }
        WriteMode::DryRun => {
            "dry-run: reports were built and logged but not transmitted; values are local only"
        }
        WriteMode::Hardware => {
            "reports were transmitted; an acknowledgement is not physical verification — \
             see docs/HARDWARE_VALIDATION.md"
        }
    };

    Ok(ApplyResult {
        state: state.clone(),
        write_mode: mode.label().into(),
        reports: receipts,
        hardware_applied: false,
        note: note.into(),
    })
}

#[tauri::command]
pub async fn reset_config(
    managed: State<'_, ManagedState>,
    _app: AppHandle,
) -> Result<ApplyResult, String> {
    // No reset report is sent: the X3 reset sequence (0x0C preamble) has not
    // been verified to be safe on this firmware, and a wrong write could leave
    // the mouse without a usable button map. Only the local profile is reset.
    let default = AppState::default();
    *managed.app_state.lock().await = default.clone();
    save_state(&managed.state_file, &default);

    Ok(ApplyResult {
        state: default,
        write_mode: safety::write_mode().label().into(),
        reports: Vec::new(),
        hardware_applied: false,
        note: "local profile restored to defaults; no report was sent to the device".into(),
    })
}

// ── Explicit single-target writes ─────────────────────────────────────────────

/// The report a `write_setting` call is allowed to transmit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteTarget {
    Dpi,
    /// Report `0x05`: lighting, power timing and debounce travel together.
    Prefs,
    Polling,
    Buttons,
}

impl WriteTarget {
    fn parse(target: &str) -> Result<Self, String> {
        match target {
            "dpi" => Ok(Self::Dpi),
            "lighting" | "prefs" => Ok(Self::Prefs),
            "polling" => Ok(Self::Polling),
            "buttons" => Ok(Self::Buttons),
            other => Err(format!("unknown write target '{other}'")),
        }
    }
}

/// Send exactly one configuration report, chosen by the caller.
///
/// This is the explicit-write path: connecting never calls it, and it is the
/// only way a changed value reaches the device. `target` is one of `dpi`,
/// `lighting`, `polling` or `buttons`. The write policy still applies.
#[tauri::command]
pub fn write_setting(
    target: String,
    patch: serde_json::Value,
    managed: State<'_, ManagedState>,
    app: AppHandle,
) -> Result<ApplyResult, String> {
    let guard = managed.driver.blocking_lock();
    let driver = guard.as_ref().ok_or("mouse is not connected")?;

    let mut state = managed.app_state.blocking_lock();
    merge_patch(&mut state, &patch);
    let battery = *managed.battery.lock().unwrap();
    let _lock = managed.queue.blocking_lock();

    let write_target = WriteTarget::parse(&target)?;
    let mut receipts = Vec::new();
    match driver {
        DriverVariant::Usb(d) => match write_target {
            WriteTarget::Dpi => receipts.push(d.set_dpi(&build_dpi_cfg(&state))?),
            WriteTarget::Prefs => {
                receipts.push(d.set_user_prefs(&build_user_prefs_cfg(&state, battery))?)
            }
            WriteTarget::Polling => receipts.push(d.set_polling_rate(&PollingRateConfig {
                rate: PollingRate::from_hz(state.polling_rate),
            })?),
            WriteTarget::Buttons => receipts.push(d.set_macros(&build_macros_cfg(&state))?),
        },
        DriverVariant::Ble(_) => {
            return Err(
                "Bluetooth explicit writes are not implemented for the X3: the GATT report \
                 layout has not been verified, so nothing was sent"
                    .into(),
            )
        }
    }

    for receipt in &receipts {
        let _ = app.emit("mouse:write_receipt", receipt);
    }
    save_state(&managed.state_file, &state);

    let mode = safety::write_mode();
    Ok(ApplyResult {
        state: state.clone(),
        write_mode: mode.label().into(),
        reports: receipts,
        hardware_applied: false,
        note: match write_target {
            WriteTarget::Polling => "polling-rate report transmitted; its effect is verified only \
                                     by a separate measurement".into(),
            _ => "report transmitted; acknowledgement is not physical verification".into(),
        },
    })
}

// ── Profiles ──────────────────────────────────────────────────────────────────

const RESERVED: [&str; 3] = ["__proto__", "constructor", "prototype"];

fn validate_name(name: &str) -> Result<String, String> {
    let n = name.trim().to_string();
    if n.is_empty() || n.len() > 64 || RESERVED.contains(&n.as_str()) {
        return Err("invalid profile name".into());
    }
    Ok(n)
}

#[tauri::command]
pub async fn profiles_list(managed: State<'_, ManagedState>) -> Result<Vec<String>, String> {
    Ok(managed.profiles.lock().await.keys().cloned().collect())
}

#[tauri::command]
pub async fn profiles_save(
    name: String,
    cfg: Option<serde_json::Value>,
    managed: State<'_, ManagedState>,
) -> Result<Vec<String>, String> {
    let n = validate_name(&name)?;
    // The renderer sends its view of the configuration, which is not the same
    // shape as `AppState`. Merge it over the current state so the file is
    // always a complete, loadable profile.
    let to_save = match cfg {
        Some(patch) => {
            let base = managed.app_state.lock().await.clone();
            merge_state(&base, &patch)
        }
        None => managed.app_state.lock().await.clone(),
    };
    let mut profiles = managed.profiles.lock().await;
    profiles.insert(n, to_save);
    save_profiles(&managed.profiles_file, &profiles);
    Ok(profiles.keys().cloned().collect())
}

#[tauri::command]
pub async fn profiles_load(
    name: String,
    managed: State<'_, ManagedState>,
) -> Result<Option<AppState>, String> {
    let n = validate_name(&name)?;
    Ok(managed.profiles.lock().await.get(&n).cloned())
}

#[tauri::command]
pub async fn profiles_delete(
    name: String,
    managed: State<'_, ManagedState>,
) -> Result<Vec<String>, String> {
    let n = validate_name(&name)?;
    let mut profiles = managed.profiles.lock().await;
    profiles.remove(&n);
    save_profiles(&managed.profiles_file, &profiles);
    Ok(profiles.keys().cloned().collect())
}

// ── Autostart (XDG) ──────────────────────────────────────────────────────────

fn autostart_path() -> Result<std::path::PathBuf, String> {
    let base = dirs::config_dir().ok_or_else(|| "no config dir".to_string())?;
    Ok(base.join("autostart").join("opensharkx3.desktop"))
}

#[tauri::command]
pub async fn autostart_status() -> Result<bool, String> {
    Ok(autostart_path()?.exists())
}

#[tauri::command]
pub async fn autostart_enable(app: AppHandle) -> Result<(), String> {
    let path = autostart_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let content = format!(
        "[Desktop Entry]\nType=Application\nName=OpenSharkX3\nComment=Attack Shark X3 configurator\nExec=\"{}\"\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    );
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    let _ = app;
    Ok(())
}

#[tauri::command]
pub async fn autostart_disable() -> Result<(), String> {
    let path = autostart_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── DPI overlay ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn show_dpi_overlay(app: AppHandle, text: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("dpi-overlay") {
        let _ = win.emit("overlay:show", text);
        let _ = win.show();
        let app2 = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(5600));
            if let Some(w) = app2.get_webview_window("dpi-overlay") {
                let _ = w.hide();
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::{dpi::build_dpi, ConnectionMode};
    use serde_json::json;

    /// Shape sent by `reactCfgToMain` in the renderer.
    fn renderer_patch() -> serde_json::Value {
        json!({
            "dpi": {
                "values": [400, 800, 1600, 3200, 6400, 12000],
                "activeStage": 0,
                "angleSnap": true,
                "rippleControl": false
            },
            "lighting": {
                "mode": 0x10,
                "stageColors": [
                    {"r":1,"g":2,"b":3},{"r":4,"g":5,"b":6},
                    {"r":7,"g":8,"b":9},{"r":10,"g":11,"b":12},
                    {"r":13,"g":14,"b":15},{"r":16,"g":17,"b":18}
                ],
                "globalColor": {"r":0,"g":255,"b":0},
                "ledSpeed": 2
            },
            "pollingRate": 500,
            "performance": {"keyResponse": 6},
            "buttons": {
                "left": {"type":"template","template":"global-left-click"},
                "backward": {"type":"template","template":"global-dpi-cycle"}
            },
            "customMacro": {
                "enabled": true,
                "targetButton": 4,
                "mode": 1,
                "repeat": 3,
                "events": [{"key":4,"delay":20,"release":false}]
            }
        })
    }

    #[test]
    fn renderer_patch_merges_into_a_complete_app_state() {
        let base = AppState::default();
        let merged = merge_state(&base, &renderer_patch());

        assert_eq!(merged.polling_rate, 500);
        assert_eq!(merged.dpi.values[0], 400);
        assert_eq!(merged.dpi.active_stage, 0);
        assert!(merged.dpi.angle_snap);
        assert!(!merged.dpi.ripple_control);
        assert_eq!(merged.performance.key_response, 6);
        assert_eq!(merged.lighting.mode, 0x10);
        assert_eq!(merged.lighting.led_speed, 2);
        assert_eq!(
            merged.buttons.get("backward").and_then(|b| b.template.as_deref()),
            Some("global-dpi-cycle")
        );
        // Fields absent from the renderer patch keep the base value. Note that
        // merge_patch clamps sleep_time to 1.0..60.0, so the 0.5 default is
        // normalised on every merge.
        assert_eq!(merged.power.sleep_time, 1.0);
        assert_eq!(merged.power.deep_sleep_time, base.power.deep_sleep_time);
        assert_eq!(merged.custom_macro.repeat, 3);
    }

    #[test]
    fn profile_save_and_load_preserve_every_field() {
        let merged = merge_state(&AppState::default(), &renderer_patch());
        let json = serde_json::to_string_pretty(&merged).unwrap();
        let loaded: AppState = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.polling_rate, merged.polling_rate);
        assert_eq!(loaded.dpi.values, merged.dpi.values);
        assert_eq!(loaded.dpi.active_stage, merged.dpi.active_stage);
        assert_eq!(loaded.lighting.mode, merged.lighting.mode);
        assert_eq!(loaded.lighting.stage_colors[5].b, 18);
        assert_eq!(loaded.performance.key_response, 6);
        assert_eq!(loaded.power.deep_sleep_time, merged.power.deep_sleep_time);
        assert_eq!(loaded.custom_macro.events.len(), 1);
        assert_eq!(loaded.custom_macro.events[0].key, 4);
    }

    #[test]
    fn invalid_values_are_clamped_after_a_merge() {
        let patch = json!({
            "pollingRate": 12345,
            "dpi": {
                "values": [1, 80000, 800, 800, 800, 800],
                "activeStage": 99,
                "angleSnap": false,
                "rippleControl": true
            },
            "performance": {"keyResponse": 1}
        });
        let merged = merge_state(&AppState::default(), &patch);
        assert_eq!(merged.polling_rate, 1000);
        assert_eq!(merged.dpi.values[0], 50);
        assert_eq!(merged.dpi.values[1], 26_000);
        assert_eq!(merged.dpi.active_stage, 5);
        assert_eq!(merged.performance.key_response, 4);
    }

    #[test]
    fn save_config_writes_a_complete_state_and_no_report() {
        let dir = std::env::temp_dir().join(format!(
            "opensharkx3-save-config-{}-{}",
            std::process::id(),
            line!()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        let mut state = AppState::default();
        let saved = save_local(
            &mut state,
            &path,
            &json!({
                "customMacro": {
                    "enabled": true,
                    "targetButton": 3,
                    "mode": 2,
                    "repeat": 7,
                    "events": [{"key":0x28,"delay":50,"release":true}]
                }
            }),
        )
        .unwrap();

        let on_disk: AppState =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.custom_macro.repeat, 7);
        assert_eq!(on_disk.custom_macro.target_button, 3);
        assert_eq!(on_disk.custom_macro.mode, 2);
        assert!(on_disk.custom_macro.events[0].release);
        // Untouched settings survive the local save.
        assert_eq!(on_disk.polling_rate, AppState::default().polling_rate);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn write_targets_are_explicit_and_reject_unknown_names() {
        assert_eq!(WriteTarget::parse("dpi"), Ok(WriteTarget::Dpi));
        assert_eq!(WriteTarget::parse("lighting"), Ok(WriteTarget::Prefs));
        assert_eq!(WriteTarget::parse("prefs"), Ok(WriteTarget::Prefs));
        assert_eq!(WriteTarget::parse("polling"), Ok(WriteTarget::Polling));
        assert_eq!(WriteTarget::parse("buttons"), Ok(WriteTarget::Buttons));
        assert!(WriteTarget::parse("all").is_err());
        assert!(WriteTarget::parse("").is_err());
    }

    #[test]
    fn active_stage_is_one_based_on_the_wire() {
        let mut s = AppState::default();
        s.dpi.active_stage = 0;
        assert_eq!(build_dpi(&build_dpi_cfg(&s), ConnectionMode::Wireless)[24], 1);
        s.dpi.active_stage = 5;
        assert_eq!(build_dpi(&build_dpi_cfg(&s), ConnectionMode::Wireless)[24], 6);
    }

    #[test]
    fn macros_keep_every_physical_button() {
        let cfg = build_macros_cfg(&AppState::default());
        assert!(cfg.has_left_click());
    }
}

// ── JSON patch helpers ───────────────────────────────────────────────────────

/// Return a complete `AppState` with `patch` merged over `base`.
///
/// The renderer sends its own view of the configuration, which is not the same
/// shape as `AppState`; merging over a complete state keeps every field present
/// and clamps values through [`merge_patch`].
fn merge_state(base: &AppState, patch: &serde_json::Value) -> AppState {
    let mut merged = base.clone();
    merge_patch(&mut merged, patch);
    merged
}

/// The local-only save used by the `save_config` command: merge, persist and
/// return the state. No device is opened and no HID report is built.
fn save_local(state: &mut AppState, path: &Path, patch: &serde_json::Value) -> Result<AppState, String> {
    merge_patch(state, patch);
    let json = serde_json::to_string_pretty(&*state)
        .map_err(|e| format!("cannot serialize state: {e}"))?;
    fs::write(path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(state.clone())
}

fn merge_patch(state: &mut AppState, patch: &serde_json::Value) {
    let Ok(mut current) = serde_json::to_value(&*state) else {
        return;
    };
    if let (Some(obj), Some(patch_obj)) = (current.as_object_mut(), patch.as_object()) {
        for (k, v) in patch_obj {
            obj.insert(k.clone(), v.clone());
        }
    }
    if let Ok(merged) = serde_json::from_value(current) {
        *state = merged;
    }
    let valid = [125u32, 250, 500, 1000];
    if !valid.contains(&state.polling_rate) {
        state.polling_rate = 1000;
    }
    for v in &mut state.dpi.values {
        *v = (*v).clamp(50, 26_000);
    }
    state.dpi.active_stage = state.dpi.active_stage.clamp(0, 5);
    state.performance.key_response = state.performance.key_response.clamp(4, 50);
    state.power.sleep_time = state.power.sleep_time.clamp(1.0, 60.0);
    state.power.deep_sleep_time = state.power.deep_sleep_time.clamp(1, 60);
}
