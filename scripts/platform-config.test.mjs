// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const json = path => JSON.parse(read(path));
const base = json('crates/app/tauri.conf.json');
const require = createRequire(new URL('../frontend/package.json', import.meta.url));

for (const [platform, targets] of Object.entries({ linux: ['deb', 'rpm', 'appimage'], windows: ['nsis'], macos: ['app', 'dmg'] })) {
  test(`${platform} overrides packaging only and inherits the shared window, security and frontend`, () => {
    const override = json(`crates/app/tauri.${platform}.conf.json`);
    assert.deepEqual(Object.keys(override).sort(), ['$schema', 'bundle']);
    assert.deepEqual(override.bundle.targets, targets);
    assert.equal(override.bundle.resources, undefined, 'do not replace shared fonts/notices');
    assert.equal(base.build.frontendDist, '../../frontend/dist');
    assert.equal(base.app.windows.length, 1);
    assert.equal(base.app.windows[0].visible, true);
    assert.equal(base.app.windows[0].center, true);
    assert.equal(base.plugins.updater, undefined);
  });
}
test('Windows embeds the offline WebView2 installer and retains the existing installer upgrade scope', () => {
  const windows = json('crates/app/tauri.windows.conf.json').bundle.windows;
  assert.deepEqual(windows.webviewInstallMode, { type: 'offlineInstaller' });
  assert.equal(windows.nsis.installMode, 'perMachine');
  assert.equal(base.bundle.targets, undefined, 'targets must come from the host override');
});
test('production assets have explicit WebView syntax targets, with no browser audit adapter', async () => {
  const { loadConfigFromFile } = require('vite');
  const { config } = await loadConfigFromFile({ command: 'build', mode: 'production' }, fileURLToPath(new URL('../frontend/vite.config.ts', import.meta.url)));
  assert.deepEqual(config.build.target, ['safari16.4', 'edge111']);
  assert.deepEqual(config.build.cssTarget, config.build.target);
  assert.deepEqual(config.resolve, {});
  assert.equal(json('crates/app/tauri.macos.conf.json').bundle.macOS.minimumSystemVersion, '13.3');
});
test('native typing target cannot display a second displaced caret', () => {
  const source = read('frontend/src/components/TypingInput.svelte');
  const { css } = require('svelte/compiler').parse(source);
  const rule = css.children.find(node => node.type === 'Rule' && source.slice(node.start, node.block.start).trim() === '.typing-input');
  const declarations = Object.fromEntries(rule.block.children.map(node => [node.property, node.value.trim()]));
  assert.equal(declarations['caret-color'], 'transparent');
  assert.equal(declarations.color, 'transparent');
});
