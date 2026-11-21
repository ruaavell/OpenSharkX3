#!/usr/bin/env python3
"""
x3_probe.py — Attack Shark X3 için SALT-OKUNUR keşif aracı.

Mouse'a HİÇBİR ŞEY YAZMAZ. Sadece /sys üzerinden HID report descriptor'larını
okur ve X11 protokolüyle (docs/protocol/PROTOCOL_EN.md) karşılaştırır.

Kullanım:
    python3 tools/x3_probe.py                 # bağlı 1d57:* cihazları tara
    python3 tools/x3_probe.py --vid 1d57 --pid fa60
    python3 tools/x3_probe.py --hex dump.txt  # usbhid-dump çıktısından/hex'ten çöz
Çıktıyı olduğu gibi kopyalayıp paylaş.
"""
import argparse, glob, os, re, sys

# X11 protokolünün beklediği Feature report ID'leri ve toplam boyutları (ID dahil)
X11_EXPECTED = {0x04: 56, 0x05: 15, 0x06: 9, 0x08: 59}
UP = {0x01: "Generic Desktop", 0x0C: "Consumer", 0x07: "Keyboard", 0x09: "Button"}

def parse_descriptor(data: bytes):
    """Report ID bazında Input/Output/Feature bit toplamlarını çıkarır."""
    i, gl, loc = 0, {}, {}
    stack, depth = [], 0
    reports = {}   # (tip, id) -> bit
    tops = []      # üst seviye collection'lar: (usage_page, usage)
    while i < len(data):
        b = data[i]
        if b == 0xFE:                       # long item
            i += 3 + (data[i + 1] if i + 1 < len(data) else 0); continue
        size = (0, 1, 2, 4)[b & 3]
        typ, tag = (b >> 2) & 3, b >> 4
        val = int.from_bytes(data[i + 1:i + 1 + size], "little") if size else 0
        i += 1 + size
        if typ == 1:                        # global
            if tag == 0: gl["page"] = val
            elif tag == 7: gl["size"] = val
            elif tag == 8: gl["id"] = val
            elif tag == 9: gl["count"] = val
            elif tag == 10: stack.append(dict(gl))
            elif tag == 11 and stack: gl = stack.pop()
        elif typ == 2:                      # local
            if tag == 0: loc["usage"] = val
        elif typ == 0:                      # main
            if tag in (8, 9, 11):           # Input, Output, Feature
                name = {8: "Input", 9: "Output", 11: "Feature"}[tag]
                key = (name, gl.get("id", 0))
                reports[key] = reports.get(key, 0) + gl.get("size", 0) * gl.get("count", 0)
            elif tag == 10:
                if depth == 0:
                    tops.append((gl.get("page", 0), loc.get("usage", 0)))
                depth += 1
            elif tag == 12:
                depth -= 1
            loc = {}
    out = {}
    for (name, rid), bits in reports.items():
        total = (bits + 7) // 8 + (1 if rid else 0)
        out.setdefault(name, {})[rid] = total
    return out, tops

def describe(label, data):
    reports, tops = parse_descriptor(data)
    print(f"  {label}: {len(data)} bayt descriptor")
    for page, usage in tops:
        print(f"    üst seviye: usage_page=0x{page:04x} ({UP.get(page, 'vendor/diğer')}) usage=0x{usage:02x}")
    for kind in ("Input", "Output", "Feature"):
        for rid, total in sorted(reports.get(kind, {}).items()):
            print(f"    {kind:7s} report_id=0x{rid:02x} toplam={total} bayt")
    feats = reports.get("Feature", {})
    hits = {r: s for r, s in X11_EXPECTED.items() if feats.get(r) == s}
    near = {r: feats[r] for r in X11_EXPECTED if r in feats and feats[r] != X11_EXPECTED[r]}
    if hits: print(f"    >> X11 ile BİREBİR uyan Feature report'lar: {[hex(r) for r in hits]}")
    if near: print(f"    >> ID aynı ama boyut FARKLI: { {hex(r): s for r, s in near.items()} } (X11 boyutları: {X11_EXPECTED})")
    return reports

def scan(vid, pid):
    found = False
    for dev in sorted(glob.glob("/sys/class/hidraw/hidraw*")):
        real = os.path.realpath(os.path.join(dev, "device"))
        try:
            uevent = open(os.path.join(real, "uevent")).read()
        except OSError:
            continue
        m = re.search(r"HID_ID=\w+:(\w+):(\w+)", uevent)
        if not m: continue
        v, p = int(m.group(1), 16), int(m.group(2), 16)
        if v != vid or (pid is not None and p != pid): continue
        found = True
        name = re.search(r"HID_NAME=(.*)", uevent)
        iface = "?"
        try: iface = open(os.path.join(real, "..", "bInterfaceNumber")).read().strip()
        except OSError: pass
        print(f"\n{os.path.basename(dev)}  {v:04x}:{p:04x}  '{name.group(1) if name else ''}'  USB interface={iface}")
        try:
            describe("report_descriptor", open(os.path.join(real, "report_descriptor"), "rb").read())
        except OSError as e:
            print(f"  descriptor okunamadı: {e}")
    if not found:
        print(f"{vid:04x}:{pid if pid is not None else 0:04x} için hidraw cihazı bulunamadı (mouse/dongle takılı mı?)")

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vid", default="1d57"); ap.add_argument("--pid", default=None)
    ap.add_argument("--hex", help="hex dosyası (usbhid-dump çıktısı da olur)")
    a = ap.parse_args()
    if a.hex:
        text = open(a.hex).read() if os.path.exists(a.hex) else a.hex
        text = "\n".join(l for l in text.splitlines() if not re.search(r"[g-zG-Z]", l))
        describe("dosya", bytes.fromhex("".join(re.findall(r"\b[0-9a-fA-F]{2}\b", text))))
    else:
        scan(int(a.vid, 16), int(a.pid, 16) if a.pid else None)

if __name__ == "__main__":
    main()
