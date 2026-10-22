# Linux stability candidate evidence

Reviewed on 2026-10-21. This is an unreleased, unsigned Linux candidate; it
does not replace the published v1.3.1 assets, create a tag or authorize promotion.

Latest Linux-only integration: `90124ec7e3d2de884913cd33713859b8afa5c4e8`,
tree `3e250dfb79467393fbca2c017a73e08554bd925c`.
[Run 39634759386](https://github.com/Deko-sudo/Racoon-Typper/actions/runs/39634759386)
passed all four jobs, including the Russian-course correction (789 Rust tests,
one ignored). Corrected production Debian package: SHA-256
`80273e129829194e7e97a861db90aca3b12a6e955207295cd688bc0d3faf926d`,
7,072,558 bytes; fresh install and two retained-data launches were repeated.
The earlier native package/results below retain their original checksums.
For the subsequent combined source and Windows/macOS/other Linux packages, see
[desktop evidence](desktop-stability-evidence.md).

## Source and integration

- Integration PR: <https://github.com/Deko-sudo/Racoon-Typper/pull/14>, targeting
  `master` (`71c61e9d7f6279c49c898b5bda0de2faeb42e9ed`). `main` and PR #5 are unchanged.
- Tested published source: `6b2b54e1d6bf33af54b3d2b2127c2a3d10835f97`.
- [CI run 39328391114](https://github.com/Deko-sudo/Racoon-Typper/actions/runs/39328391114)
  passed all four jobs: Rust, frontend, license policy and Linux candidate.
- The candidate job builds a production Debian package on Ubuntu 24.04,
  installs it, verifies nonempty data across two launches and uploads source/tree
  records and SHA-256 checksums. It does not publish a release.
- CI explicitly fetches the locked macro-fixture dependencies before its offline
  checks. Strict `-D warnings` remains on workspace Clippy, rather than being
  inherited by upstream crates through a global action default.

## Initial locally exercised production package

The package was built in official Ubuntu 24.04 userspace from source tree
`bd68b6c8444faf4217242e2bea72407828437fe9` (published commit `93bb470c264f7fb53b665de0c983776dd9254769`).
The subsequent source changes through `6b2b54e` concern CI and documentation;
application code/resources and dependency locks are identical.

The later, separately reviewed Russian-course correction replaces accidental
`导电` characters with a Russian word in `ru_m1_l5`. A production-loader regression
fails on the original embedded text and passes with the correction; it checks
every Russian lesson for foreign alphabetic characters. This content-only fix
requires a new package/manifest; the initial artifact below is retained as the
provenance of the native UI checks, not relabelled as the corrected artifact.

| Evidence | Result and boundary |
|---|---|
| `racoon-typper_1.3.1_amd64.deb` | SHA-256 `8d6ab7c61161d70260cee02f7a8e600979779fec246d0bfc6d32c942070ae35d`, 7,072,576 bytes; unsigned |
| Fresh install | apt installation into a new Ubuntu 24.04 runtime container without the build toolchain; WebKitGTK 2.52.6, glibc 2.39 |
| Restart | Two real production launches; persisted custom-text marker retained; integrity `ok`, zero FK violations |
| Upgrade | Official v1.3.1 package, checked against its published SHA256SUMS, initializes/opens a synthetic profile; candidate opens a copy twice, preserving a synthetic history row, custom text, settings and schema |
| Real input | Fcitx Pinyin, cloud addon disabled: `好世界` completed through native WebKitGTK with three correct characters, no errors, 100% accuracy and persisted history |
| Composition | Preedit is not counted; Escape cancellation leaves progress unchanged; two-character `世界` commit completed the clean test |
| Result export | PNG created under the isolated Pictures / `Raccoon Typer` directory; native UI reports save and image clipboard success. This is separate from the owner's earlier Wayland Ctrl+V confirmation |
| Abort/repeat | Repeat keeps custom/zh-tw source; abort returns to the saved text list |
| Whole-database restore through Settings | Foreign schema, missing trigger, changed trigger literal, corrupt and empty files refused; the logical database digest and history remain unchanged after each refusal. A valid backup is accepted, the UI reloads, the marker appears and history remains intact |
| Restore snapshot | The committed snapshot contains the original main database with WAL/SHM; its logical digest matches the pre-restore baseline exactly |
| Orca | Real AT-SPI/WebKitGTK speech-generation logs; this does not claim a complete auditory or assistive-technology usability review |
| Soak | 900.872 seconds, 90 initial scenarios plus 879 further sessions, 969 history rows, 15 languages; final integrity `ok`, zero FK violations; Rust backend sampled RSS 56,976–58,596 KiB |

The soak uses production input-handler bodies, the compiled Svelte store and
real Rust IPC/SQLite with synthetic event objects. It is not a 15-minute native
WebView memory test or proof of 15 installed keyboard/IME combinations.

## Gates and maintenance

Local checks passed: workspace Rust tests (788 passed, one ignored), frontend
tests (177 passed), formatting, strict workspace Clippy, frontend check/build,
IPC/version, licenses, advisory scan, vendor provenance (9), macro contracts (22),
optimized GLib regressions (3), opt-in process-crash tests (16 passed, one ignored)
and the separately invoked ignored extended crash campaign (one passed).
The additional Russian-course regression raises the workspace test count to
789; the corrected content receives a new full workspace run and package build.

The smoke regression was red before the profile-retention repair and all four
smoke tests pass after it. The old script's `new, new, pkill` trace was replaced
by `new, reused` plus a real persisted-row assertion and owned-process cleanup.

The three existing vendored backports follow the accountable roles, review
cadence, source/reference gates and removal triggers in
[vendor maintenance](../dependencies/vendor-maintenance.md). RustSec was fetched
on 2026-10-20; its revision remained `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`.

## Remaining release boundaries

- This original Linux-only pass does not verify Windows/macOS; their subsequent native CI is recorded in desktop evidence.
- AppImage/FUSE, RPM, a full desktop VM and all physical GPUs/portal combinations
  are not covered by this Debian-container result.
- One real Pinyin scenario does not validate every CJK IME or keyboard layout.
- Process-crash injection is not a physical power-loss or filesystem guarantee.
- Promotion, the next version number and signing with the owner's trusted key
  remain release-owner decisions. Historical assets/tags must remain immutable.
