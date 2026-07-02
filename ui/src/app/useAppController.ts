import { useEffect, useState } from "react"

import {
  getDashboardStatus,
  getAuthStatus,
  openSessionWindow,
  type DashboardStatus,
  type AuthStatusResponse,
  type LaunchRunResponse,
} from "@/commands/runhaven"

import { sameDashboardStatus } from "./appStatusEquality"
import { errorMessage } from "./types"
import { createSessionActions } from "./sessionActions"
import {
  sameAuthStatusResponse,
  useDiagnosticsState,
} from "./useDiagnosticsState"
import { useAppDerivedState } from "./useAppDerivedState"
import { useLaunchFlow } from "./useLaunchFlow"
import { useManagedSessionsState } from "./useManagedSessionsState"
import { useSettings } from "./useSettings"
import { useManagedSessionDiagnostics } from "./useManagedSessionDiagnostics"
import { useManagedSessionStatus } from "./useManagedSessionStatus"
import { useLaunchStatus } from "./useLaunchStatus"
import { useRunOutput } from "./useRunOutput"
import { useRunControlState } from "./useRunControlState"
import { useSessionInputState } from "./useSessionInputState"
import { useStatusLoaders } from "./useStatusLoaders"

export function useAppController() {
  const [dashboard, setDashboard] = useState<DashboardStatus | null>(null)
  const [authStatus, setAuthStatus] = useState<AuthStatusResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [lastLaunch, setLastLaunch] = useState<LaunchRunResponse | null>(null)
  const {
    managedSessions,
    navigation,
    selectedSessionId,
    selectedSessionIdRef,
    navigate,
    selectSession,
    loadManagedSessions,
  } = useManagedSessionsState(lastLaunch?.runId ?? null)
  const statusLoaders = useStatusLoaders()
  const diagnostics = useDiagnosticsState()
  const [error, setError] = useState("")
  const settingsState = useSettings()
  const launchFlow = useLaunchFlow({
    dashboard,
    defaultAgent: settingsState.defaultAgent,
    loadImageStatus: statusLoaders.loadImageStatus,
    loadRunStatus: statusLoaders.loadRunStatus,
    loadManagedSessions,
    selectSession,
    onLaunched: setLastLaunch,
    onError: setError,
  })

  // Choosing a default agent in Settings is agent intent right now, not only
  // for the next app start: point the launch draft at it too, through the same
  // path a click in the sheet's own picker would take.
  const settings = {
    ...settingsState,
    setDefaultAgent(defaultAgent: string | null) {
      settingsState.setDefaultAgent(defaultAgent)
      if (defaultAgent) {
        launchFlow.changeAgent(defaultAgent)
      }
    },
  }

  const derived = useAppDerivedState({
    dashboard,
    request: launchFlow.request,
    imageStatus: statusLoaders.imageStatus,
    plan: launchFlow.plan,
    launchConfirmation: launchFlow.launchConfirmation,
    confirmedWarnings: launchFlow.confirmedWarnings,
    managedSessions,
    navigation,
    selectedSessionId,
    lastLaunch,
  })
  const selectedSessionStatus = useManagedSessionStatus(selectedSessionId)
  const selectedSessionDiagnostics = useManagedSessionDiagnostics(selectedSessionId)
  const selectedSession =
    selectedSessionStatus.data?.session ?? derived.selectedSession
  const launchStatus = useLaunchStatus(
    lastLaunch && !selectedSession ? lastLaunch.runId : null,
  )
  const sessionInput = useSessionInputState(selectedSessionId)
  const runControlState = useRunControlState(derived.controlTargetId)
  const runOutput = useRunOutput({
    targetId: derived.outputTargetId,
    sessionId: derived.outputSessionId,
    runId: derived.outputRunId,
  })

  useEffect(() => {
    void refresh()
  }, [])

  async function refresh() {
    setLoading(true)
    setError("")
    try {
      const nextDashboard = await getDashboardStatus()
      const nextAuthStatus = await getAuthStatus()
      setDashboard((current) =>
        current && sameDashboardStatus(current, nextDashboard)
          ? current
          : nextDashboard,
      )
      setAuthStatus((current) =>
        current && sameAuthStatusResponse(current, nextAuthStatus)
          ? current
          : nextAuthStatus,
      )
      await loadManagedSessions()
      await launchFlow.applyDashboard(nextDashboard)
      if (lastLaunch) {
        await statusLoaders.loadRunStatus(lastLaunch.runId)
      }
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  const sessionActions = createSessionActions({
    selectedSession,
    lastLaunch,
    selectedSessionIdRef,
    selectedSessionStatus,
    sessionInput,
    runControlState,
    statusLoaders,
    loadManagedSessions,
  })

  function changeSessionInputDraft(value: string) {
    sessionInput.changeDraft(value)
  }

  // Open (or focus) the read-only monitor window for a session. Rust
  // validates the id, caps open monitors, and owns the window label; a
  // rejection (for example the window cap) surfaces in the main error alert.
  function openSessionInNewWindow(sessionId: string) {
    void openSessionWindow(sessionId).catch((cause: unknown) =>
      setError(errorMessage(cause)),
    )
  }

  return {
    dashboard,
    authStatus,
    launchFlow,
    settings,
    loading,
    imageLoading: statusLoaders.imageLoading,
    imageStatus: statusLoaders.imageStatus,
    imageError: statusLoaders.imageError,
    // The "Run accepted" alert lives only until the launched run shows up as a
    // managed session; after that the one-line Home notice is the durable
    // affordance, so the alert clears instead of persisting forever.
    launchMessage: derived.lastLaunchSessionId ? "" : launchFlow.launchMessage,
    lastLaunch,
    lastLaunchSessionId: derived.lastLaunchSessionId,
    managedSessions,
    navigation,
    selectedSessionId,
    sessionInputDraft: sessionInput.draft,
    sessionInputSending: sessionInput.sending,
    sessionInputError: sessionInput.error,
    sessionInputAcceptedMessage: sessionInput.acceptedMessage,
    runStatus: statusLoaders.runStatus,
    runStatusLoading: statusLoaders.runStatusLoading,
    runStatusError: statusLoaders.runStatusError,
    controlBusy: runControlState.busy,
    controlError: runControlState.error,
    controlMessage: runControlState.message,
    controlConfirmations: runControlState.confirmations,
    diagnostics: diagnostics.data,
    diagnosticsLoading: diagnostics.loading,
    diagnosticsError: diagnostics.error,
    error,
    selectedAgent: derived.selectedAgent,
    setupKnown: derived.setupKnown,
    setupReady: derived.setupReady,
    launchImageReady: derived.launchImageReady,
    launchReady: derived.launchReady,
    selectedSession,
    selectedSessionStatus: selectedSessionStatus.data,
    selectedSessionStatusLoading: selectedSessionStatus.loading,
    selectedSessionStatusError: selectedSessionStatus.error,
    selectedSessionDiagnostics: selectedSessionDiagnostics.data,
    selectedSessionDiagnosticsLoading: selectedSessionDiagnostics.loading,
    selectedSessionDiagnosticsError: selectedSessionDiagnostics.error,
    launchStatus: launchStatus.data,
    launchStatusLoading: launchStatus.loading,
    launchStatusError: launchStatus.error,
    pageTitle: derived.pageTitle,
    pageDescription: derived.pageDescription,
    runOutput,
    refresh,
    navigate,
    selectSession,
    openSessionInNewWindow,
    runControl: sessionActions.runControl,
    refreshAfterWorktree: sessionActions.refreshAfterWorktree,
    sendSelectedSessionInput: sessionActions.sendSelectedSessionInput,
    loadSelectedSessionDiagnostics: selectedSessionDiagnostics.load,
    loadDiagnostics: diagnostics.load,
    changeSessionInputDraft,
    setControlConfirmation: runControlState.setConfirmation,
  }
}
