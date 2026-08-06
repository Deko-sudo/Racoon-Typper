# Release evidence and reproducibility

Each release candidate attaches these files to its draft GitHub release:

- `SHA256SUMS` for every uploaded artifact and release-evidence file;
- `racoon-typper.sbom.cdx.json`, the repository's CycloneDX dependency inventory;
- `PROVENANCE_ATTESTATION.md`, the project-content provenance record;
- `release-manifest.json`, recording the tag, exact source commit, and evidence filenames.

The release-candidate workflow verifies that the SBOM component version matches the selected tag before publication. Checksums cover the attached SBOM, provenance record, manifest, and binary artifacts.

## Reproducibility boundary

Artifacts are rebuilt from the immutable release tag on GitHub-hosted runners with repository-pinned workflow actions. This records source-to-artifact provenance but is not a byte-for-byte reproducible-build claim: native toolchains, operating-system package state, and Tauri bundling inputs are not fully pinned or independently rebuilt and compared. Do not describe a release as reproducible until an independent, hash-comparison verification process exists.

GitHub release attachment is not cryptographic signing or a SLSA-style attestation. Signing/attestation remains a separate release gate.
