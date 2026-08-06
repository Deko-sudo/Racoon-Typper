# Screenshot capture guide

This guide defines the screenshots referenced by the `## 📸 Showcase` section
of [README.md](../README.md). Capture them in this order, then uncomment the
matching `![](...)` / `<img>` lines in the README and delete the italic
placeholders.

All captures go in [`docs/screenshots/`](screenshots/).

## Setup

1. Run the dev window (Linux — `LD_LIBRARY_PATH` must be unset for the webview):

   ```bash
   env -u LD_LIBRARY_PATH npm run tauri:dev --prefix frontend
   ```

2. Use the default **1200×800** window. Do not resize — every screenshot shares
   that frame so the showcase grid stays uniform.
3. The default theme is `racoon_dark`. Switch themes in **Settings → Theme**
   for the theme-comparison row.
4. For realistic data, complete a few short tests first (so Dashboard,
   History, Analytics, and the heatmap have content to show).

## Capture list

| File | View | How to set up the frame |
|---|---|---|
| `hero-test.png` | Test (running) | Start a `time 30s` test in `racoon_dark`, type ~15 characters so the caret, colored chars, progress bar, and the next-key glow on the virtual keyboard are all visible. |
| `theme-dark.png` | Dashboard or Test (idle) | `racoon_dark`. Same frame as the other two theme shots for a clean side-by-side. |
| `theme-light.png` | Same frame | `racoon_light`. |
| `theme-hc.png` | Same frame | `racoon_high_contrast`. |
| `results-heatmap.png` | Result overlay | Finish a short test; capture the 4-stat grid (WPM / raw WPM / accuracy / raw accuracy) plus the compact `KeyboardHeatmap`. |
| `dashboard.png` | Dashboard | Streak card (with 🔥), stat cards, and the `ProgressChart` set to 30d. |
| `weakkeys.png` | Weak Keys | `WeakKeysPanel` with the heatmap-tinted virtual keyboard and the embedded training card. |
| `replay.png` | Replay modal | History → click a row's Replay button; capture the modal with the seek slider and frame details. |

## Format

- **PNG**, 1200×800, no OS window decorations if your capture tool allows it.
- Keep file sizes reasonable (< ~500 KB each); optimize with `optipng` or
  `pngcrush` if needed.
- Filenames must match the table above exactly — the README image paths already
  point here.

## After capturing

1. Drop the PNGs into `docs/screenshots/`.
2. In `README.md`, under `## 📸 Showcase`:
   - Uncomment the `<img ...>` / `![](docs/screenshots/...)` lines.
   - Remove the italic `_placeholder_` / `_pending_` lines.
3. Visually verify the theme table renders side-by-side and the feature grid
   captions still match each screenshot.
