#!/usr/bin/env bash
# OpenSharkX3 - Arch/Linux read-only diagnostics + USB capture bundle
# Usage: bash x3-diagnose.sh [path-to-OpenSharkX3]
set -Eeuo pipefail

say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
warn() { printf '\n\033[1;33mUYARI: %s\033[0m\n' "$*"; }

PROJECT="${1:-$PWD}"
if [[ ! -f "$PROJECT/tools/x3_probe.py" ]]; then
  SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  if [[ -f "$SCRIPT_DIR/tools/x3_probe.py" ]]; then PROJECT="$SCRIPT_DIR"; fi
fi
PROJECT="$(cd -- "$PROJECT" 2>/dev/null && pwd || true)"
if [[ -z "$PROJECT" || ! -f "$PROJECT/tools/x3_probe.py" ]]; then
  echo "OpenSharkX3 proje klasörü bulunamadı. Kullanım: bash x3-diagnose.sh ~/İndirilenler/OpenSharkX3" >&2
  exit 1
fi

TS="$(date +%Y%m%d-%H%M%S)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/opensharkx3-diag.XXXXXX")"
OUT="$HOME/OpenSharkX3-diagnostic-$TS.zip"
RAW="$WORK/usbmon-raw.pcap"
FILTERED="$WORK/x3-usb.pcap"
PIDFILE="$WORK/tcpdump.pid"
CAP_PID=""

cleanup() {
  if [[ -n "${CAP_PID:-}" ]]; then
    if [[ "$EUID" -eq 0 ]]; then
      kill -INT "$CAP_PID" 2>/dev/null || true
    elif [[ ${ROOT_OK:-0} -eq 1 && -s "$PIDFILE" ]]; then
      local p
      p="$(cat "$PIDFILE" 2>/dev/null || true)"
      [[ -n "$p" ]] && sudo -n kill -INT "$p" 2>/dev/null || true
    fi
    wait "$CAP_PID" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

mkdir -p "$WORK/project-snapshot"
exec > >(tee "$WORK/run-console.txt") 2>&1

say "OpenSharkX3 tanılama paketi hazırlanıyor"
echo "Proje: $PROJECT"
echo "Zaman: $(date --iso-8601=seconds 2>/dev/null || date)"
echo "Çıktı: $OUT"

ROOT_OK=0
if [[ "$EUID" -eq 0 ]]; then
  ROOT_OK=1
elif sudo -n true >/dev/null 2>&1; then
  ROOT_OK=1
else
  echo "USB yakalama için sudo izni gerekiyor. Bir kez doğrulama denenecek."
  if sudo -v; then ROOT_OK=1; else
    warn "sudo doğrulanamadı. Salt-okunur kontroller yine yapılacak; USB yakalama atlanabilir."
  fi
fi

run_root() {
  if [[ "$EUID" -eq 0 ]]; then "$@"; else sudo -n "$@"; fi
}

say "Sistem bilgileri"
{
  echo "=== OS ==="
  cat /etc/os-release 2>/dev/null || true
  echo "=== Kernel ==="
  uname -a
  echo "=== Tool versions ==="
  for c in python3 cargo rustc tcpdump tshark lsusb udevadm; do
    if command -v "$c" >/dev/null 2>&1; then
      printf '%s: ' "$c"
      case "$c" in
        python3) python3 --version 2>&1 ;;
        cargo) cargo --version 2>&1 ;;
        rustc) rustc --version 2>&1 ;;
        tcpdump) tcpdump --version 2>&1 | head -1 ;;
        tshark) tshark --version 2>&1 | head -1 ;;
        *) command -v "$c" ;;
      esac
    else echo "$c: YOK"; fi
  done
  echo "=== Root diagnostics available ==="
  echo "$ROOT_OK"
} > "$WORK/system-info.txt"
cat "$WORK/system-info.txt"

