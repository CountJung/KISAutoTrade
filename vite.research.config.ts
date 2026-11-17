import { defineConfig, mergeConfig } from 'vite'
import baseConfig from './vite.config.ts'

// Research tools never load .env files and only expose a loopback dev server.
export default defineConfig(async (env) => {
  const base = typeof baseConfig === 'function' ? await baseConfig(env) : await baseConfig
  return mergeConfig(base, {
    envDir: false,
    server: { host: '127.0.0.1', port: 1420, strictPort: true, hmr: false, ws: false },
  })
})
