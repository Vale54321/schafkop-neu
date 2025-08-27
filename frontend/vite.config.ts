import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      // forward serial API calls (including SSE) to backend running on :3000
      '/serial': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        ws: false,
      },
    },
  },
})
