# Built-in themes

Racoon Typper ships 25 local, compiled-in themes. Themes contain no executable
code, remote URLs, or runtime dependencies.

| Identifier | Display name | Category | Purpose |
|---|---|---|---|
| `racoon_graphite` | Racoon Graphite | Racoon | Default dark graphite and soft-silver theme |
| `racoon_silver` | Racoon Silver | Racoon | Neutral daylight theme with warm silver surfaces |
| `racoon_warm` | Racoon Warm | Racoon | Warm charcoal theme with a restrained copper accent |
| `racoon_high_contrast` | Racoon High Contrast | Accessibility | Maximum state separation |
| `midnight_ink` | Midnight Ink | Dark | Deep navy-black nighttime sessions |
| `arctic_slate` | Arctic Slate | Dark | Quiet cold gray with icy-blue accents |
| `lavender_dusk` | Lavender Dusk | Dark | Desaturated purple evening palette |
| `plum` | Plum | Dark | Warm subdued dark plum |
| `ocean` | Ocean | Dark | Muted ocean and teal |
| `deep_sea` | Deep Sea | Dark | Very dark blue-green low-light palette |
| `steel_blue` | Steel Blue | Dark | Industrial steel and muted blue |
| `carbon` | Carbon | Dark | Flat near-monochrome dark palette |
| `moonlight` | Moonlight | Dark | Soft blue-gray nighttime palette |
| `racoon_forest` | Racoon Forest | Nature | Deep forest green |
| `moss` | Moss | Nature | Muted olive and moss |
| `coffee` | Coffee | Warm | Dark coffee, walnut, and cream |
| `ember` | Ember | Warm | Charcoal with restrained ember accents |
| `burgundy` | Burgundy | Warm | Subdued dark wine red |
| `paper` | Paper | Light | Soft reading-paper daylight |
| `sandstone` | Sandstone | Light | Warm sandy daylight |
| `mist` | Mist | Light | Cool light gray |
| `dawn` | Dawn | Light | Warm daylight with a muted sunrise accent |
| `sage` | Sage | Nature | Desaturated green daylight |
| `amber_terminal` | Amber Terminal | Terminal | Modern low-glare amber terminal |
| `green_terminal` | Green Terminal | Terminal | Modern low-glare green terminal |

## Runtime architecture

Each built-in theme lives in `resources/themes/<identifier>/` and contains:

- `theme.json`: stable identifier, display name, description, version, author,
  license, and representative preview palette;
- `theme.css`: one `:root` rule containing the semantic token contract and
  compatibility aliases for older components.

The desktop adapter in `crates/app/src/commands/preferences.rs` is the built-in
catalog and embeds each CSS file with `include_str!`. `get_themes` exposes the
catalog to the existing Settings selector; `get_theme_css` returns CSS only for
an allow-listed identifier. There is no filesystem theme discovery, arbitrary
CSS import, remote download, or system-theme mode.

The frontend injects the selected CSS into `#active-theme` and applies its
custom properties to the document root immediately. The same selector is
available as a native select and as keyboard-accessible preview cards; no
second selector or persistence path is used.

The selected identifier remains in the existing TOML settings file through the
existing `set_setting("theme", ...)` path. `racoon_graphite` is the fallback and
new-install default. Existing `racoon_dark` and `racoon_light` values are read
as compatibility aliases and normalized to `racoon_graphite` and
`racoon_silver` respectively. Unknown or malformed identifiers still fall back
safely and are never passed to the CSS loader.

## Semantic token contract

Component styles should consume semantic tokens, not palette names. Every
built-in theme defines the following matrix:

| Category | Required tokens |
|---|---|
| Surfaces | `--color-app-background`, `--color-surface-primary`, `--color-surface-raised`, `--color-surface-hover`, `--color-surface-active` |
| Text | `--color-text-primary`, `--color-text-secondary`, `--color-text-muted`, `--color-text-disabled` |
| Borders and interaction | `--color-border`, `--color-border-strong`, `--color-accent`, `--color-accent-hover`, `--color-accent-text`, `--color-focus-ring`, `--color-selection`, `--color-caret` |
| Active interaction | `--color-accent-active`, `--color-information` |
| Typing | `--color-typing-pending`, `--color-typing-current`, `--color-typing-correct`, `--color-typing-incorrect`, `--color-typing-corrected` |
| Keyboard | `--color-key-background`, `--color-key-border`, `--color-key-active`, `--color-key-pressed` |
| Status | `--color-success`, `--color-warning`, `--color-error`, `--color-information` |
| Charts | `--color-chart-primary`, `--color-chart-secondary`, `--color-chart-positive`, `--color-chart-negative`, `--color-chart-grid`, `--color-chart-axis`, `--color-chart-label`, `--color-chart-tooltip-background`, `--color-chart-tooltip-border`, `--color-chart-selected` |
| Progress | `--color-progress-track`, `--color-progress-fill` |
| Overlays and effects | `--color-overlay`, `--color-modal-surface`, `--color-tooltip-surface`, `--color-scrollbar`, `--color-scrollbar-hover`, `--shadow-surface`, `--shadow-elevated` |

The legacy aliases (`--bg`, `--bg-sub`, `--main`, `--sub`, `--text`,
`--error`, and `--caret`) remain temporarily available so existing components
continue to theme consistently while component-facing CSS moves to semantic
names.

## Adding a built-in theme

1. Create `resources/themes/<stable_identifier>/theme.json` and `theme.css`; the
   catalog is intentionally explicit and must contain exactly one entry per
   bundled theme.
2. Define every token in the matrix above and the compatibility aliases.
3. Add one allow-listed catalog entry and one `include_str!` branch in
   `crates/app/src/commands/preferences.rs`.
4. Add the identifier to theme validation in
   `crates/data/src/repository/settings.rs`; do not add a second persistence
   mechanism.
5. Add the description used by the existing preview cards in
   `frontend/src/components/SettingsView.svelte`.
6. Run `node --test scripts/theme-pack.test.mjs`, the frontend checks, workspace
   Rust tests, and the license-policy generator/check.
7. Verify typing states, focus, disabled controls, charts, dialogs, and settings
   at the supported window sizes before shipping.

New bundled themes must be original or have licensing compatible with the
Apache-2.0 project and must pass the repository license policy. Do not copy
theme source, palettes, assets, or metadata from another project.

Do not add remote assets or executable content to a theme. A custom or external
theme loader would change the trust boundary and requires a separate design and
security review.