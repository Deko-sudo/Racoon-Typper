// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Windows UI smoke client: drives the installed Racoon Typper through
// tauri-driver (WebDriver) so keystrokes are delivered into the WebView2
// document deterministically. Exits non-zero when the typed practice session
// does not reach its result screen; the SQLite row assertion stays in the
// PowerShell orchestrator as the hard gate.
//
// Environment:
//   APP_PATH              — absolute path to racoon-app.exe (spawned by the driver)
//   WD_URL                — tauri-driver server URL (default http://127.0.0.1:4444/)
//   TYPING_SAMPLE         — keystroke payload (optional)
//   SURFACE_TIMEOUT_MS    — wait for .text-display (default 60000)
//   RESULT_TIMEOUT_MS     — wait for .result-overlay after expiry (default 90000)

import { Builder, By, Capabilities, until } from 'selenium-webdriver';

const appPath = process.env.APP_PATH;
if (!appPath) {
  console.error('APP_PATH is required');
  process.exit(64);
}
const wdUrl = process.env.WD_URL ?? 'http://127.0.0.1:4444/';
const surfaceTimeoutMs = Number(process.env.SURFACE_TIMEOUT_MS ?? 60_000);
const resultTimeoutMs = Number(process.env.RESULT_TIMEOUT_MS ?? 90_000);
const typingSample =
  process.env.TYPING_SAMPLE ?? 'asdf jkl; ewq poiuy '.repeat(12);

let driver;

process.on('exit', () => {
  // The session (and the driver-spawned application) must not outlive the
  // orchestrator's cleanup window; quit() is best-effort here because the
  // happy path awaits it explicitly.
  if (driver) void driver.quit();
});

try {
  const capabilities = new Capabilities();
  capabilities.set('tauri:options', { application: appPath });
  capabilities.setBrowserName('wry');

  driver = await new Builder()
    .withCapabilities(capabilities)
    .usingServer(wdUrl)
    .build();

  await driver.wait(until.titleIs('Racoon Typper'), surfaceTimeoutMs);
  console.log('[ui-smoke] first screen rendered (title matched)');

  const surface = await driver.wait(
    until.elementLocated(By.css('.text-display')),
    surfaceTimeoutMs,
  );
  await surface.click();
  console.log('[ui-smoke] typing surface focused');

  // First delivered character starts the time-mode timer (by design); keep a
  // human-like cadence so every subsequent key lands while the test runs.
  const chunk = Math.max(1, Math.ceil(typingSample.length / 12));
  for (let i = 0; i < typingSample.length; i += chunk) {
    await driver.actions().sendKeys(typingSample.slice(i, i + chunk)).perform();
    await new Promise((resolve) => setTimeout(resolve, 700));
  }
  console.log('[ui-smoke] typing sample delivered; waiting for result screen');

  await driver.wait(
    until.elementLocated(By.css('.result-overlay')),
    resultTimeoutMs,
  );
  console.log('[ui-smoke] result overlay rendered — practice session completed');
  process.exitCode = 0;
} catch (error) {
  console.error(`[ui-smoke] FAILED: ${error?.message ?? error}`);
  process.exitCode = 1;
} finally {
  if (driver) {
    try {
      await driver.quit();
    } catch {
      // Session teardown failures must not mask the primary result.
    }
  }
}
