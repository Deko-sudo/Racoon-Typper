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
Download URLs are released after click dispatch rather than immediately.

Remove the unused Rust renderer and `export_result_png` from the handler,
AppManifest, frontend IPC wrapper and capability. Keep the Rust `image`
dependency and the separate heatmap export. Preserve the font source and
license metadata; distribute the full DejaVu license with the application.

This preserves the CONTRIBUTING/ARCHITECTURE boundary: the frontend renders
backend-confirmed results; it neither recomputes scores nor decides completion,
achievement eligibility or persistence. No remote fonts, HTTP client,
additional permissions or weaker CSP are introduced.

Replacing `ab_glyph` with `skrifa` was considered but adds a new dependency family
under the current no-new-dependencies constraint. Removing PNG export would
unnecessarily remove existing behavior.

## Verification and limits

`npm run test:ui --prefix frontend` includes `scripts/share-card.test.mjs` for
rendering inputs, color fallback, bounded failures/retry, download cleanup,
duplicate requests and removal of the obsolete IPC/dependency chain.
Those tests use doubles for Canvas; they do not prove rasterization.

For real WebKitGTK rasterization, build the test-only fixture with existing Vite:

```bash
node scripts/build-share-card-fixture.mjs /tmp/racoon-share-card-fixture
/usr/bin/python3 scripts/check-share-card-webkit.py /tmp/racoon-share-card-fixture /tmp/racoon-share-card-results
```

The latter requires an existing GTK display, system PyGObject and WebKit2 4.1.
It creates an ephemeral context and offscreen window, uses software rendering,
decodes the generated PNG, checks dimensions/text/Unicode/theme/heatmap pixels
and records requested resources. It installs nothing and uses no user profile.
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

## Rollback and remaining advisories

Revert the focused PNG commit, including the lockfile, manifests, ACL and
license inventories, as one unit. No database or settings migration is needed.
GTK/GLib findings remain governed by ADR 0001 and must not be declared fixed by
this presentation change.
