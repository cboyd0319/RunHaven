import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { LaunchStatusCard } from "../app/LaunchStatusCard"

describe("LaunchStatusCard", () => {
  it("renders backend-owned launch progress", () => {
    const html = renderToStaticMarkup(
      <LaunchStatusCard
        status={{
          runId: "launch-card-run",
          state: "starting",
          label: "Starting",
          detail: "Waiting for the managed session.",
          updatedAt: "2026-07-01T00:00:00Z",
        }}
        loading={false}
        error=""
      />,
    )

    expect(html).toContain("Launch status")
    expect(html).toContain("Starting")
    expect(html).toContain("Waiting for the managed session.")
    expect(html).not.toContain("/Users/")
  })
})
