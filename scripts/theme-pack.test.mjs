import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { test } from 'node:test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const themesRoot = path.join(repositoryRoot, 'resources', 'themes');
const preferencesSourcePath = path.join(repositoryRoot, 'crates', 'app', 'src', 'commands', 'preferences.rs');
const frontendComponentsRoot = path.join(repositoryRoot, 'frontend', 'src', 'components');

const legacyAliases = new Map([
  ['--bg', 'var(--color-app-background)'],
  ['--bg-sub', 'var(--color-surface-primary)'],
  ['--main', 'var(--color-accent)'],
  ['--sub', 'var(--color-text-secondary)'],
  ['--text', 'var(--color-text-primary)'],
  ['--error', 'var(--color-error)'],
  ['--caret', 'var(--color-caret)'],
]);

const expectedThemes = new Map([
  ['abyss', { displayName: 'Abyss', description: 'Near-black navy with a distant starlight-blue accent.', isDark: true, category: 'Dark' }],
  ['amber_terminal', { displayName: 'Amber Terminal', description: 'Modern low-glare amber terminal reinterpretation.', isDark: true, category: 'Terminal' }],
  ['arctic_slate', { displayName: 'Arctic Slate', description: 'Cold neutral gray with quiet icy-blue accents.', isDark: true, category: 'Dark' }],
  ['burgundy', { displayName: 'Burgundy', description: 'Sophisticated subdued wine red with ivory text.', isDark: true, category: 'Warm' }],
  ['carbon', { displayName: 'Carbon', description: 'Near-monochrome dark theme, flatter than Graphite.', isDark: true, category: 'Dark' }],
  ['catppuccin_mocha', { displayName: 'Catppuccin Mocha', description: 'Soothing pastel dark with warm lavender.', isDark: true, category: 'Pastel' }],
  ['chestnut', { displayName: 'Chestnut', description: 'Warm brown surfaces with a soft golden accent.', isDark: true, category: 'Warm' }],
  ['coffee', { displayName: 'Coffee', description: 'Dark coffee, walnut, and cream with a soft copper accent.', isDark: true, category: 'Warm' }],
  ['coral', { displayName: 'Coral', description: 'Warm coral reef with soft orange-pink accents.', isDark: false, category: 'Warm' }],
  ['dark', { displayName: 'Dark', description: 'Minimal neutral dark theme.', isDark: true, category: 'Dark' }],
  ['dawn', { displayName: 'Dawn', description: 'Soft warm daylight with a muted sunrise accent.', isDark: false, category: 'Light' }],
  ['deep_sea', { displayName: 'Deep Sea', description: 'Very dark blue-green low-light palette.', isDark: true, category: 'Dark' }],
  ['dracula', { displayName: 'Dracula', description: 'Classic vampire dark with neon pink-cyan.', isDark: true, category: 'Pop' }],
  ['ember', { displayName: 'Ember', description: 'Charcoal with restrained ember-red and copper accents.', isDark: true, category: 'Warm' }],
  ['foamy', { displayName: 'Foamy', description: 'Tidal seafoam green light palette.', isDark: false, category: 'Light' }],
  ['glacier', { displayName: 'Glacier', description: 'Cool pale ice-blue daylight with crisp contrast.', isDark: false, category: 'Light' }],
  ['green_terminal', { displayName: 'Green Terminal', description: 'Modern low-glare green terminal palette.', isDark: true, category: 'Terminal' }],
  ['gruvbox_dark', { displayName: 'Gruvbox Dark', description: 'Retro groove dark with amber and olive.', isDark: true, category: 'Retro' }],
  ['lavender_dusk', { displayName: 'Lavender Dusk', description: 'Elegant desaturated purple for quiet evening sessions.', isDark: true, category: 'Dark' }],
  ['light', { displayName: 'Light', description: 'Minimal neutral light theme.', isDark: false, category: 'Light' }],
  ['lilac', { displayName: 'Lilac', description: 'Soft purple-gray light with floral undertones.', isDark: false, category: 'Light' }],
  ['matrix', { displayName: 'Matrix', description: 'Pure digital rain green-on-black.', isDark: true, category: 'Terminal' }],
  ['midnight_ink', { displayName: 'Midnight Ink', description: 'Deep navy-black surfaces for long nighttime sessions.', isDark: true, category: 'Dark' }],
  ['mint_frost', { displayName: 'Mint Frost', description: 'Light mint-green with cool refreshing tones.', isDark: false, category: 'Light' }],
  ['mist', { displayName: 'Mist', description: 'Cool light gray with a restrained neutral-blue accent.', isDark: false, category: 'Light' }],
  ['moonlight', { displayName: 'Moonlight', description: 'Soft blue-gray nighttime palette for low eye strain.', isDark: true, category: 'Dark' }],
  ['moss', { displayName: 'Moss', description: 'Relaxed olive and moss tones for focused practice.', isDark: true, category: 'Nature' }],
  ['nautilus', { displayName: 'Nautilus', description: 'Deep ocean shell with pearlescent blue-green.', isDark: true, category: 'Dark' }],
  ['nord', { displayName: 'Nord', description: 'Arctic north-bluish color palette.', isDark: true, category: 'Cool' }],
  ['obsidian', { displayName: 'Obsidian', description: 'Pure dark glass with electric cyan accent.', isDark: true, category: 'Dark' }],
  ['ocean', { displayName: 'Ocean', description: 'Dark muted ocean and teal without a cyberpunk glow.', isDark: true, category: 'Dark' }],
  ['paper', { displayName: 'Paper', description: 'Soft reading-paper daylight without a pure-white canvas.', isDark: false, category: 'Light' }],
  ['plum', { displayName: 'Plum', description: 'Warm dark plum with a restrained rose accent.', isDark: true, category: 'Dark' }],
  ['porcelain', { displayName: 'Porcelain', description: 'Clean bright white with soft blue-gray accents.', isDark: false, category: 'Light' }],
  ['racoon_forest', { displayName: 'Racoon Forest', description: 'Deep forest surfaces with muted natural highlights.', isDark: true, category: 'Nature' }],
  ['racoon_graphite', { displayName: 'Racoon Graphite', description: 'Calm graphite surfaces with soft silver contrast.', isDark: true, category: 'Racoon' }],
  ['racoon_high_contrast', { displayName: 'Racoon High Contrast', description: 'Maximum contrast and clearly separated typing states.', isDark: true, category: 'Accessibility' }],
  ['racoon_silver', { displayName: 'Racoon Silver', description: 'Neutral daylight theme with warm silver surfaces.', isDark: false, category: 'Racoon' }],
  ['racoon_warm', { displayName: 'Racoon Warm', description: 'Warm charcoal surfaces with a restrained copper accent.', isDark: true, category: 'Racoon' }],
  ['rose_pine', { displayName: 'Rosé Pine', description: 'Soho vibes with muted rose and iris.', isDark: true, category: 'Pastel' }],
  ['sage', { displayName: 'Sage', description: 'Quiet desaturated green daylight for professional practice.', isDark: false, category: 'Nature' }],
  ['sandstone', { displayName: 'Sandstone', description: 'Subtle sandy daylight with warm, readable contrast.', isDark: false, category: 'Light' }],
  ['serika', { displayName: 'Serika', description: 'Warm beige with dark text and golden accent.', isDark: false, category: 'Light' }],
  ['serika_dark', { displayName: 'Serika Dark', description: 'Dark variant of serika with golden accent.', isDark: true, category: 'Dark' }],
  ['serika_light', { displayName: 'Serika Light', description: 'Bright daylight variant of serika.', isDark: false, category: 'Light' }],
  ['solar_flare', { displayName: 'Solar Flare', description: 'Warm dark with golden-amber radiance.', isDark: true, category: 'Warm' }],
  ['steel_blue', { displayName: 'Steel Blue', description: 'Industrial steel and muted blue for metallic focus.', isDark: true, category: 'Dark' }],
  ['terra', { displayName: 'Terra', description: 'Earthen clay and ochre with warm stone surfaces.', isDark: true, category: 'Warm' }],
  ['toxic', { displayName: 'Toxic', description: 'Deep black with neon green terminal accent.', isDark: true, category: 'Dark' }],
  ['volcanic', { displayName: 'Volcanic', description: 'Black basalt with molten orange-red fissures.', isDark: true, category: 'Warm' }],
]);

