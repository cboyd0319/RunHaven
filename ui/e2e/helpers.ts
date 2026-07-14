import AxeBuilder from "@axe-core/playwright"

import { expect, type Page } from "./fixtures"

export async function expectNoAccessibilityViolations(page: Page) {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze()
  expect(results.violations).toEqual([])
}

export async function expectNoHorizontalOverflow(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true)
}

export async function expectNewSessionTypography(
  page: Page,
  sizes: { body: string; support: string; heading: string },
) {
  const progress = page.getByRole("list", { name: "Session setup progress" })
  await expect(
    page.getByText(/Choose an assistant and project, then check the access/),
  ).toHaveCSS("font-size", sizes.body)
  await expect(progress.getByText("Choose assistant", { exact: true })).toHaveCSS(
    "font-size",
    sizes.body,
  )
  await expect(progress.getByText(/Complete ·/)).toHaveCSS(
    "font-size",
    sizes.support,
  )
  await expect(
    page.getByRole("heading", { level: 2, name: "Choose assistant" }),
  ).toHaveCSS("font-size", sizes.heading)
}

export async function expectRunningTerminalFocus(page: Page) {
  await openProjectStep(page, "Claude", "/tmp/terminal-focus", true)
  await page.getByRole("button", { name: "Continue" }).click()
  await page.getByRole("button", { name: "Start session" }).click()
  await page.getByRole("button", { name: "New session", exact: true }).click()
  await page.getByRole("button", { name: /Claude · terminal-focus/ }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Claude · terminal-focus" }),
  ).toBeFocused()
  await page.keyboard.press("Tab")
  await expect(
    page.getByRole("button", { name: "Close", exact: true }),
  ).toBeFocused()
}

export async function openNewSession(
  page: Page,
  project?: string,
  terminal = false,
) {
  const params = new URLSearchParams()
  if (project) params.set("project", project)
  if (terminal) params.set("preview", "terminal")
  await page.goto(`/${params.size > 0 ? `?${params}` : ""}`)
  await expect(
    page.getByRole("heading", { level: 1, name: "New session" }),
  ).toBeVisible()
  await expect(
    page.getByRole("heading", { level: 2, name: "Choose assistant" }),
  ).toBeVisible()
}

export async function openProjectStep(
  page: Page,
  assistant: "Claude" | "Codex" | "Antigravity" | "Copilot",
  project: string,
  terminal = false,
) {
  await openNewSession(page, project, terminal)
  await page.getByRole("combobox", { name: "Assistant" }).click()
  await page.getByRole("option", { name: assistant, exact: true }).click()
  await page.getByRole("button", { name: "Continue" }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Choose project" }),
  ).toBeVisible()
  await expect(
    page.getByText(`Pick the project folder ${assistant} can use.`),
  ).toBeVisible()
  await page.getByRole("button", { name: "Choose folder" }).click()
  await expect(
    page.getByText(project.split("/").at(-1)!, { exact: true }),
  ).toBeVisible()
}

export async function startManagedSession(
  page: Page,
  project: string,
  { protectedCopy = true }: { protectedCopy?: boolean } = {},
) {
  await openProjectStep(page, "Codex", project)
  if (!protectedCopy) {
    await page.getByText("Advanced options", { exact: true }).click()
    await page.getByRole("radio", { name: /^Original folder with changes/ }).check()
  }
  await page.getByRole("button", { name: "Continue" }).click()
  await expect(
    page.getByRole("heading", { level: 2, name: "Review and start" }),
  ).toBeVisible()
  await expectNoHorizontalOverflow(page)
  await page.getByRole("button", { name: "Start session" }).click()
  const projectName = project.split("/").at(-1)!
  await expect(
    page.getByRole("heading", { level: 1, name: `Codex - ${projectName}` }),
  ).toBeVisible()
}
