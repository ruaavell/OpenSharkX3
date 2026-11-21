#!/usr/bin/env python3
"""usbmon pcapng dosyasından X3 dongle'ının ayar/olay paketlerini ayıklar (mouse hareketini atar).
Kullanım: x3_capture_check.py yakalama.pcapng DEVNO ciktisi.txt"""
import struct, sys, collections
path, dev, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
data = open(path, "rb").read()
pos, rows = 0, []
while pos + 12 <= len(data):
    bt, bl = struct.unpack_from("<II", data, pos)
    if bl < 12: break
    if bt == 6:
        b = data[pos + 8:pos + bl - 4]
        _, th, tl, cap, _ = struct.unpack_from("<IIIII", b, 0)
        p = b[20:20 + cap]
        if len(p) >= 64:
            (_, typ, xfer, ep, d, _, fs, _, _, _, st, ul, dl) = struct.unpack_from("<QBBBBHbbqiiII", p, 0)
            rows.append(dict(ts=((th << 32) | tl) * 1e-6, t=chr(typ), x=xfer, ep=ep, dev=d,
                             setup=p[40:48] if fs == 0 else b"", data=p[64:64 + dl]))
    pos += bl
if not rows:
    print("Yakalama dosyasında kayıt yok."); sys.exit(1)
t0 = rows[0]["ts"]
lines, sets = [], collections.Counter()
for r in rows:
    if r["dev"] != dev or r["ep"] == 0x82:
        continue
    s = r["setup"]
    if r["x"] == 2 and s and s[0] == 0x80:   # standart descriptor okumaları
        continue
    lines.append("%7.2f %s x=%d ep=%#04x setup=%s data=%s" % (r["ts"] - t0, r["t"], r["x"], r["ep"], s.hex() or "-", r["data"].hex()))
    if r["x"] == 2 and s and s[0] == 0x21 and s[1] == 0x09 and r["t"] == "S":
        sets[(s[3], len(r["data"]))] += 1
open(out, "w").write("\n".join(lines) + "\n")
print("toplam kayıt: %d, süre: %.1f sn" % (len(rows), rows[-1]["ts"] - t0))
print("dongle'a SET_REPORT (ayar yazma) sayısı: %d" % sum(sets.values()))
for (rid, ln), n in sorted(sets.items()):
    print("  report 0x%02x, %d bayt: %d kez" % (rid, ln, n))
print("olay/ayar dökümü: %s (%d satır)" % (out, len(lines)))
