# MySQLDesk

<div align="center">
  <img src="data/icons/org.mysqldesk.MySQLDesk.svg" width="96" height="96" alt="MySQLDesk Logo"/>
  <h3>Modern Native Linux Control Panel for MySQL, MariaDB & phpMyAdmin</h3>
  <p>Pengontrol dan pengelola server database native Linux yang ringan, terisolasi, aman, dan berestetika GNOME modern (Rust + GTK4 + Libadwaita).</p>
</div>

---

## 📋 Daftar Isi
- [Latar Belakang & Konsep](#latar-belakang--konsep)
- [Fitur Utama](#fitur-utama)
- [Arsitektur Aplikasi](#arsitektur-aplikasi)
- [Prinsip Keamanan (Security Considerations)](#prinsip-keamanan-security-considerations)
- [Persyaratan Sistem & Dependensi](#persyaratan-sistem--dependensi)
- [Panduan Pengembangan (Development Setup)](#panduan-pengembangan-development-setup)
- [Kompilasi & Eksekusi](#kompilasi--eksekusi)
- [Pengujian (Testing)](#pengujian-testing)
- [Pembuatan Paket Distribusi (Packaging)](#pembuatan-paket-distribusi-packaging)
- [Lisensi](#lisensi)

---

## 🚀 Latar Belakang & Konsep

Banyak developer Linux yang menyukai kesederhanaan **XAMPP Control Panel** di Windows ketika mengelola database lokal dan phpMyAdmin untuk kebutuhan prototyping/pengembangan web. Namun di Linux:
- XAMPP bawaan sering kali merusak library sistem asli atau menggunakan bundel binary kuno.
- Mengontrol service native Linux via terminal (`systemctl`, `journalctl`, `mysql`) memerlukan banyak langkah manual.
- Aplikasi web wrapper berbasis Electron/Tauri memakan ratusan megabyte memori RAM hanya untuk menampilkan panel sederhana.

**MySQLDesk** diciptakan untuk menyelesaikan masalah tersebut: aplikasi native desktop Linux murni yang berinteraksi langsung dengan service MariaDB/MySQL bawaan sistem operasi Linux Anda, terintegrasi dengan runtime PHP lokal untuk menjalankan phpMyAdmin secara instan tanpa perlu web server berat (Apache/Nginx) yang berjalan di latar belakang.

---

## ✨ Fitur Utama

1. **Autodeteksi Cerdas Multi-Distro:**
   - Mendeteksi otomatis MariaDB atau MySQL di Ubuntu, Debian, Fedora, Arch Linux, dan openSUSE.
   - Menemukan lokasi binary, nomor versi, unit service systemd, lokasi soket UNIX, data directory, dan berkas konfigurasi (`my.cnf` / `mariadb.cnf`).
2. **Kontrol Service Tanpa Freeze (Non-blocking):**
   - Mengontrol `Start`, `Stop`, dan `Restart` database service secara asynchronous.
   - Status visual real-time: `● Running`, `● Stopped`, `● Starting`, `● Stopping`, `● Error`.
3. **Pemberitahuan Konflik Port:**
   - Mendeteksi ketersediaan port 3306.
   - Memberikan pesan informatif jika port database sedang digunakan oleh proses lain.
4. **Integrasi phpMyAdmin Terisolasi:**
   - Deteksi PHP CLI dan modul-modul penting (`mysqli`, `mbstring`, `json`, `session`).
   - Fitur **Setup / Unduh phpMyAdmin Resmi** dengan validasi checksum hash SHA256 integritas file secara otomatis.
   - Menjalankan server lokal via PHP Built-in Server yang terikat (*strictly bound*) hanya ke `127.0.0.1`.
   - Tombol satu klik untuk membuka browser default Linux.
5. **Penampil Log Database Real-time (Logs Viewer):**
   - Menampilkan riwayat log dari `journalctl` secara non-blocking.
   - Fitur filter/pencarian kata kunci instan, salin log ke clipboard, pembersihan tampilan, auto-scroll, dan auto-refresh.
6. **Desain Linux Modern & Dark Mode:**
   - Dibangun dengan **GTK4** dan **Libadwaita**.
   - Mendukung tema Light Mode, Dark Mode, dan Mengikuti Pengaturan Sistem GNOME.

---

## 🏗️ Arsitektur Aplikasi

Aplikasi dibangun dengan arsitektur modular dan pemisahan tanggung jawab (*separation of concerns*):

```
src/
├── main.rs                 # Inisialisasi AdwApplication, styling CSS, dan aktivasi window
├── app.rs                  # State holder global (thread-safe Arc<Mutex>)
├── error.rs                # Model error terpadu AppError & AppResult yang ramah pengguna
├── database/               # Domain Database
│   ├── mod.rs
│   ├── detector.rs         # Scanner instalasi MariaDB / MySQL & parser variabel
│   ├── mysql.rs            # Spesifikasi & konfigurasi MySQL
│   ├── mariadb.rs          # Spesifikasi & konfigurasi MariaDB
│   ├── service.rs          # Abstraksi kontrol service database
│   └── config.rs           # Parser & pemanggil editor file konfigurasi
├── phpmyadmin/             # Domain phpMyAdmin
│   ├── mod.rs
│   ├── detector.rs         # Scanner ketersediaan PHP runtime & phpMyAdmin
│   ├── manager.rs          # Downloader resmi terverifikasi SHA256 & generator konfigurasi
│   └── launcher.rs         # Manajer siklus hidup PHP server & pembuka browser
├── system/                 # Domain Sistem Operasi Linux
│   ├── mod.rs
│   ├── process.rs          # Eksekusi command aman tanpa injeksi shell
│   ├── service.rs          # Integrasi systemctl & journalctl via PolicyKit
│   ├── port.rs             # Pemeriksa probe port TCP & diagnostik konflik
│   └── package_manager.rs  # Abstraksi pengelola paket distro (Apt, Dnf, Pacman)
├── config/                 # Konfigurasi Pengguna
│   ├── mod.rs
│   └── settings.rs         # Serialisasi JSON ke ~/.config/mysqldesk/settings.json
└── ui/                     # Antarmuka Desktop Native GTK4/Libadwaita
    ├── mod.rs
    ├── window.rs           # AdwApplicationWindow, HeaderBar, ViewSwitcher, ToastOverlay
    ├── dashboard.rs        # Halaman ringkasan status utama & aksi cepat
    ├── server.rs           # Halaman detail server, socket, datadir & config
    ├── phpmyadmin.rs       # Halaman server lokal phpMyAdmin & installer
    ├── logs.rs             # Halaman penampil log journalctl non-blocking
    ├── settings.rs         # Dialog preferensi tema & konfigurasi port
    └── components.rs       # Reusable UI widgets (status badge, card, metrics)
```

---

## 🛡️ Prinsip Keamanan (Security Considerations)

1. **Tanpa Hak Akses Root untuk GUI:**
   Aplikasi desktop GUI tidak boleh dan tidak pernah meminta dijalankan sebagai `root` atau `sudo`. Seluruh antarmuka berjalan di bawah sesi user biasa non-root.
2. **Eskalasi Akses Terstandarisasi (PolicyKit / pkexec):**
   Ketika user menekan tombol Start/Stop database server, aplikasi memanggil perintah lewat PolicyKit (`pkexec systemctl <action> <service>`). Sistem Linux akan menampilkan dialog autentikasi native desktop yang aman.
3. **Pencegahan Injeksi Shell (No Shell Injection):**
   Tidak ada pemanggilan `sh -c` dengan penggabungan string mentah. Semua pemanggilan proses menggunakan `std::process::Command` dengan argument terpisah dan sanitasi ketat.
4. **Isolasi Jaringan phpMyAdmin (Local Loopback Only):**
   PHP development server phpMyAdmin hanya diikat (*bind*) ke alamat `127.0.0.1` (loopback lokal). Tidak pernah diekspos ke antarmuka publik `0.0.0.0` sehingga aman dari akses jaringan luar/LAN.
5. **Verifikasi Integritas Unduhan (SHA256 Verification):**
   Pemasangan phpMyAdmin mengunduh arsip langsung dari server mirror resmi `files.phpmyadmin.net` dan memvalidasi hash kriptografi SHA256 sebelum dilakukan ekstraksi.

---

## 📦 Persyaratan Sistem & Dependensi

### 1. Kebutuhan Runtime (Di Sistem Pengguna):
- **Linux Distribution**: Ubuntu 22.04+, Debian 12+, Fedora 38+, Arch Linux, atau distro Linux modern lainnya.
- **Database Server**: MariaDB Server (`mariadb-server`) atau MySQL Server (`mysql-server`).
- **PHP CLI**: PHP versi 8.0+ beserta ekstensi `php-mysqli`, `php-mbstring`, `php-zip`, `php-json`.

### 2. Kebutuhan Build (Kompilasi):
- Rust & Cargo versi 1.80+ (dapat dipasang via [rustup.rs](https://rustup.rs))
- `pkg-config`
- `libgtk-4-dev`
- `libadwaita-1-dev`
- `build-essential`

#### Instalasi Dependensi di Ubuntu / Debian:
```bash
sudo apt update
sudo apt install -y pkg-config libgtk-4-dev libadwaita-1-dev build-essential
```

#### Instalasi Dependensi di Fedora:
```bash
sudo dnf install -y pkg-config gtk4-devel libadwaita-devel gcc
```

#### Instalasi Dependensi di Arch Linux:
```bash
sudo pacman -S --needed pkgconf gtk4 libadwaita base-devel
```

---

## 🛠️ Panduan Pengembangan (Development Setup)

1. Clone repositori:
   ```bash
   git clone https://github.com/your-username/mysqldesk.git
   cd mysqldesk
   ```
2. Pastikan toolchain Rust sudah siap:
   ```bash
   rustc --version
   cargo --version
   ```
3. Cek dependensi GTK4 & Libadwaita:
   ```bash
   pkg-config --modversion gtk4 libadwaita-1
   ```

---

## 🚀 Kompilasi & Eksekusi

### Menjalankan Mode Development:
```bash
cargo run
```
Untuk mengaktifkan log debug terperinci:
```bash
RUST_LOG=mysqldesk=debug cargo run
```

### Mengompilasi Rilis Produksi Teroptimasi:
```bash
cargo build --release
```
Biner biner hasil build akan berada di `target/release/mysqldesk`. Biner ini telah di-strip dan dioptimasi dengan Link-Time Optimization (LTO) untuk ukuran kecil dan performa cepat.

---

## 🧪 Pengujian (Testing)

Proyek ini dilengkapi rangkaian unit tests untuk memvalidasi parser versi, status systemd, port conflict checker, dan konfigurasi tanpa menyentuh database production:

```bash
cargo test
```

Hasil test mencakup:
- `test_mariadb_version_parsing_debian_ubuntu`
- `test_mariadb_version_parsing_fedora_arch`
- `test_mysql_version_parsing`
- `test_systemd_status_parsing`
- `test_port_detection_available_port`
- `test_port_conflict_reporting`
- `test_settings_serialization_roundtrip`
- `test_parse_port_from_my_cnf`
- `test_package_manager_detection`

---

## 📦 Pembuatan Paket Distribusi (Packaging)

### 1. Pembuatan Paket Debian (.deb):
Telah disediakan skrip otomatis satu klik:
```bash
./scripts/build-deb.sh
```
Paket `.deb` siap pakai akan dihasilkan di `target/mysqldesk_0.1.0_amd64.deb` (ukuran hanya ~1.4 MB).

Atau bila Anda menggunakan perkakas `cargo-deb`:
```bash
cargo install cargo-deb
cargo deb
```

### 2. Pembuatan Paket RPM (.rpm):
Gunakan `cargo-generate-rpm`:
```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
```

### 3. Pembuatan AppImage:
Gunakan `linuxdeploy` dan plugin GTK:
```bash
linuxdeploy-x86_64.AppImage \
  --appdir AppDir \
  -e target/release/mysqldesk \
  -d data/org.mysqldesk.MySQLDesk.desktop \
  -i data/icons/org.mysqldesk.MySQLDesk.svg \
  --plugin gtk \
  --output appimage
```

---

## 📄 Lisensi

Proyek ini dilisensikan di bawah **GNU General Public License v3.0 (GPL-3.0-or-later)**.
