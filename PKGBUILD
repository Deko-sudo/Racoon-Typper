# Maintainer: Racoon Typper Contributors
pkgname=racoon-typper
pkgver=0.9.0
pkgrel=1
pkgdesc="Local desktop touch-typing trainer for Linux — combines Monkeytype and Stamina"
arch=('x86_64')
url="https://github.com/racoon-typper/racoon-typper"
license=('MIT')
depends=('webkit2gtk-4.1')
makedepends=('rust' 'npm' 'base-devel')
source=("$pkgname-$pkgver.tar.gz::https://github.com/racoon-typper/racoon-typper/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    npm install --prefix frontend
    cargo tauri build --release
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

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
}