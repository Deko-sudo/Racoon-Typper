# Release Baseline Record

**Phase:** 0 — Release Baseline  
**Baseline reference:** `f88d579 fix replay and typing session persistence`  
**Review date:** 2026-07-12  
**Status:** Working-tree baseline; not a production release

## Baseline rule

The verified historical application reference is commit `f88d579`. The working tree was already dirty when Phase 0 began. Existing modifications and untracked files remain the user's work and are not silently folded into the release baseline.

The repository must not create a release from a dirty worktree. A future release candidate must be built from a reviewed commit containing the intended Phase 0 changes and separately reviewed feature work.

## Pre-existing work in progress

The following categories were present before Phase 0 implementation and remain outside the verified baseline until separately reviewed:

- modifications to Tauri commands, application startup, data models/settings/repository tests, and frontend components/API/types;
- changes to `.github/workflows/ci.yml`;
- an untracked imported theme catalog, copied license/notice material, and its importer. These incompatible resources are removed in the current Phase 1 worktree and must not be restored to a release build.

The imported theme catalog and related files were explicitly excluded from release approval and are removed by Phase 1. No copied theme catalog is permitted in the distributed project.

## Phase 0 changes

Phase 0 adds only baseline/release-topology documentation, version verification, a deterministic Tauri command wrapper, and corrections to stale packaging-helper commands/path defects. It does not add product functionality, change scoring, alter the database schema, or clear/relicense any asset.

## Verification interpretation

Passing workspace/frontend checks on the dirty worktree demonstrates that the current files compile and test together. It does not establish that the pre-existing work-in-progress is correct, licensed, or release-ready. Release claims require a clean reviewed commit and the later licensing, foundation, security, and release gates in `ROADMAP.md`.
