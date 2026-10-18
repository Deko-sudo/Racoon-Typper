# ADR 0001 — GLib/GTK3 dependency risks

**Status:** Local backport candidates in ADR 0007/0008; release review remains
**Date:** 2026-08-01
**Last reviewed:** 2026-10-18
**Decision owner:** Racoon Typper maintainers
**Maps to:** TECH_DEBT DD4, ROADMAP R-009 and Gate G5

## Current evidence

The original bounded decision below is superseded for the two specific local
backport candidates: ADR 0007 fixes the GLib iterator, and ADR 0008 removes the
retired diagnostic helper from the two pinned macro crates. The latter adds
two maintained local patch deltas, no new packages, and diagnostic regressions.
The effective-graph scan is now clean for the reviewed RustSec revision; the
registry reference preserves the original GLib advisory and rejects future
macro findings. Earlier red-scan statements below describe the pre-ADR-0008
graph, not the current candidate. Release readiness is still not established.

The locally verified lockfile uses Tauri 2.12.4, wry 0.57.0, GTK 0.18.2,
GLib 0.18.5, glib-macros 0.18.5 and gtk3-macros 0.18.2. The compatible SDK
update keeps the GTK3/GLib 0.18 chain. A standalone bump to an incompatible
GLib major-minor line is not a verified replacement for this SDK graph.

The unmodified registry graph, with both `unsound = "all"` and
`unmaintained = "all"`, reported these two findings before the local backport:

| Package | Finding | Classification | Current status |
|---|---|---|---|
| glib 0.18.5 | [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) | Unsound | Reproduced; narrow local backport candidate verified in ADR 0007; registry release remains affected |
| proc-macro-error 1.0.4 | [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html) | Unmaintained | Removed from the local application graph by ADR 0008; original registry graph remains affected |

This is evidence for the locally cached advisory database used in that scan,
not a promise that future advisory databases contain the same findings.
Historical GTK maintenance notices and older scan totals are not the current
scan result. Dependency reviews must refresh/revise the recorded evidence.

`proc-macro-error` is used by glib-macros and gtk3-macros at compile time. Its
maintenance finding is distinct from the GLib runtime soundness defect.
Updating or patching those macros does not fix `VariantStrIter`.

## GLib exposure and limits of the analysis

The affected GLib code passes an immutable reference to an out-argument that
C mutates. The upstream fix uses a mutable reference. Optimized execution of
the affected iterator can otherwise lead to undefined behavior and a crash.

The current Linux graph routes GLib through Tauri/wry/tao/muda and
webkit2gtk/GTK. The application does not directly use `VariantStrIter` or
`Variant::array_iter_str`. A textual search of the Linux normal/build source
graph (340 packages, 7315 Rust source files, including all workspace features)
found these symbols only inside GLib itself, including its docs/tests.

That search is **not proof of runtime unreachability**. Generated code, macro
expansion, FFI and platform/feature differences are not exhaustively covered.
The earlier wording that our process never executes the path was too strong
and is superseded by this evidence-limited statement. The unsafe implementation
remains present in the registry 0.18.5 source. ADR 0007 now records a locally
verified source fix and optimized regression for the vendored candidate.
Windows/macOS runtime behavior is not verified by the Linux checks; see
SUPPORT_MATRIX.md.

## Bounded decision and its explicit local exceptions

Keep the supported Tauri/GTK upstream stack for the current local development
candidate. Do not create further private forks of GTK macros or replace the GUI stack
solely to remove a maintenance warning. ADR 0008 is the explicit exception for
the two diagnostic-only macro backports, justified by the inspected upstream
removal, positive/compile-fail regressions and no-new-packages graph diff. This avoids taking ownership of foreign
macro crates while preserving the project's desktop architecture and existing
no-new-dependencies constraint.

Keep both findings visible (including the registry reference scan in ADR 0007).
Do not add a blanket ignore, reduce scan scope or
claim a clean dependency audit. This bounded development decision does not
itself establish supported production-release readiness or waive Gate G5.

GLib unsoundness receives higher review priority than the macros' maintenance
finding. Before a supported release, the maintainer must explicitly reassess
the available upstream fixes, exposure evidence and G5 decision. A runtime
reproduction is not required before reopening a soundness decision if new
credible reachability evidence appears.

Revisit during every dependency review, and immediately when any of these
occurs:

1. A compatible Tauri/backend release resolves the affected GLib/macro chain.
2. New evidence shows the affected GLib API is invoked by a supported path.
3. A new soundness or security advisory affects a used SDK path.
4. Production-release preparation reaches the G5 dependency decision.

If an urgent GLib backport becomes necessary, review it as a separate decision
with a regression test, pinned provenance, explicit ownership and rollback.
Keeping a patched old package version may still trigger a version-based scan;
a backport must not be hidden by an unexplained ignore or fake version bump.
ADR 0007 is that separate decision: a reproduced runtime crash, exact upstream
delta, provenance/license gates and optimized regression. No macro-crate fork
is included. Its review candidate is not a published release.

## Verification and release gap

After ADR 0008 the effective graph scan returns zero for the recorded RustSec
revision: the retired helper is absent, rather than ignored:

```bash
cargo deny --locked --offline check advisories -D unmaintained -D unsound
```

`--offline` uses cached advisory data. A release review must also record the
advisory database revision and refresh it through the approved development/CI
process. This is build/review tooling, not network access inside the product.
In cargo-deny 0.20.2, severity flags alone do not widen the `unsound` scope;
retain the explicit `all` settings in deny.toml.

The local CI configuration now includes a full workspace advisory gate and a
registry-reference source gate for all three vendored crates; it has not run
on GitHub for this unpublished branch. The reference retains the original
GLib finding and rejects new macro-crate findings, because cargo-deny does
not match crates.io advisories to path packages. See ADR 0007/0008. The source
backport and actual helper removal provide the dispositions; passing a build
alone would not. Maintainer review of local patch ownership and Gate G5
remains a release requirement.

The earlier quick-xml/unic findings are no longer present in the current scan
after compatible dependency upgrades. The separate font-parser finding was
removed with the Canvas result-card change described in ADR 0002. Those fixes
do not fix GTK/GLib.

## References

- [GLib advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
- [Upstream GLib fix](https://github.com/gtk-rs/gtk-rs-core/pull/1343)
- [proc-macro-error advisory](https://rustsec.org/advisories/RUSTSEC-2024-0370.html)
- ROADMAP Gate G5 and R-009; TECH_DEBT DD4; SUPPORT_MATRIX.md
