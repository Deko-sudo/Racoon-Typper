# PKGBUILD release-source blocker

The current package version is `1.1.0`, but its local tag is not advertised by `origin` and the configured GitHub release archive returns HTTP 404. A source URL plus a SHA-256 must identify the same immutable public release artifact; a locally generated archive cannot safely substitute for that distribution input.

Until the `v1.1.0` tag/archive is published at the canonical upstream, `PKGBUILD` intentionally remains unmodified rather than replacing `SKIP` with an unverifiable checksum. Once published, download the source archive from the canonical upstream, record its SHA-256 in `PKGBUILD`, and run `makepkg --verifysource` followed by a clean package build.
