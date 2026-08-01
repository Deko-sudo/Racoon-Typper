# Maintainer: Racoon Typper Contributors
pkgname=racoon-typper
pkgver=1.1.0
pkgrel=1
pkgdesc="Local desktop touch-typing trainer for Linux"
arch=('x86_64')
url="https://github.com/racoon-typper/racoon-typper"
license=('Apache-2.0')
depends=('webkit2gtk-4.1')
makedepends=('rust' 'npm' 'base-devel')
source=("$pkgname-$pkgver.tar.gz::https://github.com/racoon-typper/racoon-typper/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    npm ci --prefix frontend
    npm run tauri:build:binary --prefix frontend
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
    install -Dm644 "THIRD_PARTY_NOTICES.md" "$pkgdir/usr/share/doc/$pkgname/THIRD_PARTY_NOTICES.md"
}
