import AxeBuilder from "@axe-core/playwright"
import { expect, test, type Page } from "./fixtures"

async function expectNoAccessibilityViolations(page: Page) {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze()
  expect(results.violations).toEqual([])
}

async function expectNoHorizontalOverflow(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true)
}

async function openHome(page: Page, project?: string, terminal = false) {
  const params = new URLSearchParams()
  if (project) params.set("project", project)
  if (terminal) params.set("preview", "terminal")
  await page.goto(`/${params.size > 0 ? `?${params}` : ""}`)
  await expect(
    page.getByRole("heading", { level: 1, name: "Start a session" }),
  ).toBeVisible()
}

async function openProjectStep(
  page: Page,
  assistant: "Claude" | "Codex" | "Antigravity" | "Copilot",
  project: string,
  terminal = false,
) {
  await openHome(page, project, terminal)
  await page.getByRole("button", { name: assistant, exact: true }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Choose project" }),
  ).toBeVisible()
  await expect(page.getByText(`Pick the project folder ${assistant} can use.`)).toBeVisible()
  await page.getByRole("button", { name: "Choose folder" }).click()
  await expect(page.getByText(project.split("/").at(-1)!, { exact: true })).toBeVisible()
}

async function startManagedSession(
  page: Page,
  project: string,
  { protectedCopy = true }: { protectedCopy?: boolean } = {},
) {
  await openProjectStep(page, "Codex", project)
  if (!protectedCopy) {
    await page.getByText("Advanced options", { exact: true }).click()
    await page.getByRole("radio", { name: /^Original folder with changes/ }).check()
  }
  await page.getByRole("button", { name: "Start session" }).click()
  const projectName = project.split("/").at(-1)!
  await expect(
    page.getByRole("heading", { level: 1, name: `Codex - ${projectName}` }),
  ).toBeVisible()
}

test("Home is a focused one-click launchpad at minimum and normal sizes", async ({
  page,
}) => {
  for (const viewport of [
    { width: 800, height: 560 },
    { width: 1080, height: 720 },
  ]) {
    await page.setViewportSize(viewport)
    await openHome(page)
    for (const assistant of ["Claude", "Codex", "Antigravity", "Copilot"]) {
      await expect(
        page.getByRole("button", { name: assistant, exact: true }),
      ).toBeVisible()
    }
    await expect(
      page.getByText("Choose the assistant account you already use."),
    ).toBeVisible()
    await page.getByRole("button", { name: "Not sure?" }).click()
    await expect(
      page.getByRole("heading", { level: 1, name: "RunHaven Help" }),
    ).toBeVisible()
    await expect(page.getByRole("heading", { name: "Start safely" })).toBeVisible()
    await expect(page.getByText("Needs attention", { exact: true })).toHaveCount(0)
    await expect(page.getByText("Recent activity", { exact: true })).toHaveCount(0)
    await expect(page.getByText("Run history", { exact: true })).toHaveCount(0)
    await expectNoHorizontalOverflow(page)
    await expectNoAccessibilityViolations(page)
  }
})

test("every Home choice opens its project step without asking again", async ({
  page,
}) => {
  for (const assistant of [
    "Claude",
    "Codex",
    "Antigravity",
    "Copilot",
  ] as const) {
    await openProjectStep(page, assistant, `/tmp/runhaven-${assistant.toLowerCase()}`)
    await expect(page.getByRole("combobox", { name: "Agent" })).toHaveCount(0)
    await expect(page.getByText("Project folder path", { exact: true })).toHaveCount(0)
    await expectNoHorizontalOverflow(page)
  }
})

test("sidebar, appearance, and keyboard paths behave like Mac controls", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" })
  await openHome(page)
  await expect(page.locator("html")).toHaveClass(/dark/)

  const sidebarToggle = page.locator('[data-sidebar="trigger"]')
  const desktopSidebar = page.locator('[data-slot="sidebar"]')
  const sidebarContainer = page.locator('[data-slot="sidebar-container"]')
  await page.getByRole("button", { name: "Home", exact: true }).focus()
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

  const codex = page.getByRole("button", { name: "Codex", exact: true })
  await codex.focus()
  await page.keyboard.press("Enter")
  await expect(page.getByRole("heading", { name: "Choose project" })).toBeVisible()
  await expectNoAccessibilityViolations(page)
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
  await expect(page.getByLabel("CPU cores")).toHaveAttribute("aria-invalid", "true")
  await expect(page.getByLabel("Memory")).toHaveAttribute("aria-invalid", "true")
  await expect(
    page.getByText("Enter CPU cores as a number, such as 4 or 2.5."),
  ).toBeVisible()
  await expect(page.getByText("Enter a value such as 4g or 4096m.")).toBeVisible()

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
  await expect(page.getByRole("button", { name: "Start session" })).toBeEnabled()

  await page.getByRole("radio", { name: /^Full internet/ }).check()
  await expect(page.getByText("Sign-in method changes")).toBeVisible()
  await expect(page.getByText("This assistant will be offline")).toHaveCount(0)
})

