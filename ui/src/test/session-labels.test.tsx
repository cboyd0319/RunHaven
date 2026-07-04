// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { beforeEach, describe, expect, it } from "vitest"

import { RecentActivityPanel } from "../app/dashboard/RecentActivityPanel"
import { SessionLabelsProvider } from "../app/sessionLabels"
import { sessionDisplayTitle } from "../app/sessionIdentity"
import { SessionView } from "../app/views/SessionView"
import type { ManagedSessionSummary } from "../commands/runhaven"

import { installLocalStorageStub } from "./local-storage"
import { mainViewProps, managedSession } from "./main-view-props"
import { act } from "./render-hook"

const SESSION_LABELS_KEY = "runhaven.sessionLabels"

installLocalStorageStub()

function seedLabels(labels: Record<string, string>): void {
  window.localStorage.setItem(SESSION_LABELS_KEY, JSON.stringify(labels))
}

function nativeInputValue(input: HTMLInputElement, value: string): void {
  const setter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    "value",
  )?.set
  setter?.call(input, value)
  input.dispatchEvent(new Event("input", { bubbles: true }))
}

describe("sessionDisplayTitle", () => {
  const session = managedSession({ id: "s1", title: "Codex - alpha" })

  it("uses the label when one is set", () => {
    expect(sessionDisplayTitle(session, { s1: "Login bug" })).toBe("Login bug")
  })

  it("falls back to the derived title when no label is set", () => {
    expect(sessionDisplayTitle(session, {})).toBe("Codex - alpha")
  })

  it("falls back when the label is blank", () => {
    expect(sessionDisplayTitle(session, { s1: "   " })).toBe("Codex - alpha")
  })
})

describe("session labels in the run tables", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("shows the label for a labeled run and the derived title otherwise", () => {
    seedLabels({ s1: "Login bug" })
    const sessions: ManagedSessionSummary[] = [
      managedSession({ id: "s1", title: "Codex - alpha" }),
      managedSession({
        id: "s2",
        title: "Codex - beta",
        startedAt: "2026-07-01T00:00:00Z",
      }),
    ]
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <RecentActivityPanel
          sessions={sessions}
          onSelectSession={() => undefined}
          onOpenHistory={() => undefined}
        />
      </SessionLabelsProvider>,
    )

    expect(html).toContain("Login bug")
    expect(html).not.toContain("Codex - alpha")
    expect(html).toContain("Codex - beta")
  })
})

describe("session labels in the session view", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("shows the label as the session title when set", () => {
    seedLabels({ s1: "Login bug" })
    const session = managedSession({ id: "s1", title: "Codex - alpha" })
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <SessionView
          {...mainViewProps({ selectedSession: session })}
          selectedSession={session}
        />
      </SessionLabelsProvider>,
    )

    expect(html).toContain("Login bug")
    expect(html).toContain("Rename session")
  })

  it("saves a typed label so the title updates and persists", async () => {
    const session = managedSession({ id: "s1", title: "Codex - alpha" })
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(
        createElement(
          SessionLabelsProvider,
          null,
          createElement(SessionView, {
            ...mainViewProps({ selectedSession: session }),
            selectedSession: session,
          }),
        ),
      )
    })

    const nameButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Name this session",
    )
    expect(nameButton).toBeDefined()
    await act(async () => {
      nameButton?.click()
    })

    const input = container.querySelector<HTMLInputElement>("#session-label-s1")
    expect(input).not.toBeNull()
    await act(async () => {
      if (input) {
        nativeInputValue(input, "Login bug")
      }
    })

    const form = input?.closest("form")
    await act(async () => {
      form?.dispatchEvent(
        new Event("submit", { bubbles: true, cancelable: true }),
      )
    })

    expect(container.textContent).toContain("Login bug")
    expect(
      JSON.parse(window.localStorage.getItem(SESSION_LABELS_KEY) ?? "{}"),
    ).toEqual({ s1: "Login bug" })

    await act(async () => {
      root.unmount()
    })
  })
})
