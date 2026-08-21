# PKGBUILD release-source blocker — RESOLVED

The blocker is resolved: the `v1.2.0` tag/archive is published at the canonical
upstream, and `PKGBUILD` pins the verified SHA-256 of that immutable archive.

## Evidence (2026-08-21)

- Source archive: `https://github.com/Deko-sudo/Racoon-Typper/archive/v1.2.0.tar.gz`
- SHA-256: `93e132a63752b3ffcd16521e35aa5ec6e314dccf841f6fba17456960f25c9ec7`
- `makepkg --verifysource` passes against the pinned checksum.
- Full local `makepkg -f` succeeds: `racoon-typper-1.2.0-1-x86_64.pkg.tar.zst`
  (binary, desktop file, icons, THIRD_PARTY_NOTICES.md).

## Remaining owner actions

- Publish the PKGBUILD to the AUR (owner account).
- Re-verify the checksum whenever the tag is recreated for a new version.
