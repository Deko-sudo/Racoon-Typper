# Maintainer: Racoon Typper Contributors
pkgname=racoon-typper
pkgver=1.3.0
pkgrel=1
pkgdesc="Local desktop touch-typing trainer for Linux"
arch=('x86_64')
url="https://github.com/Deko-sudo/Racoon-Typper"
license=('Apache-2.0')
depends=('webkit2gtk-4.1')
makedepends=('rust' 'npm' 'base-devel')
source=("$pkgname-$pkgver.tar.gz::https://github.com/Deko-sudo/Racoon-Typper/archive/v$pkgver.tar.gz")
sha256sums=('93e132a63752b3ffcd16521e35aa5ec6e314dccf841f6fba17456960f25c9ec7')

build() {
    cd "$srcdir/Racoon-Typper-$pkgver"
    # makepkg's default CFLAGS include -flto=auto, which breaks linking of the
    # bundled SQLite/ring C code (undefined sqlite3_* / ring_core_* symbols).
    # Strip the LTO flag for the C compiler; Rust's own thin LTO still applies.
    export CFLAGS="${CFLAGS//-flto=auto/}"
    export CXXFLAGS="${CXXFLAGS//-flto=auto/}"
    npm ci --prefix frontend
    npm run tauri:build:binary --prefix frontend
}

package() {
    cd "$srcdir/Racoon-Typper-$pkgver"

    # Binary
    install -Dm755 "target/release/racoon-app" "$pkgdir/usr/bin/racoon-typper"

    # Desktop file
    install -Dm644 "racoon-typper.desktop" "$pkgdir/usr/share/applications/racoon-typper.desktop"

    # Icons
    for size in 32 128; do
        install -Dm644 "crates/app/icons/${size}x${size}.png" \
            "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/racoon-typper.png"
    done
    install -Dm644 "crates/app/icons/128x128@2x.png" \
        "$pkgdir/usr/share/icons/hicolor/256x256/apps/racoon-typper.png"

    # License
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "THIRD_PARTY_NOTICES.md" "$pkgdir/usr/share/doc/$pkgname/THIRD_PARTY_NOTICES.md"
}
