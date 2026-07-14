import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { HelpView } from "../app/views/HelpView"

describe("HelpView", () => {
  it("sets honest expectations for generative AI and provider data", () => {
    const html = renderToStaticMarkup(
      <HelpView
        backLabel="New session"
        onBack={() => undefined}
        supportReport={{
          busy: false,
          message: "",
          error: "",
          create: async () => undefined,
        }}
      />,
    )

    expect(html).toContain("use generative AI")
    expect(html).toContain("can make mistakes")
    expect(html).toContain("Review their output and file changes")
    expect(html).toContain("provider’s services")
    expect(html).toContain("provider’s terms")
    expect(html).toContain("stores this content")
    expect(html).toContain("uses it for training")
    expect(html).toContain("Start with recommended settings")
    expect(html).not.toContain("Start safely")
    expect(html).toContain("Get support")
    expect(html).toContain("RunHaven does not upload the report")
    expect(html).toContain("Get Support…")
  })

  it("announces support success and failure with distinct live-region roles", () => {
    const html = renderToStaticMarkup(
      <HelpView
        backLabel="New session"
        onBack={() => undefined}
        supportReport={{
          busy: false,
          message: "Report created.",
          error: "Finder did not open.",
          create: async () => undefined,
        }}
      />,
    )

    expect(html).toContain('role="status"')
    expect(html).toContain('role="alert"')
    expect(html).not.toContain('aria-live="polite"')
  })
})
