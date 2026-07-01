import { expect, test, type Page } from "@playwright/test";

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
  await expect(page.getByText("Sign-in method")).toBeVisible();
  await expect(page.getByText("Auth status")).toBeVisible();
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Setup ready" })).toHaveCount(0);
  expect(runtimeErrors).toEqual([]);
});

test("reviews and starts a preview run", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("alert").filter({ hasText: "Agent ready" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Choose project" })).toBeVisible();
  await expect(page.getByText("Reuse the login from runhaven login for this agent across projects on this Mac.")).toBeVisible();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-preview");
  await page.getByRole("button", { name: "Review access" }).click();

  await expect(page.getByRole("heading", { name: "Review access" })).toBeVisible();
  await expect(page.getByRole("rowheader", { name: "Login state" })).toBeVisible();
  await expect(page.getByText("RunHaven will use the shared claude login state across projects on this Mac.")).toBeVisible();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();

  await expect(page.getByText(/Run accepted: preview-/)).toBeVisible();
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Claude - runhaven-preview" })).toBeVisible();
  await expect(page.getByText("Monitor session")).toBeVisible();
  await expect(page.getByText("Run record")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Message" })).toBeVisible();
  await page.getByRole("button", { name: "Load session diagnostics" }).click();
  await expect(page.getByText("Auth state: api-key-broker")).toBeVisible();
  await expect(page.getByText("api.openai.com")).toBeVisible();
  await expect(page.getByRole("button", { name: "Send" })).toBeDisabled();
  await page.getByLabel("Message").fill("draft for selected session");
  await page.getByRole("button", { name: "Start" }).click();
  await page.getByText("Claude - runhaven-preview").click();
  await expect(page.getByLabel("Message")).toHaveValue("");
  await page.getByLabel("Message").fill("hello session");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();
  await expect(page.getByText(/Sent \d+ bytes\./)).toBeVisible();
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
  await expect(page.getByText("codex: api-key-broker")).toBeVisible();
  await page.getByRole("tab", { name: "Network" }).click();
  await expect(page.getByText("No provider egress decisions recorded.")).toBeVisible();
});

test("keeps selected-session message drafts isolated", async ({ page }) => {
  await page.goto("/");

  await startPreviewRun(page, "/tmp/runhaven-alpha");
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
  await page.getByLabel("Message").fill("alpha draft");
  await page.getByLabel("Refresh status").click();
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
  await expect(page.getByLabel("Message")).toHaveValue("alpha draft");
  await expect(page.getByRole("heading", { name: "Choose agent" })).toHaveCount(0);

  await page.getByRole("button", { name: "Start" }).click();
  await startPreviewRun(page, "/tmp/runhaven-beta");
  await expect(page.getByLabel("Message")).toHaveValue("");
  await page.getByLabel("Message").fill("beta message");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();

  await page.getByText("Claude - runhaven-alpha").click();
  await expect(page.getByLabel("Message")).toHaveValue("");
  await expect(page.getByText("beta message")).toHaveCount(0);
});

async function startPreviewRun(page: Page, workspacePath: string) {
  await expect(page.getByRole("heading", { level: 1, name: "Launch" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByLabel("Project folder path").fill(workspacePath);
  await page.getByRole("button", { name: "Review access" }).click();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();
  await expect(page.getByText(/Run accepted: preview-/)).toBeVisible();
}
