import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  outputDir: "../target/playwright/ui/test-results",
  reporter: [["list"], ["html", { open: "never", outputFolder: "../target/playwright/ui/report" }]],
  use: {
    trace: "on-first-retry"
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] }
    }
  ]
});
