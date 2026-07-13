import { useState } from "react"

import {
  loadSettings,
  persistSettings,
  type AppSettings,
} from "./uiStatePersistence"

export type SettingsState = ReturnType<typeof useSettings>

// State for the one native View-menu preference. The choice persists through
// the sanitized uiStatePersistence allowed set and never changes a security
// default or suppresses a confirmation.
export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(loadSettings)

  function update(patch: Partial<AppSettings>) {
    const next = { ...settings, ...patch }
    persistSettings(next)
    setSettings(next)
  }

  return {
    dockBadge: settings.dockBadge,
    setDockBadge: (dockBadge: boolean) => update({ dockBadge }),
  }
}