const requiredTokens = [
  '--color-app-background',
  '--color-surface-primary',
  '--color-surface-raised',
  '--color-surface-hover',
  '--color-surface-active',
  '--color-text-primary',
  '--color-text-secondary',
  '--color-text-muted',
  '--color-text-disabled',
  '--color-border',
  '--color-border-strong',
  '--color-accent',
  '--color-accent-hover',
  '--color-accent-active',
  '--color-accent-text',
  '--color-focus-ring',
  '--color-selection',
  '--color-caret',
  '--color-typing-pending',
  '--color-typing-current',
  '--color-typing-correct',
  '--color-typing-incorrect',
  '--color-typing-corrected',
  '--color-key-background',
  '--color-key-border',
  '--color-key-active',
  '--color-key-pressed',
  '--color-success',
  '--color-warning',
  '--color-error',
  '--color-information',
  '--color-chart-primary',
  '--color-chart-secondary',
  '--color-chart-positive',
  '--color-chart-negative',
  '--color-chart-grid',
  '--color-chart-axis',
  '--color-chart-label',
  '--color-chart-tooltip-background',
  '--color-chart-tooltip-border',
  '--color-chart-selected',
  '--color-progress-track',
  '--color-progress-fill',
  '--color-overlay',
  '--color-modal-surface',
  '--color-tooltip-surface',
  '--color-scrollbar',
  '--color-scrollbar-hover',
  '--shadow-surface',
  '--shadow-elevated',
];

