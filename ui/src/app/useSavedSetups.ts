import { useEffect, useState } from "react"

import {
  chooseProjectFolder,
  getSavedSetups,
  pinSavedSetup,
  selectSavedSetup,
  setSavedSetupPinned,
  type RunPlanRequest,
  type SavedSetupSummary,
} from "@/commands/runhaven"

import type { NavDestination } from "./navigation"
import { errorMessage } from "./types"

export function useSavedSetups({
  applyRequest,
  navigate,
}: {
  applyRequest: (request: RunPlanRequest) => void
  navigate: (destination: NavDestination) => void
}) {
  const [setups, setSetups] = useState<SavedSetupSummary[]>([])
  const [error, setError] = useState("")
  const [message, setMessage] = useState("")
  const [unavailableSetupId, setUnavailableSetupId] = useState<string | null>(null)

  async function refresh() {
    try {
      setSetups(await getSavedSetups())
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  useEffect(() => {
    void refresh()
  }, [])

  async function selectSetup(id: string) {
    setError("")
    setMessage("")
    setUnavailableSetupId(null)
    const summary = setups.find((setup) => setup.id === id)
    navigate({ kind: "new-session" })
    if (summary && !summary.available) {
      setUnavailableSetupId(id)
      setError("Project folder can’t be found. Choose the folder again or unpin this setup.")
      return
    }
    try {
      applyRequest(await selectSavedSetup(id))
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  async function repairUnavailable() {
    const summary = setups.find((setup) => setup.id === unavailableSetupId)
    if (!summary) {
      setError("This saved setup is unavailable. Select it again, then choose its folder.")
      return
    }
    try {
      const projectPath = await chooseProjectFolder()
      if (!projectPath) {
        return
      }
      setError("")
      setMessage("")
      const replacementId = await pinSavedSetup(summary.assistant, projectPath)
      await setSavedSetupPinned(summary.id, false)
      applyRequest(await selectSavedSetup(replacementId))
      setUnavailableSetupId(null)
      await refresh()
      setMessage("Setup updated.")
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  async function pinDraft(assistant: string, projectPath: string) {
    setError("")
    setMessage("")
    try {
      await pinSavedSetup(assistant, projectPath)
      await refresh()
      setMessage("Setup saved.")
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  async function unpin(id: string) {
    setError("")
    setMessage("")
    try {
      await setSavedSetupPinned(id, false)
      await refresh()
      setMessage("Setup removed from the sidebar.")
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  return {
    pinnedSetups: setups.filter((setup) => setup.pinned),
    error,
    message,
    refresh,
    selectSetup,
    repairUnavailable,
    pinDraft,
    unpin,
  }
}

export type SavedSetupsState = ReturnType<typeof useSavedSetups>
