import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// Config for building to docs/demo/ for GitHub Pages
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  // Base path for GitHub Pages - app is at /deciduous/demo/
  base: '/deciduous/demo/',
  build: {
    outDir: '../docs/demo',
    emptyDir: false, // Don't delete graph-data.json
    sourcemap: false,
  },
})
