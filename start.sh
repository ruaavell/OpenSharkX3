#!/usr/bin/env bash
# Attack Shark X3 — tek komutla kurulum + probe + uygulamayı aç.
#
#   ./start.sh               # kur, probe çalıştır, uygulamayı READ-ONLY aç (hiç yazmaz)
#   ./start.sh --probe-only  # sadece probe çıktısını üret, uygulamayı açma
#   ./start.sh --dryrun      # yazma raporlarını üret + logla, göndermeden aç
#   ./start.sh --write       # HARDWARE: doğrulanmış raporları gönder (riskli!)
set -uo pipefail
cd "$(dirname "$(readlink -f "$0")")"

PROBE_ONLY=0; WRITE=0; DRYRUN=0
for a in "$@"; do
  case "$a" in
    --probe-only) PROBE_ONLY=1 ;;
    --dryrun) DRYRUN=1 ;;
    --write) WRITE=1 ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
    *) echo "Bilinmeyen seçenek: $a"; exit 1 ;;
  esac
done

say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }

# 1) Bağımlılıklar (Arch)
if command -v pacman >/dev/null 2>&1; then
  say "Bağımlılıklar kontrol ediliyor (pacman)"
  sudo pacman -S --needed --noconfirm webkit2gtk-4.1 libusb gtk3 libayatana-appindicator \
       nodejs npm rust git pkgconf usbutils || echo "UYARI: bazı paketler kurulamadı."
else
  echo "pacman yok: webkit2gtk-4.1, libusb, gtk3, libayatana-appindicator, nodejs, npm, rust'ı elle kur."
fi
for c in python3 node npm cargo; do
  command -v "$c" >/dev/null 2>&1 || { echo "HATA: '$c' bulunamadı."; exit 1; }
done

# 2) udev kuralı (hidraw + usb erişimi)
RULE_DST=/etc/udev/rules.d/70-attack-shark-x3-hidraw.rules
if [ ! -f "$RULE_DST" ]; then
  say "udev kuralı kuruluyor"
  sudo rm -f /etc/udev/rules.d/99-attack-shark-x3-hidraw.rules
  sudo cp tools/70-attack-shark-x3-hidraw.rules "$RULE_DST"
  sudo cp aur/99-attack-shark-x11.rules /etc/udev/rules.d/99-attack-shark-x11.rules
  sudo udevadm control --reload-rules && sudo udevadm trigger
  echo "Kural kuruldu. Mouse/dongle'ı bir kez çıkarıp tak, sonra Enter'a bas."
  read -r _
fi

# 3) Probe (salt-okunur) — çıktı probe-output.txt dosyasına da yazılır
say "Probe çalışıyor (mouse'a yazmaz)"
{
  echo "# $(date -Is)"; echo "## lsusb"; lsusb | grep -i "1d57" || echo "1d57 cihazı yok"
  for pid in fa60 fa55; do echo; echo "## x3_probe.py --pid $pid"; python3 tools/x3_probe.py --pid "$pid"; done
  if command -v usbhid-dump >/dev/null 2>&1; then
    echo; echo "## usbhid-dump"; sudo usbhid-dump -d 1d57:fa60 2>&1 || true
  fi
} | tee probe-output.txt
echo; echo "Probe çıktısı kaydedildi: $(pwd)/probe-output.txt  (bunu paylaş)"

[ "$PROBE_ONLY" = 1 ] && exit 0

# 4) JS bağımlılıkları
if [ ! -d node_modules ]; then
  say "npm install"
  npm install || { echo "HATA: npm install başarısız."; exit 1; }
fi

# 5) Uygulamayı aç
if [ "$WRITE" = 1 ]; then
  echo
  echo "!! HARDWARE WRITE: doğrulanmış raporlar mouse'a gönderilecek."
  echo "!! Bağlantı kurmak tek başına asla yazmaz; her yazma arayüzde açık bir tıklama ister."
  echo "!! DPI/buton/ışık raporları bu cihazda fiziksel olarak doğrulanmadı."
  echo "!! Devam için 'evet' yaz:"
  read -r ans; [ "$ans" = "evet" ] || { echo "İptal."; exit 1; }
  export OPENSHARKX3_WRITE_MODE=hardware
  say "Uygulama açılıyor (HARDWARE WRITE)"
elif [ "$DRYRUN" = 1 ]; then
  export OPENSHARKX3_WRITE_MODE=dryrun
  say "Uygulama açılıyor (DRY-RUN: paket üretir, göndermez)"
else
  say "Uygulama açılıyor (READ-ONLY: hiçbir yazma gönderilmez)"
fi
exec npm run dev
