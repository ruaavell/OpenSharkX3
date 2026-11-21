//! Linux-native HID transport for the Attack Shark X3.
//!
//! The X3 exposes four HID interfaces. Interface 2 is the configuration
//! interface: it carries vendor-defined **feature** reports and a small
//! **input** report channel (report `0x03`) used for status events.
//!
//! This module deliberately talks to `/dev/hidraw*` with `HIDIOCSFEATURE` and
//! `HIDIOCGFEATURE` instead of claiming the USB interface through libusb. The
//! kernel driver stays bound, the mouse keeps working during a configuration
//! write, and no `USBDEVFS_DISCONNECT` is ever issued.
//!
//! Device discovery is by sysfs identity (VID/PID/interface), never by a
//! hard-coded `/dev/hidrawN` number.

use serde::Serialize;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::{self, File, OpenOptions},
    io::{self, Read},
    os::{
        fd::AsRawFd,
        raw::{c_int, c_short, c_ulong},
    },
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::debug;

pub const VID: u16 = 0x1D57;
pub const PID_WIRELESS: u16 = 0xFA60;
pub const PID_WIRED: u16 = 0xFA55;
pub const CONFIG_INTERFACE: u8 = 2;

// HID_MAX_BUFFER_SIZE in the kernel is 16384. A single report is far smaller,
// but reads must provide a buffer at least as large as the report.
const READ_BUFFER: usize = 64;

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

const POLLIN: c_short = 0x0001;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: c_int) -> c_int;
}

/// `_IOC()` from `<asm-generic/ioctl.h>`.
const fn ioc(dir: c_ulong, ty: u8, nr: u8, size: usize) -> c_ulong {
    (dir << 30) | ((size as c_ulong) << 16) | ((ty as c_ulong) << 8) | (nr as c_ulong)
}

/// `HIDIOCSFEATURE(len)` — `_IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, len)`.
const fn hidiocsfeature(len: usize) -> c_ulong {
    ioc(3, b'H', 0x06, len)
}

/// `HIDIOCGFEATURE(len)` — `_IOC(_IOC_WRITE|_IOC_READ, 'H', 0x07, len)`.
const fn hidiocgfeature(len: usize) -> c_ulong {
    ioc(3, b'H', 0x07, len)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HidrawInfo {
    /// `/dev/hidrawN` as discovered for this boot.
    pub node: String,
    /// Canonical sysfs path of the HID device (`.../0003:1D57:FA60.0008`).
    pub sysfs: String,
    pub bus: u16,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: Option<u8>,
    pub name: String,
    /// Feature report ID → total bytes (report ID byte included).
    pub feature_reports: BTreeMap<u8, usize>,
    /// Input report ID → total bytes (report ID byte included).
    pub input_reports: BTreeMap<u8, usize>,
    /// Output report ID → total bytes (report ID byte included).
    pub output_reports: BTreeMap<u8, usize>,
    pub descriptor_len: usize,
    /// Top-level collection usage pages/usages, for diagnostics.
    pub top_level_usages: Vec<(u16, u16)>,
    #[serde(skip_serializing)]
    pub descriptor: Vec<u8>,
}

impl HidrawInfo {
    /// The largest feature report declared by this interface.
    pub fn max_feature_len(&self) -> usize {
        self.feature_reports.values().copied().max().unwrap_or(0)
    }

    pub fn has_feature(&self, report_id: u8) -> bool {
        self.feature_reports.contains_key(&report_id)
    }
}

fn read_hex_u16(value: &str) -> Option<u16> {
    u16::from_str_radix(value.trim(), 16).ok()
}

fn read_uevent_field(uevent: &str, key: &str) -> Option<String> {
    uevent.lines().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
            .map(|v| v.trim().to_string())
    })
}

fn parse_hid_id(uevent: &str) -> Option<(u16, u16, u16)> {
    // HID_ID=0003:00001D57:0000FA60
    let raw = read_uevent_field(uevent, "HID_ID")?;
    let mut parts = raw.split(':');
    let bus = read_hex_u16(parts.next()?)?;
    let vid = read_hex_u16(parts.next()?)?;
    let pid = read_hex_u16(parts.next()?)?;
    Some((bus, vid, pid))
}

