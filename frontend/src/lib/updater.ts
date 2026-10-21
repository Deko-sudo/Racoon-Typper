// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Auto-update wrapper around @tauri-apps/plugin-updater. Kept separate so the
// UI can call a small, typed surface and so the check logic is testable.

import { check } from '@tauri-apps/plugin-updater';

export interface UpdateCheckResult {
  available: boolean;
  version?: string;
  error?: string;
}

/**
 * Check whether an update is available. Returns the new version when one is
 * found, or an error string when the check fails (e.g. no network, no release).
 */
export async function checkForUpdate(): Promise<UpdateCheckResult> {
  try {
    const update = await check();
    if (!update) return { available: false };
    return { available: true, version: update.version };
  } catch (error) {
    return { available: false, error: String(error) };
  }
}

/**
 * Download and install the available update. Returns true on success.
 * The app restarts after installation.
 */
export async function installUpdate(): Promise<boolean> {
  try {
    const update = await check();
    if (!update) return false;
    await update.downloadAndInstall();
    return true;
  } catch {
    return false;
  }
}
