import { useState } from "react"

import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"

import { AppHeader } from "./AppHeader"
import { AppMainContent } from "./AppMainContent"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { userSessions } from "./sessionList"
import { terminalWorkspacePageTitle } from "./terminalWorkspace"
import { sidebarDefaultOpen } from "./uiStatePersistence"
import { useAppController } from "./useAppController"
import { useMenuEvents } from "./useMenuEvents"
import { useMenuStateSync } from "./useMenuStateSync"

function keepEditingContextMenu(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest('input, textarea, [contenteditable="true"], pre, code, .xterm') !==
      null
  )
}

export default function App() {
  const app = useAppController()
  const sessions = userSessions(app.managedSessions.sessions)
  const [sidebarOpen, setSidebarOpen] = useState(sidebarDefaultOpen)
  useMenuEvents(app)
  useMenuStateSync(
    sessions,
    app.navigation,
    sidebarOpen,
  )
  const pageTitle =
    app.navigation.kind === "session"
      ? (app.selectedSession?.title ?? app.pageTitle)
      : terminalWorkspacePageTitle(
          app.navigation,
          app.launchFlow.runTerminals,
          app.pageTitle,
        )
  return (
    <TooltipProvider>
      <SidebarProvider
        open={sidebarOpen}
        onOpenChange={setSidebarOpen}
        onContextMenu={(event) => {
          // WKWebView otherwise exposes Reload and Inspect Element on app
          // chrome. Keep the normal editing menu where it is useful; session
          // actions remain in the visible controls and native Run menu.
          if (!keepEditingContextMenu(event.target)) {
            event.preventDefault()
          }
        }}
      >
        <RunHavenSidebar
          navigation={app.navigation}
          onNavigate={app.navigate}
          runTerminals={app.launchFlow.runTerminals}
          savedSetups={app.savedSetups.pinnedSetups}
          savedSetupsError={app.savedSetups.error}
          onSelectSavedSetup={(id) => void app.savedSetups.selectSetup(id)}
          onUnpinSavedSetup={(id) => void app.savedSetups.unpin(id)}
          appVersion={app.dashboard?.appVersion ?? ""}
        />
        <SidebarInset className="h-screen overflow-hidden">
          <AppHeader
            title={pageTitle}
            session={
              app.navigation.kind === "session" ? app.selectedSession : null
            }
            controlBusy={app.controlBusy}
            onControl={app.runControl}
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
              savedSetups={app.savedSetups}
              supportReport={app.supportReport}
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
    </TooltipProvider>
  )
}
