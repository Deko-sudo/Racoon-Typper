# ADR 0009: share production fixes across desktop candidates

Status: accepted for candidate configuration; Windows/macOS support is unverified
until the exact artifacts pass their native and manual gates.

## Problem and decision

Linux fixes live in shared Rust/Svelte code, not a Linux-only UI. Rebuilding an
old tag does not include them. The previous bundle target list had NSIS and
Linux formats but no macOS application/DMG, and CI never compiled the current
candidate on macOS. Production `esnext` also left WKWebView syntax requirements
unspecified.

Use Tauri's automatic platform overrides only for packaging. Keep the same
window, security policy, identifier, embedded resources, frontend and IPC on all
three OSes. Keep NSIS's existing per-machine installation/upgrade scope, but
embed WebView2's offline installer rather than downloading it on a user's first
installation. The build still downloads tooling/runtime from its existing
vendors. WebView2 remains the existing Windows OS runtime prerequisite, not an
Apache-licensed crate or a new app network feature. Its vendor terms still apply.
The installer grows substantially; byte-for-byte reproducibility is not claimed.

Declare frontend targets Safari 16.4 and Edge 111. macOS's packaging floor is
13.3, where the corresponding system WKWebView is available. This is a technical
floor, not a claim that macOS 13.3 has been tested. JS syntax lowering does not
polyfill missing Web APIs. Explicitly hide the editable textarea's native caret
as well as its text; the visual caret remains rendered with the practice text.

PR/manual candidate CI builds the exact head commit for Windows x86_64, macOS
Intel and Apple Silicon. It runs the existing shared UI/PNG and Rust tests,
builds NSIS/DMG, verifies a real visible native window and durable synthetic text
through restart, and records source/tree/artifact hashes. No tag, release,
updater, accounts, server or telemetry is added. The separate macOS candidate
does not become an automatic prerequisite for a Linux-only release promotion.

## Safety, limits and verification

Native smoke may seed only a nonexistent profile on a fresh GitHub-hosted
runner. APPDATA/XDG overrides do not isolate these OS known folders. Refuse any
existing profile before changing it; stop only owned processes. A process,
window title or database alone cannot prove that the UI rendered or input/PNG
clipboard delivery worked. The smoke explicitly excludes those claims.

An optional, separately reported Windows UIA/SendInput investigation uses only a
fresh hosted runner, an EN custom-text fixture and OS input events. It must verify
two exact persisted results independently; its own failure cannot produce a UI
pass. PNG file and OS image clipboard checks are narrower than hardware layouts,
pixel/DPI review or pasting into another application. A corresponding macOS
AX/CGEvent probe uses the same independent result checks when its controller is
already trusted. Accessibility trust is queried without prompting, requesting
permission or changing OS protections; untrusted access remains a failed probe.
Required package/startup gates remain strict if the optional experiment is blocked.

Keep Windows/macOS experimental until hardware checks cover RU/EN restart,
abort, moving/resizing/high DPI, PNG contents, Pictures-folder delivery, image
paste and OS launch/security prompts. macOS DMGs are not Developer-ID signed or
notarized; CI does not test Gatekeeper's downloaded-app path. Do not disable
Gatekeeper, SmartScreen or the WebView sandbox to make a smoke pass.

Rollback: revert this focused commit, leaving the Linux stability baseline and
profiles intact. No migrations, dependency locks or data formats change.

## Windows findings from native CI

Strict Clippy on Windows exposed a Unix-only import and two directory builders
whose mutable access previously existed only under `cfg(unix)`. Scope the
import and explicitly retain non-recursive staging-directory creation; do not
waive the lint gate. File sync needs a writable handle on Windows because
`FlushFileBuffers` requires `GENERIC_WRITE`; keep read-only handles for Unix
directory sync. The file-sync regression checks byte preservation and rejects
missing files without creating them. This does not add a Windows directory-sync
or physical power-loss guarantee.

The next Windows run compiled with strict Clippy and found nine test failures:
fixtures used Rust test-thread names containing `::` as filenames. Replace those
test-only suffixes with debug thread IDs, valid on Windows too. Keep the tests
enabled and preserve production settings-path validation.

References: [Tauri configuration](https://v2.tauri.app/reference/config/),
[WebView2 installation](https://v2.tauri.app/distribute/windows-installer/),
[WebView versions](https://v2.tauri.app/reference/webview-versions/).
