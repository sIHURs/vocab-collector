import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  root: fileURLToPath(new URL('.', import.meta.url)),
  plugins: [tailwindcss(), svelte()],
  resolve: { alias: { $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) }, conditions: ['browser'] },
  server: { host: '127.0.0.1', port: 5174, strictPort: true },
  preview: { host: '127.0.0.1', port: 4174, strictPort: true },
  build: { outDir: 'dist-landing', emptyOutDir: true, rollupOptions: {
    input: { index: fileURLToPath(new URL('./landing/index.html', import.meta.url)), demo: fileURLToPath(new URL('./landing/demo.html', import.meta.url)) },
  } },
});
