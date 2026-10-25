import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({
  plugins: [react()],
  cacheDir: 'target/vite-e2e-cache',
  build: { outDir: 'target/e2e-site', rollupOptions: { input: 'tests/e2e/harness.html' } },
});
