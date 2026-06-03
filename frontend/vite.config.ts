import { defineConfig } from 'vite'
import { tanstackStart } from '@tanstack/react-start/plugin/vite'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  server: {
    // 3000 is the Rust backend (see AGENTS.md); Vite auto-falls-back
    // to 3001 when 3000 is busy. Kept explicit so the choice is visible.
    port: 3000,
  },
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    tanstackStart(),
    // react's vite plugin must come after start's vite plugin
    viteReact(),
    tailwindcss(),
  ],
})
