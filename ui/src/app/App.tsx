import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"

import { AppHeader } from "./AppHeader"
import { AppMainContent } from "./AppMainContent"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { userSessions } from "./sessionList"
import { SessionLabelsProvider } from "./sessionLabels"
import { sidebarDefaultOpen } from "./uiStatePersistence"
import { useAppController } from "./useAppController"
import { useAttentionBadge } from "./useAttentionBadge"
import { useMenuEvents } from "./useMenuEvents"
import { useMenuStateSync } from "./useMenuStateSync"

export default function App() {
  const app = useAppController()
  const sessions = userSessions(app.managedSessions.sessions)
  useMenuEvents(app)
  useMenuStateSync(sessions, app.navigation)
  useAttentionBadge(sessions, app.settings.dockBadge)

  return (
    <TooltipProvider>
      <SessionLabelsProvider>
        <SidebarProvider defaultOpen={sidebarDefaultOpen()}>
          <RunHavenSidebar
            navigation={app.navigation}
            onNavigate={app.navigate}
            onOpenSettings={() => app.settings.setOpen(true)}
            settings={app.settings}
            agents={app.dashboard?.agents ?? []}
            runTerminalActive={app.launchFlow.runTerminal !== null}
            appVersion={app.dashboard?.appVersion ?? ""}
          />
          <SidebarInset className="h-screen overflow-hidden">
            <AppHeader
              title={app.pageTitle}
              description={app.pageDescription}
              loading={app.loading}
              onRefresh={() => void app.refresh()}
            />
            <AppMainContent
              navigation={app.navigation}
              error={app.error}
              launchMessage={app.launchMessage}
              loading={app.loading}
              lastRefreshedAt={app.lastRefreshedAt}
              dashboard={app.dashboard}
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
              sessions={sessions}
              onNavigate={app.navigate}
              onRefresh={() => void app.refresh()}
              lastLaunch={app.lastLaunch}
              lastLaunchSessionId={app.lastLaunchSessionId}
              onSelectSession={app.selectSession}
              runStatus={app.runStatus}
              runStatusLoading={app.runStatusLoading}
              runStatusError={app.runStatusError}
            />
          </SidebarInset>
        </SidebarProvider>
      </SessionLabelsProvider>
    </TooltipProvider>
  )
}
