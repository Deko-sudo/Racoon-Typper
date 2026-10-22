# Desktop stability candidate evidence

Reviewed 2026-10-22. Unreleased, unsigned/unnotarized candidate; version metadata
remains 1.3.1. Historical release assets/tags are unchanged. Promotion, a new
version, trusted signing keys and review/merge require the release owner.

## Integration and source

`release/desktop-stability` descends linearly from Linux candidate `90124ec` and
Windows/macOS follow-up `a4eca3696d75e414d62278b4e8f57c4c4e4770f8` (tree
`d74f8218e12a7d5298e557ece51f4a72d16b8364`). No extra merge or runtime feature is
needed. PR #14 targets master; PR #16 targets the Linux integration. Neither is
merged by this preparation. Main/master refs are not moved.

Candidate evidence must always name the exact tested source/tree and package
checksum. Documentation/QA follow-ups must retain their own CI status; these
baseline results cannot be silently attributed to different binary bytes.

## Verified baseline

| Source / environment | Result | Boundary |
|---|---|---|
| `a4eca369`, Windows Server 2022 x64 | [Native CI 39701329233](https://github.com/Deko-sudo/Racoon-Typper/actions/runs/39701329233): strict Clippy, 784 Rust tests, 177 frontend tests, NSIS install, two visible-window/SQLite launches | No physical Windows 10/11, visual/DPI or image-paste claim |
| `a4eca369`, macOS 15.7.9 Intel and ARM | Same run: 790 Rust tests on each architecture, 177 frontend tests, DMG/app, two visible-window/SQLite launches | Not macOS 13.3, Gatekeeper downloaded-app acceptance or hardware testing |
| Combined runtime code, Ubuntu 24.04 x64 | deb and AppImage built; fresh runtime containers, two launches and retained custom-text marker, integrity ok / FK 0 | AppImage extract-and-run, not kernel FUSE; no full desktop/GPU claim |
| Combined runtime code, Fedora 43 x64 | Ubuntu-built RPM installed with DNF; two unprivileged launches retaining marker, integrity ok / FK 0 | Container/userspace, not all RPM distributions |

The local Linux build initially lacked FUSE access and then the build-only
`file` utility. Both failed attempts were retained; adding container-only tools
and using extraction completed packaging. The host OS/profile was not modified.
Linux package hashes and full runtime logs are generated with each candidate;
do not substitute the older Debian-only artifact manifest.

The common Rust/frontend source preserves offline operation, V001–V009,
architectural boundaries, existing dependency locks and the profile identifier.
The additional work here is QA/packaging/documentation; native probe code is not
embedded into the production frontend or IPC.

## Gates and remaining acceptance

Canonical checks include fmt, strict Clippy, workspace tests, frontend check/test/
build, platform/IPC/version contracts, license/source/advisory gates and smoke
harness tests. Linux candidate CI builds all three package formats, verifies
Debian/AppImage restart retention and installs RPM in a pinned Fedora container.

The optional native UI investigations are separate outcomes: Windows UIA/SendInput
or macOS AX/CGEvent controls, OS-injected EN text, repeat/abort, PNG file and OS
image clipboard. A missing
controller/window/control, blocked input, missing results or failed PNG delivery
is not a passed UI check. No OS permissions or protections are disabled. Its
logs/images and experiment outcome accompany the installer artifacts. macOS
controller trust is checked without requesting Accessibility permission; a
trusted controller can attempt the AX probe without changing OS permissions.

Physical Windows/macOS RU/EN/layout switching, fast caret movement and resize/DPI,
PNG pixels/image paste and OS launch/security prompts remain manual. Signing,
notarization, every GPU/portal/IME, full auditory accessibility, kernel FUSE and
physical power loss are outside the verified boundary. Existing Linux native
Pinyin/restore/Orca/soak evidence remains under its original source/checksum.

See [support matrix](../../SUPPORT_MATRIX.md), [release checklist](../../RELEASE_CHECKLIST.md),
[draft notes](RELEASE_NOTES.next.md) and [ADR 0009](../adr/0009-desktop-candidate-parity.md).
