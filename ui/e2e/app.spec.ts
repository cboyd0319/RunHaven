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

  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
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
  await expect(page.getByText("RunHaven will use the shared codex login state across projects on this Mac.")).toBeVisible();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
  // Once the run shows up as a managed session, the "Run accepted" launch
  // alert clears instead of persisting forever.
  await expect(page.getByText(/Run accepted: preview-/)).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Codex - runhaven-preview" })).toBeVisible();
  await expect(page.getByText("Monitor session")).toBeVisible();
  await expect(page.getByText("Run record")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Message" })).toBeVisible();
  // The session view has its own Session diagnostics card; the app-wide
  // Diagnostics surface is a sidebar destination, never a second panel in the
  // session view's content pane.
  await expect(
    page.getByRole("main").getByText("Diagnostics", { exact: true }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Load session diagnostics" }).click();
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
  await expect(page.getByRole("heading", { name: "Session controls" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toBeDisabled();
  await page.getByLabel("Confirm stopping this session.").check();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(page.getByText(/Stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm hard-stopping this session.").check();
  await page.getByRole("button", { name: "Hard stop" }).click();
  await expect(page.getByText(/Hard stop requested for preview-/)).toBeVisible();

  await page.getByLabel("Confirm clearing this session's stale session record.").check();
  await page.getByRole("button", { name: "Repair session record" }).click();
  await expect(page.getByText(/Stale session record cleared for preview-/)).toBeVisible();
  await page.getByRole("button", { name: "Home" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();

  // Once the launched run is a managed session, the Start screen must not offer
  // a second control surface for it: only the one-line notice remains, and the
  // run's controls live in the per-session view.
  await expect(page.getByText("Run started - find it in your sidebar")).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop run" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Hard stop" })).toHaveCount(0);
  await page.getByRole("button", { name: "Open session" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
});

test("loads secret-free diagnostics", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Diagnostics" }).click();
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

  await page.getByRole("button", { name: "Home" }).click();
  await startPreviewRun(page, "/tmp/runhaven-beta");
  await expect(page.getByLabel("Message")).toHaveValue("");
  await page.getByLabel("Message").fill("beta message");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText("Message sent")).toBeVisible();

  await page.getByText("Codex - runhaven-alpha").click();
  await expect(page.getByLabel("Message")).toHaveValue("alpha draft");
  await expect(page.getByText("beta message")).toHaveCount(0);
});

test("signs in a preview agent from the Agent step", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Choose agent" })).toBeVisible();
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  // The preview sign-in surfaces the device link and code, then succeeds.
  await expect(
    page.getByText("Open https://example.com/device and enter CODE-1234."),
  ).toBeVisible();
  await expect(page.getByText("Signed in (preview)")).toBeVisible();
  await expect(page.getByText("Sign-in complete.")).toBeVisible();
});

test("runs the maintenance preview flows behind confirm gates", async ({ page }) => {
  await page.goto("/");

  // Image rebuild lives in the Agents destination and stays gated until the
  // user confirms.
  await page.getByRole("button", { name: "Agents" }).click();
  const rebuildButton = page.getByRole("button", { name: "Rebuild image" });
  await expect(rebuildButton).toBeDisabled();
  await page.getByLabel("Confirm agent image rebuild").check();
  await rebuildButton.click();
  await expect(page.getByText("Succeeded (preview)")).toBeVisible();
  await expect(page.getByText("Preview build output.")).toBeVisible();

  // Network cleanup lives in the Maintenance destination: it previews a count
  // first, then gates the prune.
  await page.getByRole("button", { name: "Maintenance" }).click();
  await page.getByRole("button", { name: "Check for stale networks" }).click();
  await expect(page.getByText(/Stale RunHaven networks found:/)).toBeVisible();
  const removeButton = page.getByRole("button", { name: "Remove stale networks" });
  await expect(removeButton).toBeDisabled();
  await page.getByLabel("Confirm network cleanup").check();
  await removeButton.click();
  await expect(page.getByText("Cleanup finished")).toBeVisible();
  await expect(page.getByText("Removed 1. Skipped 0 still in use.")).toBeVisible();
});

test("reviews a finished worktree session with per-action confirm gating", async ({ page }) => {
  await page.goto("/");

  await startPreviewRun(page, "/tmp/runhaven-worktree", { worktree: true });

  // Stopping the run finishes the preview session with a saved isolated copy.
  await page.getByLabel("Confirm stopping this session.").check();
  await page.getByRole("button", { name: "Stop run" }).click();
  await expect(
    page.getByRole("heading", { name: "Review the isolated copy" }),
  ).toBeVisible();
  await expect(page.getByText("2 files changed in the isolated copy.")).toBeVisible();

  // Every action is gated on its own confirmation checkbox.
  const mergeButton = page.getByRole("button", { name: "Merge into project" });
  const keepButton = page.getByRole("button", { name: "Keep for later" });
  await expect(mergeButton).toBeDisabled();
  await expect(keepButton).toBeDisabled();
  await expect(page.getByRole("button", { name: "Recover by hand" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Discard changes" })).toBeDisabled();

  // Confirming one action does not unlock the others, and confirmations reset
  // after the action runs.
  await page.getByLabel("Confirm keeping the isolated copy for now.").check();
  await expect(mergeButton).toBeDisabled();
  await keepButton.click();
  await expect(
    page.getByText("Kept the isolated copy so you can review it later."),
  ).toBeVisible();
  await expect(keepButton).toBeDisabled();

  // Discard removes the copy, so the review card closes and the session stays.
  await page.getByLabel("Confirm discarding the isolated copy and its changes.").check();
  await page.getByRole("button", { name: "Discard changes" }).click();
  await expect(
    page.getByRole("heading", { name: "Review the isolated copy" }),
  ).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
});

async function startPreviewRun(
  page: Page,
  workspacePath: string,
  { worktree = false } = {},
) {
  await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByLabel("Project folder path").fill(workspacePath);
  if (worktree) {
    await page.getByText("Advanced options").click();
    await page.getByLabel("Run in an isolated copy (git worktree)").check();
  }
  await page.getByRole("button", { name: "Review access" }).click();
  await page.getByLabel("I reviewed this plan and want to start this run.").check();
  await page.getByRole("button", { name: "Launch run" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Session" })).toBeVisible();
}
