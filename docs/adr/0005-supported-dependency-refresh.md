# ADR 0005: Refresh vulnerable toolchain dependencies together

Status: implemented locally; validation evidence accompanies the change.

## Context

The locked Vite 5 / Svelte plugin 4 / inspector 3 / esbuild 0.21 group produced
npm advisory findings. Tauri's transitive `plist` 1.9 used quick-xml 0.39,
affected by RUSTSEC-2026-0194 and RUSTSEC-2026-0195. Updating quick-xml alone
cannot satisfy plist 1.9's 0.39 requirement.

## Decision

- Use the supported Vite 7.3 line with Svelte plugin 6.2 and inspector 5.
  Their peer ranges agree; keep Rollup and the locked Svelte version.
  Vite 8's Rolldown migration is unnecessary for this focused correction.
- Keep explicit compatible minor ranges for the two direct toolchain packages
  and commit the generated lockfile. Do not use force/legacy-peer-deps or
  dependency overrides to bypass incompatible requirements.
- Require Node 22.12+ on the 22.x line or Node 24+. CI's Node 22 selection
  resolves within the supported line; the developer .nvmrc remains Node 26.
- Update the locked parent plist to 1.10.1, which uses quick-xml 0.42, rather
  than introducing a replacement parser or a maintained local fork.
- Add a full npm audit gate at moderate severity to frontend CI. This is a
  development/CI check, not a network feature of the shipped application.
- Regenerate dependency inventories, notices and SBOM. Keep the license
  allow-list, existing exceptions and application offline boundary unchanged.

## Validation and rollback

Run clean npm installation, full npm audit, UI tests, Svelte check and build;
verify the default bundle excludes the browser audit adapter. Run the Rust
workspace tests, fmt/clippy, license/advisory checks, and native Tauri build.
Smoke-test the local browser harness with real commands and a separate profile.

Keep remaining Rust unmaintained advisories visible; an XML upgrade does not
resolve the GTK chain or every transitive maintenance risk. Passing known
advisories does not establish that all dependency behavior is safe.

Rollback is a revert of the dependency/configuration/documentation commit,
followed by npm ci. No application migration or user-data transformation occurs.
