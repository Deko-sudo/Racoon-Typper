#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { existsSync } from "node:fs";
import { spawn } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..");
const appDirectory = join(repositoryRoot, "crates", "app");
const tauriEntrypoint = join(
  repositoryRoot,
  "frontend",
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);

if (!existsSync(tauriEntrypoint)) {
  console.error(
    "Tauri CLI is not installed. Run `npm ci --prefix frontend` first.",
  );
  process.exit(1);
}

const child = spawn(process.execPath, [tauriEntrypoint, ...process.argv.slice(2)], {
  cwd: appDirectory,
  stdio: "inherit",
  windowsHide: false,
});

child.on("error", (error) => {
  console.error(`Unable to start Tauri CLI: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 1);
});
