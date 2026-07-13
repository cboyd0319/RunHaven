import type {
  AgentProfile,
  DashboardStatus,
  ImageStatusResponse,
  LaunchRunResponse,
  LaunchStatusResponse,
  ManagedSessionStatusResponse,
  ManagedSessionDiagnosticsResponse,
  ManagedSessionSummary,
  RunStatusResponse,
} from "@/commands/runhaven"

import type { NavDestination } from "../navigation"
import type { RunControlAction } from "../types"
import type { LaunchFlow } from "../useLaunchFlow"
import type { useRunOutput } from "../useRunOutput"

// The prop bag the app controller feeds the main content router. Home and
// Session take the whole bag (they own most of the app's state surface); the
// System views take narrow slices of it.
export type MainViewProps = {
  error: string
  launchMessage: string
  loading: boolean
  // ISO time of the last successful status read, for the Home freshness line.
  lastRefreshedAt: string | null
  dashboard: DashboardStatus | null
  setupReady: boolean
  selectedSession: ManagedSessionSummary | null
  selectedSessionStatus: ManagedSessionStatusResponse | null
  selectedSessionStatusLoading: boolean
  selectedSessionStatusError: string
  selectedSessionDiagnostics: ManagedSessionDiagnosticsResponse | null
  selectedSessionDiagnosticsLoading: boolean
  selectedSessionDiagnosticsError: string
  onLoadSelectedSessionDiagnostics: () => Promise<void>
  launchStatus: LaunchStatusResponse | null
  launchStatusLoading: boolean
  launchStatusError: string
  sessionInputDraft: string
  sessionInputSending: boolean
  sessionInputError: string
  sessionInputAcceptedMessage: string
  onSessionInputDraftChange: (value: string) => void
  onSendSelectedSessionInput: () => Promise<void>
  runOutput: ReturnType<typeof useRunOutput>
  controlBusy: boolean
  controlError: string
  controlMessage: string
  onControl: (action: RunControlAction) => Promise<void>
  onWorktreeResolved: () => Promise<void>
  // The New Session flow's state and actions travel as one object so the
  // router never has to unpack its implementation details.
  launchFlow: LaunchFlow
  launchReady: boolean
  launchImageReady: boolean
  selectedAgent: AgentProfile | undefined
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  onSignedIn: () => void
  sessions: ManagedSessionSummary[]
  onNavigate: (destination: NavDestination) => void
  onRefresh: () => void
  lastLaunch: LaunchRunResponse | null
  lastLaunchSessionId: string | null
  onSelectSession: (sessionId: string) => void
  runStatus: RunStatusResponse | null
  runStatusLoading: boolean
  runStatusError: string
}
