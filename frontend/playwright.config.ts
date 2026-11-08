import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';

const evidence = process.env.DREAM_MACHINE_EVIDENCE_DIR;
export default defineConfig({
  testDir: 'tests',
  workers: 1,
  outputDir: evidence ? path.join(evidence, 'playwright-results') : 'test-results',
  reporter: evidence ? [['json', { outputFile: path.join(evidence, 'browser-report.json') }]] : 'list',
  use: {
    baseURL: 'http://127.0.0.1:5173',
    trace: 'on',
    launchOptions: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH
      ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH }
      : undefined,
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 5'] } },
  ],
});
