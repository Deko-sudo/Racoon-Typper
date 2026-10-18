# ADR 0008 — Remove the retired macro diagnostic helper

**Status:** Local review candidate, not a published SDK release
**Date:** 2026-10-18
**Decision owner:** Racoon Typper maintainers
**Maps to:** DD4 / Gate G5; supersedes ADR 0001's no-macro-backport decision
for this specific, tested diagnostic-only delta

## Problem and choice

The pinned Linux GTK stack uses glib-macros 0.18.5 and gtk3-macros 0.18.2.
Both depend on proc-macro-error 1.0.4, a retired build-time diagnostic helper
([RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html)).
There is no patched version of that package. The similarly named replacement
proc-macro-error2 is also retired
([RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173.html)).

Backport the upstream move to explicit syn errors, using dependencies already
present in both macro crates. Keep their package versions, generated GTK 0.18
API and the rest of the GUI stack pinned. Do not upgrade the macro crates to
a different GTK generation, add a replacement diagnostic library, rename the
retired helper or suppress its advisory.

The upstream sources are:

- [GLib c74a40a16b23a1b9c8e67842d32d02bc187f551d](https://github.com/gtk-rs/gtk-rs-core/commit/c74a40a16b23a1b9c8e67842d32d02bc187f551d)
- [GTK 45782251fa35ecf8bfe759ffceb3db32659a0c97](https://github.com/gtk-rs/gtk3-rs/commit/45782251fa35ecf8bfe759ffceb3db32659a0c97)

The older 0.18 code lacks newer dynamic-type features. Adapt only diagnostic
propagation to its existing entry points, including the older flags path;
do not import those newer features. An additional early validation rejects
data-carrying Variant enums in enum/flags modes before lazy token generation
can reach an existing unimplemented branch. The original registry macros
panic on these inputs; the candidate reports a compile error at the variant.

## Ownership, provenance and scope

This is substantially larger than ADR 0007's two-line GLib fix: two additional
vendored crates, 14 modified Rust files and four manifest changes. Their
separate patches contain +275/-244 lines relative to the original archives,
including formatting in the modified modules. The full vendored source diff
must not be confused with the maintained patch delta.

Keep the original upstream licenses and copyright notices. The two
`licenses/vendor-*-macros.json` records pin archive checksums, original and
candidate file hashes, upstream commits, local adaptations and patch hashes.
The archives were checked against the registry checksums in the original
application lockfile. Source verification rejects unexpected files/edits and
symlinks; the records are review evidence, not an independent trust root.
The bundle retains licenses, copyright and provenance for both crates.

Maintainers own these additional patches until a compatible upstream SDK
removes the need for them. This candidate follows the explicit local
experiment approved after the no-new-dependencies plan; it does not silently
reclassify the original maintenance warning as fixed by upstream.

The application lockfile removes proc-macro-error, proc-macro-error-attr and
syn 1.0.109; no new package/version pair is added. Migrations, domain/data
behavior, IPC contracts, offline policy and application features are unchanged.

## Verification and diagnostic behavior

```bash
node scripts/check-macro-diagnostics.mjs
node --test scripts/vendor-provenance.test.mjs
node scripts/check-vendored-glib-advisories.mjs
cargo deny --locked check advisories -D unmaintained -D unsound
```

The standalone pinned fixture covers valid Enum, flags, ErrorDomain, Boxed,
SharedBoxed, Variant, clone/closure, ObjectSubclass, ObjectInterface,
Properties/derived_properties and CompositeTemplate expansion. Runtime tests
exercise values, variants, closures and properties without initializing a GUI.
Twenty invalid-input cases check failure, diagnostic text and primary input
spans, and reject macro panics. The exact same runner accepts `--baseline`
to exercise the original registry macro crates: 11/22 contracts pass there,
versus 22/22 with the candidate. Baseline valid-input runtime tests pass;
the red contracts concern helper removal, better input spans and macro panics.

Diagnostic presentation intentionally changes: some errors previously pointed
at the derive invocation and now point at the invalid declaration. The
upstream duplicate-watch diagnostic points at the existing capture and no
longer emits the separate previous-capture note. These changes affect build
diagnostics, not a new runtime language/IPC behavior.

Path packages are not matched by cargo-deny's registry advisory matcher.
The registry reference now pins GLib plus GTK and verifies all three vendored
package identities. It preserves the known registry GLib finding and rejects
any advisory against either macro crate: this diagnostic-only backport is not
a disposition for an unknown future parent-crate defect. The old helper may
appear in that intentionally unpatched reference graph; the application
contract independently requires its absence from the effective graph.
Keep the full workspace advisory gate in CI as well as this source gate.

## Limits and removal

Passing these contracts is not an exhaustive proof of every upstream macro
expansion, generated-code equivalence, GTK safety or cross-platform support.
Retain application builds/tests and native platform smoke. GitHub CI and new
release artifacts have not been executed/published by this local experiment.
Gate G5 and maintainer release review remain separate.

At each dependency review refresh RustSec data and record its revision.
Reassess when compatible Tauri/GTK packages remove the helper or introduce a
new advisory. To remove the local patches, first verify that the compatible
upstream graph needs neither retired helper; then remove both overrides,
vendor records/sources and related notices, regenerate inventory/SBOM and
repeat diagnostics/application/native checks. Reverting to the original
registry macros restores the known maintenance debt and diagnostic panics.