fn read_interface(sysfs: &Path) -> Option<u8> {
    // .../3-3.3:1.2/0003:1D57:FA60.0008 — bInterfaceNumber lives in the
    // parent USB interface directory.
    let path = sysfs.parent()?.join("bInterfaceNumber");
    let text = fs::read_to_string(path).ok()?;
    u8::from_str_radix(text.trim().trim_start_matches("0x"), 16).ok()
}

/// Parse a HID report descriptor into per-report-ID byte lengths.
///
/// Feature/input/output items accumulate into the current report ID. A report
/// ID byte is counted once when the descriptor has report IDs.
pub fn parse_report_sizes(
    descriptor: &[u8],
) -> (
    BTreeMap<u8, usize>,
    BTreeMap<u8, usize>,
    BTreeMap<u8, usize>,
    Vec<(u16, u16)>,
) {
    let mut feature = BTreeMap::new();
    let mut input = BTreeMap::new();
    let mut output = BTreeMap::new();
    let mut top_level = Vec::new();

    let mut report_size: usize = 0;
    let mut report_count: usize = 0;
    let mut report_id: u8 = 0;
    let mut usage_page: u16 = 0;
    let mut usage: u16 = 0;
    let mut depth: usize = 0;

    let mut i = 0usize;
    while i < descriptor.len() {
        let prefix = descriptor[i];
        if prefix == 0xFE {
            // Long item: 0xFE, size, tag, data...
            let size = descriptor.get(i + 1).copied().unwrap_or(0) as usize;
            i += 3 + size;
            continue;
        }

        let n = match prefix & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        };
        let typ = (prefix >> 2) & 0x03;
        let tag = prefix >> 4;
        let value = if n > 0 && i + 1 + n <= descriptor.len() {
            descriptor[i + 1..i + 1 + n]
                .iter()
                .enumerate()
                .fold(0u32, |acc, (idx, b)| acc | ((*b as u32) << (8 * idx)))
        } else {
            0
        };
        i += 1 + n;

        match typ {
            0 => match tag {
                8 => add_report(&mut input, report_id, report_size, report_count),
                9 => add_report(&mut output, report_id, report_size, report_count),
                10 => {
                    if depth == 0 {
                        top_level.push((usage_page, usage));
                    }
                    depth += 1;
                }
                12 => depth = depth.saturating_sub(1),
                11 => add_report(&mut feature, report_id, report_size, report_count),
                _ => {}
            },
            1 => match tag {
                0 => usage_page = value as u16,
                7 => report_size = value as usize,
                8 => report_id = value as u8,
                9 => report_count = value as usize,
                _ => {}
            },
            2 => {
                if tag == 0 {
                    usage = value as u16;
                }
            }
            _ => {}
        }
    }

    (feature, input, output, top_level)
}

fn add_report(map: &mut BTreeMap<u8, usize>, report_id: u8, size: usize, count: usize) {
    let bits = size.saturating_mul(count);
    let bytes = bits.div_ceil(8);
    if bytes == 0 {
        return;
    }
    // Report ID 0 is not transmitted on the wire. Every other report needs one
    // byte for the ID itself.
    let entry = map.entry(report_id).or_insert(if report_id == 0 { 0 } else { 1 });
    *entry += bytes;
}

/// Enumerate every hidraw node on the system.
pub fn find_all() -> Vec<HidrawInfo> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hidraw") else {
        return out;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("hidraw") {
            continue;
        }

        let link = entry.path().join("device");
        let Ok(sysfs) = fs::canonicalize(&link) else {
            continue;
        };
        let Ok(uevent) = fs::read_to_string(sysfs.join("uevent")) else {
            continue;
        };
        let Some((bus, vid, pid)) = parse_hid_id(&uevent) else {
            continue;
        };
        let descriptor = fs::read(sysfs.join("report_descriptor")).unwrap_or_default();
        let (feature, input, output, top_level) = parse_report_sizes(&descriptor);
        let hid_name = read_uevent_field(&uevent, "HID_NAME").unwrap_or_default();

        out.push(HidrawInfo {
            node: format!("/dev/{name}"),
            sysfs: sysfs.display().to_string(),
            bus,
            vendor_id: vid,
            product_id: pid,
            interface: read_interface(&sysfs),
            name: hid_name,
            feature_reports: feature,
            input_reports: input,
            output_reports: output,
            descriptor_len: descriptor.len(),
            top_level_usages: top_level,
            descriptor,
        });
    }

    out.sort_by(|a, b| a.node.cmp(&b.node));
    out
}

