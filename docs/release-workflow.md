# Release workflow

Releases are deliberately split into verification, a draft release candidate, and an approved promotion. A pushed tag does not publish an artifact.

## 1. Verify the candidate

The normal `CI` workflow runs on pull requests, protected branches, and tags with read-only repository permissions. It verifies Rust, frontend, licensing, and build artifacts but cannot create a release.

## 2. Build a draft release candidate

A maintainer runs **Release candidate** manually and supplies an existing immutable tag such as `v1.2.3`.

The workflow checks that the tag exists and matches the project version, rebuilds Linux and Windows artifacts from that tag, generates `SHA256SUMS`, and creates a draft prerelease. Verify the checksums before installation. A draft release is not public and is not a promotion decision.

## 3. Promote after approval

A maintainer runs **Promote release** with the same tag only after release evidence is reviewed. Its `release-promotion` environment must be protected in GitHub repository settings with the required maintainer reviewers. This is the approval boundary: the workflow makes the existing draft non-draft and non-prerelease.

## Security properties

- Workflow defaults are `contents: read`; only the draft publisher and protected promotion job receive `contents: write`.
- Workflows do not print tokens or repository secrets.
- Candidate and promotion both validate the exact tag name before mutation.
- GitHub environment protection is repository configuration, not a source-controlled guarantee; configure `release-promotion` before using promotion.

## Scope limits

This workflow produces release artifacts and checksums. It also blocks draft publication on the isolated Debian-package install/launch/persistence/restart smoke. SBOM/provenance attachments are a separate release task and must be complete before promotion. The runtime smoke intentionally does not claim browser-download export coverage; export remains covered by the application integration suite until a stable desktop automation contract exists.