function parseTokens(css) {
  return new Map(
    [...css.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)]
      .map((match) => [match[1], match[2].trim()]),
  );
}

async function loadThemeDirectories() {
  const entries = await readdir(themesRoot, { withFileTypes: true });
  const directories = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const files = await readdir(path.join(themesRoot, entry.name));
    if (files.includes('theme.json') || files.includes('theme.css')) directories.push(entry.name);
  }
  return directories.sort();
}

function parseRegistryIdentifiers(source) {
  const catalogEntries = [...source.matchAll(/theme_info\(\s*"([a-z0-9_]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)\s*,/g)]
    .map((match) => ({
      identifier: match[1],
      displayName: match[2],
      isDark: match[3] === 'true',
    }));
  const cssIdentifiers = [...source.matchAll(/"([a-z0-9_]+)"\s*=>\s*(?:\{\s*)?include_str!\("\.\.\/\.\.\/\.\.\/\.\.\/resources\/themes\/([^/]+)\/theme\.css"\)/g)]
    .map((match) => ({ identifier: match[1], resource: match[2] }));
  return { catalogEntries, cssIdentifiers };
}

function assertBalancedCss(css, identifier) {
  let depth = 0;
  for (const character of css) {
    if (character === '{') depth += 1;
    if (character === '}') depth -= 1;
    assert.ok(depth >= 0, `${identifier} contains an unmatched closing brace`);
  }
  assert.equal(depth, 0, `${identifier} contains unmatched CSS braces`);
}

function assertThemeCssIsSafe(css, identifier) {
  assert.doesNotMatch(
    css,
    /(?:@import\b|url\(|javascript:|<script|https?:\/\/)/i,
    `${identifier} contains executable, imported, or remote content`,
  );
}

function hexToRgb(hex) {
  const normalized = hex.replace('#', '');
  assert.match(normalized, /^[0-9a-f]{6}$/i, `expected six-digit hex color, received ${hex}`);
  return [0, 2, 4].map((offset) => Number.parseInt(normalized.slice(offset, offset + 2), 16));
}

function luminance(hex) {
  const channels = hexToRgb(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(foreground, background) {
  const light = Math.max(luminance(foreground), luminance(background));
  const dark = Math.min(luminance(foreground), luminance(background));
  return (light + 0.05) / (dark + 0.05);
}

async function loadTheme(identifier) {
  const directory = path.join(themesRoot, identifier);
  const metadata = JSON.parse(await readFile(path.join(directory, 'theme.json'), 'utf8'));
  const css = await readFile(path.join(directory, 'theme.css'), 'utf8');
  return { metadata, css, tokens: parseTokens(css) };
}

async function loadFrontendComponent(filename) {
  return readFile(path.join(frontendComponentsRoot, filename), 'utf8');
}

function tokenColor(tokens, name) {
  const value = tokens.get(name);
  assert.ok(value, `missing ${name}`);
  assert.match(value, /^#[0-9a-f]{6}$/i, `${name} must be a six-digit hex color for contrast validation`);
  return value;
}

test('the built-in catalog contains exactly the 50 approved Racoon themes', async () => {
  const directories = await loadThemeDirectories();
  assert.deepEqual(directories, [...expectedThemes.keys()].sort());

  const metadataIdentifiers = [];
  for (const [identifier, expected] of expectedThemes) {
    const { metadata } = await loadTheme(identifier);
    metadataIdentifiers.push(metadata.name);
    assert.equal(metadata.name, identifier);
    assert.equal(metadata.display_name, expected.displayName);
    assert.equal(metadata.description, expected.description);
    assert.equal(metadata.is_dark, expected.isDark);
    assert.equal(metadata.category, expected.category);
    assert.equal(metadata.license, 'Apache-2.0');
    assert.equal(metadata.source, 'Racoon Typper repository');
    assert.equal(JSON.stringify(metadata).includes('http://'), false);
    assert.equal(JSON.stringify(metadata).includes('https://'), false);
  }
  assert.equal(new Set(metadataIdentifiers).size, expectedThemes.size);
  assert.equal(expectedThemes.size, 50);
});

test('theme resources and static Rust registries stay one-to-one', async () => {
  const directories = await loadThemeDirectories();
  const manifestIdentifiers = [];
  const manifestMetadata = new Map();
  for (const identifier of directories) {
    const directory = path.join(themesRoot, identifier);
    const metadata = JSON.parse(await readFile(path.join(directory, 'theme.json'), 'utf8'));
    await readFile(path.join(directory, 'theme.css'), 'utf8');
    assert.equal(metadata.name, identifier, `${identifier} manifest ID must match its directory`);
    manifestIdentifiers.push(metadata.name);
    manifestMetadata.set(identifier, metadata);
  }

  const preferencesSource = await readFile(preferencesSourcePath, 'utf8');
  const { catalogEntries, cssIdentifiers } = parseRegistryIdentifiers(preferencesSource);
  const catalogIdentifiers = catalogEntries.map(({ identifier }) => identifier);
  const cssRegistryIdentifiers = cssIdentifiers.map(({ identifier }) => identifier);
  const cssResourceIdentifiers = cssIdentifiers.map(({ resource }) => resource);

  assert.equal(directories.length, 50);
  assert.equal(new Set(directories).size, 50);
  assert.deepEqual(manifestIdentifiers.sort(), directories);
  assert.deepEqual([...new Set(catalogIdentifiers)].sort(), directories);
  assert.equal(catalogIdentifiers.length, 50);
  assert.deepEqual([...new Set(cssRegistryIdentifiers)].sort(), directories);
  assert.deepEqual([...new Set(cssResourceIdentifiers)].sort(), directories);
  assert.equal(cssIdentifiers.length, 50);
  assert.deepEqual(cssRegistryIdentifiers, cssResourceIdentifiers);

  for (const catalogEntry of catalogEntries) {
    const expected = expectedThemes.get(catalogEntry.identifier);
    const manifest = manifestMetadata.get(catalogEntry.identifier);
    assert.ok(expected, `${catalogEntry.identifier} Rust catalog entry must be accepted`);
    assert.ok(manifest, `${catalogEntry.identifier} Rust catalog entry must have a manifest`);
    assert.equal(catalogEntry.displayName, expected.displayName, `${catalogEntry.identifier} Rust display name must match the accepted catalog`);
    assert.equal(catalogEntry.isDark, expected.isDark, `${catalogEntry.identifier} Rust dark-mode flag must match the accepted catalog`);
    assert.equal(catalogEntry.displayName, manifest.display_name, `${catalogEntry.identifier} Rust display name must match the manifest`);
    assert.equal(catalogEntry.isDark, manifest.is_dark, `${catalogEntry.identifier} Rust dark-mode flag must match the manifest`);
  }
});

test('every built-in theme defines the complete semantic token contract', async () => {
  for (const identifier of expectedThemes.keys()) {
    const { css, tokens } = await loadTheme(identifier);
    assertBalancedCss(css, identifier);
    for (const token of requiredTokens) {
      assert.ok(tokens.has(token), `${identifier} is missing ${token}`);
      assert.notEqual(tokens.get(token), '', `${identifier} has an empty ${token}`);
    }
    for (const [alias, expectedValue] of legacyAliases) {
      assert.equal(tokens.get(alias), expectedValue, `${identifier} ${alias} must remain a semantic compatibility alias`);
    }
    for (const token of requiredTokens.filter((name) => name.startsWith('--color-'))) {
      assert.match(tokens.get(token), /^#[0-9a-f]{6}$/i, `${identifier} ${token} must be a six-digit hex color`);
    }
    for (const shadowToken of ['--shadow-surface', '--shadow-elevated']) {
      assert.match(tokens.get(shadowToken), /^(?:none|-?\d+(?:\.\d+)?(?:px)?(?:\s+-?\d+(?:\.\d+)?(?:px)?){1,3}\s+(?:#[0-9a-f]{6}|rgba?\([^)]*\)))$/i, `${identifier} ${shadowToken} has unsupported syntax`);
    }
    assertThemeCssIsSafe(css, identifier);
  }
});

test('built-in theme CSS rejects imports, including protocol-relative imports', () => {
  assert.throws(
    () => assertThemeCssIsSafe('@IMPORT "//example.invalid/theme.css";', 'fixture'),
    /fixture contains executable, imported, or remote content/,
  );
});

test('solid semantic colors meet contrast targets on their actual surfaces', async () => {
  for (const identifier of expectedThemes.keys()) {
    const { tokens } = await loadTheme(identifier);
    const background = tokenColor(tokens, '--color-app-background');
    const surface = tokenColor(tokens, '--color-surface-primary');
    const surfaceActive = tokenColor(tokens, '--color-surface-active');
    for (const textToken of ['--color-text-primary', '--color-text-secondary']) {
      assert.ok(contrast(tokenColor(tokens, textToken), background) >= 4.5, `${identifier} ${textToken} must be WCAG AA on the app background`);
      assert.ok(contrast(tokenColor(tokens, textToken), surface) >= 4.5, `${identifier} ${textToken} must be WCAG AA on the primary surface`);
    }
    assert.ok(contrast(tokenColor(tokens, '--color-text-disabled'), surface) >= 3, `${identifier} disabled text must remain readable on the primary surface`);
    assert.ok(contrast(tokenColor(tokens, '--color-caret'), surface) >= 3, `${identifier} caret must remain visible`);
    assert.ok(contrast(tokenColor(tokens, '--color-focus-ring'), surface) >= 3, `${identifier} focus ring must remain visible on the primary surface`);
    assert.ok(contrast(tokenColor(tokens, '--color-typing-pending'), surface) >= 4.5, `${identifier} pending text must remain readable on the typing surface`);
    assert.ok(contrast(tokenColor(tokens, '--color-typing-current'), surfaceActive) >= 4.5, `${identifier} current character must remain readable on the active surface`);
    assert.ok(contrast(tokenColor(tokens, '--color-typing-incorrect'), surface) >= 4.5, `${identifier} incorrect characters must remain immediately visible`);
    assert.ok(contrast(tokenColor(tokens, '--color-accent-text'), tokenColor(tokens, '--color-accent')) >= 4.5, `${identifier} accent foreground must remain readable`);
  }
});

test('high contrast uses stronger text contrast and distinct typing-state tokens', async () => {
  const { tokens } = await loadTheme('racoon_high_contrast');
  const background = tokenColor(tokens, '--color-app-background');
  assert.ok(contrast(tokenColor(tokens, '--color-text-primary'), background) >= 7);
  assert.ok(contrast(tokenColor(tokens, '--color-text-secondary'), background) >= 7);
  const typingStateValues = [
    '--color-typing-pending',
    '--color-typing-current',
    '--color-typing-correct',
    '--color-typing-incorrect',
    '--color-typing-corrected',
  ].map((token) => tokens.get(token));
  assert.equal(new Set(typingStateValues).size, typingStateValues.length);
  assert.notEqual(tokens.get('--color-caret'), tokens.get('--color-surface-primary'));
});

test('accent-filled frontend controls use the semantic accent foreground', async () => {
  const accentConsumers = [
    'KeyboardTrainer.svelte',
    'WeakKeysPanel.svelte',
    'CustomTextsView.svelte',
    'ResultOverlay.svelte',
    'ProfileTransferPanel.svelte',
  ];

  for (const filename of accentConsumers) {
    const source = await loadFrontendComponent(filename);
    assert.match(
      source,
      /background(?:-color)?:\s*var\(--main\)[\s\S]{0,180}color:\s*var\(--color-accent-text\)/,
      `${filename} must use --color-accent-text on accent-filled controls`,
    );
    assert.doesNotMatch(
      source,
      /background(?:-color)?:\s*var\(--main\)[\s\S]{0,180}color:\s*var\(--bg\)/,
      `${filename} must not use --bg as an accent-filled control foreground`,
    );
  }
});

test('primary typing surfaces consume semantic typing-state tokens with explicit current precedence', async () => {
  for (const filename of ['TestView.svelte', 'WeakKeysPanel.svelte']) {
    const source = await loadFrontendComponent(filename);
    for (const [status, token] of [
      ['pending', '--color-typing-pending'],
      ['correct', '--color-typing-correct'],
      ['incorrect', '--color-typing-incorrect'],
      ['backspaced', '--color-typing-corrected'],
    ]) {
      assert.match(source, new RegExp(`\\.char\\.${status}\\s*\\{[^}]*color:\\s*var\\(${token}\\)`), `${filename} must map ${status} to ${token}`);
    }
    assert.match(source, /\.char\.current\.pending\s*\{[^}]*color:\s*var\(--color-typing-current\)/, `${filename} must apply the current token only to pending current characters`);
    // TestView uses the unified smooth caret element; WeakKeysPanel keeps the
    // per-char pseudo-element. Both must consume the semantic caret token.
    assert.match(
      source,
      /(?:\.caret-element\s*\{[^}]*background:\s*var\(--color-caret\)|\.char\.caret::before\s*\{[^}]*background:\s*var\(--color-caret\))/,
      `${filename} must use the semantic caret token`,
    );
    assert.doesNotMatch(source, /\.char\.backspaced\s*\{[^}]*#[0-9a-f]{3,8}/i, `${filename} must not hardcode a corrected-character color`);
  }
});
