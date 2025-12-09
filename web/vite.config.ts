import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  // For GitHub Pages deployment - adjust base as needed
  base: './',
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  server: {
    port: 3001,
    // Proxy API requests to deciduous serve (when running locally)
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
