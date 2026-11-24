import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

const gateway = process.env.NX_GATEWAY || 'http://127.0.0.1:18006'

export default defineConfig({
  base: '/app/',
  plugins: [vue()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  build: {
    outDir: fileURLToPath(new URL('../rust-gateway/resources/public/app', import.meta.url)),
    emptyOutDir: true,
    target: 'es2020',
    chunkSizeWarningLimit: 900,
  },
  server: {
    port: 5173,
    proxy: {
      '/api': { target: gateway, changeOrigin: true },
      '/theme': { target: gateway, changeOrigin: true },
    },
  },
})
