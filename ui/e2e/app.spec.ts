import { expect, test, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

async function expectNoAccessibilityViolations(page: Page) {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(results.violations).toEqual([]);
}

async function expectNoHorizontalOverflow(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
}

test("meets automated WCAG checks on the primary workflow", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Choose an agent" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Start with Codex" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Start with Codex" })).toHaveCSS("opacity", "1");
  await expectNoAccessibilityViolations(page);

  await page.getByRole("button", { name: "Start with Codex" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "New session" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByRole("button", { name: "Cancel" }).click();

  await page.getByRole("button", { name: "History", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Run history" })).toBeVisible();
  await expectNoAccessibilityViolations(page);

  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveClass(/dark/);
  await expectNoAccessibilityViolations(page);

  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await expect(page.getByText("App icon count", { exact: true })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByRole("button", { name: "System details", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "System details" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Remove stale networks" })).toBeEnabled();
  await expectNoAccessibilityViolations(page);
});

test("renders the dashboard without runtime errors", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("pageerror", (error) => runtimeErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(message.text());
    }
  });

  await page.goto("/");

  // Home is only the agent launchpad. Operational status and run history live
  // on their own destinations instead of competing with the common path.
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Choose an agent" })).toBeVisible();
  for (const name of ["Claude", "Codex", "Antigravity", "Copilot"]) {
    await expect(page.getByRole("button", { name: `Start with ${name}` })).toBeEnabled();
  }
  await expect(page.getByText("System ready", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Recent work", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Needs attention", { exact: true })).toHaveCount(0);

  // New Session is a page, not a modal. Draft values survive ordinary
  // navigation so a user can check Home without losing work.
  await page.getByRole("button", { name: "Start with Codex" }).click();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expect(page.getByText("Sign-in method")).toHaveCount(0);
  await expect(page.getByText("API key handling")).toHaveCount(0);
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByText("Advanced options", { exact: true }).click();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-draft");
  await page.getByRole("button", { name: "Home", exact: true }).click();
  await page.getByRole("button", { name: "Start with Codex" }).click();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Choose project" })).toBeVisible();
  await page.getByText("Advanced options", { exact: true }).click();
  await expect(page.getByLabel("Project folder path")).toHaveValue("/tmp/runhaven-draft");
  await page.getByRole("button", { name: "Back" }).click();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();

  expect(runtimeErrors).toEqual([]);
});

test("fits the minimum desktop window and follows system appearance", async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 560 });
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/");

  await expect(page.locator("html")).not.toHaveClass(/dark/);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
  await expect(page.getByRole("button", { name: "Start with Codex" })).toBeVisible();

  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveClass(/dark/);
  await page.getByRole("button", { name: "Start with Codex" }).click();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
  const continueBox = await page.getByRole("button", { name: "Continue" }).boundingBox();
  expect(continueBox).not.toBeNull();
  expect(continueBox!.y).toBeGreaterThanOrEqual(0);
  await expectNoHorizontalOverflow(page);
});

for (const viewport of [
  { label: "minimum", width: 800, height: 560 },
  { label: "normal", width: 1080, height: 720 },
]) {
  test(`fits every primary surface at the ${viewport.label} desktop window`, async ({
    page,
  }) => {
    const runtimeErrors: string[] = [];
    page.on("pageerror", (error) => runtimeErrors.push(error.message));
    page.on("console", (message) => {
      if (message.type() === "error") runtimeErrors.push(message.text());
    });
    await page.setViewportSize(viewport);
    await page.goto("/");
    await expectNoHorizontalOverflow(page);

    await page.getByRole("button", { name: "History", exact: true }).click();
    await expectNoHorizontalOverflow(page);

    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await expect(page.getByText("App icon count", { exact: true })).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await page.getByRole("button", { name: "System details", exact: true }).click();
    await expectNoHorizontalOverflow(page);

    await page.getByRole("button", { name: "Home", exact: true }).click();
    await page.getByRole("button", { name: "Start with Claude", exact: true }).click();
    for (const name of ["Claude", "Codex", "Antigravity", "Copilot"]) {
      await selectAgent(page, name);
      await expectNoHorizontalOverflow(page);
    }
    await selectAgent(page, "Claude");
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await page.getByText("Advanced options", { exact: true }).click();
    await page
      .getByLabel("Project folder path")
      .fill(`/tmp/runhaven-${viewport.label}-surface-qa`);
    await page.getByRole("button", { name: "Review access", exact: true }).click();
    await page.getByRole("button", { name: "Start run", exact: true }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Run terminal" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Close", exact: true })).toBeVisible();
    await expectNoHorizontalOverflow(page);
    expect(runtimeErrors).toEqual([]);
  });
}

