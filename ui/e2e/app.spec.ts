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

  await expect(page.getByRole("heading", { name: "Agent runs" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Setup checks" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Launch plan" })).toBeVisible();
  await expect(page.getByText("Preview mode")).toBeVisible();
  expect(runtimeErrors).toEqual([]);
});

test("reviews and starts a preview run", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByText("Image ready")).toBeVisible();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-preview");
  await page.getByRole("button", { name: "Review plan" }).click();

  await expect(page.getByRole("heading", { name: "Plan review" })).toBeVisible();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();

  await expect(page.getByText(/Run started: preview-/)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Last launch" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Run status" })).toBeVisible();
  await expect(page.getByText("Container state")).toBeVisible();
  await expect(page.getByText("running").first()).toBeVisible();
  await expect(page.getByText("Container", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Run output" })).toBeVisible();
  await expect(page.getByRole("button", { name: "View latest output" })).toBeDisabled();
  await page.getByLabel("Show raw container output for this run.").check();
  await page.getByRole("button", { name: "View latest output" }).click();
  await expect(page.getByText("Preview log line")).toBeVisible();

  await expect(page.getByRole("button", { name: "Stop run" })).toBeDisabled();
  await page.getByLabel("Confirm stopping this run.").check();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();
});
