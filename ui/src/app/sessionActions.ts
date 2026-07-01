import type { RefObject } from "react"

import {
  cleanupManagedSession,
  killManagedSession,
  killRun,
  repairManagedSession,
  repairRun,
  stopManagedSession,
  stopRun,
  writeSessionInput,
  type LaunchRunResponse,
  type ManagedSessionList,
  type ManagedSessionSummary,
} from "@/commands/runhaven"

import { errorMessage, type RunControlAction } from "./types"
import type { useManagedSessionStatus } from "./useManagedSessionStatus"
import type { useRunControlState } from "./useRunControlState"
import type { useSessionInputState } from "./useSessionInputState"
import type { useStatusLoaders } from "./useStatusLoaders"

type SessionActionsInput = {
  selectedSession: ManagedSessionSummary | null
  lastLaunch: LaunchRunResponse | null
  selectedSessionIdRef: RefObject<string | null>
  selectedSessionStatus: ReturnType<typeof useManagedSessionStatus>
  sessionInput: ReturnType<typeof useSessionInputState>
  runControlState: ReturnType<typeof useRunControlState>
  statusLoaders: ReturnType<typeof useStatusLoaders>
  loadManagedSessions: () => Promise<ManagedSessionList>
}

// Actions on the selected session (or the last-launch fallback): run control,
// worktree-resolution refresh, and session input sending. Split from
// useAppController by ownership; all state lives in the passed hook returns,
// this file owns only the action flows.
export function createSessionActions({
  selectedSession,
  lastLaunch,
  selectedSessionIdRef,
  selectedSessionStatus,
  sessionInput,
  runControlState,
  statusLoaders,
  loadManagedSessions,
}: SessionActionsInput) {
  async function runControl(action: RunControlAction) {
    if (!selectedSession && !lastLaunch) {
      return
    }
    runControlState.setBusy(true)
    runControlState.setError("")
    runControlState.setMessage("")
    try {
      if (selectedSession) {
        const sessionId = selectedSession.id
        if (action === "stop") {
          const { result } = await stopManagedSession(sessionId)
          runControlState.setMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const { result } = await killManagedSession(sessionId)
          runControlState.setMessage(`Hard stop requested for ${result.runId}.`)
        } else if (action === "repair") {
          const { result } = await repairManagedSession(sessionId)
          runControlState.setMessage(
            result.markerRemoved
              ? `Stale session record cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        } else {
          const { result } = await cleanupManagedSession(sessionId)
          runControlState.setMessage(
            `Cleaned up ${result.target} for ${result.runId}.`,
          )
        }
        await selectedSessionStatus.loadStatus()
        await loadManagedSessions()
      } else {
        const runId = lastLaunch?.runId
        if (!runId) {
          return
        }
        if (action === "stop") {
          const result = await stopRun(runId)
          runControlState.setMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const result = await killRun(runId)
          runControlState.setMessage(`Hard stop requested for ${result.runId}.`)
        } else if (action === "repair") {
          const result = await repairRun(runId)
          runControlState.setMessage(
            result.markerRemoved
              ? `Stale session record cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        } else {
          return
        }
        await statusLoaders.loadRunStatus(runId)
        await loadManagedSessions()
      }
    } catch (cause) {
      runControlState.setError(errorMessage(cause))
    } finally {
      runControlState.resetConfirmations()
      runControlState.setBusy(false)
    }
  }

  async function refreshAfterWorktree() {
    await selectedSessionStatus.loadStatus()
    await loadManagedSessions()
  }

  async function sendSelectedSessionInput() {
    if (!selectedSession || sessionInput.sending) {
      return
    }
    const targetSessionId = selectedSession.id
    const input = sessionInput.draft
    sessionInput.setSendingFor(targetSessionId, true)
    sessionInput.setErrorFor(targetSessionId, "")
    sessionInput.setAcceptedMessageFor(targetSessionId, "")
    try {
      const response = await writeSessionInput(targetSessionId, input)
      sessionInput.setDraftFor(targetSessionId, "")
      sessionInput.setAcceptedMessageFor(
        targetSessionId,
        `Sent ${response.acceptedBytes} bytes.`,
      )
      if (selectedSessionIdRef.current === targetSessionId) {
        await selectedSessionStatus.loadStatus()
      }
      await loadManagedSessions()
    } catch (cause) {
      sessionInput.setErrorFor(targetSessionId, errorMessage(cause))
    } finally {
      sessionInput.setSendingFor(targetSessionId, false)
    }
  }

  return {
    runControl,
    refreshAfterWorktree,
    sendSelectedSessionInput,
  }
}