say "USB cihaz listesi ve ağaç"
if command -v lsusb >/dev/null 2>&1; then
  lsusb > "$WORK/lsusb.txt" 2>&1 || true
  lsusb -t > "$WORK/lsusb-tree.txt" 2>&1 || true
  grep -Ei '1d57:(fa60|fa61|fa55)' "$WORK/lsusb.txt" || echo "X3 VID/PID lsusb listesinde bulunamadı."
else
  echo "usbutils / lsusb kurulu değil." > "$WORK/lsusb.txt"
fi

say "HID descriptor keşfi (salt-okunur)"
if command -v python3 >/dev/null 2>&1; then
  (cd "$PROJECT" && python3 tools/x3_probe.py --vid 1d57 --pid fa60) > "$WORK/probe-fa60.txt" 2>&1 || true
  (cd "$PROJECT" && python3 tools/x3_probe.py --vid 1d57 --pid fa61) > "$WORK/probe-fa61.txt" 2>&1 || true
  (cd "$PROJECT" && python3 tools/x3_probe.py --vid 1d57 --pid fa55) > "$WORK/probe-fa55.txt" 2>&1 || true
  cat "$WORK/probe-fa60.txt"
  cat "$WORK/probe-fa61.txt"
  cat "$WORK/probe-fa55.txt"
fi

say "HIDRAW ve udev bilgileri (seri numarası toplanmaz)"
{
  for h in /sys/class/hidraw/hidraw*; do
    [[ -e "$h" ]] || continue
    ue="$(cat "$h/device/uevent" 2>/dev/null || true)"
    if grep -Eqi '1D57.*(FA60|FA61|FA55)' <<< "$ue"; then
      echo "--- /dev/$(basename "$h") ---"
      grep -E '^(HID_ID|HID_NAME|HID_PHYS|HID_UNIQ)=' <<< "$ue" | sed -E 's/^HID_UNIQ=.*/HID_UNIQ=[REDACTED]/'
      printf 'interface='
      cat "$h/device/../bInterfaceNumber" 2>/dev/null || echo '?'
      printf 'driver='
      readlink -f "$h/device/driver" 2>/dev/null || echo '(none)'
      printf 'udev-path='
      udevadm info --query=path --name="/dev/$(basename "$h")" 2>/dev/null || true
    fi
  done
} > "$WORK/hidraw-udev.txt"
cat "$WORK/hidraw-udev.txt"

say "Yerel proje durumu"
{
  echo "project=$PROJECT"
  if git -C "$PROJECT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "=== git status ==="
    git -C "$PROJECT" status --short --branch 2>&1 || true
    echo "=== commit ==="
    git -C "$PROJECT" rev-parse --short HEAD 2>&1 || true
  else echo "Git deposu değil veya .git klasörü yok."; fi
  echo "=== file sizes ==="
  du -sh "$PROJECT" 2>/dev/null || true
} > "$WORK/project-status.txt"
cat "$WORK/project-status.txt"

# Eski x3_read_feature.py bilerek çalıştırılmıyor: X3 için GET_FEATURE readback
# upstream asctl araştırmasında desteklenmiyor / write-only olarak belgelenmiş.
cat > "$WORK/readback-note.txt" <<'EOF'
GET_FEATURE testi bu tanılama scriptinde çalıştırılmadı.
Upstream yourchocomate/attacksharkx3 araştırması X3 ayar protokolünü write-only
olarak belgeliyor; mevcut ayarları GET_FEATURE ile geri okuma desteklenmiyor.
Bu yüzden x3_read_feature.py timeout sonucu tek başına donanım arızası değildir.
Bir sonraki doğrulama üretici yazılımının gönderdiği SET_REPORT paketini yakalamaktır.
Kaynak: https://github.com/yourchocomate/attacksharkx3
EOF

