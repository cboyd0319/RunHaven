import { expect, test } from "./fixtures"
import {
  expectNoAccessibilityViolations,
  expectNoHorizontalOverflow,
  expectNewSessionTypography,
  expectRunningTerminalFocus,
  openNewSession,
  openProjectStep,
  startManagedSession,
} from "./helpers"

test("New session uses wide windows and collapses before content gets cramped", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await openNewSession(page)

  const progress = page.getByRole("list", { name: "Session setup progress" })
  const currentStep = page.getByRole("region", {
    name: "Choose assistant step",
  })
  const wideProgress = await progress.boundingBox()
  const wideStep = await currentStep.boundingBox()
  expect(wideProgress).not.toBeNull()
  expect(wideStep).not.toBeNull()
  expect(wideProgress!.x + wideProgress!.width).toBeLessThanOrEqual(wideStep!.x)

  await page.setViewportSize({ width: 800, height: 560 })
  const narrowProgress = await progress.boundingBox()
  const narrowStep = await currentStep.boundingBox()
  expect(narrowProgress).not.toBeNull()
  expect(narrowStep).not.toBeNull()
  expect(narrowProgress!.y + narrowProgress!.height).toBeLessThanOrEqual(
    narrowStep!.y,
  )
  await expectNoHorizontalOverflow(page)

  await page.setViewportSize({ width: 1440, height: 900 })
  await page.evaluate(() => localStorage.setItem("runhaven.text-size", "200"))
  await page.reload()
  await expect(page.locator("html")).toHaveCSS("font-size", "32px")
  await expectNewSessionTypography(page, {
    body: "30px", support: "26px", heading: "34px",
  })
  const scaledProgress = await progress.boundingBox()
  const scaledStep = await currentStep.boundingBox()
  expect(scaledProgress).not.toBeNull()
  expect(scaledStep).not.toBeNull()
  expect(scaledProgress!.y + scaledProgress!.height).toBeLessThanOrEqual(
    scaledStep!.y,
  )
  await expectNoHorizontalOverflow(page)
})

test("200% text remains usable at the minimum window size", async ({
  page,
}) => {
  await page.setViewportSize({ width: 800, height: 560 })
  await openNewSession(page)
  await page.evaluate(() => localStorage.setItem("runhaven.text-size", "200"))
  await page.reload()

  await expect(page.locator("html")).toHaveCSS("font-size", "32px")
  await expect(page.locator('[data-slot="sidebar-container"]')).toHaveCSS(
    "width",
    "264px",
  )
  await expect(page.locator('[data-slot="sidebar-header"] img')).toHaveCSS(
    "width",
    "32px",
  )
  const mainBounds = await page.getByRole("main").boundingBox()
  expect(mainBounds?.width).toBeGreaterThanOrEqual(520)
  expect(
    await page
      .getByRole("button", { name: "New session", exact: true })
      .locator("span")
      .evaluate((label) => label.scrollWidth <= label.clientWidth),
  ).toBe(true)
  await expect(page.getByRole("button", { name: "Continue" })).toBeVisible()
  await expectNoHorizontalOverflow(page)

  await page.getByRole("button", { name: "History", exact: true }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "History" }),
  ).toBeVisible()
  await expect(
    page.getByRole("button", { name: "Start a session" }),
  ).toBeVisible()
  await expectNoHorizontalOverflow(page)

  await page.getByRole("button", { name: "System", exact: true }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "System" }),
  ).toBeVisible()
  await expect(page.getByRole("button", { name: "Check again" })).toBeVisible()
  await expectNoHorizontalOverflow(page)

  await startManagedSession(page, "/tmp/runhaven-large-text", {
    protectedCopy: false,
  })
  await expect(page.getByLabel("Message")).toBeVisible()
  await expect(page.getByRole("button", { name: "Show details" })).toBeVisible()
  await expectNoHorizontalOverflow(page)
})

test("every assistant choice opens its project step", async ({ page }) => {
  for (const assistant of [
    "Claude",
    "Codex",
    "Antigravity",
    "Copilot",
  ] as const) {
    await openProjectStep(
      page,
      assistant,
      `/tmp/runhaven-${assistant.toLowerCase()}`,
    )
    await expect(page.getByRole("combobox", { name: "Agent" })).toHaveCount(0)
    await expect(
      page.getByText("Project folder path", { exact: true }),
    ).toHaveCount(0)
    await expectNoHorizontalOverflow(page)
  }
})

