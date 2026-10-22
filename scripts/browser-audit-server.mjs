// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { spawn } from 'node:child_process';
import { randomBytes, randomUUID } from 'node:crypto';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';

export const AUDIT_ORIGIN = 'http://127.0.0.1:1440';
export const MAX_UPLOAD = 64 * 1024 * 1024;

export function authorized(headers, token) {
  return headers.host === '127.0.0.1:1440'
    && headers.origin === AUDIT_ORIGIN
    && headers.authorization === `Bearer ${token}`;
}

export async function readBody(stream, maximum) {
  const chunks = [];
  let size = 0;
  for await (const chunk of stream) {
    size += chunk.length;
    if (size > maximum) throw new Error('Audit request is too large.');
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

export async function startBackend(binary, profile) {
  const child = spawn(binary, ['--browser-audit'], {
    // Defense in depth if the operator accidentally selects a native binary.
    env: { ...process.env, RACOON_BROWSER_PROFILE: profile,
      XDG_DATA_HOME: join(profile, 'xdg-data'), XDG_CONFIG_HOME: join(profile, 'xdg-config'),
      XDG_CACHE_HOME: join(profile, 'xdg-cache') },
    stdio: ['pipe', 'pipe', 'inherit'],
  });
  let pending;
  let stopped = false;
  let tail = Promise.resolve();
  const read = () => new Promise((resolve, reject) => {
    if (stopped) { reject(new Error('Audit backend stopped.')); return; }
    const timeout = setTimeout(() => {
      reject(new Error('Audit backend timed out. Restart the audit server.'));
      child.kill();
    }, 30000);
    pending = {
      resolve: (value) => { clearTimeout(timeout); pending = undefined; resolve(value); },
      reject: (error) => { clearTimeout(timeout); pending = undefined; reject(error); },
    };
  });
  createInterface({ input: child.stdout }).on('line', (line) => {
    try { pending?.resolve(JSON.parse(line)); }
    catch { pending?.reject(new Error('Invalid audit backend response.')); child.kill(); }
  });
  const stop = () => {
    stopped = true;
    pending?.reject(new Error('Audit backend stopped. Restart the audit server.'));
  };
  child.once('error', stop);
  child.once('exit', stop);
  const ready = await read();
  if (!ready.ready) { child.kill(); throw new Error('Audit backend did not initialize.'); }
  return {
    invoke(request) {
      const job = tail.then(() => {
        const response = read();
        if (!stopped) child.stdin.write(`${JSON.stringify(request)}\n`);
        return response;
      });
      tail = job.catch(() => {});
      return job;
    },
    close: () => child.kill(),
  };
}

export function browserAuditPlugin() {
  const token = randomBytes(32).toString('hex');
  return {
    name: 'racoon-local-browser-audit',
    configResolved(config) {
      if (config.command !== 'serve') throw new Error('Browser audit is development-only.');
      if (config.server.host !== '127.0.0.1' || config.server.port !== 1440 || !config.server.strictPort) {
        throw new Error('Browser audit requires its fixed loopback address.');
      }
    },
    transformIndexHtml() {
      return [
        { tag: 'meta', attrs: { name: 'audit-token', content: token }, injectTo: 'head' },
        { tag: 'aside', attrs: { style: 'padding:8px;text-align:center;background:#ffe59b;color:#202020;font:14px sans-serif' },
          children: 'Browser audit — real Rust / SQLite · isolated test profile · desktop shell is not tested', injectTo: 'body-prepend' },
      ];
    },
    async configureServer(server) {
      const binary = process.env.RACOON_BROWSER_BINARY;
      if (!binary) throw new Error('Build the browser-audit feature and set RACOON_BROWSER_BINARY.');
      const profile = await mkdtemp(join(tmpdir(), 'racoon-browser-audit-'));
      await mkdir(join(profile, 'uploads'), { mode: 0o700 });
      const backend = await startBackend(binary, profile);
      console.log(`Browser audit profile (test data only): ${profile}`);
      const close = () => backend.close();
      process.once('exit', close);
      server.httpServer?.once('close', () => { close(); process.removeListener('exit', close); });
      server.middlewares.use('/__audit', async (req, res) => {
        res.setHeader('Content-Type', 'application/json');
        res.setHeader('Cache-Control', 'no-store');
        if (req.method !== 'POST' || !authorized(req.headers, token)) {
          res.statusCode = 403;
          res.end(JSON.stringify({ error: 'Forbidden audit request.' }));
          return;
        }
        try {
          if (req.url === '/invoke' && req.headers['content-type'] === 'application/json') {
            const request = JSON.parse((await readBody(req, 8 * 1024 * 1024)).toString('utf8'));
            res.end(JSON.stringify(await backend.invoke(request)));
          } else if (req.url === '/upload' && req.headers['content-type'] === 'application/octet-stream') {
            const body = await readBody(req, MAX_UPLOAD);
            const path = join(profile, 'uploads', randomUUID());
            await writeFile(path, body, { mode: 0o600, flag: 'wx' });
            res.end(JSON.stringify({ path }));
          } else {
            res.statusCode = 404;
            res.end(JSON.stringify({ error: 'Unknown audit operation.' }));
          }
        } catch {
          res.statusCode = 400;
          res.end(JSON.stringify({ error: 'Audit operation failed; see local development logs.' }));
        }
      });
    },
  };
}
