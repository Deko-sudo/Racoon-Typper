# ADR 0001 — GLib/GTK3 advisory chain (RUSTSEC-2024-0429, gtk3-rs unmaintained)

**Status:** Accepted (documented acceptance with isolation + upgrade trigger)
**Date:** 2026-08-01
**Decision owner:** Racoon Typper maintainers
**Maps to:** `TECH_DEBT.md` DD4, `ROADMAP.md` risk R-009, Phase 5 dependency-decision gate (G5)
**Review trigger:** Tauri 2.x landing a gtk4-rs / `glib >= 0.20` webview backend; or a *soundness* (not maintenance) advisory that is reachable from our process.

## Context

`cargo audit` against the current lockfile (`Cargo.lock`, Tauri 2.11.3 / wry 0.55.1) reports
one **unsound** advisory on the GLib line and a cluster of **unmaintained** warnings on the
GTK3 (`gtk3-rs`) bindings. These map to ROADMAP risk R-009 and dependency debt DD4.

### The advisory chain

| Crate (locked) | Advisory | Class | Affected range | Fix |
|---|---|---|---|---|
| `glib` 0.18.5 | [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429) (GHSA-wrw7-89jp-8q8g) | Unsound | `>=0.15.0, <0.20.0` | `>=0.20.0` |
| `atk` / `atk-sys` 0.18.2 | RUSTSEC-2024-0413 / -0416 | Unmaintained | gtk3-rs line | gtk4-rs line |
| `gdk` / `gdk-sys` 0.18.2 | RUSTSEC-2024-0412 / -0418 | Unmaintained | gtk3-rs line | gtk4-rs line |
| `gdkx11` / `gdkx11-sys` 0.18.2 | RUSTSEC-2024-0417 / -0414 | Unmaintained | gtk3-rs line | gtk4-rs line |
| `gdkwayland-sys` 0.18.2 | RUSTSEC-2024-0411 | Unmaintained | gtk3-rs line | gtk4-rs line |
| `gtk` / `gtk-sys` 0.18.2 | RUSTSEC-2024-0415 / -0420 | Unmaintained | gtk3-rs line | gtk4-rs line |
| `gtk3-macros` 0.18.2 | RUSTSEC-2024-0419 | Unmaintained | gtk3-rs line | gtk4-rs line |

The GLib unsoundness: `glib::VariantStrIter` passed a `&T` where the underlying variadic C
function mutates through it (out-argument). Optimized builds could drop the in-place write,
leaving a null pointer handed to `CStr::from_ptr` and a potential crash. The maintainer fix is
to pass the pointer as `&mut`.

### Reachability in Racoon Typper

- `glib`, `gtk`, `webkit2gtk` and the rest of the chain are **transitive only**. A workspace
  grep finds **no direct use** of `glib` in any `crates/` source or the frontend.
- The whole chain is the **Linux webview backend of Tauri** (`racoon-app → tauri → wry →
  webkit2gtk → gtk/glib`, plus `tao`/`muda`). It is the standard, supported way Tauri renders
  on Linux.
- It is **Linux-only**. `cargo tree --target x86_64-pc-windows-msvc` and
  `--target aarch64-apple-darwin` resolve **no** `webkit2gtk`/`gtk`/`glib` at all; Windows uses
  WebView2 and macOS uses WKWebView. (See `SUPPORT_MATRIX.md` — Linux x86_64 is the only
  verified target; the others are unverified.)
- The unsound symbol (`VariantStrIter`) is an iterator over a GLib `Variant` string collection.
  Our process never constructs or iterates such a value; the path is exercised only inside
  Tauri/wry GTK glue, not by our IPC or rendering code.

### Options considered

1. **Upgrade.** The GLib fix is in `glib >= 0.20`, i.e. the **gtk4-rs** line. Tauri's Linux
   backend on wry 0.55.1 still depends on the gtk3-rs (glib 0.18) line. Moving to glib 0.20 is
   an **upstream Tauri/wry migration to GTK4**, not a version bump we control. As of
   2026-08-01, bumping `tauri = "2"` (latest 2.11.5) / `wry` (latest 0.56.0) does not lift glib
   out of the 0.18 line.
2. **Isolate / avoid.** There is no supported Tauri Linux webview that avoids GTK/GLib
   (webkit2gtk, and the experimental webkit6 path, both sit on it). Removing it would mean
   leaving Tauri, which ROADMAP §1 and §4.3 have explicitly rejected for this product.