/// Find the configuration interface for a known Attack Shark identity.
///
/// Preference order:
/// 1. USB interface 2 with feature reports (the vendor configuration interface)
/// 2. any interface with more than one feature report
/// 3. any interface that has at least one feature report
pub fn find_config(product_id: u16) -> Option<HidrawInfo> {
    let candidates: Vec<HidrawInfo> = find_all()
        .into_iter()
        .filter(|d| d.vendor_id == VID && d.product_id == product_id)
        .collect();

    candidates
        .iter()
        .find(|d| d.interface == Some(CONFIG_INTERFACE) && !d.feature_reports.is_empty())
        .or_else(|| candidates.iter().find(|d| d.feature_reports.len() > 1))
        .or_else(|| candidates.iter().find(|d| !d.feature_reports.is_empty()))
        .cloned()
}

/// Find any X3-family receiver or cable, preferring the configuration interface.
pub fn find_any_x3() -> Option<HidrawInfo> {
    find_config(PID_WIRELESS).or_else(|| find_config(PID_WIRED))
}

/// Find the mouse data interface (the one carrying movement reports).
///
/// Polling-rate measurement has to observe real input reports, so it uses this
/// interface rather than the vendor configuration interface. Interface 1 is the
/// mouse endpoint on the FA60 descriptor; as a fallback the top-level HID usage
/// (Generic Desktop / Mouse) is used. No report is ever written here.
pub fn find_mouse_input(product_id: u16) -> Option<HidrawInfo> {
    let candidates: Vec<HidrawInfo> = find_all()
        .into_iter()
        .filter(|d| d.vendor_id == VID && d.product_id == product_id)
        .collect();

    candidates
        .iter()
        .find(|d| d.interface == Some(1) && !d.input_reports.is_empty())
        .or_else(|| {
            candidates.iter().find(|d| {
                d.top_level_usages
                    .iter()
                    .any(|&(page, usage)| page == 0x01 && usage == 0x02)
            })
        })
        .cloned()
}

