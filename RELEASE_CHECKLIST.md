# Release Checklist — Racoon Typper

## Pre-Release

### Code Quality
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` — 0 warnings
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo audit` — 0 vulnerabilities (unmaintained warnings OK)
- [ ] No TODO/FIXME in code
- [ ] No dead code warnings

### Build
- [ ] `npm run build` (frontend) passes
- [ ] `cargo tauri build` passes without errors
- [ ] Release binary starts and opens window
- [ ] Database creates on first run
- [ ] Settings file creates on first run

### Migrations
- [ ] Fresh DB: all 3 migrations apply (V001, V002, V003)
- [ ] Existing DB: upgrade without data loss
- [ ] Migration idempotent: re-opening DB safe

### Packaging
- [ ] PKGBUILD: `makepkg -si` works on clean Arch
- [ ] AppImage: `build-appimage.sh` produces working AppImage
- [ ] Flatpak: `flatpak-builder` produces working Flatpak
- [ ] Windows: NSIS installer builds
- [ ] All artifacts start without errors

## Release Process

1. Update version in `Cargo.toml` (workspace), `tauri.conf.json`, `PKGBUILD`
2. Run full test suite
3. Create git tag: `git tag -a v1.0.0 -m "Release v1.0.0"`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions builds artifacts automatically
6. Create GitHub Release (draft)
7. Attach artifacts: AppImage, tarball, NSIS exe, torrent
8. Publish release
9. Update AUR package (if applicable)

## Post-Release

- [ ] Verify download links work
- [ ] Verify AppImage runs on clean Ubuntu
- [ ] Verify NSIS installs on clean Windows
- [ ] Verify PKGBUILD installs on clean Arch

## Rollback Process

1. Delete GitHub Release
2. Delete git tag: `git tag -d v1.0.0 && git push origin :refs/tags/v1.0.0`
3. Revert to previous tag: `git checkout v0.9.0`
4. Rebuild and re-release as v1.0.1

## Version Numbering

- `vX.Y.Z` — stable release
- `vX.Y.0-rc.N` — release candidate
- `vX.Y.0-alpha.N` — alpha