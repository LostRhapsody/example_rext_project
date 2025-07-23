import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      '@/appearance': fileURLToPath(new URL('./src/appearance', import.meta.url)),
      '@/bridge': fileURLToPath(new URL('./src/bridge', import.meta.url)),
      '@/components': fileURLToPath(new URL('./src/appearance/components', import.meta.url)),
      '@/views': fileURLToPath(new URL('./src/appearance/views', import.meta.url)),
      '@/pages': fileURLToPath(new URL('./src/appearance/pages', import.meta.url)),
      '@/layouts': fileURLToPath(new URL('./src/appearance/layouts', import.meta.url)),
      '@/styles': fileURLToPath(new URL('./src/appearance/styles', import.meta.url)),
      '@/api': fileURLToPath(new URL('./src/bridge/api', import.meta.url)),
      '@/stores': fileURLToPath(new URL('./src/bridge/stores', import.meta.url)),
      '@/types': fileURLToPath(new URL('./src/bridge/types', import.meta.url)),
      '@/router': fileURLToPath(new URL('./src/bridge/router', import.meta.url)),
    },
  },
})