/// Hex dump of a report descriptor, one report per line, for diagnostics.
pub fn descriptor_hex(descriptor: &[u8]) -> String {
    descriptor
        .chunks(16)
        .map(|chunk| {
            chunk
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A single open hidraw file description.
///
/// Reads and ioctls are issued on the raw file descriptor; no USB interface is
/// claimed or detached.
pub struct HidrawHandle {
    file: File,
    path: PathBuf,
}

impl HidrawHandle {
    pub fn open(info: &HidrawInfo, writable: bool) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true).write(writable);
        let file = options.open(&info.node)?;
        debug!(
            "opened {} (writable={writable}) for {:04x}:{:04x} interface {:?}",
            info.node, info.vendor_id, info.product_id, info.interface
        );
        Ok(Self {
            file,
            path: PathBuf::from(&info.node),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Send one HID feature report.
    ///
    /// The length is the full report, including the report ID byte. hidraw
    /// accepts reports longer than the descriptor's declared size; this is how
    /// the vendor preamble (10 bytes for a descriptor that declares 6) reaches
    /// the device.
    pub fn set_feature(&self, report: &[u8]) -> io::Result<usize> {
        if report.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a feature report needs at least the report ID and one data byte",
            ));
        }
        let mut buffer = report.to_vec();
        let request = hidiocsfeature(buffer.len());
        let ret = unsafe {
            ioctl(
                self.file.as_raw_fd(),
                request,
                buffer.as_mut_ptr() as *mut c_void,
            )
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    /// Read one HID feature report.
    ///
    /// The X3's configuration reports are write-only in practice: a
    /// `GET_FEATURE` on interface 2 blocks until the USB timeout. The function
    /// exists for diagnostics and future firmware, not for the normal path.
    pub fn get_feature(&self, report_id: u8, len: usize) -> io::Result<Vec<u8>> {
        if len < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "feature report length must be at least 1",
            ));
        }
        let mut buffer = vec![0u8; len];
        buffer[0] = report_id;
        let request = hidiocgfeature(len);
        let ret = unsafe {
            ioctl(
                self.file.as_raw_fd(),
                request,
                buffer.as_mut_ptr() as *mut c_void,
            )
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            buffer.truncate(ret as usize);
            Ok(buffer)
        }
    }

    /// Wait for one input report. Returns `Ok(None)` on timeout.
    pub fn read_input(&mut self, timeout: Duration) -> io::Result<Option<Vec<u8>>> {
        let mut pfd = PollFd {
            fd: self.file.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        };
        let millis = timeout.as_millis().min(i32::MAX as u128) as c_int;
        let ret = unsafe { poll(&mut pfd, 1, millis) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        if ret == 0 {
            return Ok(None);
        }
        if pfd.revents & POLLIN == 0 {
            return Ok(None);
        }

        let mut buffer = [0u8; READ_BUFFER];
        match self.file.read(&mut buffer) {
            Ok(0) => Ok(None),
            Ok(n) => Ok(Some(buffer[..n].to_vec())),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Utility used by tests: parse a hex string into bytes.
pub fn hex_bytes(text: &str) -> Vec<u8> {
    text.split_whitespace()
        .filter_map(|token| u8::from_str_radix(token.trim_start_matches("0x"), 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioctl_numbers_match_the_kernel_abi() {
        // HIDIOCSFEATURE(10) == 0xC00A4806 on Linux.
        assert_eq!(hidiocsfeature(10), 0xC00A_4806);
        // HIDIOCGFEATURE(52) == 0xC0344807.
        assert_eq!(hidiocgfeature(52), 0xC034_4807);
    }

    #[test]
    fn parses_the_captured_fa60_descriptor() {
        // The complete 236-byte interface-2 report descriptor read from the
        // live receiver. Keeping it in the test suite pins the parser to the
        // hardware we actually support.
        let descriptor = hex_bytes(include_str!("../tests/data/fa60-iface2-descriptor.hex"));
        assert_eq!(descriptor.len(), 236);

        let (feature, input, output, _) = parse_report_sizes(&descriptor);
        assert_eq!(feature.get(&0x04), Some(&52));
        assert_eq!(feature.get(&0x05), Some(&13));
        assert_eq!(feature.get(&0x06), Some(&9));
        assert_eq!(feature.get(&0x08), Some(&59));
        assert_eq!(feature.get(&0x09), Some(&64));
        assert_eq!(feature.get(&0x0C), Some(&6));
        assert_eq!(feature.get(&0x23), Some(&262));
        assert_eq!(feature.get(&0xA0), Some(&8));
        assert_eq!(input.get(&0x03), Some(&5));
        assert!(output.is_empty());
    }

    #[test]
    fn parses_a_synthetic_report_descriptor() {
        // Usage Page (Vendor), Usage (1), Collection (Application),
        //   Report ID 4, Report Size 8, Report Count 3, Feature (Data,Var,Abs),
        // End Collection
        let descriptor = [
            0x06, 0x00, 0xFF, // Usage Page (Vendor 0xFF00)
            0x09, 0x01, // Usage (1)
            0xA1, 0x01, // Collection (Application)
            0x85, 0x04, // Report ID (4)
            0x75, 0x08, // Report Size (8)
            0x95, 0x03, // Report Count (3)
            0xB1, 0x02, // Feature
            0xC0, // End Collection
        ];
        let (feature, _, _, top) = parse_report_sizes(&descriptor);
        assert_eq!(feature.get(&0x04), Some(&4)); // 1 ID byte + 3 data bytes
        assert_eq!(top, vec![(0xFF00, 1)]);
    }

    #[test]
    fn ignores_zero_length_items() {
        let descriptor = [0x85, 0x05, 0x75, 0x00, 0x95, 0x00, 0xB1, 0x02];
        let (feature, _, _, _) = parse_report_sizes(&descriptor);
        assert!(feature.is_empty());
    }
}
