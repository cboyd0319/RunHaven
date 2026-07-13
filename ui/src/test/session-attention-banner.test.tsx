import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionAttentionBanner } from "../app/SessionAttentionBanner"

import { managedSession } from "./main-view-props"

const onNavigate = () => undefined
const onShowOutput = () => undefined

describe("SessionAttentionBanner", () => {
  it("renders nothing for a quiet session", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession()}
        onNavigate={onNavigate}
        onShowOutput={onShowOutput}
      />,
    )
    expect(html).toBe("")
  })

  it("leads a failed session with its reason and direct recovery actions", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({ lifecycle: "failed" })}
        onNavigate={onNavigate}
        onShowOutput={onShowOutput}
      />,
    )
    expect(html).toContain("Session ended")
    expect(html).toContain("RunHaven couldn’t determine why this session ended.")
    expect(html).toContain("Show output")
    expect(html).toContain("Check System")
    expect(html).not.toContain("Start a new session")
    expect(html).not.toContain("Show error details")
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
        onShowOutput={onShowOutput}
      />,
    )
    // Failed outranks not-responding: one banner, highest state wins.
    expect(html).toContain("Session ended")
    expect(html).toContain("The container exited early.")
    expect(html).not.toContain("Not responding")
  })

  it("removes the output action after past output is known unavailable", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({ lifecycle: "failed" })}
        onNavigate={onNavigate}
        onShowOutput={onShowOutput}
        outputError="RunHaven keeps session output only until the app closes. Output for this run is no longer available."
      />,
    )

    expect(html).not.toContain("Show output")
    expect(html).toContain("Open System")
    expect(html).toContain("Check System")
  })

  it("offers one contextual sign-in fix", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({ authStatus: "needs-sign-in" })}
        onNavigate={onNavigate}
        onShowOutput={onShowOutput}
      />,
    )
    expect(html).toContain("Sign in to continue")
    expect(html).toContain("Sign in to Codex")
    expect(html).not.toContain("Agents")
    expect(html).toContain(">Sign in<")
  })

  it("names the repair control for a not-responding session", () => {
    const html = renderToStaticMarkup(
      <SessionAttentionBanner
        session={managedSession({
          nextAction: { kind: "recover", label: "Repair", detail: "" },
        })}
        onNavigate={onNavigate}
        onShowOutput={onShowOutput}
        onRepair={() => undefined}
      />,
    )
    expect(html).toContain("Session ended unexpectedly")
    expect(html).toContain("RunHaven can no longer find this session.")
    expect(html).toContain("Remove from active list")
    expect(html).not.toContain("More actions")
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
        onShowOutput={onShowOutput}
      />,
    )
    expect(html).toContain("Ready to review")
    expect(html).toContain("Review the assistant&#x27;s changes below")
  })
})