3. **Documented acceptance with a defined trigger.** Accept the transitive advisory with a
   written exposure analysis, scope it to Linux, record the upgrade trigger, and keep it on the
   dependency-debt register.

## Decision

**Option 3 — documented acceptance**, because the advisory is transitive, Linux-only, in code
our application does not execute (`VariantStrIter`), and the real fix is gated on an upstream
Tauri migration we cannot force. This satisfies the Phase 5 requirement that no
dependency-decision go undocumented, and pins a concrete remediation trigger rather than an
open-ended "accept."

Acceptance is bounded:

- **Scope the risk.** The exposure is confined to Tauri/wry GTK glue on Linux. We do not
  accept it for any code we own, and we do not accept reaching `glib::VariantStrIter` from our
  own IPC/rendering paths (none do).
- **Track, don't silence.** We will not add a blanket `[advisories]` ignore for
  RUSTSEC-2024-0429 in `deny.toml`. The advisory stays visible in `cargo audit`/`cargo deny`
  output so it is re-evaluated every dependency review. (CI currently runs only
  `cargo deny check licenses`; an advisory gate is itself Phase 6 work — see Consequences.)
- **Remediation trigger (automatic revisit).** Re-open this decision when **any** of:
  1. a Tauri 2.x (or 3.x) release ships a Linux backend on `glib >= 0.20` / gtk4-rs — then we
     bump and drop the accepted advisory;
  2. a **soundness** advisory (not a maintenance/info one) appears in a part of the GLib/GTK
     chain that is actually reachable from Tauri's rendering or IPC path;
  3. a CVE with a network/IPC-reachable vector is published against this chain.
- **Operational note preserved.** Running Tauri locally on this host already requires
  `env -u LD_LIBRARY_PATH npm run tauri:dev --prefix frontend` to avoid GLib-symbol clashes
  from the Flatpak/Zed sandbox. That workaround is orthogonal to this advisory but is recorded
  here so it is not mistaken for an advisory symptom.

## Consequences

- **Positive.** No artificial fork, no out-of-tree Tauri patches, no blanket advisory ignores
  that would hide future real issues. The risk is named, scoped, and trigger-bounded.
- **Negative / residual risk.** Linux builds keep a transitive unsound crate. The unsound path
  is not reachable from our code, but a future Tauri/wry change could begin using
  `VariantStrIter`; the trigger above covers that.
- **CI gap to close (separate task).** `ci.yml` runs `cargo deny check licenses` but not
  `cargo deny check advisories` (and `deny.toml` has no advisory `ignore` list). Adding an
  advisory gate is Phase 6 release-work; when it lands, it must either (a) accept
  RUSTSEC-2024-0429 with a comment pointing here and an expiry/trigger, or (b) fail open until
  the Tauri gtk4 migration arrives. Either way, this ADR is the justification record.
- **Other audit findings (not in scope of this ADR).** `cargo audit` also reports
  RUSTSEC-2026-0194 / -0195 (`quick-xml` < 0.41, via `plist → tauri-utils`, also Tauri
  transitive) and RUSTSEC-2026-0190 (`anyhow` unsound `downcast_mut`, also Tauri transitive),
  plus several `unic-*` maintenance warnings. These are recorded for the dependency-review
  cadence; none are reachable from our own code and each is gated on the same upstream Tauri
  update path.

## Verification performed

- `cargo audit` on the committed lockfile (2026-08-01): 2 vulnerabilities, 18 warnings.
- `cargo tree -i glib@0.18.5`: all paths route through `tauri/wry/webkit2gtk/gtk`.
- `cargo tree --target x86_64-pc-windows-msvc -i webkit2gtk` and
  `--target aarch64-apple-darwin`: nothing to print (Linux-only).
- Workspace grep for `glib` in `crates/` and `frontend/src/`: no matches (transitive only).

## References

- Advisory: https://rustsec.org/advisories/RUSTSEC-2024-0429 (GHSA-wrw7-89jp-8q8g)
- gtk3-rs unmaintained cluster: RUSTSEC-2024-0411 through -0420
- ROADMAP.md §10 R-009, Phase 5 gate G5 validation checklist
- TECH_DEBT.md DD4
- SUPPORT_MATRIX.md (target verification status)
