import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { Alert, AlertTitle } from "../components/ui/alert"
import { SidebarInset } from "../components/ui/sidebar"

describe("Apple HIG control semantics", () => {
  it("keeps static inline information out of the urgent alert channel", () => {
    const html = renderToStaticMarkup(
      <Alert>
        <AlertTitle>Live terminal</AlertTitle>
      </Alert>,
    )

    expect(html).not.toContain('role="alert"')
  })

  it("preserves an explicit live-region role for changing feedback", () => {
    const html = renderToStaticMarkup(
      <Alert role="status">
        <AlertTitle>Setup updated</AlertTitle>
      </Alert>,
    )

    expect(html).toContain('role="status"')
  })

  it("does not create a second main landmark around page content", () => {
    const html = renderToStaticMarkup(
      <SidebarInset>
        <main>Page content</main>
      </SidebarInset>,
    )

    expect(html.match(/<main/g) ?? []).toHaveLength(1)
  })
})
