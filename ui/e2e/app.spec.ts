import { expect, test } from "@playwright/test";

test("renders the dashboard without runtime errors", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("pageerror", (error) => runtimeErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(message.text());
    }
  });

  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1, name: "Launch" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Setup ready" })).toHaveCount(0);
  expect(runtimeErrors).toEqual([]);
});

test("reviews and starts a preview run", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("alert").filter({ hasText: "Agent ready" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Choose project" })).toBeVisible();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-preview");
  await page.getByRole("button", { name: "Review access" }).click();

  await expect(page.getByRole("heading", { name: "Review access" })).toBeVisible();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();

  await expect(page.getByText(/Run started: preview-/)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Last launch" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Run status" })).toBeVisible();
  await expect(page.getByText("Container state")).toBeVisible();
  await expect(page.getByText("running").first()).toBeVisible();
  await expect(page.getByText("Container", { exact: true })).toBeVisible();

  await expect(page.getByText("Active sessions")).toBeVisible();
  await page.getByText("Claude - runhaven-preview").click();
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Claude - runhaven-preview" })).toBeVisible();
  await expect(page.getByText("Monitor session")).toBeVisible();
  await expect(page.getByText("Run record")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Run output" })).toBeVisible();
  await expect(page.getByRole("button", { name: "View latest output" })).toBeDisabled();
  await expect(page.getByLabel("Keep output updated")).toBeDisabled();
  await page.getByLabel("Show raw container output for this run.").check();
  await page.getByRole("button", { name: "View latest output" }).click();
  await expect(page.getByText("Preview log line")).toBeVisible();
  await page.getByLabel("Keep output updated").check();
  await expect(page.getByText("Updating output every few seconds.")).toBeVisible();
  await expect(page.getByText("Preview log line 2")).toBeVisible();
  await expect(page.getByText("/tmp/runhaven-preview")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Session controls" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toBeDisabled();
  await page.getByLabel("Confirm stopping this session.").check();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm hard-stopping this session.").check();
  await page.getByRole("button", { name: "Hard stop" }).click();
  await expect(page.getByText(/Hard stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm clearing this session's stale marker.").check();
  await page.getByRole("button", { name: "Repair marker" }).click();
  await expect(page.getByText(/Stale marker cleared for preview-/)).toBeVisible();
  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Launch" })).toBeVisible();

  await expect(page.getByRole("button", { name: "Stop run" })).toBeDisabled();
  await page.getByLabel("Confirm stopping this run.").check();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm hard-stopping this run.").check();
  await page.getByRole("button", { name: "Hard stop" }).click();
  await expect(page.getByText(/Hard stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm clearing this run's stale marker.").check();
  await page.getByRole("button", { name: "Repair marker" }).click();
  await expect(page.getByText(/Stale marker cleared for preview-/)).toBeVisible();
});

test("loads secret-free diagnostics", async ({ page }) => {
  await page.goto("/");

  await page.getByText("Diagnostics", { exact: true }).click();
  await expect(page.getByRole("heading", { name: "Trust checks" })).toBeVisible();
  await page.getByRole("button", { name: "Load diagnostics" }).click();
  await expect(page.getByRole("heading", { name: "Auth broker status" })).toBeVisible();
  await expect(page.getByText("codex: api-key-prototype")).toBeVisible();
  await page.getByRole("tab", { name: "Network" }).click();
  await expect(page.getByText("No provider egress decisions recorded.")).toBeVisible();
});
