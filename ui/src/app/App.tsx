import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"

import { AppHeader } from "./AppHeader"
import { AppMainContent } from "./AppMainContent"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { SessionLabelsProvider } from "./sessionLabels"
import { SettingsDialog } from "./SettingsDialog"
import { sidebarDefaultOpen } from "./uiStatePersistence"
import { useAppController } from "./useAppController"
import { useAttentionBadge } from "./useAttentionBadge"
import { useMenuEvents } from "./useMenuEvents"
import { useMenuStateSync } from "./useMenuStateSync"

export default function App() {
  const app = useAppController()
  useMenuEvents(app)
  useMenuStateSync(app.managedSessions.sessions, app.navigation)
  useAttentionBadge(app.managedSessions.sessions, app.settings.dockBadge)

  return (
    <TooltipProvider>
      <SessionLabelsProvider>
        <SidebarProvider defaultOpen={sidebarDefaultOpen()}>
          <RunHavenSidebar
            sessions={app.managedSessions.sessions}
            navigation={app.navigation}
            onNavigate={app.navigate}
            runTerminalActive={app.launchFlow.runTerminal !== null}
          />
          <SidebarInset className="h-screen overflow-hidden">
            <AppHeader
              title={app.pageTitle}
              description={app.pageDescription}
              setupKnown={app.setupKnown}
              setupReady={app.setupReady}
              loading={app.loading}
              onRefresh={() => void app.refresh()}
            />
            <AppMainContent
              navigation={app.navigation}
              error={app.error}
              launchMessage={app.launchMessage}
              loading={app.loading}
              dashboard={app.dashboard}
              authStatus={app.authStatus}
              setupReady={app.setupReady}
              selectedSession={app.selectedSession}
              selectedSessionStatus={app.selectedSessionStatus}
              selectedSessionStatusLoading={app.selectedSessionStatusLoading}
              selectedSessionStatusError={app.selectedSessionStatusError}
              selectedSessionDiagnostics={app.selectedSessionDiagnostics}
              selectedSessionDiagnosticsLoading={app.selectedSessionDiagnosticsLoading}
              selectedSessionDiagnosticsError={app.selectedSessionDiagnosticsError}
              onLoadSelectedSessionDiagnostics={app.loadSelectedSessionDiagnostics}
              launchStatus={app.launchStatus}
              launchStatusLoading={app.launchStatusLoading}
              launchStatusError={app.launchStatusError}
              sessionInputDraft={app.sessionInputDraft}
              sessionInputSending={app.sessionInputSending}
              sessionInputError={app.sessionInputError}
              sessionInputAcceptedMessage={app.sessionInputAcceptedMessage}
              onSessionInputDraftChange={app.changeSessionInputDraft}
              onSendSelectedSessionInput={app.sendSelectedSessionInput}
              runOutput={app.runOutput}
              controlBusy={app.controlBusy}
              controlError={app.controlError}
              controlMessage={app.controlMessage}
              controlConfirmations={app.controlConfirmations}
              onControlConfirmationChange={app.setControlConfirmation}
              onControl={app.runControl}
              onWorktreeResolved={app.refreshAfterWorktree}
              launchFlow={app.launchFlow}
              launchReady={app.launchReady}
              launchImageReady={app.launchImageReady}
              selectedAgent={app.selectedAgent}
              imageLoading={app.imageLoading}
              imageStatus={app.imageStatus}
              imageError={app.imageError}
              onSignedIn={() => void app.refresh()}
              sessions={app.managedSessions.sessions}
              onNavigate={app.navigate}
              onRefresh={() => void app.refresh()}
              lastLaunch={app.lastLaunch}
              lastLaunchSessionId={app.lastLaunchSessionId}
              onSelectSession={app.selectSession}
              runStatus={app.runStatus}
              runStatusLoading={app.runStatusLoading}
              runStatusError={app.runStatusError}
              diagnostics={app.diagnostics}
              diagnosticsLoading={app.diagnosticsLoading}
              diagnosticsError={app.diagnosticsError}
              onLoadDiagnostics={app.loadDiagnostics}
            />
          </SidebarInset>
          <SettingsDialog
            settings={app.settings}
            agents={app.dashboard?.agents ?? []}
          />
        </SidebarProvider>
      </SessionLabelsProvider>
    </TooltipProvider>
  )
}
