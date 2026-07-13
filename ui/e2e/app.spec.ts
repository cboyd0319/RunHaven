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
  await expect(page.getByRole("button", { name: "New run" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "New run" })).toHaveCSS("opacity", "1");
  await expectNoAccessibilityViolations(page);

  await page.getByRole("button", { name: "New run" }).click();
  const newRunDialog = page.getByRole("dialog", { name: "New run" });
  await expect(newRunDialog).toBeVisible();
  await expect(newRunDialog).toHaveCSS("opacity", "1");
  await expectNoAccessibilityViolations(page);
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: "Agents", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Agents" })).toBeVisible();
  await expectNoAccessibilityViolations(page);

  await page.getByRole("button", { name: "System Check", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "System Check" })).toBeVisible();
  await expectNoAccessibilityViolations(page);

  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveClass(/dark/);
  await expectNoAccessibilityViolations(page);

  await page.getByRole("button", { name: "Settings", exact: true }).click();
  const settingsDialog = page.getByRole("dialog", { name: "Settings" });
  await expect(settingsDialog).toBeVisible();
  await expect(settingsDialog).toHaveCSS("opacity", "1");
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

  // Home is the welcome surface: setup health, agent readiness, and the one
  // prominent New run action. The wizard is not inline anymore.
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
  await expect(page.getByText("Ready to run", { exact: true })).toBeVisible();
  await expect(page.locator("header").getByText("Ready", { exact: true })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Setup ready" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Choose agent" })).toHaveCount(0);

  // The launch flow lives in a modal sheet.
  await page.getByRole("button", { name: "New run" }).click();
  const sheet = page.getByRole("dialog", { name: "New run" });
  await expect(sheet).toBeVisible();
  await expect(sheet.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expect(sheet.getByText("Sign-in method")).toHaveCount(0);
  await expect(sheet.getByText("API key handling")).toHaveCount(0);

  // Escape cancels with no side effects: the sheet closes, focus returns to
  // the New run button, and entered values survive for the next open.
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByText("Advanced options", { exact: true }).click();
  await page.getByLabel("Project folder path").fill("/tmp/runhaven-draft");
  await page.keyboard.press("Escape");
  await expect(sheet).toBeHidden();
  await expect(page.getByRole("button", { name: "New run" })).toBeFocused();
  await page.getByRole("button", { name: "New run" }).click();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByText("Advanced options", { exact: true }).click();
  await expect(page.getByLabel("Project folder path")).toHaveValue("/tmp/runhaven-draft");
  await page.getByRole("button", { name: "Back" }).click();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(sheet).toBeHidden();

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
  await expect(page.getByRole("button", { name: "New run" })).toBeVisible();

  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveClass(/dark/);
  await page.getByRole("button", { name: "New run" }).click();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
  const dialogBox = await page.getByRole("dialog", { name: "New run" }).boundingBox();
  const continueBox = await page.getByRole("button", { name: "Continue" }).boundingBox();
  expect(dialogBox).not.toBeNull();
  expect(continueBox).not.toBeNull();
  expect(continueBox!.y + continueBox!.height).toBeLessThanOrEqual(
    dialogBox!.y + dialogBox!.height,
  );
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

    await page.getByRole("button", { name: "Agents", exact: true }).click();
    for (const name of [
      "Claude Signed in",
      "Codex Needs sign-in",
      "Antigravity Needs sign-in",
      "Copilot Needs sign-in",
    ]) {
      await page.getByRole("button", { name, exact: true }).click();
      await expectNoHorizontalOverflow(page);
    }

    await page.getByRole("button", { name: "System Check", exact: true }).click();
    await page.getByRole("tab", { name: "Network", exact: true }).click();
    await expectNoHorizontalOverflow(page);
    await page.getByRole("button", { name: "Cleanup", exact: true }).click();
    await expectNoHorizontalOverflow(page);

    await page.getByRole("button", { name: "Needs you", exact: true }).click();
    const filters = page.getByRole("group", { name: "Filter run history" });
    for (const name of ["Running", "Past", "All"]) {
      await filters.getByRole("button", { name, exact: true }).click();
    }
    await expectNoHorizontalOverflow(page);

    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await expect(page.getByRole("dialog", { name: "Settings" })).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await page.getByRole("button", { name: "Close", exact: true }).click();

    await page.getByRole("button", { name: "Home", exact: true }).click();
    await page.getByRole("button", { name: "New run", exact: true }).click();
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

  await page.getByRole("button", { name: "New run" }).click();
  await selectAgent(page, "Codex");
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
  await expect(page.getByText("Auth state: api-key-broker")).toBeVisible();
  await expect(page.getByText("api.openai.com")).toBeVisible();
  await expect(page.getByRole("button", { name: "Send" })).toBeDisabled();
  await page.getByLabel("Message").fill("draft for selected session");
  await page.getByRole("button", { name: "Home" }).click();
  await page.getByText("Codex - runhaven-preview").click();
  await expect(page.getByLabel("Message")).toHaveValue("draft for selected session");
  await page.getByLabel("Message").fill("hello session");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();
  await expect(page.getByText(/Sent \d+ bytes\./)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Latest output" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Load latest output" })).toBeDisabled();
  await expect(page.getByLabel("Keep output updated")).toBeDisabled();
  await page.getByLabel("I understand raw output may include private data.").check();
  await page.getByRole("button", { name: "Load latest output" }).click();
  await expect(page.getByText("Preview log line")).toBeVisible();
  await page.getByLabel("Keep output updated").check();
  await expect(page.getByText("Updates every few seconds.")).toBeVisible();
  await expect(page.getByText("Preview log line 2")).toBeVisible();
  await expect(page.getByText("/tmp/runhaven-preview")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Run actions" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toBeEnabled();
  await expect(page.getByText("More actions", { exact: true })).toBeVisible();
  await page.getByText("More actions", { exact: true }).click();
  await expect(page.getByRole("button", { name: "Hard stop" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Repair run record" })).toBeEnabled();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();
  await page.getByRole("button", { name: "Home" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();

  // Once the launched run is a managed session, the Start screen must not offer
  // a second control surface for it: only the one-line notice remains, and the
  // run's controls live in the per-session view.
  await expect(page.getByText("Run started. Find it in Recent activity.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Hard stop" })).toHaveCount(0);
  await page.getByRole("button", { name: "Open session" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Run details" })).toBeVisible();
});

test("loads secret-free diagnostics on entry", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "System Check", exact: true }).click();
  // The destination is read-only, so it loads by itself: full doctor detail
  // plus the trust checks, with no manual load button.
  await expect(page.getByRole("heading", { name: "Setup checks" })).toBeVisible();
  await expect(page.getByText("Preview mode")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Trust checks" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Load diagnostics" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Auth broker status" })).toBeVisible();
  await expect(page.getByText("codex: api-key-broker")).toBeVisible();
  await page.getByRole("tab", { name: "Network" }).click();
  await expect(page.getByText("No provider egress decisions recorded.")).toBeVisible();
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

  // Variant 06 lists sessions in Home's Recent activity, not the sidebar, so
  // switching sessions routes through Home.
  await page.getByRole("button", { name: "Home" }).click();
  await page.getByText("Codex - runhaven-alpha").click();
  await expect(page.getByLabel("Message")).toHaveValue("alpha draft");
  await expect(page.getByText("beta message")).toHaveCount(0);
});

test("keeps ready-agent launch choices compact", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "New run" }).click();
  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await expect(page.getByText("Sign-in details")).toHaveCount(0);
  await expect(page.getByText("API key handling")).toHaveCount(0);
  await expect(
    page.getByRole("dialog").getByText("Signed in", { exact: true }),
  ).toBeVisible();
});

