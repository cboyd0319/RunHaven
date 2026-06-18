import { defineConfig, devices } from "@playwright/test";

const requestedPort = process.env.RUNHAVEN_PLAYWRIGHT_PORT ?? "5174";
const port = /^\d+$/.test(requestedPort) ? requestedPort : "5174";
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: "./e2e",
  outputDir: "../target/playwright/ui/test-results",
  reporter: [["list"], ["html", { open: "never", outputFolder: "../target/playwright/ui/report" }]],
  use: {
    baseURL,
    trace: "on-first-retry"
  },
  webServer: {
    command: `npm run dev -- --host 127.0.0.1 --port ${port} --strictPort`,
    url: baseURL,
    reuseExistingServer: false,
    timeout: 120_000
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] }
    }
  ]
});
