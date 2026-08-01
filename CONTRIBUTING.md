# Contributing to Racoon Typper

Thank you for contributing. Racoon Typper is being developed as a long-term open-source desktop application, so changes should be small, testable, documented, and easy to review.

## Development setup

Install the platform prerequisites from [INSTALL.md](INSTALL.md), then run:

```bash
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
npm ci --prefix frontend
npm run check:version --prefix frontend
npm run license:check --prefix frontend
npm run tauri:dev --prefix frontend
```

The Tauri wrapper runs from `crates/app`; this is the canonical project topology. Do not substitute the obsolete `cargo tauri` command unless the repository explicitly adopts and verifies that toolchain.

## Architecture rules

Read [ARCHITECTURE.md](ARCHITECTURE.md) and [ROADMAP.md](ROADMAP.md) before making a cross-layer change.

- `domain` contains shared types and invariants, without Tauri, SQLite, filesystem, or UI dependencies.
- `core` contains typing/session behavior and must be testable without the desktop shell.
- `data` owns SQLite, migrations, SQL, repository mapping, and persistence-specific errors.
- `resources` owns embedded content loading and validation.
- `app` adapts Tauri commands and application state; command handlers must not grow new raw SQL or unrelated business policy.
- `frontend` owns presentation and interaction state; backend results remain authoritative for scores and persistence.
- New assets, resources, dependencies, or generated files require provenance and license metadata.
- Schema, IPC, release, or platform changes require a short decision note and rollback plan.

The current code contains known foundation debt. Do not disguise a refactor as a feature; follow the approved phase order in `ROADMAP.md`.

## Checks before opening a pull request

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check --prefix frontend
npm run build --prefix frontend
npm run check:version --prefix frontend
npm run license:check --prefix frontend
git diff --check
```

Use focused tests during development, then run the full relevant suite before requesting review. A passing unit test does not prove packaging, licensing, migration safety, or cross-platform support.

## Change scope and work in progress

- Keep pull requests focused and reviewable.
- Do not mix feature work with release, licensing, or database migrations without an explicit dependency and rollback plan.
- Never publish or tag from a dirty worktree.
- Existing worktree changes are recorded in [BASELINE.md](BASELINE.md); review them independently rather than assuming they are part of the release baseline.
- Do not add copied themes, icons, fonts, quotes, courses, or scripts without a source, license, attribution, and maintainer review.
- Do not commit secrets, local databases, generated frontend output, or private user content.

## Rust and frontend style

### Rust

- Run `cargo fmt --all` before commit.
- Treat clippy warnings as errors in CI.
- Prefer typed errors with stable context at boundaries.
- Add unit or integration tests for new behavior and failure paths.
- Avoid `unwrap`/`expect` on user-data, startup, migration, or IPC paths unless the invariant is locally proved and documented.

### Svelte/TypeScript

- Components live in `frontend/src/components/`.
- IPC wrappers and contracts live under `frontend/src/lib/`.
- Keep presentation state separate from server/persistence state.
- Prefer explicit types over `any`; cover loading, validation, and failure states.
- Follow the existing Svelte 5/Vite toolchain; do not add a framework or state library without a decision note.

## Commit and pull request conventions

Use focused conventional prefixes:

```text
feat: <description>
fix: <description>
refactor: <description>
docs: <description>
test: <description>
chore: <description>
```

Pull requests should explain:

1. the problem and intended behavior;
2. the files and boundaries changed;
3. tests and commands run;
4. migration, security, licensing, and release impact;
5. rollback or follow-up work;
6. screenshots or recordings for visible UI changes.

## Versioning and releases

The canonical product version is the root `Cargo.toml` workspace version. Update mirrored metadata only with the version check in mind:

```bash
npm run check:version --prefix frontend
```

Release preparation follows [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md). Do not describe a package format or platform as supported until it has passed clean-install and smoke validation.