test("keeps the interactive run terminal accessible", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "New run" }).click();
  await page.getByLabel("Agent").click();
  await page.getByRole("option", { name: /Claude/ }).click();
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

test("runs repair and cleanup through native-confirmed actions", async ({ page }) => {
  await page.goto("/");

  // Agents uses a compact list with one detail panel. Repair stays behind a
  // disclosure and Rust owns the confirmation.
  await page.getByRole("button", { name: "Agents", exact: true }).click();
  await expect(
    page.getByRole("alert").filter({ hasText: "Agent ready" }).first(),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Available agents" })).toBeVisible();
  await page.getByText("Repair agent installation", { exact: true }).click();
  const rebuildButton = page.getByRole("button", { name: "Reinstall agent" });
  await expect(rebuildButton).toBeEnabled();
  await rebuildButton.click();
  await expect(page.getByText("Succeeded (preview)")).toBeVisible();
  await expect(page.getByText("Preview build output.")).toBeVisible();

  // Network cleanup previews a count, then the action uses native confirmation.
  await page.getByRole("button", { name: "Cleanup" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Cleanup" })).toBeVisible();
  await expectNoAccessibilityViolations(page);
  await page.getByRole("button", { name: "Check for stale networks" }).click();
  await expect(page.getByText(/Stale RunHaven networks found:/)).toBeVisible();
  const removeButton = page.getByRole("button", { name: "Remove stale networks" });
  await expect(removeButton).toBeEnabled();
  await removeButton.click();
  await expect(page.getByText("Cleanup finished")).toBeVisible();
  await expect(page.getByText("Removed 1.")).toBeVisible();
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
  // The sidebar lists no individual sessions, so the run is reached through the
  // Run history view opened from the Runs counts. A running run sits under Running.
  await page.getByRole("button", { name: "Running" }).click();
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
  await page.getByRole("button", { name: "New run" }).click();
  await selectAgent(page, "Codex");
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
