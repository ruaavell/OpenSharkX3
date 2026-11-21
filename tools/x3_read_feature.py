#!/usr/bin/env python3
"""
x3_read_feature.py — X3 dongle'ından MEVCUT ayarları okur (GET_FEATURE).

Mouse'a ayar YAZMAZ. Sadece şu Feature report'ları okur (sabit izin listesi):
    0x04 (52 B)  0x05 (13 B)  0x06 (9 B)  0x08 (59 B)
0x0b ve büyük report'lara (0x22, 0x23, 0x28, 0x29, ...) DOKUNMAZ.

Kullanım:
    python3 tools/x3_read_feature.py            # interface 2'yi otomatik bulur
    python3 tools/x3_read_feature.py /dev/hidraw5
Önce mouse'ta bir ayar değiştir (örn. DPI düğmesiyle stage'i değiştir), sonra
tekrar çalıştır ve iki çıktıyı karşılaştır; hangi bayt değişti, onu buluruz.
"""
import fcntl, glob, os, re, sys

ALLOWED = {0x04: 52, 0x05: 13, 0x06: 9, 0x08: 59}

def hidiocgfeature(n):
    return (3 << 30) | (n << 16) | (ord("H") << 8) | 0x07

def find_iface2(vid=0x1d57, pid=0xfa60):
    for dev in sorted(glob.glob("/sys/class/hidraw/hidraw*")):
        real = os.path.realpath(os.path.join(dev, "device"))
        try:
            ue = open(os.path.join(real, "uevent")).read()
            iface = open(os.path.join(real, "..", "bInterfaceNumber")).read().strip()
        except OSError:
            continue
        m = re.search(r"HID_ID=\w+:(\w+):(\w+)", ue)
        if m and int(m.group(1), 16) == vid and int(m.group(2), 16) == pid and iface == "02":
            return "/dev/" + os.path.basename(dev)
    return None

def main():
    path = sys.argv[1] if len(sys.argv) > 1 else find_iface2()
    if not path:
        sys.exit("1d57:fa60 interface 2 bulunamadı (dongle takılı mı?)")
    print(f"# cihaz: {path}")
    try:
        fd = os.open(path, os.O_RDWR)
    except PermissionError:
        sys.exit("İzin yok. udev kuralını kurup mouse'u çıkar-tak (./start.sh bunu yapar).")
    for rid, size in ALLOWED.items():
        buf = bytearray(size)
        buf[0] = rid
        try:
            n = fcntl.ioctl(fd, hidiocgfeature(size), buf, True)
            data = bytes(buf[:n if isinstance(n, int) and n > 0 else size])
            print(f"report 0x{rid:02x} ({len(data)} bayt):")
            for i in range(0, len(data), 16):
                print(f"  {i:02d}: " + " ".join(f"{b:02x}" for b in data[i:i+16]))
        except OSError as e:
            print(f"report 0x{rid:02x}: okunamadı ({e})")
    os.close(fd)

if __name__ == "__main__":
    main()
