# Maintenance contract for the three pinned GTK backports

**Decision:** Linux candidate policy, 2026-10-20. **Accountable role:** repository
maintainer; the person approving each release also verifies the evidence below.
This records the existing three-patch exception; it authorizes no new vendor
forks, advisory suppressions or GUI-stack migration.

## Maintained deltas

| Package | Purpose | Immutable source / patch record |
|---|---|---|
| glib 0.18.5 | Upstream mutable out-argument fix for VariantStrIter | `licenses/vendor-glib.json`, ADR 0007 |
| glib-macros 0.18.5 | Diagnostic-only removal of retired helper | `licenses/vendor-glib-macros.json`, ADR 0008 |
| gtk3-macros 0.18.2 | Diagnostic-only removal of retired helper | `licenses/vendor-gtk3-macros.json`, ADR 0008 |

Each record binds the upstream archive hash, complete local file list/hashes,
patch hash and upstream provenance. Licenses/notices stay in source and bundles.
The dependency license policy remains Apache-2.0-compatible, including the
explicit existing MIT upstream notices; project-owned code remains Apache-2.0.

## Required before every release and monthly maintenance review

1. Refresh RustSec advisory data; record revision and fetch time. An offline
   cached scan is labelled with that revision, never called a live audit.
2. Run the effective-workspace advisory/license gates, provenance tests and
   `node scripts/check-vendored-glib-advisories.mjs`. Path overrides alone are
   not covered by cargo-deny's crates.io matcher. Keep the unpatched registry
   reference: preserve the reviewed GLib finding, reject new parent-crate
   findings and do not add ignores to obtain a passing run.
3. Run optimized GLib regression, all 22 macro diagnostic contracts, workspace
   and frontend checks, and the applicable native/package smoke. Record exact
   versions/commits and failed attempts as well as successful runs.
4. Compare compatible Tauri/GTK upstream updates. Remove backports only when
   the compatible dependency graph makes them unnecessary and all gates pass.

A new advisory against any patched parent crate, a hash/list mismatch, SDK
version change, failing optimized/macro test, or changed unsafe upstream code
blocks release pending a documented review. Source/hash records must change in
an independently reviewable commit with an updated patch and regression.

## Exit and rollback

Remove overrides, vendor directories and records together, regenerate notices/
inventory/SBOM, then repeat all gates on the compatible upstream stack. Reverting
to the original registry GLib restores the known defect; it is not a safe fallback.
When evidence is missing, retain the last known-good release and do not publish
the candidate. No amount of scan output proves the absence of future defects.

The monthly review is a maintainer obligation, not a scheduler installed by this
change. Platform claims remain governed by SUPPORT_MATRIX.md.