say "Rust cargo check (offline, en fazla 90 saniye)"
if command -v cargo >/dev/null 2>&1 && [[ -f "$PROJECT/src-tauri/Cargo.toml" ]]; then
  if command -v timeout >/dev/null 2>&1; then
    (cd "$PROJECT" && timeout 90s cargo check --offline --manifest-path src-tauri/Cargo.toml) > "$WORK/cargo-check.txt" 2>&1 || echo "cargo check exit=$? (ayrıntı dosyada)" >> "$WORK/cargo-check.txt"
  else
    (cd "$PROJECT" && cargo check --offline --manifest-path src-tauri/Cargo.toml) > "$WORK/cargo-check.txt" 2>&1 || echo "cargo check başarısız (ayrıntı dosyada)" >> "$WORK/cargo-check.txt"
  fi
  tail -80 "$WORK/cargo-check.txt"
else
  echo "Cargo veya src-tauri/Cargo.toml bulunamadı; derleme kontrolü atlandı." > "$WORK/cargo-check.txt"
fi

say "İlgili proje kaynaklarının anlık kopyası"
for f in \
  docs/X3_FINDINGS.md docs/X3_BRINGUP.md \
  tools/x3_probe.py tools/x3_capture_check.py \
  src-tauri/src/driver/attack_shark.rs \
  src-tauri/src/driver/attack_shark_ble.rs \
  src-tauri/src/protocols/dpi.rs \
  src-tauri/src/protocols/polling_rate.rs \
  src-tauri/src/safety.rs; do
  if [[ -f "$PROJECT/$f" ]]; then
    mkdir -p "$WORK/project-snapshot/$(dirname "$f")"
    cp "$PROJECT/$f" "$WORK/project-snapshot/$f"
  fi
done

# Identify the receiver and capture its USB bus. Supports the currently observed FA60/FA61 IDs.
BUS=""; DEV=""; FOUND_LINE=""
if command -v lsusb >/dev/null 2>&1; then
  FOUND_LINE="$(lsusb -d 1d57:fa60 2>/dev/null | head -1 || true)"
  [[ -n "$FOUND_LINE" ]] || FOUND_LINE="$(lsusb -d 1d57:fa61 2>/dev/null | head -1 || true)"
  if [[ -n "$FOUND_LINE" ]]; then
    BUS="$(awk '{print $2}' <<< "$FOUND_LINE")"
    DEV="$(awk '{print $4}' <<< "$FOUND_LINE" | tr -d ':')"
  fi
fi

say "USB trafği yakalama"
if [[ -z "$BUS" || -z "$DEV" ]]; then
  warn "1d57:fa60 veya 1d57:fa61 dongle bulunamadı; USB yakalama atlandı."
elif ! command -v tcpdump >/dev/null 2>&1; then
  warn "tcpdump kurulu değil; USB yakalama atlandı. Paket kurmadım."
elif [[ "$ROOT_OK" -ne 1 ]]; then
  warn "sudo doğrulanmadığı için USB yakalama atlandı. Diğer teşhisler ZIP'e eklenecek."
