import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  // External macOS volumes can contain AppleDouble metadata beside test sources.
  testIgnore: '**/._*',
  timeout: 30_000,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  expect: {
    timeout: 5_000,
  },
  use: {
    baseURL: 'http://127.0.0.1:1430',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  // Source harnesses intercept Vite-transformed modules; UI tests exercise dist in CI.
  webServer: process.env.CI ? [
    {
      command: 'npm run dev -- --host 127.0.0.1 --port 1430 --strictPort',
      url: 'http://127.0.0.1:1430',
      reuseExistingServer: false,
      timeout: 60_000,
    },
    {
      command: 'npm run preview -- --host 127.0.0.1 --port 1431 --strictPort',
      url: 'http://127.0.0.1:1431',
      reuseExistingServer: false,
      timeout: 60_000,
    },
  ] : {
    command: 'npm run dev -- --host 127.0.0.1 --port 1430 --strictPort',
    url: 'http://127.0.0.1:1430',
    reuseExistingServer: true,
    timeout: 60_000,
  },
  projects: process.env.CI ? [
    {
      name: 'chromium',
      testIgnore: ['**/._*', '**/balance-events.spec.ts', '**/toss-commission-ratio.spec.ts', '**/strategy-deviation-policy.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: 'http://127.0.0.1:1431' },
    },
    {
      name: 'chromium-source-harness',
      testMatch: ['**/balance-events.spec.ts', '**/toss-commission-ratio.spec.ts', '**/strategy-deviation-policy.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: 'http://127.0.0.1:1430' },
    },
  ] : [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
