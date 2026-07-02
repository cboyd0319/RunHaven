import { useState } from "react"

import {
  loadSettings,
  persistSettings,
  type AppSettings,
} from "./uiStatePersistence"

export type SettingsState = ReturnType<typeof useSettings>

// State for the in-app Settings dialog (spec 3.9, shipped lean inside the
// main window; see docs/plans/native-gui-navigation-gap.md B3). The dialog
// opens from RunHaven > Settings... (Cmd-,) through the menu event channel;
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
    claudeReauthDays: settings.claudeReauthDays,
    setDockBadge: (dockBadge: boolean) => update({ dockBadge }),
    setDefaultAgent: (defaultAgent: string | null) => update({ defaultAgent }),
    setClaudeReauthDays: (claudeReauthDays: number) =>
      update({ claudeReauthDays }),
  }
}