else
  BUSNUM=$((10#$BUS))
  DEVNUM=$((10#$DEV))
  USBMON="usbmon$BUSNUM"
  echo "Dongle: $FOUND_LINE"
  echo "Arayüz: $USBMON, USB device address: $DEVNUM"
  run_root modprobe usbmon || warn "usbmon modülü yüklenemedi."
  if [[ "$EUID" -eq 0 ]]; then
    tcpdump -i "$USBMON" -s 0 -U -w "$RAW" >/dev/null 2> "$WORK/tcpdump-stderr.txt" &
    CAP_PID=$!
  else
    sudo -n sh -c 'echo $$ > "$1"; exec tcpdump -i "$2" -s 0 -U -w "$3"' sh "$PIDFILE" "$USBMON" "$RAW" >/dev/null 2> "$WORK/tcpdump-stderr.txt" &
    CAP_PID=$!
  fi
  sleep 2
  if [[ "$EUID" -eq 0 ]]; then
    if ! kill -0 "$CAP_PID" 2>/dev/null; then CAP_PID=""; fi
  else
    if [[ ! -s "$PIDFILE" ]]; then CAP_PID=""; fi
  fi

  if [[ -z "$CAP_PID" ]]; then
    warn "tcpdump başlatılamadı. Ayrıntı tcpdump-stderr.txt dosyasına kaydedildi."
  else
    cat <<EOF

USB yakalama başladı. Bu yakalama yalnızca dongle'ın bulunduğu USB bus'ını dinler;
paketler ZIP'e eklenmeden önce 1d57:$([[ "$FOUND_LINE" == *fa61* ]] && echo fa61 || echo fa60) cihaz adresine göre filtrelenecek.

ŞİMDİ:
  1. Resmî Attack Shark X3 ayar yazılımını (varsa Wine üzerinden) aç.
  2. Yalnızca TEK bir ayarı değiştir (ör. polling rate) ve Apply/Uygula'ya bas.
  3. Yazılım sende yoksa DPI tuşuna 5-6 kez basıp fareyi hareket ettir.
  4. İşin bitince bu terminale dönüp ENTER'a bas.

Firmware güncellemesi, fabrika sıfırlama veya eşleştirme işlemi yapma.
EOF
    read -r -p "Yakalamayı bitirmek için ENTER: " _ || true
    if [[ "$EUID" -eq 0 ]]; then
      kill -INT "$CAP_PID" 2>/dev/null || true
    else
      P="$(cat "$PIDFILE" 2>/dev/null || true)"
      [[ -n "$P" ]] && sudo -n kill -INT "$P" 2>/dev/null || true
    fi
    wait "$CAP_PID" 2>/dev/null || true
    CAP_PID=""
    if [[ -s "$RAW" && "$EUID" -ne 0 ]]; then
      run_root chown "$(id -u):$(id -g)" "$RAW" 2>/dev/null || true
      chmod 600 "$RAW" 2>/dev/null || true
    fi

    if [[ -s "$RAW" ]]; then
      python3 - "$RAW" "$FILTERED" "$BUSNUM" "$DEVNUM" "$WORK/usb-analysis.txt" <<'PY'
import collections, os, struct, sys
raw, out, wanted_bus, wanted_dev, report = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
lines=[]
with open(raw,'rb') as f:
    gh=f.read(24)
    if len(gh)!=24: raise SystemExit('PCAP global header eksik')
    magic=gh[:4]
    endian='<' if magic in (b'\xd4\xc3\xb2\xa1',b'\x4d\x3c\xb2\xa1') else '>'
    network=struct.unpack_from(endian+'I',gh,20)[0]
    if network != 220:
        raise SystemExit(f'Desteklenmeyen PCAP linktype={network}; USB_LINUX_MMAPPED (220) bekleniyordu.')
    total=kept=0
    xfers=collections.Counter(); endpoints=collections.Counter(); ctrl=collections.Counter(); events=collections.Counter()
    event_samples=[]; set_examples=[]; start=None; last=None
    with open(out,'wb') as g:
        g.write(gh)
        while True:
            rh=f.read(16)
            if not rh: break
            if len(rh)<16: break
            sec,usec,caplen,origlen=struct.unpack(endian+'IIII',rh)
            pkt=f.read(caplen)
            if len(pkt)<caplen: break
            total+=1
            if len(pkt)<64: continue
            bus=struct.unpack_from('<H',pkt,12)[0]
            dev=pkt[11]
            if bus != wanted_bus or dev != wanted_dev: continue
            kept+=1; g.write(rh); g.write(pkt)
            typ=chr(pkt[8]); xfer=pkt[9]; ep=pkt[10]
            status=struct.unpack_from('<i',pkt,28)[0]
            dlen=struct.unpack_from('<I',pkt,36)[0]
            data=pkt[64:64+dlen]
            xfers[(typ,xfer)] += 1
            endpoints[(typ,xfer,ep)] += 1
            t=(sec,usec)
            if start is None: start=t
            last=t
            if xfer==2 and typ=='S':
                setup=pkt[40:48]
                bm,req,wval,widx,wlen=struct.unpack('<BBHHH',setup)
                key=(bm,req,(wval>>8)&0xff,wval&0xff,widx,wlen)
                ctrl[key]+=1
                if bm==0x21 and req==0x09:
                    set_examples.append((sec,usec,wval>>8,wval&0xff,widx,wlen,data[:64].hex(' ')))
            if xfer==1 and ep==0x83 and typ=='C' and data:
                hx=data.hex(' ')
                events[hx]+=1
                if len(event_samples)<20 and hx not in event_samples:
                    event_samples.append(hx)
    def elapsed(a,b): return (b[0]-a[0])+(b[1]-a[1])/1e6
    lines.append(f'USB bus/device: {wanted_bus}/{wanted_dev}')
    lines.append(f'PCAP linktype: {network} (USB_LINUX_MMAPPED)')
    lines.append(f'Tüm kayıtlardan dongle filtresinden geçen: {kept} / {total}')
    lines.append(f'Yakalama süresi: {elapsed(start,last):.3f} s' if start and last else 'Yakalama süresi: bilinmiyor')
    lines.append('Transfer türleri (S=submit, C=complete):')
    for k,v in sorted(xfers.items()): lines.append(f'  {k}: {v}')
    lines.append('Endpoint kayıtları:')
    for k,v in sorted(endpoints.items()): lines.append(f'  {k}: {v}')
    lines.append('Control setup istekleri (bmRequestType, bRequest, reportType, reportID, interface, length):')
    for k,v in sorted(ctrl.items()): lines.append(f'  {k}: {v}')
    lines.append(f'HID SET_REPORT (0x21/0x09) adet: {sum(v for k,v in ctrl.items() if k[0]==0x21 and k[1]==0x09)}')
    lines.append(f'HID GET_REPORT Feature (0xA1/0x01/type 3) adet: {sum(v for k,v in ctrl.items() if k[0]==0xA1 and k[1]==0x01 and k[2]==3)}')
    for row in set_examples[:25]: lines.append('  SET_REPORT örneği: '+repr(row))
    lines.append('EP 0x83 input event paketleri:')
    for hx,n in events.most_common(30): lines.append(f'  {n} x  {hx}')
    with open(report,'w') as r: r.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
print(f'Filtered PCAP: {out} ({os.path.getsize(out)} bytes)')
PY
      rm -f "$RAW"
    else
      warn "PCAP dosyası oluşmadı veya boş."
    fi
  fi
fi

# Avoid adding an empty or unusable capture file.
[[ -s "$FILTERED" ]] || rm -f "$FILTERED"

cat > "$WORK/README-OUTPUT.txt" <<EOF
OpenSharkX3 diagnostics
Generated: $TS
Project: $PROJECT

Included files:
- system-info.txt: OS/kernel/tool availability
- lsusb*.txt: USB topology
- probe-fa60/61/55.txt: HID report descriptors discovered by the project's read-only probe
- hidraw-udev.txt: matching hidraw/interface info; HID_UNIQ is redacted
- project-status.txt: local git state
- cargo-check.txt: offline Rust compile check result
- usb-analysis.txt + x3-usb.pcap: filtered USB traffic for the X3 receiver, when capture succeeded
- readback-note.txt: why GET_FEATURE is not tested
- project-snapshot/: relevant local source/docs snapshot

No settings are written to the mouse. The old GET_FEATURE read test is deliberately not run.
Raw bus capture is removed after filtering to the X3 USB device.
EOF

say "ZIP oluşturuluyor"
python3 - "$WORK" "$OUT" <<'PY'
import os, sys, zipfile
root, dest=sys.argv[1:]
with zipfile.ZipFile(dest,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
    for base,dirs,files in os.walk(root):
        dirs.sort()
        for name in sorted(files):
            path=os.path.join(base,name)
            arc=os.path.relpath(path,root)
            z.write(path,arc)
print(dest)
print(f'{os.path.getsize(dest)} bytes')
PY

say "Tamamlandı"
echo "Bana göndereceğin dosya: $OUT"
echo "Bu script fare ayarlarına yazma yapmadı."
