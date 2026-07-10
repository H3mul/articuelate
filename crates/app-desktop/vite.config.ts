import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri expects a fixed port and ignores changes to the Rust sources.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/target/**', '**/src/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
});
