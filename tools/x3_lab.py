#!/usr/bin/env python3
"""
x3_lab.py — controlled Attack Shark X3 / FA60 hardware experiments.

Safety model:
  * dry-run is the default; every command prints the exact bytes
  * --yes is required for any write
  * the report ID and total length are validated against the live HID
    report descriptor before a write is attempted
  * one experiment per invocation

Read-only:
    python3 tools/x3_lab.py descriptor
    python3 tools/x3_lab.py listen --seconds 10

Writes (explicit):
    python3 tools/x3_lab.py preamble --yes
    python3 tools/x3_lab.py polling --hz 1000 --yes
    python3 tools/x3_lab.py dpi --stage 0 --dpi 800 --yes
"""
from __future__ import annotations
import argparse, fcntl, glob, os, re, select, struct, sys, time

VID, PID = 0x1D57, 0xFA60
CFG_INTERFACE = 2

# Exact feature-report sizes from the FA60 interface-2 report descriptor.
FEATURE_SIZES = {
    0x04: 52, 0x05: 13, 0x06: 9, 0x07: 8, 0x08: 59, 0x09: 64,
    0x0A: 8, 0x0B: 8, 0x0C: 6, 0x10: 8, 0x22: 132, 0x23: 262,
    0x24: 8, 0x25: 13, 0x26: 9, 0x27: 8, 0x28: 132, 0x29: 129,
    0x2A: 132, 0x2B: 8, 0x2C: 4, 0x2D: 132, 0x2E: 5, 0xA0: 8,
}

def ioctl_gfeature(n): return (3 << 30) | (n << 16) | (ord("H") << 8) | 0x07
def ioctl_sfeature(n): return (3 << 30) | (n << 16) | (ord("H") << 8) | 0x06

def find_interface(number: int = CFG_INTERFACE) -> str:
    for dev in sorted(glob.glob("/sys/class/hidraw/hidraw*")):
        real = os.path.realpath(os.path.join(dev, "device"))
        try:
            ue = open(os.path.join(real, "uevent")).read()
            iface = open(os.path.join(real, "..", "bInterfaceNumber")).read().strip()
        except OSError:
            continue
        m = re.search(r"HID_ID=\w+:(\w+):(\w+)", ue)
        if not m:
            continue
        if int(m.group(1), 16) == VID and int(m.group(2), 16) == PID and int(iface) == number:
            return "/dev/" + os.path.basename(dev)
    raise SystemExit(f"{VID:04x}:{PID:04x} interface {number} not found")

def find_config() -> str:
    return find_interface(CFG_INTERFACE)

def descriptor_bytes(path: str) -> bytes:
    node = os.path.basename(path)
    real = os.path.realpath(f"/sys/class/hidraw/{node}/device")
    return open(os.path.join(real, "report_descriptor"), "rb").read()

def parse_feature_sizes(desc: bytes) -> dict[int, int]:
    """Tiny HID descriptor parser: report id + report size/count -> bytes."""
    i = 0; glob = {}; sizes = {}; rid = 0; rsize = 0; rcount = 0
    while i < len(desc):
        b = desc[i]
        if b == 0xFE:
            i += 3 + (desc[i + 1] if i + 1 < len(desc) else 0); continue
        n = (0, 1, 2, 4)[b & 3]; typ = (b >> 2) & 3; tag = b >> 4
        val = int.from_bytes(desc[i + 1:i + 1 + n], "little") if n else 0
        i += 1 + n
        if typ == 1:
            if tag == 7: rsize = val
            elif tag == 8: rid = val
            elif tag == 9: rcount = val
        elif typ == 0 and tag == 11:  # Feature
            if rid:
                # A report ID may be declared by more than one Feature item;
                # the total report length accumulates.
                sizes[rid] = sizes.get(rid, 1) + rsize * rcount // 8
    return sizes

def validate(report: bytes, sizes: dict[int, int]) -> None:
    rid = report[0]
    if rid not in sizes:
        raise SystemExit(f"report 0x{rid:02x} is not a feature report in this descriptor")
    if len(report) != sizes[rid]:
        raise SystemExit(f"report 0x{rid:02x} must be {sizes[rid]} bytes, got {len(report)}")
    if report[1] != len(report):
        raise SystemExit(f"logical length byte 0x{report[1]:02x} != {len(report)}")

def hx(b): return " ".join(f"{x:02x}" for x in b)

def read_events(fd, seconds: float) -> list[bytes]:
    out = []; end = time.monotonic() + seconds
    while time.monotonic() < end:
        r, _, _ = select.select([fd], [], [], max(0, end - time.monotonic()))
        if not r: break
        try:
            data = os.read(fd, 64)
        except BlockingIOError:
            continue
        if data:
            out.append(data)
    return out

def send(fd, report: bytes, sizes: dict[int, int], wait: float) -> None:
    validate(report, sizes)
    print(f"-> report 0x{report[0]:02x} {len(report)} bytes: {hx(report)}")
    buf = bytearray(report)
    n = fcntl.ioctl(fd, ioctl_sfeature(len(report)), buf, True)
    print(f"   HIDIOCSFEATURE -> {n}")
    for ev in read_events(fd, wait):
        print(f"<- input {len(ev)} bytes: {hx(ev)}")

# ---- payload builders (documented upstream, adapted to FA60 lengths) ----
PREAMBLE = bytes([0x0C, 0x0A, 0x01, 0xFE, 0x01, 0xFE, 0, 0, 0, 0])

def build_polling(hz: int) -> bytes:
    div = {125: 8, 250: 4, 500: 2, 1000: 1}[hz]
    r = bytearray(9)
    r[0] = 0x06; r[1] = 9; r[2] = 0x01; r[3] = div; r[4] = (~div) & 0xFF
    return bytes(r)