test("a managed session supports message, output, diagnostics, and controls", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-managed", { protectedCopy: false })

  await page.getByLabel("Message").fill("hello session")
  await page.getByRole("button", { name: "Send", exact: true }).click()
  await expect(page.getByText("Message sent", { exact: true })).toBeVisible()

  await page.getByRole("button", { name: "Show details" }).click()
  await page.getByRole("button", { name: "Show latest output" }).click()
  await expect(page.getByText("Preview log line")).toBeVisible()
  await expect(page.getByText("/tmp/runhaven-managed")).toHaveCount(0)

  await expect(page.getByRole("heading", { name: "Runtime status" })).toBeVisible()
  await page.getByRole("button", { name: "Load session diagnostics" }).click()
  await expect(page.getByText("Sign-in method: API key from this Mac")).toBeVisible()
  await expect(page.getByText("api.openai.com")).toBeVisible()
  await expectNoAccessibilityViolations(page)

  await page.getByRole("button", { name: "Stop session" }).click()
  await expect(page.getByText("Stopping this session…", { exact: true })).toBeVisible()
})

test("protected-copy work is reviewable and removable from the session", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-worktree")
  await page.getByRole("button", { name: "Stop session" }).click()
  await expect(page.getByRole("heading", { name: "Review changes" })).toBeVisible()
  await expect(page.getByText("Assistant changes", { exact: true })).toBeVisible()
  await expect(page.getByText("src/app.ts", { exact: true })).toBeVisible()
  await expect(page.getByText("src/session.ts", { exact: true })).toBeVisible()
  await expect(page.getByRole("button", { name: "Apply changes" })).toBeEnabled()
  await expect(page.getByRole("button", { name: "Discard changes" })).toBeEnabled()
  await expect(page.getByRole("button", { name: "Show recovery steps" })).toHaveCount(0)
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
    .toBe("#ff453a")
  await page.mouse.move(0, 0)
  await page.waitForTimeout(250)
  await expectNoAccessibilityViolations(page)

  await page.getByRole("button", { name: "Discard changes" }).click()
  await expect(page.getByRole("heading", { name: "Review changes" })).toHaveCount(0)
})

test("History owns session state and opens a session with the keyboard", async ({
  page,
}) => {
  await startManagedSession(page, "/tmp/runhaven-history", { protectedCopy: false })
  await page.getByRole("button", { name: "History", exact: true }).click()
  await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible()
  const rowButton = page.getByRole("button", { name: /Codex - runhaven-history/ })
  expect(
    await rowButton.evaluate((button) =>
      button.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true })),
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
  await openHome(page)
  await page.getByRole("button", { name: "System", exact: true }).click()
  await expect(page.getByRole("heading", { level: 1, name: "System" })).toBeVisible()
  await expect(page.getByText("Your Mac is ready for RunHaven.")).toBeVisible()
  await page.getByText("Advanced system tools", { exact: true }).click()
  await expect(page.getByRole("button", { name: "Restart container service" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Remove old networks" })).toBeEnabled()
  await page.getByRole("button", { name: "Restart container service" }).click()
  await expect(page.getByText("Desktop app required", { exact: true })).toBeVisible()
  await expect(page.getByText("Internet service refreshed", { exact: true })).toHaveCount(0)
  await page.getByRole("button", { name: "Remove old networks" }).click()
  await expect(page.getByText("Cleanup complete", { exact: true })).toBeVisible()
  await expectNoAccessibilityViolations(page)
})

test("the explicit terminal fixture keeps the interactive terminal reachable", async ({
  page,
}) => {
  await openProjectStep(page, "Claude", "/tmp/runhaven-terminal", true)
  await page.getByRole("button", { name: "Start session" }).click()
  await expect(
    page.getByRole("heading", { level: 1, name: "Session terminal" }),
  ).toBeVisible()
  await expect(page.getByRole("button", { name: "Close", exact: true })).toBeVisible()
  await expect(page.getByText(/needs the desktop app/)).toBeVisible()
  await expectNoHorizontalOverflow(page)
  await expectNoAccessibilityViolations(page)
})
