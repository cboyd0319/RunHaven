import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { HelpView } from "../app/views/HelpView"

describe("HelpView", () => {
  it("sets honest expectations for generative AI and provider data", () => {
    const html = renderToStaticMarkup(
      <HelpView backLabel="New session" onBack={() => undefined} />,
    )

    expect(html).toContain("use generative AI")
    expect(html).toContain("can make mistakes")
    expect(html).toContain("Review their output and file changes")
    expect(html).toContain("provider’s services")
    expect(html).toContain("provider’s terms")
    expect(html).toContain("stores this content")
    expect(html).toContain("uses it for training")
  })
})