test("reviews and starts a preview run", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Start with Codex" }).click();
  await expect(page.getByRole("alert").filter({ hasText: "Agent ready" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Choose project" })).toBeVisible();
  await expect(page.getByText("Use the sign-in you did in RunHaven for this agent across your projects on this Mac.")).toBeHidden();
  await page.getByText("Advanced options", { exact: true }).click();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-preview");
  await page.getByRole("button", { name: "Review access" }).click();

  await expect(page.getByRole("heading", { name: "Review access" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByText("Technical details", { exact: true }).click();
  await expect(page.getByText("Sign-in sharing", { exact: true })).toBeVisible();
  await expect(page.getByText("RunHaven will use the shared codex sign-in across projects on this Mac.")).toBeVisible();
  await page.getByRole("button", { name: "Start run" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
  // Once the run shows up as a managed session, the "Run accepted" launch
  // alert clears instead of persisting forever.
  await expect(page.getByText(/Run accepted: preview-/)).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Codex - runhaven-preview" })).toBeVisible();
  await expect(page.getByText("Monitor session")).toHaveCount(0);
  await page.getByText("Run details and events", { exact: true }).click();
  await expect(page.getByText("Run record", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Message" })).toBeVisible();
  // The session view has its own Session diagnostics card; the app-wide
  // Diagnostics surface is a sidebar destination, never a second panel in the
  // session view's content pane.
  await expect(
    page.getByRole("main").getByText("Diagnostics", { exact: true }),
  ).toHaveCount(0);
  // Runtime status and per-session diagnostics sit behind the Details
  // disclosure instead of stacking as permanent panes.
  await expect(page.getByRole("heading", { name: "Runtime status" })).toHaveCount(0);
  await page.getByRole("button", { name: "Show details" }).click();
  await expect(page.getByRole("heading", { name: "Runtime status" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByRole("button", { name: "Load run diagnostics" }).click();
  await expect(page.getByText("Credential access: API key broker")).toBeVisible();
  await expect(page.getByText("api.openai.com")).toBeVisible();
  await expect(page.getByRole("button", { name: "Send" })).toBeDisabled();
  await page.getByLabel("Message").fill("draft for selected session");
  await page.getByRole("button", { name: "History", exact: true }).click();
  await page.getByText("Codex - runhaven-preview").click();
  await expect(page.getByLabel("Message")).toHaveValue("draft for selected session");
  await page.getByLabel("Message").fill("hello session");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();
  await expect(page.getByText(/Sent \d+ bytes\./)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Latest output" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Show latest output" })).toBeEnabled();
  await expect(page.getByText("Keep output updated")).toHaveCount(0);
  await page.getByRole("button", { name: "Show latest output" }).click();
  await expect(page.getByText("Preview log line")).toBeVisible();
  await expect(page.getByText("/tmp/runhaven-preview")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Run actions" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toBeEnabled();
  await expect(page.getByText("More actions", { exact: true })).toBeVisible();
  await page.getByText("More actions", { exact: true }).click();
  await expect(page.getByRole("button", { name: "Hard stop" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Repair run record" })).toHaveCount(0);
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();
  await page.getByRole("button", { name: "Home" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();

  // Home remains a launchpad after a run. Existing runs and their controls live
  // only in History and the per-session view.
  await expect(page.getByRole("heading", { name: "Choose an agent" })).toBeVisible();
  await expect(page.getByText("Run started. Find it in Recent work.")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Stop run" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Hard stop" })).toHaveCount(0);
  await page.getByRole("button", { name: "History", exact: true }).click();
  await page.getByText("Codex - runhaven-preview").click();
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
});

test("shows actionable system details without raw trust logs", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await page.getByRole("button", { name: "System details", exact: true }).click();
  await expect(page.getByText("Your Mac is ready for RunHaven.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Setup problems" })).toHaveCount(0);
  await expect(page.getByText("Preview mode")).toHaveCount(0);
  await expect(page.getByText("Trust checks")).toHaveCount(0);
  await expect(page.getByText("Auth broker status")).toHaveCount(0);
  await expect(page.getByText("Network trust")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Leftover networks" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Check again" })).toBeVisible();
});

test("keeps selected-session message drafts isolated", async ({ page }) => {
  await page.goto("/");

  await startPreviewRun(page, "/tmp/runhaven-alpha");
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
  await page.getByLabel("Message").fill("alpha draft");
  await page.getByLabel("Refresh status").click();
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
  await expect(page.getByLabel("Message")).toHaveValue("alpha draft");
  await expect(page.getByRole("heading", { name: "Choose agent" })).toHaveCount(0);

  await page.getByRole("button", { name: "Home" }).click();
  await startPreviewRun(page, "/tmp/runhaven-beta");
  await expect(page.getByLabel("Message")).toHaveValue("");
  await page.getByLabel("Message").fill("beta message");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();

  // Sessions live in History, not Home or the sidebar.
  await page.getByRole("button", { name: "History", exact: true }).click();
  await page.getByText("Codex - runhaven-alpha").click();
  await expect(page.getByLabel("Message")).toHaveValue("alpha draft");
  await expect(page.getByText("beta message")).toHaveCount(0);
});

test("keeps agent launch choices compact", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Start with Codex" }).click();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expect(page.getByText("Sign-in details")).toHaveCount(0);
  await expect(page.getByText("API key handling")).toHaveCount(0);
  await expect(page.getByText("Sign-in", { exact: true })).toBeVisible();
  await expect(page.getByText("Needs sign-in", { exact: true })).toBeVisible();
});

test("keeps the interactive run terminal accessible", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Start with Claude" }).click();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByText("Advanced options", { exact: true }).click();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-terminal");
  await page.getByLabel("Run in an isolated copy (git worktree)").check();
  await page.getByRole("button", { name: "Review access" }).click();
  await page.getByRole("button", { name: "Start run" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Run terminal" })).toBeVisible();
  await expect(page.getByText("The run terminal needs the desktop app.")).toBeVisible();
  await expectNoAccessibilityViolations(page);
});

test("cleans stale networks from System details in one click", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await page.getByRole("button", { name: "System details", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "System details" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  const removeButton = page.getByRole("button", { name: "Remove stale networks" });
  await expect(removeButton).toBeEnabled();
  await removeButton.click();
  await expect(page.getByText("Cleanup complete")).toBeVisible();
  await expect(page.getByText("Removed 1 RunHaven networks.")).toBeVisible();
});

test("reviews a finished worktree session with native-confirmed actions", async ({ page }) => {
  await page.goto("/");

  await startPreviewRun(page, "/tmp/runhaven-worktree", { worktree: true });

  // Stopping the run finishes the preview session with a saved isolated copy.
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(
    page.getByRole("heading", { name: "Review the isolated copy" }),
  ).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await expect(page.getByText("2 files changed in the isolated copy.")).toBeVisible();

  // Every action is ready to choose; Rust presents the native confirmation.
  const mergeButton = page.getByRole("button", { name: "Merge into project" });
  const keepButton = page.getByRole("button", { name: "Keep for later" });
  await expect(mergeButton).toBeEnabled();
  await expect(keepButton).toBeEnabled();
  await expect(page.getByRole("button", { name: "Recover by hand" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Discard changes" })).toBeEnabled();

  await keepButton.click();
  await expect(
    page.getByText("Kept the isolated copy so you can review it later."),
  ).toBeVisible();
  await expect(keepButton).toBeEnabled();

  // Discard removes the copy, so the review card closes and the session stays.
  await page.getByRole("button", { name: "Discard changes" }).click();
  await expect(
    page.getByRole("heading", { name: "Review the isolated copy" }),
  ).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
});

test("reaches a run from the full Run history view", async ({ page }) => {
  await page.goto("/");

  await startPreviewRun(page, "/tmp/runhaven-history");
  await page.getByRole("button", { name: "History" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Run history" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByText("Codex - runhaven-history").click();
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
});

async function startPreviewRun(
  page: Page,
  workspacePath: string,
  { worktree = false } = {},
) {
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
  await page.getByRole("button", { name: "Start with Codex" }).click();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByText("Advanced options", { exact: true }).click();
  await page.getByLabel("Project folder path").fill(workspacePath);
  if (worktree) {
    await page.getByLabel("Run in an isolated copy (git worktree)").check();
  } else {
    await page.getByLabel("Run in an isolated copy (git worktree)").uncheck();
  }
  await page.getByRole("button", { name: "Review access" }).click();
  if (!worktree) {
    await expectNoAccessibilityViolations(page);
    await page.getByLabel("Confirm direct-workspace warning").check();
  }
  await page.getByRole("button", { name: "Start run" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
}

async function selectAgent(page: Page, name: string) {
  await page.getByLabel("Agent").click();
  await page.getByRole("option", { name: new RegExp(name, "i") }).click();
}
