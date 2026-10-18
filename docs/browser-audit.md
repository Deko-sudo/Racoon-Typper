# Local browser audit harness

This branch adds a developer-only way to exercise the Svelte UI in a browser
with the existing Rust command handlers, real embedded resources, session
engine, migrations and SQLite repositories. It is not a supported web edition.
No account, remote service, telemetry or new dependency is introduced.

## Start

Use the repository's documented Rust/Node toolchains and installed dependencies:

```bash
cargo build -p racoon-app --features browser-audit
RACOON_BROWSER_BINARY="$(pwd)/target/debug/racoon-app" npm run dev:browser-audit --prefix frontend
```

If `CARGO_TARGET_DIR` is set, point `RACOON_BROWSER_BINARY` to the binary in that
directory instead. Open <http://127.0.0.1:1440> in the browser. The audit banner
must be visible. The native development command remains unchanged.

The Vite process creates a new private `racoon-browser-audit-*` directory under
the OS temporary directory and prints its location locally. The Rust backend
uses ONLY that directory: it does not resolve Tauri user paths or copy legacy
profiles. Browser reloads preserve that process's database and settings; each
server start or configuration reload creates a fresh profile. Multiple tabs share the same test
session. Stop the server before removing its directory; it contains test data,
uploaded files and any restore snapshots. Nothing is automatically pruned.

## Boundary and checks

- Only the explicit `browser-audit` Vite mode enables the HTTP adapter/aliases.
  A normal frontend build uses Tauri IPC. Building the audit mode is rejected.
- The Rust entry point additionally requires the `browser-audit` feature,
  a debug build and the explicit `--browser-audit` argument.
- The dev server binds only to `127.0.0.1:1440`; API requests require that Host,
  matching Origin, and a random token created for that server process. Do not
  expose this development server through a public tunnel.
- Rust command handlers are registered through the same function as native
  builds. Scores, persistence and application validation are not mocked.
- Tauri's **test runtime** replaces its desktop shell. Its command invocation
  does not establish production capability/CSP enforcement. File selection is
  a browser picker; clipboard access follows browser permissions.
- Uploads are limited to 64 MiB, stored under random names with private file
  permissions. Whole-database restore accepts paths only inside the audit
  upload directory. Other filesystem/plugin access is not provided.
- This does not verify WebKitGTK/native rendering, system keyboard/IME events,
  desktop dialogs, packaging, OS locks across native instances or power loss.
  Chromium browser-input checks are evidence for that browser only.

Run the transport contracts against the freshly built backend:

```bash
RACOON_BROWSER_BINARY="$(pwd)/target/debug/racoon-app" node scripts/browser-audit.test.mjs
cargo test -p racoon-app --features browser-audit browser_audit
npm run test:ui --prefix frontend
npm run check --prefix frontend
npm run build --prefix frontend
```

The transport contract checks isolated startup/migrations, Russian input,
completion/history, clear-statistics, settings/themes and forbidden filesystem
requests. The normal Rust workspace checks still apply. Browser automation
must separately exercise the actual UI rather than calling the adapter as a
substitute for clicks and typing.

Rollback: stop the audit server and return to the original checkout/branch.
No migration, production profile or release capability is changed.
