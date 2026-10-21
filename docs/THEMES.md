# Built-in themes

Racoon Typper ships 50 local, compiled-in themes. Themes contain no executable
code, remote URLs, or runtime dependencies.

| Identifier | Display name | Category | Purpose |
|---|---|---|---|
| `abyss` | Abyss | Dark | Near-black navy with a distant starlight-blue accent |
| `amber_terminal` | Amber Terminal | Terminal | Modern low-glare amber terminal reinterpretation |
| `arctic_slate` | Arctic Slate | Dark | Cold neutral gray with quiet icy-blue accents |
| `burgundy` | Burgundy | Warm | Sophisticated subdued wine red with ivory text |
| `carbon` | Carbon | Dark | Near-monochrome dark theme, flatter than Graphite |
| `catppuccin_mocha` | Catppuccin Mocha | Pastel | Soothing pastel dark with warm lavender |
| `chestnut` | Chestnut | Warm | Warm brown surfaces with a soft golden accent |
| `coffee` | Coffee | Warm | Dark coffee, walnut, and cream with a soft copper accent |
| `coral` | Coral | Warm | Warm coral reef with soft orange-pink accents |
| `dark` | Dark | Dark | Minimal neutral dark theme |
| `dawn` | Dawn | Light | Soft warm daylight with a muted sunrise accent |
| `deep_sea` | Deep Sea | Dark | Very dark blue-green low-light palette |
| `dracula` | Dracula | Pop | Classic vampire dark with neon pink-cyan |
| `ember` | Ember | Warm | Charcoal with restrained ember-red and copper accents |
| `foamy` | Foamy | Light | Tidal seafoam green light palette |
| `glacier` | Glacier | Light | Cool pale ice-blue daylight with crisp contrast |
| `green_terminal` | Green Terminal | Terminal | Modern low-glare green terminal palette |
| `gruvbox_dark` | Gruvbox Dark | Retro | Retro groove dark with amber and olive |
| `lavender_dusk` | Lavender Dusk | Dark | Elegant desaturated purple for quiet evening sessions |
| `light` | Light | Light | Minimal neutral light theme |
| `lilac` | Lilac | Light | Soft purple-gray light with floral undertones |
| `matrix` | Matrix | Terminal | Pure digital rain green-on-black |
| `midnight_ink` | Midnight Ink | Dark | Deep navy-black surfaces for long nighttime sessions |
| `mint_frost` | Mint Frost | Light | Light mint-green with cool refreshing tones |
| `mist` | Mist | Light | Cool light gray with a restrained neutral-blue accent |
| `moonlight` | Moonlight | Dark | Soft blue-gray nighttime palette for low eye strain |
| `moss` | Moss | Nature | Relaxed olive and moss tones for focused practice |
| `nautilus` | Nautilus | Dark | Deep ocean shell with pearlescent blue-green |
| `nord` | Nord | Cool | Arctic north-bluish color palette |
| `obsidian` | Obsidian | Dark | Pure dark glass with electric cyan accent |
| `ocean` | Ocean | Dark | Dark muted ocean and teal without a cyberpunk glow |
| `paper` | Paper | Light | Soft reading-paper daylight without a pure-white canvas |
| `plum` | Plum | Dark | Warm dark plum with a restrained rose accent |
| `porcelain` | Porcelain | Light | Clean bright white with soft blue-gray accents |
| `racoon_forest` | Racoon Forest | Nature | Deep forest surfaces with muted natural highlights |
| `racoon_graphite` | Racoon Graphite | Racoon | Calm graphite surfaces with soft silver contrast |
| `racoon_high_contrast` | Racoon High Contrast | Accessibility | Maximum contrast and clearly separated typing states |
| `racoon_silver` | Racoon Silver | Racoon | Neutral daylight theme with warm silver surfaces |
| `racoon_warm` | Racoon Warm | Racoon | Warm charcoal surfaces with a restrained copper accent |
| `rose_pine` | Rosé Pine | Pastel | Soho vibes with muted rose and iris |
| `sage` | Sage | Nature | Quiet desaturated green daylight for professional practice |
| `sandstone` | Sandstone | Light | Subtle sandy daylight with warm, readable contrast |
| `serika` | Serika | Light | Warm beige with dark text and golden accent |
| `serika_dark` | Serika Dark | Dark | Dark variant of serika with golden accent |
| `serika_light` | Serika Light | Light | Bright daylight variant of serika |
| `solar_flare` | Solar Flare | Warm | Warm dark with golden-amber radiance |
| `steel_blue` | Steel Blue | Dark | Industrial steel and muted blue for metallic focus |
| `terra` | Terra | Warm | Earthen clay and ochre with warm stone surfaces |
| `toxic` | Toxic | Dark | Deep black with neon green terminal accent |
| `volcanic` | Volcanic | Warm | Black basalt with molten orange-red fissures |

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