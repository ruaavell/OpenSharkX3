#!/usr/bin/env bash
# Attack Shark X3 — resmi yazılımı Wine'da çalıştırıp USB trafiğini otomatik yakalar.
#
#   ./x3-capture.sh                    # installer'ı İndirilenler'de arar
#   ./x3-capture.sh ~/İndirilenler/X3_Setup.exe
#
# Yazılım ZATEN kuruluysa (tekrar çalıştırma) installer gerekmez.
set -uo pipefail
cd "$(dirname "$(readlink -f "$0")")"
say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\n\033[1;31mHATA: %s\033[0m\n' "$*"; exit 1; }

sudo -v || die "sudo gerekli."

# 1) Araçlar
say "Araçlar kuruluyor"
sudo pacman -S --needed --noconfirm wine wine-mono wine-gecko winetricks wireshark-cli zip usbutils psmisc python \
  || echo "UYARI: bazı paketler kurulamadı."
sudo modprobe usbmon || die "usbmon yüklenemedi."

# 2) Dongle
LS=$(lsusb -d 1d57:fa60 | head -1)
[ -n "$LS" ] || die "Dongle (1d57:fa60) takılı değil."
BUS=$(echo "$LS" | awk '{print $2}'); DEV=$(echo "$LS" | awk '{print $4}' | tr -d ':')
USBMON="usbmon$((10#$BUS))"; DEVNO=$((10#$DEV))
echo "Dongle: bus $BUS, cihaz $DEVNO -> $USBMON"

# 3) udev (hidraw erişimi)
RULE=/etc/udev/rules.d/70-attack-shark-x3-hidraw.rules
if [ ! -f "$RULE" ]; then
  say "udev kuralı kuruluyor"
  sudo cp tools/70-attack-shark-x3-hidraw.rules "$RULE"
  sudo udevadm control --reload-rules && sudo udevadm trigger
  echo "Dongle'ı çıkarıp tak, sonra scripti tekrar çalıştır."; exit 0
fi

# 4) Wine prefix + yazılım
export WINEPREFIX="$HOME/.wine-x3" WINEDEBUG=-all
[ -d "$WINEPREFIX" ] || { say "Wine prefix hazırlanıyor (ilk seferde biraz sürer)"; wineboot -u >/dev/null 2>&1; }
EXEFILE="$WINEPREFIX/.x3-exe"
if [ ! -s "$EXEFILE" ]; then
  INST="${1:-}"
  if [ -z "$INST" ]; then
    INST=$(find "$HOME/İndirilenler" "$HOME/Downloads" "$HOME/Masaüstü" "$HOME/Desktop" -maxdepth 2 -type f -iname '*.exe' 2>/dev/null \
           | grep -iE 'x3|shark|attack|driver|setup' | head -1)
  fi
  [ -n "$INST" ] && [ -f "$INST" ] || die "Installer bulunamadı. Resmi sayfadan 'ATTACKSHARK X3 Software'i indir:
  https://attackshark.com/pages/driver-download
  sonra: ./x3-capture.sh ~/İndirilenler/<dosya>.exe"
  say "Yazılım kuruluyor: $INST"
  echo "Kurulum penceresini normal şekilde ilerlet (İleri/Next). Bitince yazılımı KAPAT."
  wine "$INST"
  mapfile -t C < <(find "$WINEPREFIX/drive_c/Program Files" "$WINEPREFIX/drive_c/Program Files (x86)" -maxdepth 3 -type f -iname '*.exe' 2>/dev/null \
                   | grep -viE 'unins|uninstall|update|crash|redist|setup|wine')
  [ "${#C[@]}" -gt 0 ] || die "Kurulu .exe bulunamadı (kurulum tamamlanmadı mı?). Tekrar dene."
  if [ "${#C[@]}" -eq 1 ]; then SEL="${C[0]}"; else
    echo "Hangisi ayar yazılımı?"; select SEL in "${C[@]}"; do [ -n "$SEL" ] && break; done
  fi
  echo "$SEL" > "$EXEFILE"
fi
APP=$(cat "$EXEFILE")

# 5) Yakalama + yazılım
TS=$(date +%Y%m%d-%H%M%S); PCAP="/tmp/x3-capture-$TS.pcapng"
say "Yakalama başlıyor ($USBMON)"
sudo tshark -q -i "$USBMON" -w "$PCAP" >/dev/null 2>&1 &
sleep 3
say "Yazılım açılıyor"
wine "$APP" >/dev/null 2>&1 &
cat <<MSG

Yazılım açıldı. ŞİMDİ ŞUNLARI YAP (her adımdan sonra 'Uygula/Apply' de, 5 sn bekle):
  1) Önce ekranda X3'ün görünüp görünmediğine bak (görünmüyorsa not al)
  2) Bir DPI stage değerini değiştir (örn. 800 -> 1600)
  3) Polling rate'i değiştir
  4) LOD'u değiştir
  5) Bir butonu başka işleve ata
YAPMA: firmware güncelleme, fabrika sıfırlama, eşleştirme düğmeleri.
MSG
sleep 20
say "Teşhis: Wine dongle'ın hidraw cihazlarını açmış mı?"
NODES=()
for d in /sys/class/hidraw/hidraw*; do
  grep -qi "1D57:0000FA60" "$d/device/uevent" 2>/dev/null && NODES+=("/dev/$(basename "$d")")
done
sudo fuser -v "${NODES[@]}" 2>&1 | tee /tmp/x3-fuser-$TS.txt
echo "(Boşsa Wine dongle'ı hiç açmamış demektir.)"
echo
read -rp "Tüm ayarları değiştirip bitirdiysen ENTER'a bas... " _

# 6) Durdur + paketle
say "Yakalama durduruluyor"
sudo pkill -INT -f "tshark -q -i $USBMON -w $PCAP"; sleep 2
sudo chown "$USER" "$PCAP"
python3 tools/x3_capture_check.py "$PCAP" "$DEVNO" "$HOME/x3-events-$TS.txt" || true
cp /tmp/x3-fuser-$TS.txt "$HOME/" 2>/dev/null
ZIP="$HOME/x3-capture-$TS.zip"
zip -q -j "$ZIP" "$PCAP" "$HOME/x3-events-$TS.txt" "$HOME/x3-fuser-$TS.txt"
say "Bitti"
echo "Yüklenecek dosya: $ZIP"
echo "İstersen sadece küçük metin dosyasını da yapıştırabilirsin: $HOME/x3-events-$TS.txt"
