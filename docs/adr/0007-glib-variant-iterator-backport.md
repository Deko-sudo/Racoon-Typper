# ADR 0007 — Narrow GLib variant iterator backport

**Status:** Locally verified review candidate; not published
**Date:** 2026-10-18
**Decision owner:** Racoon Typper maintainers
**Maps to:** TECH_DEBT DD4, ROADMAP R-009 / Gate G5; supersedes the no-backport
statement for GLib in ADR 0001 only

## Evidence and decision

The compatible Tauri 2.12.4 / wry 0.57.0 graph still selects GLib 0.18.5.
[RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
describes undefined behavior in `VariantStrIter::impl_get`: C writes through
an immutable Rust reference to an out-parameter. The registry release is
affected; its published patched range starts at 0.20.0.

An isolated, unmodified copy of 0.18.5 crashed with SIGSEGV in an optimized
regression run on Linux x86_64 / Rust 1.96.0. Applying the two-line upstream
fix made all three iterator tests pass in debug and release. This demonstrates
the library defect, not its reachability from a particular application flow.

Vendor the original 0.18.5 crate with only the change from
[gtk-rs PR #1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343/files):
`let p` becomes `let mut p`, and the C out-argument becomes `&mut p`.
There is no version/API change, new application dependency, migration change,
new IPC permission, GUI-stack replacement or fork of the GTK macro crates.
The demonstrated runtime defect justifies this narrow exception to keeping
all SDK sources upstream. A maintenance warning alone would not justify it.

## Source ownership and licensing

`vendor/glib-0.18.5` contains the original 121 archive files (approximately
1.7 MB), with only `src/variant_iter.rs` modified. The large source diff is
vendoring, not a rewrite of GLib. The upstream MIT license and copyright are
retained. The archive SHA-256 is
`233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`.

`licenses/vendor-glib.json` records every source hash, original/patched
iterator hashes, archive provenance, upstream fix and the separate patch.
The license inventory/SBOM includes local third-party packages and rejects
untracked local packages; GLib must not disappear merely because Cargo marks
a path dependency with `source: null`. The app bundle includes the vendor
license, copyright and provenance record. Hash checks reject missing/extra/edited files
and symlinked sources. They detect drift against the reviewed record; they do
not independently authenticate a maliciously replaced record and source.

Maintainers own this exact two-line delta and its review until removal. Do
not reformat or otherwise modify vendored sources as incidental cleanup.

## Verification and advisory visibility

The standalone optimized contract covers forward/reverse iteration,
`next`/`nth`/`last`/`nth_back`, exhaustion, extreme offsets, UTF-8, empty arrays
and repeated strings:

```bash
cargo test --manifest-path scripts/glib-variant-regression/Cargo.toml --locked --release
node --test scripts/vendor-provenance.test.mjs
npm run license:check --prefix frontend
node scripts/check-vendored-glib-advisories.mjs
cargo deny --locked check advisories -D unmaintained -D unsound
```

Cargo-deny 0.20.2 does not match the crates.io advisory to the local path
package. Its disappearance from the effective graph scan is **not evidence
of a fix**. The separate registry-only reference project pins unmodified
0.18.5 and preserves its raw advisory diagnostics. Its gate requires the
known finding plus a verified source disposition; an additional or missing
GLib advisory fails closed for review. CI also runs the optimized contract.
The registry project is review tooling, not shipped runtime code. CI network
access to refresh advisories does not add network access to the offline app.

The original ADR 0007 graph remained nonzero for
`proc-macro-error` 1.0.4 / RUSTSEC-2024-0370. ADR 0008 now removes that helper
from the two pinned macro crates with separate diagnostic-only backports and
regressions; the effective-graph scan is clean for the reviewed database.
No ignore, fake version bump or reduced advisory scope is added. Passing the reference gate means the known
GLib disposition is internally consistent; it does not mean all dependencies
are clean. CI changes have been checked locally but not run on GitHub.

## Limits, removal and rollback

This candidate fixes the reproduced out-parameter defect. It does not certify
all GLib/GTK code, prove application reachability, establish support for
Windows/macOS, run the entire upstream GLib test suite or Miri, or complete
Gate G5. Native file dialogs, packaged install, real OS layout/IME and GPU
paths need their own evidence. Release approval remains separate.

At every dependency review, refresh advisory data and reassess the pinned
delta. Remove the vendor override, vendor/reference fixtures and backport
gates when a compatible upstream SDK resolves to a verified fixed GLib;
regenerate notices/SBOM and repeat the application and iterator checks.
Do not simply remove the override while retaining the affected registry
version. Reverting this candidate restores the known defective library and
requires an explicit risk decision before release.

## Candidate maintenance ownership — 2026-10-20

The repository maintainer/release reviewer owns the three existing deltas under
[the maintenance contract](../dependencies/vendor-maintenance.md). Before release,
record a fresh advisory revision and repeat its fail-closed source/reference,
optimized regression, macro diagnostic and native validation gates. This closes
the ownership/procedure question, not all future dependency risk.
