#!/usr/bin/env bash
set -e

# Build script for creating a native Debian package (.deb)
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

VERSION="0.1.0"
PACKAGE_NAME="mysqldesk"
ARCH="amd64"
DIST_DIR="$PROJECT_DIR/target/debian"
STAGE_DIR="$DIST_DIR/${PACKAGE_NAME}_${VERSION}_${ARCH}"
DEB_FILE="$PROJECT_DIR/target/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo "==> 1. Membangun biner rilis teroptimasi..."
cargo build --release

echo "==> 2. Menyiapkan struktur direktori paket .deb..."
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/DEBIAN"
mkdir -p "$STAGE_DIR/usr/bin"
mkdir -p "$STAGE_DIR/usr/share/applications"
mkdir -p "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$STAGE_DIR/usr/share/doc/$PACKAGE_NAME"

echo "==> 3. Menyalin berkas biner dan desktop assets..."
install -m 755 "$PROJECT_DIR/target/release/mysqldesk" "$STAGE_DIR/usr/bin/mysqldesk"
install -m 644 "$PROJECT_DIR/data/org.mysqldesk.MySQLDesk.desktop" "$STAGE_DIR/usr/share/applications/org.mysqldesk.MySQLDesk.desktop"
install -m 644 "$PROJECT_DIR/data/icons/org.mysqldesk.MySQLDesk.svg" "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps/org.mysqldesk.MySQLDesk.svg"
install -m 644 "$PROJECT_DIR/README.md" "$STAGE_DIR/usr/share/doc/$PACKAGE_NAME/README.md"

# Calculate installed size in KB
INSTALLED_SIZE=$(du -sk "$STAGE_DIR/usr" | cut -f1)

echo "==> 4. Membuat metadata DEBIAN/control..."
cat << EOF > "$STAGE_DIR/DEBIAN/control"
Package: $PACKAGE_NAME
Version: $VERSION
Section: database
Priority: optional
Architecture: $ARCH
Installed-Size: $INSTALLED_SIZE
Maintainer: MySQLDesk Contributors <developer@mysqldesk.org>
Depends: libgtk-4-1 (>= 4.6), libadwaita-1-0 (>= 1.2), pkexec | policykit-1, php-cli, mariadb-server | mysql-server
Homepage: https://github.com/mysqldesk/mysqldesk
Description: Native Linux MySQL/MariaDB & phpMyAdmin Management Control Panel
 MySQLDesk is a lightweight, modern, native Linux desktop application
 built with Rust and GTK4/Libadwaita to easily control MariaDB/MySQL
 and launch local phpMyAdmin without web wrapper bloat.
EOF

echo "==> 5. Mengompresi paket .deb via dpkg-deb..."
dpkg-deb --build --root-owner-group "$STAGE_DIR" "$DEB_FILE"

echo ""
echo "========================================================="
echo " SUKSES! Paket .deb berhasil dibuat:"
echo " File: $DEB_FILE"
echo " Ukuran: $(du -h "$DEB_FILE" | cut -f1)"
echo ""
echo " Cara Instalasi:"
echo "   sudo dpkg -i $DEB_FILE"
echo "   sudo apt-get install -f   # jika butuh resolusi dependensi"
echo "========================================================="
