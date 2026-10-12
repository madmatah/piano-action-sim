import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    // wasm-pack --target web emits an ES module with an explicit async init(),
    // so neither vite-plugin-wasm nor vite-plugin-top-level-await is needed.
    target: 'esnext',
  },
  worker: {
    // Vite still defaults worker.format to 'iife'; module workers have been
    // Baseline since Firefox 114.
    format: 'es',
  },
  server: {
    // Mirrors the production _headers file so that cross-origin isolation —
    // and therefore SharedArrayBuffer — behaves identically in dev and in prod.
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
})
