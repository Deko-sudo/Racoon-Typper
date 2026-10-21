// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { auditRequest } from './core';

const selectedFiles = new Map<string, File>();

export async function open(options: { filters?: { extensions: string[] }[] } = {}): Promise<string | null> {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = options.filters?.flatMap((filter) => filter.extensions.map((ext) => `.${ext}`)).join(',') ?? '';
  const file = await new Promise<File | null>((resolve) => {
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.oncancel = () => resolve(null);
    input.click();
  });
  if (!file) return null;
  if (file.size > 64 * 1024 * 1024) throw new Error('Audit uploads are limited to 64 MiB.');
  const { path } = await auditRequest('upload', file, 'application/octet-stream');
  // Retain only the last selection, rather than accumulating browser Files.
  selectedFiles.clear();
  selectedFiles.set(path, file);
  return path;
}

export async function readTextFile(path: string): Promise<string> {
  const file = selectedFiles.get(path);
  if (!file) throw new Error('Select a file in this browser first.');
  return file.text();
}

export async function readText(): Promise<string> {
  return navigator.clipboard.readText();
}
