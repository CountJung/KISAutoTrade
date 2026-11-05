import { defineConfig, type UserConfig } from 'vite'
import react from '@vitejs/plugin-react'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async (): Promise<UserConfig> => ({
  plugins: [react()],
  clearScreen: false,
  // @tauri-apps/api를 Vite pre-bundling에서 제외
  // Vite가 @tauri-apps/api를 pre-bundle하면 window.__TAURI_INTERNALS__ 초기화 전에
  // 모듈이 평가되어 "Cannot read properties of undefined (reading 'invoke')" 에러 발생
  optimizeDeps: {
    exclude: ['@tauri-apps/api'],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || '0.0.0.0',
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target:
      process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: 'react', priority: 20, test: /[/\\]node_modules[/\\](?:react|react-dom|scheduler)(?:[/\\]|$)/ },
            { name: 'mui', test: /[/\\]node_modules[/\\]@mui[/\\]/ },
            { name: 'emotion', priority: 10, test: /[/\\]node_modules[/\\]@emotion[/\\]/ },
            { name: 'tanstack', test: /[/\\]node_modules[/\\]@tanstack[/\\]/ },
            { name: 'charts', test: /[/\\]node_modules[/\\]lightweight-charts[/\\]/ },
          ],
        },
      },
    },
  },
}))
