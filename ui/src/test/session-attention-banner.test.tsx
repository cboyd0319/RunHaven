import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionAttentionBanner } from "../app/SessionAttentionBanner"

import { managedSession } from "./main-view-props"

const onNavigate = () => undefined

describe("SessionAttentionBanner", () => {
  it("renders nothing for a quiet session", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession()}
        onNavigate={onNavigate}
      />,
    )
    expect(html).toBe("")
  })

  it("leads a failed run with the failure and points at output", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({ lifecycle: "failed" })}
        onNavigate={onNavigate}
      />,
    )
    expect(html).toContain("This run failed")
    expect(html).toContain("Check the latest output below")
  })

  it("prefers the backend fix detail for a failed run when present", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({
          lifecycle: "failed",
          nextAction: {
            kind: "recover",
            label: "Recover",
            detail: "The container exited early.",
          },
        })}
        onNavigate={onNavigate}
      />,
    )
    // Failed outranks not-responding: one banner, highest state wins.
    expect(html).toContain("This run failed")
    expect(html).toContain("The container exited early.")
    expect(html).not.toContain("Not responding")
  })

  it("offers the Agents view as the sign-in fix", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({ authStatus: "needs-sign-in" })}
        onNavigate={onNavigate}
      />,
    )
    expect(html).toContain("Needs sign-in")
    expect(html).toContain("Sign in to Codex in Agents")
    expect(html).toContain("Open Agents")
  })

  it("names the repair control for a not-responding session", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({
          nextAction: { kind: "recover", label: "Repair", detail: "" },
        })}
        onNavigate={onNavigate}
      />,
    )
    expect(html).toContain("Not responding")
    expect(html).toContain("Repair session record")
  })

  it("points a finished worktree run at the review below", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({
          lifecycle: "finished",
          cleanup: {
            available: true,
            target: "worktree",
            label: "Delete worktree",
            detail: "Removes the saved isolated copy.",
          },
        })}
        onNavigate={onNavigate}
      />,
    )
    expect(html).toContain("Ready to review")
    expect(html).toContain("Review the isolated copy below")
  })
})