test("Review keeps the launch boundary clear and editable", async ({ page }) => {
  const project = "/tmp/runhaven-review"
  await openProjectStep(page, "Codex", project)

  await expect(
    page.locator('[aria-label="Session setup progress"] [aria-current="step"]'),
  ).toContainText("Choose project")
  await page.getByRole("button", { name: "Continue" }).click()

  await expect(
    page.getByRole("heading", { level: 2, name: "Review and start" }),
  ).toBeVisible()
  await expect(
    page.getByRole("heading", { level: 2, name: "Review and start" }),
  ).toBeFocused()
  await expect(page.getByText("Review this session's access")).toBeVisible()
  await expect(page.getByText("runhaven-review", { exact: true })).toBeVisible()
  await expect(page.getByText(project, { exact: true })).toHaveCount(0)
  await expect(page.getByRole("button", { name: "Edit assistant" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Edit project" })).toBeVisible()
  await expectNoHorizontalOverflow(page)
  await expectNoAccessibilityViolations(page)

  await page.getByRole("button", { name: "Edit project" }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Choose project" }),
  ).toBeVisible()
  await expect(page.getByText("runhaven-review", { exact: true })).toBeVisible()
})

test("sidebar, appearance, and keyboard paths behave like Mac controls", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" })
  await openNewSession(page)
  await expect(page.locator("html")).toHaveClass(/dark/)

  const sidebarToggle = page.locator('[data-sidebar="trigger"]')
  const desktopSidebar = page.locator('[data-slot="sidebar"]')
  const sidebarContainer = page.locator('[data-slot="sidebar-container"]')
  await page.getByRole("button", { name: "New session", exact: true }).focus()
  await expect(sidebarToggle).toHaveAccessibleName("Hide Sidebar")
  await sidebarToggle.evaluate((button: HTMLElement) => button.click())
  await expect(desktopSidebar).toHaveAttribute("data-state", "collapsed")
  await expect(sidebarContainer).toHaveAttribute("aria-hidden", "true")
  await expect(sidebarContainer).toHaveAttribute("inert", "")
  await expect(sidebarToggle).toHaveAccessibleName("Show Sidebar")
  await expect(sidebarToggle).toBeFocused()
  await expectNoHorizontalOverflow(page)
  await sidebarToggle.click()
  await expect(desktopSidebar).toHaveAttribute("data-state", "expanded")

  await page.getByRole("combobox", { name: "Assistant" }).click()
  await page.getByRole("option", { name: "Codex", exact: true }).click()
  await page.getByRole("button", { name: "Continue" }).focus()
  await page.keyboard.press("Enter")
  await expect(
    page.getByRole("heading", { name: "Choose project" }),
  ).toBeVisible()
  const projectHeading = page.getByRole("heading", { level: 2, name: "Choose project" })
  await expect(projectHeading).toBeFocused()
  await expect(projectHeading).not.toHaveCSS("box-shadow", "none")
  await expectNoAccessibilityViolations(page)
})

test("saved-setup warning text stays readable in both appearances", async ({
  page,
}) => {
  for (const colorScheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme })
    await openNewSession(page)
    await page.locator('[data-slot="sidebar-header"]').evaluate((header) => {
      const warning = document.createElement("span")
      warning.className = "text-xs text-warning-text"
      warning.textContent = "Project folder can’t be found"
      header.append(warning)
    })
    await expect(
      page.getByText("Project folder can’t be found", { exact: true }),
    ).toBeVisible()
    await expectNoAccessibilityViolations(page)
  }
})

test("Advanced options keep safe defaults first and explain supported exceptions", async ({
  page,
}) => {
  await openProjectStep(page, "Codex", "/tmp/runhaven-advanced")
  await page.getByText("Advanced options", { exact: true }).click()

  await expect(page.getByText("Use a number such as 4 or 2.5.")).toBeVisible()
  await expect(
    page.getByText("Use a number and unit, such as 4g or 4096m."),
  ).toBeVisible()
  await expect(page.getByLabel("CPU cores")).toHaveAccessibleName("CPU cores")
  await expect(page.getByLabel("Memory")).toHaveAccessibleName("Memory")
  await page.getByLabel("CPU cores").fill("four")
  await page.getByLabel("Memory").fill("4 GB")
  await expect(page.getByLabel("CPU cores")).toHaveAccessibleName("CPU cores")
  await expect(page.getByLabel("Memory")).toHaveAccessibleName("Memory")
  await expect(page.getByLabel("CPU cores")).toHaveAttribute(
    "aria-invalid",
    "true",
  )
  await expect(page.getByLabel("Memory")).toHaveAttribute(
    "aria-invalid",
    "true",
  )
  await expect(
    page.getByText("Enter CPU cores as a number, such as 4 or 2.5."),
  ).toBeVisible()
  await expect(
    page.getByText("Enter a value such as 4g or 4096m."),
  ).toBeVisible()

  await expect(
    page.getByRole("radio", { name: /^Protected copy/ }),
  ).toBeChecked()
  await expect(
    page.getByText("Assistant services only", { exact: true }),
  ).toBeVisible()
  await page.getByRole("radio", { name: /^Local only/ }).check()
  await expect(page.getByText("This assistant will be offline")).toBeVisible()
  await expect(
    page.getByText(/Choose Assistant services only to let it connect safely/),
  ).toBeVisible()
  await expect(page.getByRole("button", { name: "Continue" })).toBeEnabled()

  await page.getByRole("radio", { name: /^Full internet/ }).check()
  await expect(page.getByText("Sign-in method changes")).toBeVisible()
  await expect(page.getByText("This assistant will be offline")).toHaveCount(0)
})

