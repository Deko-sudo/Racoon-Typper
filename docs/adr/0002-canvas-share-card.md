# ADR 0002 — Render the result PNG in the presentation layer

**Status:** Accepted for local implementation; platform/release checks remain required
**Date:** 2026-10-18
**Scope:** Result share card only; scores, persistence and analytics remain backend-owned

## Problem and boundary

The result overlay passed already computed scores and theme colors to
`export_result_png`. Rust rendered a 900×470 PNG using embedded DejaVu fonts and
returned bytes for a frontend Blob download. The direct `ab_glyph` dependency
pulled `owned_ttf_parser` and the unmaintained `ttf-parser`
([RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html)).
The export never authenticated scores or wrote history; it was presentation.

## Decision

Use Canvas 2D and the existing WebView PNG encoder, without adding Cargo/npm
dependencies. `frontend/src/lib/shareCard.ts` draws the card with the same
bundled DejaVu fonts. Font loading and encoding are bounded and return explicit
errors. The result button prevents overlapping exports and permits retries.
The native `save_result_card` adapter receives bounded PNG bytes and returns independent file/clipboard acknowledgements. It does not render or recompute scores.

The result heatmap is a character grid, not a reconstruction of physical keys.
Backend heatmap entries describe the first attempt at an expected Unicode
scalar in the task. The previous fixed Latin keyboard omitted Cyrillic, CJK,
diacritics, uppercase letters, spaces and punctuation. Keep the exact keys
without case folding, normalization or transliteration, and account for every
positive entry once. Per-character percentages present backend first-attempt
counts; they do not redefine WPM or overall accuracy.

The card stays 900 pixels wide; its height grows from 470 to at most 900 pixels.
Show at most 60 characters ordered by frequency, then Unicode scalar value.
For larger sets explicitly state how many are shown and sum the remaining
character count, first attempts, correct and incorrect counts. The remainder
must conserve the original totals. New grid labels follow the UI language,
independently of the practice language. Non-ASCII characters also carry an
ASCII code-point caption; spaces and combining/invisible characters have
readable labels. No remote font or new font asset is added.

Remove the unused Rust renderer and `export_result_png` from the handler,
AppManifest, frontend IPC wrapper and capability. Keep the Rust `image`
dependency and the separate heatmap export. Preserve the font source and
license metadata; distribute the full DejaVu license with the application.

This preserves the CONTRIBUTING/ARCHITECTURE boundary: the frontend renders
backend-confirmed results; it neither recomputes scores nor decides completion,
achievement eligibility or persistence. No remote fonts, HTTP client,
new dependencies or weaker CSP are introduced. The sole added application permission is `allow-save-result-card`; it accepts no path and grants no general filesystem or clipboard-read access.

Replacing `ab_glyph` with `skrifa` was considered but adds a new dependency family
under the current no-new-dependencies constraint. Removing PNG export would
unnecessarily remove existing behavior.

## Native delivery amendment — 2026-10-19

Browser download dispatch is not an acknowledgement that a native WebView saved
anything. The result button now sends Canvas PNG bytes to an asynchronous app
adapter, which validates PNG decoding (4 MiB encoded, at most 900×900 pixels,
900-pixel width, minimum 470-pixel height, decoder allocation bound). It writes
into the OS Pictures directory / `Raccoon Typer` with generated collision-safe
names, flushes the file, and copies decoded RGBA image pixels through the
already installed clipboard plugin. No clipboard text/path substitution is used.
Neither delivery failure suppresses the other attempt. The response exposes the
saved path only on acknowledged file success and a separate clipboard boolean;
private IO error details do not cross IPC. Progress has an indeterminate
animation, a reduced-motion alternative and a live status region. Explicit
messages for all 15 UI languages report complete or partial success.

Pictures uses the OS resolver (Linux XDG_PICTURES_DIR, macOS Pictures, Windows
Known Folder). No fallback silently saves elsewhere when it cannot be resolved.
Generated files are create-new, Unix mode 0600; newly created directories use
0700. Existing files, including filename symlinks, are never overwritten; a
symlink in the application's gallery directory is rejected. This is not a
sandbox against another process replacing parent directories concurrently.
Write/flush errors attempt to remove the partial file; power failure can still
leave a partial file, and no directory-fsync durability is promised. Exports are
retained until the user deletes them. A retry can create another PNG.

Clipboard success means the OS accepted image data while the application is
running. Pasting depends on the receiving application accepting images. On
Linux the clipboard may stop being available after app exit unless a clipboard
manager retains it. Windows/macOS and native Wayland paste still require smoke
validation; do not infer platform success from unit tests or an X11 run.

## Verification and limits

`npm run test:ui --prefix frontend` includes `scripts/share-card.test.mjs` for
rendering inputs, color fallback, bounded failures/retry, OS acknowledgements,
duplicate requests and removal of the obsolete IPC/dependency chain.
Those tests use doubles for Canvas; they do not prove rasterization.
They cover all 15 selectable language fixtures, exact Unicode identities,
single-count conservation, empty/invalid counts, limits around 60 characters,
large-set overflow, explicit translations and UI/practice language separation.

For real WebKitGTK rasterization, build the test-only fixture with existing Vite:

```bash
node scripts/build-share-card-fixture.mjs /tmp/racoon-share-card-fixture
/usr/bin/python3 scripts/check-share-card-webkit.py /tmp/racoon-share-card-fixture /tmp/racoon-share-card-results
```

The latter requires an existing GTK display, system PyGObject and WebKit2 4.1.
It creates an ephemeral context and offscreen window, uses software rendering,
decodes the generated PNG, checks dimensions/text/Unicode/theme/heatmap pixels
and records requested resources. It also writes individual PNGs for all 15
languages plus an overflow case. Tile surface pixels and readable contents
are checked; inspect actual Cyrillic/CJK glyphs visually as well. It installs
nothing and uses no user profile.
The fixture uses the application's production CSP. It is not an ordinary
application entrypoint and is not included in the production frontend bundle.

The initial local run passed on WebKitGTK 2.52.6. Browser-audit also completed
a short Russian session with real Rust/SQLite and downloaded the PNG through
the result button. This is not native Tauri dialog/download E2E, hardware-GPU
coverage, Windows/macOS validation or a clean-install check.

Canvas rasterization and font fallback can vary between WebViews. Exact PNG
bytes across platforms are not promised; glyphs absent from bundled DejaVu may
use WebView fallback. Font parsing still exists inside the system WebView; this
change removes a particular Rust dependency, not all font-parser risk.
An ASCII code-point caption keeps a non-ASCII character identifiable even if
the platform lacks its glyph. It is not a promise that every OS has CJK fonts.
The separate HTML heatmap and its key/finger mappings are outside this change.

## Rollback and remaining advisories

Revert the focused PNG commit, including the lockfile, manifests, ACL and
license inventories, as one unit. For the delivery change, revert its frontend/adapter/ACL changes together; existing exported PNG files are retained. No database or settings migration is needed.
GTK/GLib findings remain governed by ADR 0001 and must not be declared fixed by
this presentation change.
