import { expect, test } from "./fixtures"
import {
  expectNoAccessibilityViolations,
  expectNoHorizontalOverflow,
  expectNewSessionTypography,
  openNewSession,
  openProjectStep,
} from "./helpers"

test("New session is focused at minimum and normal sizes", async ({ page }) => {
  for (const viewport of [
    { width: 800, height: 560 },
    { width: 1080, height: 720 },
  ]) {
    await page.setViewportSize(viewport)
    await openNewSession(page)
    await expect(
      page.getByRole("combobox", { name: "Assistant" }),
    ).toBeVisible()
    await expect(
      page.getByText("Pick the coding assistant you want to use."),
    ).toBeVisible()
    await expectNewSessionTypography(page, {
      body: "15px", support: "13px", heading: "17px",
    })
    await expect(
      page.getByText("Needs attention", { exact: true }),
    ).toHaveCount(0)
    await expect(
      page.getByText("Recent activity", { exact: true }),
    ).toHaveCount(0)
    await expect(page.getByText("Run history", { exact: true })).toHaveCount(0)
    await expectNoHorizontalOverflow(page)
    await expectNoAccessibilityViolations(page)
  }
})

test("standalone error text is accessible in dark appearance", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" })
  await openProjectStep(page, "Codex", "/tmp/runhaven-error-contrast")
  await page.getByText("Advanced options", { exact: true }).click()
  await page.getByRole("textbox", { name: "Memory" }).fill("invalid")
  await expect(page.getByText("Enter a value such as 4g or 4096m.")).toBeVisible()
  await expectNoAccessibilityViolations(page)
})
