import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { viteSingleFile } from 'vite-plugin-singlefile'
import path from 'path'

// Config for building a single embeddable HTML file for Rust include_str!
export default defineConfig({
  plugins: [react(), viteSingleFile()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  base: './',
  build: {
    outDir: 'dist-embed',
    // Inline everything
    assetsInlineLimit: 100000000, // 100MB - inline everything
    cssCodeSplit: false,
    minify: true,
    rollupOptions: {
      output: {
        // Prevent code-splitting
        inlineDynamicImports: true,
      },
    },
  },
})
