#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(readFileSync(join(root, "com.racoon.typper.json"), "utf8"));
const module = manifest.modules?.find(({ name }) => name === "racoon-typper");

function requireCondition(condition, message) {
  if (!condition) throw new Error(message);
}

requireCondition(manifest["app-id"] === "com.racoon.typper", "Unexpected Flatpak app ID");
requireCondition(typeof manifest.runtime === "string" && typeof manifest["runtime-version"] === "string", "Runtime must be explicitly pinned");
requireCondition(module?.sources?.some((source) => source.type === "dir" && source.path === "."), "Flatpak must build the checked-out source tree");
requireCondition(module?.["build-commands"]?.includes("npm run tauri:build:binary --prefix frontend"), "Flatpak must build the application binary from source");
requireCondition(!manifest["finish-args"]?.some((argument) => argument.startsWith("--filesystem=")), "Flatpak runtime must not grant host filesystem access");
requireCondition(!manifest["finish-args"]?.includes("--share=network"), "Flatpak runtime must not grant network access");

console.log("Flatpak manifest policy checks passed");
