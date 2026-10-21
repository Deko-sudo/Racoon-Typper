# Flatpak source build

`com.racoon.typper.json` builds `racoon-app` from the checked-out repository source, rather than installing a prebuilt host binary. Release-candidate workflows check out an immutable version tag before building; that tag is the source revision for the local `dir` source.

The runtime is pinned to `org.gnome.Platform`/`org.gnome.Sdk` **50** (the current stable GNOME runtime). The GNOME SDK contains neither cargo nor Node, so the manifest declares the `org.freedesktop.Sdk.Extension.rust-stable` and `org.freedesktop.Sdk.Extension.node22` SDK extensions and appends their `bin` directories to the build path. The rust-stable extension for the 50 runtime ships Rust 1.97.1, which satisfies the repository's pinned `rust-toolchain.toml` (1.96.0); node22 satisfies the frontend `engines` requirement (Node >=22).

Runtime permissions are deliberately restricted to Wayland/X11, IPC, and GPU rendering. The application uses Flatpak-managed storage rather than direct host `xdg-data` or `xdg-config` filesystem grants, and it has no runtime network permission.

`npm ci` and the Rust build execute during the build phase. Build-phase network access remains required until npm and Cargo source generators/vendor inputs are checked into the Flatpak source manifest. This is not a runtime permission and must not be described as fully offline/reproducible.

Run the policy gate with:

```sh
node scripts/validate-flatpak-manifest.mjs
```

A runtime Flatpak smoke needs `flatpak-builder`, the matching GNOME runtime/SDK, and their Rust/Node build extensions. This host has Flatpak but not `flatpak-builder`; the release workflow therefore keeps the policy gate deterministic and does not claim a local runtime smoke. A smoke build remains an owner/CI action (installing `org.flatpak.Builder` plus the ~2 GB GNOME 50 runtime).
