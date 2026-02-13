# GeoPic: Fotoğraf Konum Bulucu 📍

GeoPic, fotoğraflardan yüksek hassasiyetli GPS koordinatlarını çıkaran ve bu koordinatları OpenStreetMap kullanarak gerçek adreslere dönüştüren güçlü bir Rust tabanlı komut satırı aracıdır.

## 🚀 Özellikler
- **🎨 Renkli Arayüz**: Linux terminalleri için optimize edilmiş, kolay okunabilir çıktı.
- **📱 HEIC Desteği**: iPhone (HEIC/HEIF) ve modern tüm fotoğraf formatlarını destekler.
- **🌐 Kesin Adres**: Koordinatları Nominatim API üzerinden sokak detaylarına kadar çevirir.
- **💾 Teknik Detaylar**: Kamera markası, modeli, lens bilgisi ve pozlama değerlerini gösterir.
- **⚖️ Gizlilik Odaklı**: Etik kullanım uyarısı ve veri güvenliği önlemleri içerir.

## 🛠️ Linux Hızlı Kurulum

Linux üzerinde en hızlı başlangıç için sağlanan kurulum betiğini kullanabilirsiniz:
```bash
chmod +x setup.sh
./setup.sh
```
Bu betik sisteminizi tanır, gerekli bağımlılıkları yükler ve projeyi derler.

## 📦 Manuel Kurulum

### 1. Gereksinimler
Rust çalışma ortamına ihtiyacınız vardır:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Bağımlılıkları Yükleme
**Ubuntu/Debian:** `sudo apt install pkg-config libssl-dev build-essential ca-certificates`  
**Fedora:** `sudo dnf install pkg-config openssl-devel`  
**Arch Linux:** `sudo pacman -S pkgconf openssl base-devel`

### 3. Derleme ve Yükleme
`Makefile` kullanarak standart kurulum yapabilirsiniz:
```bash
make build
# İsteğe bağlı: Sistemi genelinde kullanmak için (/usr/local/bin)
sudo make install
```

## 📖 Kullanım
Eğer `make install` yaptıysanız:
```bash
geopic path/to/photo.jpg
```
Aksi halde release binary dosyasını çalıştırın:
```bash
./target/release/rust-map path/to/photo.jpg
```

**İngilizce çıktı için:**
```bash
geopic path/to/photo.jpg --en
```

## ⚙️ Nasıl Çalışır?
1. **Çıkarma**: `nom-exif` kütüphanesi ile fotoğrafın EXIF başlıkları taranır.
2. **Dönüştürme**: Karmaşık rasyonel GPS verileri (Derece, Dakika, Saniye) yüksek hassasiyetli ondalık koordinatlara çevrilir.
3. **Sorgulama**: Elde edilen koordinatlar OpenStreetMap Nominatim sunucularına gönderilerek en yakın kayıtlı adres bilgisi çekilir.
4. **Teknik Veri**: Kameranın donanım bilgileri (Lens, ISO, Diyafram) ayıklanarak raporlanır.

## ⚖️ Gizlilik ve Etik
Bu araç eğitim ve kişisel kullanım için tasarlanmıştır.
1. **Sahiplik**: Sadece sahibi olduğunuz veya analiz etmek için izniniz olan fotoğrafları işleyin.
2. **Yasal Uyum**: Konum verileriyle ilgili yerel yasalarına (KVKK vb.) uyun.
3. **Kötüye Kullanım**: GeoPic'i takip, taciz veya ifşa amaçlı kullanmayın.

## 📁 Desteklenen Formatlar
- **JPEG / JPG**: Standart kamera fotoğrafları.
- **HEIC / HEIF**: Modern iPhone ve Android yüksek verimlilik formatları.
- **TIFF / PNG / AVIF**: Meta veri içeren profesyonel formatlar.

> [!WARNING]
> **Fotoğrafım Neden Çalışmıyor?**
> Sosyal medya platformları (Instagram, WhatsApp, X) kullanıcı gizliliğini korumak için GPS verilerini fotoğraftan otomatik olarak siler. Eğer fotoğrafınız bu uygulamalar üzerinden geldiyse konum bilgisi içermeyecektir. Telefonunuzdan direkt aktardığınız orijinal fotoğrafları deneyin.
