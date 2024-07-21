import { fileURLToPath, URL } from 'node:url'
import AutoImport from "unplugin-auto-import/vite"
import Components from "unplugin-vue-components/vite"

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  clearScreen: false,
  server:{
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_PLATFORM', 'TAURI_ARCH', 'TAURI_FAMILY', 'TAURI_PLATFORM_VERSION', 'TAURI_PLATFORM_TYPE', 'TAURI_DEBUG'],

  plugins: [
    AutoImport({
      eslintrc: {
        enabled: true,
      },
      imports: [
        "vue",
        "vue-router"
      ],
      dts: "src/auto-imports.d.ts",
      dirs: ["src/composables/**", "src/stores", "src/utils/**"]
    }),
    Components({
      dts: 'src/components.d.ts',
      directoryAsNamespace: true,
      extensions: ["vue"],
      include: [/\.vue$/, /\.vue\?vue/,],
    }),
    vue(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