test("a managed session supports message, output, diagnostics, and controls", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-managed", {
    protectedCopy: false,
  })

  await page.getByLabel("Message").fill("hello session")
  await page.getByRole("button", { name: "Send…", exact: true }).click()
  await expect(page.getByText("Message sent", { exact: true })).toBeVisible()

  await page.getByRole("button", { name: "Show details" }).click()
  await page.getByRole("button", { name: "Show output…", exact: true }).click()
  await expect(page.getByText("Preview log line")).toBeVisible()
  await expect(page.getByText("/tmp/runhaven-managed")).toHaveCount(0)

  await expect(
    page.getByRole("heading", { name: "Runtime status" }),
  ).toBeVisible()
  await page.getByRole("button", { name: "Load session diagnostics" }).click()
  await expect(
    page.getByText("Sign-in method: API key from this Mac"),
  ).toBeVisible()
  await expect(page.getByText("api.openai.com")).toBeVisible()
  await expectNoAccessibilityViolations(page)

  await page.getByRole("button", { name: "Stop session" }).click()
  await expect(
    page.getByText("Stopping this session…", { exact: true }),
  ).toBeVisible()
})

test("protected-copy work is reviewable and removable from the session", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-worktree")
  await page.getByRole("button", { name: "Stop session" }).click()
  await expect(
    page.getByRole("heading", { name: "Review changes" }),
  ).toBeVisible()
  await expect(
    page.getByText("Assistant changes", { exact: true }),
  ).toBeVisible()
  await expect(page.getByText("src/app.ts", { exact: true })).toBeVisible()
  await expect(page.getByText("src/session.ts", { exact: true })).toBeVisible()
  await expect(
    page.getByRole("button", { name: "Apply changes" }),
  ).toBeEnabled()
  await expect(
    page.getByRole("button", { name: "Discard changes" }),
  ).toBeEnabled()
  await expect(
    page.getByRole("button", { name: "Show recovery steps" }),
  ).toHaveCount(0)
  await expectNoAccessibilityViolations(page)
  await page.emulateMedia({ colorScheme: "dark" })
  await expect(page.locator("html")).toHaveClass(/dark/)
  await expect
    .poll(() =>
      page.evaluate(() =>
        getComputedStyle(document.documentElement)
          .getPropertyValue("--destructive")
          .trim(),
      ),
    )
    .toBe("oklch(0.577 0.245 27.325)")
  await page.mouse.move(0, 0)
  await page.waitForTimeout(250)
  await expectNoAccessibilityViolations(page)

  await page.getByRole("button", { name: "Discard changes" }).click()
  await expect(
    page.getByRole("heading", { name: "Review changes" }),
  ).toHaveCount(0)
})

test("History owns session state and opens a session with the keyboard", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-history", {
    protectedCopy: false,
  })
  await page.getByRole("button", { name: "History", exact: true }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "History" }),
  ).toBeVisible()
  const rowButton = page.getByRole("button", {
    name: /Codex - runhaven-history/,
  })
  expect(
    await rowButton.evaluate((button) =>
      button.dispatchEvent(
        new MouseEvent("contextmenu", { bubbles: true, cancelable: true }),
      ),
    ),
  ).toBe(false)
  await rowButton.focus()
  await page.keyboard.press("Enter")
  await expect(
    page.getByRole("heading", { level: 1, name: "Codex - runhaven-history" }),
  ).toBeVisible()
  await expectNoAccessibilityViolations(page)
})

test("System keeps direct fixes visible and technical maintenance disclosed", async ({
  page,
}) => {
  await openNewSession(page)
  await page.getByRole("button", { name: "System", exact: true }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "System" }),
  ).toBeVisible()
  await expect(page.getByText("Your Mac is ready for RunHaven.")).toBeVisible()
  await page.getByText("Advanced system tools", { exact: true }).click()
  await expect(
    page.getByRole("button", { name: "Restart container service" }),
  ).toBeVisible()
  await expect(
    page.getByRole("button", { name: "Remove old networks" }),
  ).toBeEnabled()
  await page.getByRole("button", { name: "Restart container service" }).click()
  await expect(
    page.getByText("Desktop app required", { exact: true }),
  ).toBeVisible()
  await expect(
    page.getByText("Internet service refreshed", { exact: true }),
  ).toHaveCount(0)
  await page.getByRole("button", { name: "Remove old networks" }).click()
  await expect(
    page.getByText("Cleanup complete", { exact: true }),
  ).toBeVisible()
  await expectNoAccessibilityViolations(page)
})

test("the explicit terminal fixture keeps the interactive terminal reachable", async ({
  page,
}) => {
  await openProjectStep(page, "Claude", "/tmp/runhaven-terminal", true)
  await page.getByRole("button", { name: "Continue" }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Review and start" }),
  ).toBeVisible()
  await page.getByRole("button", { name: "Start session" }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "Claude · runhaven-terminal" }),
  ).toBeVisible()
  await expect(
    page.getByRole("button", { name: "Close", exact: true }),
  ).toBeVisible()
  await expect(page.getByText(/needs the desktop app/)).toBeVisible()
  await expectNoHorizontalOverflow(page)
  await expectNoAccessibilityViolations(page)
})

test("a Running row returns focus to its terminal before the controls", async ({
  page,
}) => {
  await expectRunningTerminalFocus(page)
})