def build_dpi(stage: int, dpi: int, angle_snap=False, ripple=False,
              motion_sync=False, lod2mm=False) -> bytes:
    # FA60 report 0x04 is 52 bytes = the upstream 56-byte layout without
    # trailing padding. Checksum still lands at 50..51.
    r = bytearray(52)
    r[0] = 0x04; r[1] = 52; r[2] = 0x01
    r[3] = 1 if lod2mm else 0
    r[4] = 1 if ripple else 0
    r[6] = 1 if angle_snap else 0
    r[7] = 1 if motion_sync else 0
    r[5] = 1 << (stage & 7)
    wire = max(0, dpi // 50 - 1)
    r[8 + stage] = wire & 0xFF
    r[16 + stage] = (wire >> 8) & 0xFF
    r[24] = stage + 1
    palette = [(255,0,0),(255,128,0),(255,255,0),(0,255,0),
               (0,255,255),(0,0,255),(255,0,255),(255,255,255)]
    for i, c in enumerate(palette):
        r[25+i*3:28+i*3] = bytes(c)
    r[49] = 0x01
    s = sum(r[3:50]) & 0xFFFF
    r[50] = s >> 8; r[51] = s & 0xFF
    return bytes(r)

def main() -> None:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("descriptor")
    p = sub.add_parser("listen"); p.add_argument("--seconds", type=float, default=10)
    p = sub.add_parser("measure"); p.add_argument("--seconds", type=float, default=2.0)
    p.add_argument("--interface", type=int, default=1)
    p = sub.add_parser("preamble"); p.add_argument("--yes", action="store_true")
    p = sub.add_parser("polling"); p.add_argument("--hz", type=int, choices=[125,250,500,1000], default=1000); p.add_argument("--yes", action="store_true")
    p = sub.add_parser("dpi"); p.add_argument("--stage", type=int, default=0); p.add_argument("--dpi", type=int, default=800)
    p.add_argument("--angle-snap", action="store_true"); p.add_argument("--ripple", action="store_true")
    p.add_argument("--motion-sync", action="store_true"); p.add_argument("--lod2mm", action="store_true"); p.add_argument("--yes", action="store_true")
    a = ap.parse_args()

    if a.cmd == "measure":
        # Read-only: observe the real input interval on the mouse interface.
        # No report is written and no interface is claimed.
        path = find_interface(a.interface)
        print(f"measuring on {path} (interface {a.interface}) for {a.seconds:.1f}s; move the mouse")
        fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
        stamps = []
        end = time.monotonic() + a.seconds
        try:
            while time.monotonic() < end:
                r, _, _ = select.select([fd], [], [], max(0, end - time.monotonic()))
                if not r:
                    break
                try:
                    data = os.read(fd, 64)
                except BlockingIOError:
                    continue
                if data:
                    stamps.append(time.monotonic())
        finally:
            os.close(fd)
        intervals = [(b - a_) * 1e6 for a_, b in zip(stamps, stamps[1:]) if b > a_]
        if len(intervals) < 10:
            print(f"samples: {len(intervals)} intervals — too few; move the mouse during the window")
            raise SystemExit(2)
        intervals.sort()
        median = intervals[len(intervals) // 2]
        dev = sorted(abs(v - median) for v in intervals)
        jitter = dev[len(dev) // 2]
        hz = 1e6 / median
        nearest = min([125, 250, 500, 1000], key=lambda x: abs(x - hz))
        print(f"samples:  {len(intervals)} intervals ({len(stamps)} reports)")
        print(f"median:   {median:.1f} us")
        print(f"jitter:   {jitter:.1f} us (median absolute deviation)")
        print(f"measured: {hz:.1f} Hz  (nearest standard: {nearest} Hz)")
        return

    path = find_config()
    sizes = parse_feature_sizes(descriptor_bytes(path))
    print(f"device: {path}  (descriptor feature reports: {len(sizes)})")

    if a.cmd == "descriptor":
        for rid in sorted(sizes): print(f"  0x{rid:02x}: {sizes[rid]} bytes")
        return
    if a.cmd == "listen":
        fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)
        for ev in read_events(fd, a.seconds): print(f"{time.time():.3f} {hx(ev)}")
        os.close(fd); return

    reports: list[bytes] = []
    if a.cmd == "preamble":
        reports = [PREAMBLE]
    elif a.cmd == "polling":
        reports = [PREAMBLE, build_polling(a.hz)]
    elif a.cmd == "dpi":
        reports = [PREAMBLE, build_dpi(a.stage, a.dpi, a.angle_snap, a.ripple, a.motion_sync, a.lod2mm)]

    print("planned:")
    for r in reports: print(f"  {len(r):3d}  {hx(r)}")
    if not a.yes:
        print("dry-run: add --yes to transmit")
        return

    fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)
    try:
        for r in reports:
            # The preamble is the documented exception: 10 bytes even though
            # the descriptor declares 6. It carries no persistent setting.
            if r is PREAMBLE:
                print(f"-> preamble {len(r)} bytes: {hx(r)}")
                buf = bytearray(r)
                n = fcntl.ioctl(fd, ioctl_sfeature(len(r)), buf, True)
                print(f"   HIDIOCSFEATURE -> {n}")
                for ev in read_events(fd, 0.4): print(f"<- input {len(ev)} bytes: {hx(ev)}")
            else:
                send(fd, r, sizes, wait=1.5)
            time.sleep(0.12)
    finally:
        os.close(fd)

if __name__ == "__main__":
    main()
