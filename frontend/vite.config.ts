import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'node:url';
import { browserAuditPlugin } from '../scripts/browser-audit-server.mjs';

export default defineConfig(({ mode }) => ({
  plugins: [svelte(), ...(mode === 'browser-audit' ? [browserAuditPlugin()] : [])],
  resolve: mode === 'browser-audit' ? {
    alias: {
      '@tauri-apps/api/core': fileURLToPath(new URL('./audit/core.ts', import.meta.url)),
      '@tauri-apps/plugin-dialog': fileURLToPath(new URL('./audit/files.ts', import.meta.url)),
      '@tauri-apps/plugin-fs': fileURLToPath(new URL('./audit/files.ts', import.meta.url)),
      '@tauri-apps/plugin-clipboard-manager': fileURLToPath(new URL('./audit/files.ts', import.meta.url)),
    },
  } : {},
  clearScreen: false,
  server: {
    port: mode === 'browser-audit' ? 1440 : 1420,
    strictPort: true,
    host: mode === 'browser-audit' ? '127.0.0.1' : 'localhost',
    cors: mode === 'browser-audit' ? false : undefined,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // WKWebView follows macOS, unlike evergreen Windows WebView2. Keep both
    // release packages on a declared syntax floor rather than latest syntax.
    target: ['safari16.4', 'edge111'],
    cssTarget: ['safari16.4', 'edge111'],
    outDir: 'dist',
  },
}));
