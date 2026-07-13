import { useState } from "react"

import {
  loadSettings,
  persistSettings,
  type AppSettings,
} from "./uiStatePersistence"

export type SettingsState = ReturnType<typeof useSettings>

// State for the small inline Settings panel. It opens from the sidebar or
// RunHaven > Settings... (Cmd-,);
// choices persist through the sanitized uiStatePersistence allowed set and
// hold only convenience preferences, never security defaults or confirmation
// suppression.
export function useSettings() {
  const [open, setOpen] = useState(false)
  const [settings, setSettings] = useState<AppSettings>(loadSettings)

  function update(patch: Partial<AppSettings>) {
    const next = { ...settings, ...patch }
    persistSettings(next)
    setSettings(next)
  }

  return {
    open,
    setOpen,
    dockBadge: settings.dockBadge,
    defaultAgent: settings.defaultAgent,
    setDockBadge: (dockBadge: boolean) => update({ dockBadge }),
    setDefaultAgent: (defaultAgent: string | null) => update({ defaultAgent }),
  }
}
