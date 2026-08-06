# Flatpak source build

`com.racoon.typper.json` builds `racoon-app` from the checked-out repository source, rather than installing a prebuilt host binary. Release-candidate workflows check out an immutable version tag before building; that tag is the source revision for the local `dir` source.

The runtime uses the pinned `org.gnome.Platform`/SDK version declared in the manifest. Runtime permissions are deliberately restricted to Wayland/X11, IPC, and GPU rendering. The application uses Flatpak-managed storage rather than direct host `xdg-data` or `xdg-config` filesystem grants, and it has no runtime network permission.

`npm ci` and the Rust build execute during the build phase. Build-phase network access remains required until npm and Cargo source generators/vendor inputs are checked into the Flatpak source manifest. This is not a runtime permission and must not be described as fully offline/reproducible.

Run the policy gate with:

```sh
node scripts/validate-flatpak-manifest.mjs
```

A runtime Flatpak smoke needs `flatpak-builder`, the matching GNOME runtime/SDK, and their Rust/Node build extensions. This host has Flatpak but not `flatpak-builder`; the release workflow therefore keeps the policy gate deterministic and does not claim a local runtime smoke.
