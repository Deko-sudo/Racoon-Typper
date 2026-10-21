// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Resolved ONLY by Vite's browser-audit mode. Ordinary desktop IPC is unchanged.
export async function auditRequest(path: string, body: BodyInit, contentType: string) {
  const token = document.querySelector<HTMLMetaElement>('meta[name="audit-token"]')?.content;
  if (!token) throw new Error('Browser audit transport is unavailable.');
  const response = await fetch(`/__audit/${path}`, {
    method: 'POST',
    headers: { 'Content-Type': contentType, Authorization: `Bearer ${token}` },
    body,
  });
  if (!response.ok) throw new Error('Local audit backend failed.');
  return response.json();
}

export async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const response = await auditRequest('invoke', JSON.stringify({ cmd, args }), 'application/json');
  if (!response.ok) throw response.error;
  return response.value as T;
}
