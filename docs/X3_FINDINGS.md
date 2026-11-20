# X3 yakalama bulguları (usbmon, bus 3, dongle 1d57:fa60)

Kaynak: `x3-capture.pcapng` (37,9 sn, 34 260 usbmon kaydı). Dongle cihaz no. 7.

## Yapı (config descriptor)
4 interface: 0 klavye (EP 0x81), 1 mouse (EP 0x82), 2 vendor/medya (EP 0x83, interrupt IN, 16 bayt),
3 klavye (EP 0x84). Interface 2'nin report descriptor'ı 236 bayt (probe çıktısıyla aynı).

## EP 0x83 olayları (5 bayt)
| Paket | Anlamı | Not |
|---|---|---|
| `03 10 40 01 07` | Her ~2,0 sn'de bir tekrarlayan durum/pil paketi | X11'de `03 55 40 01 <%>`; X3'te byte[1]=0x10 ve son bayt 0x07 (pil % mi, kod mu: BİLİNMİYOR) |
| `03 00 10 0N 00` | DPI düğmesi, N = yeni stage (1..6) | X11'de `03 55 10 <N> 00`; X3'te byte[1]=0x00 |

DPI düğmesine art arda basınca N sırası: 2,3,4,5,6,1,2,3,... yani **6 DPI stage** (X11 gibi).

## Sonuç
- Olay paketleri X11'e çok benziyor ama byte[1] farklı (0x55 yerine 0x10 / 0x00).
  X11 sürücüsü `data[1] == 0x55` aradığı için X3'te pil ve DPI olaylarını **yakalayamaz**.
- Yakalamada hiç host→device kontrol transferi (SET_REPORT) yok: yazılım mouse'a bir şey
  göndermedi. X3'ün ayar yazma paketleri HÂLÂ BİLİNMİYOR. X11 paketlerinin X3'te geçerli
  olduğu doğrulanmadı; yazma kapalı (dry-run) kalmalı.

## Yapılacaklar
1. Resmi yazılım çalışırken yeniden yakala (DPI, polling rate, LOD, buton değişiklikleri).
2. `attack_shark.rs` olay ayrıştırıcısını X3 formatına uyarla (byte[1] kontrolünü gevşet).
3. Pil baytının anlamını doğrula (şarj durumu farklı iken iki yakalama karşılaştır).
