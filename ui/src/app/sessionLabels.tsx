import { createContext, useContext, useState, type ReactNode } from "react"

import {
  loadSessionLabels,
  persistSessionLabels,
  sanitizeSessionLabel,
} from "./uiStatePersistence"

// GUI-only session display labels, kept entirely client-side. A label is a
// cosmetic name the user types for a session; it never renames the run's real
// volume identity key and never crosses a backend command, so it needs no
// capability and no native dialog. The map is keyed by the opaque session id
// and shared through context so the sidebar and the session view show the same
// name from one source of truth.
type SessionLabelsContextValue = {
  labels: Record<string, string>
  setLabel: (sessionId: string, label: string) => void
}

// A default value means components rendered without a provider (for example the
// read-only session monitor window, or a focused unit test) simply see no
// labels and fall back to the derived title, never crash.
const SessionLabelsContext = createContext<SessionLabelsContextValue>({
  labels: {},
  setLabel: () => {},
})

export function SessionLabelsProvider({ children }: { children: ReactNode }) {
  const [labels, setLabels] = useState<Record<string, string>>(loadSessionLabels)

  function setLabel(sessionId: string, label: string): void {
    const clean = sanitizeSessionLabel(label)
    setLabels((current) => {
      const next = { ...current }
      if (clean) {
        next[sessionId] = clean
      } else {
        // A blank label clears the override, so the row falls back to the
        // derived "Agent - project" title.
        delete next[sessionId]
      }
      persistSessionLabels(next)
      return next
    })
  }

  return (
    <SessionLabelsContext.Provider value={{ labels, setLabel }}>
      {children}
    </SessionLabelsContext.Provider>
  )
}

export function useSessionLabels(): SessionLabelsContextValue {
  return useContext(SessionLabelsContext)
}
