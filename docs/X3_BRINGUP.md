# Attack Shark X3 — Kurulum ve Doğrulama Rehberi

Bu proje, MIT lisanslı **OpenSharkX11** (Clevs) projesinin X3 için hazırlanmış fork'udur.
Arayüz: Tauri 2 + Rust + React. Orijinal kod X11 içindir.

## Hızlı başlangıç

```bash
chmod +x start.sh
./start.sh               # kurar, probe alır (probe-output.txt), uygulamayı read-only açar
./start.sh --probe-only  # sadece probe
./start.sh --write       # yazma açık (sadece doğrulamadan sonra)
```

## Otomatik yakalama (resmi yazılım + Wine + usbmon)

```bash
./x3-capture.sh ~/İndirilenler/<X3-installer>.exe   # ilk seferde kurar, sonra installer gerekmez
```
Script: araçları kurar, yazılımı Wine'a kurar, usbmon yakalamasını başlatır, yazılımı açar,
Enter'a basınca durdurup `~/x3-capture-<zaman>.zip` üretir (içinde küçük bir olay dökümü de var).

## ÖNEMLİ: Durum

- X3 dongle'ı `1d57:fa60` olarak görünüyor; X11 dongle'ı ile aynı kimlik. Bu, protokolün
  aynı olduğunu **kanıtlamaz**.
- Bu yüzden uygulama varsayılan olarak **READ-ONLY** modunda: mouse'a hiçbir şey yazmaz,
  göndereceği paketleri log'a basar. (Orijinal uygulama bağlanır bağlanmaz DPI, ışık ve
  buton ayarlarını yazıyordu; X3'te bu ayarları bozabilirdi.)
- Güncel durum: `cargo check`, `cargo test` (36 test), `tsc --noEmit` ve `vite build` geçiyor.
  Ayrıntılar: `docs/PROJECT_AUDIT.md`, `docs/HARDWARE_VALIDATION.md`, `docs/INSTALLATION_ARCH.md`.
- Yazma modu `OPENSHARKX3_WRITE_MODE` ile seçilir. Varsayılan **read-only**; bağlanmak
  hiçbir ayar yazmaz.

## Adım 1 — Salt-okunur probe (önce bunu yap)

```bash
sudo cp tools/70-attack-shark-x3-hidraw.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
# mouse/dongle'ı çıkarıp tak

python3 tools/x3_probe.py --pid fa60          # dongle
python3 tools/x3_probe.py --pid fa55          # kablolu (USB-C ile bağlıyken)
sudo usbhid-dump -d 1d57:fa60                 # ham descriptor
```

Çıktının tamamını paylaş. Bakacağımız şey: X11'in kullandığı **interface 2** ve
Feature report'lar `0x04 (56 bayt) / 0x05 (15) / 0x06 (9) / 0x08 (59)` X3'te de var mı,
boyutları aynı mı.

### Adım 1b — Mevcut ayarları oku (yine yazmaz)

```bash
python3 tools/x3_read_feature.py | tee read-before.txt
# mouse'ta DPI düğmesiyle stage değiştir, sonra:
python3 tools/x3_read_feature.py | tee read-after.txt
diff read-before.txt read-after.txt
```

## Adım 2 — Derle ve dry-run ile çalıştır

```bash
sudo pacman -S --needed webkit2gtk-4.1 libusb gtk3 libayatana-appindicator \
                        nodejs npm rust git pkgconf
npm install
npm run dev          # geliştirme modu
```

Uygulama açılınca konsolda `[DRY-RUN] yazma engellendi ...` satırlarını görürsün.
Bu satırlar X11 paketlerinin X3'e gönderilecek hali; probe çıktısıyla karşılaştırırız.
Pil okuma ve DPI düğmesi olayları dry-run'da da çalışır (sadece dinleme).

> Not: Uygulama **hidraw** kullanır; USB interface'ini talep etmez ve çekirdek
> sürücüsünü ayırmaz. Bu nedenle yazma sırasında mouse çalışmaya devam eder.

## Adım 3 — Yazmayı açma (SADECE doğrulamadan sonra)

```bash
OPENSHARKX3_WRITE_MODE=hardware npm run dev
# DPI / ışık / buton raporları ayrıca OPENSHARKX3_EXPERIMENTAL=1 ister
```

Kurallar (X11 belgesinden, `docs/protocol/PROTOCOL_EN.md` §8):
1. İlk yazma denemelerini **kablolu (USB-C)** modda yap; dongle eşleşmesi kaybolmasın.
2. Sadece Report ID `0x04`, `0x05`, `0x06` (ve doğrulandıysa `0x08`). **`0x0b` kullanma**:
   X11'de dongle eşleşmesini sildi.
3. Bir seferde tek ayar değiştir.
4. Eşleşme kaybolursa: mouse'u kapat/aç, sonra dongle LED'i yanıp sönmeyi bırakana
   kadar DPI düğmesini basılı tut (X11 için geçerli; X3 kılavuzunu da kontrol et).

## Adım 4 — X3'e özel uyarlama (probe sonrası)

X3 ile X11 arasında farklar olabilir (buton sayısı, RGB var mı, DPI aralığı, polling
rate seçenekleri). Fark çıkarsa `src-tauri/src/protocols/` ve `src/data.jsx` içinde
bir X3 model profili ekleyip desteklenmeyen bölümleri arayüzden gizleriz.
Tutarsızlık büyükse Windows yazılımıyla Wireshark (USBPcap) yakalamasına geçeriz.

## Değişen dosyalar (orijinale göre)

- `src-tauri/src/safety.rs` (yeni): `OPENSHARKX3_WRITE_MODE` politikası
- `src-tauri/src/lib.rs`: `mod safety;`
- `src-tauri/src/driver/attack_shark.rs`, `attack_shark_ble.rs`: yazma öncesi dry-run kontrolü
- `tools/x3_probe.py`, `tools/70-attack-shark-x3-hidraw.rules` (yeni)
- `docs/X3_BRINGUP.md` (bu dosya)
