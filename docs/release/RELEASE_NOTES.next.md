# Desktop stability update — release draft

Version and publication date: pending maintainer decision. This draft describes
the unreleased candidate; it does not replace v1.3.1 assets or announce a release.

- Validate whole-database restore on a private migrated copy before replacement;
  retain complete snapshots and journal replacement/recovery. Reject incompatible,
  changed-trigger, corrupt and empty files without discarding the live history.
- Preserve custom-test language/source through repeat, recover the saved-text list
  on abort, and keep input/caret/session state consistent. Localize achievements
  and require speed/accuracy thresholds within the same completed test.
- Reject stale theme/search responses. Keep all training, data and diagnostics
  local; remove network updater and URL-import entry points.
- Render Unicode result PNGs and heatmaps, save under Pictures/Raccoon Typer,
  copy an image to the OS clipboard, and report save/copy outcomes separately.
- Build shared fixes for Linux deb/RPM/AppImage, Windows x64 NSIS and macOS Intel/
  Apple Silicon DMG/app. Windows embeds its offline WebView2 installer.
- Maintain the existing pinned dependency backports with source/provenance and
  optimized/advisory regression gates; no new runtime dependencies or migrations.

Linux container package/restart checks and Windows/macOS native CI are documented
in [candidate evidence](desktop-stability-evidence.md). Windows/macOS hardware
RU/EN, DPI/caret, PNG/paste and OS security acceptance remain pending. Packages
are unsigned/unnotarized candidates; no physical power-loss guarantee is made.

Before updating, export the profile and fully close older application instances.
Use the exact source/checksum manifest. Keep the previous known-good package and
a pre-upgrade profile copy; do not downgrade a migrated live database as rollback.
Do not disable Gatekeeper, SmartScreen or WebView protections to install a candidate.
