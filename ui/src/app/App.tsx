import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"

import { AppHeader } from "./AppHeader"
import { AppMainContent } from "./AppMainContent"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { useAppController } from "./useAppController"

export default function App() {
  const app = useAppController()

  return (
    <TooltipProvider>
      <SidebarProvider>
        <RunHavenSidebar
          sessions={app.managedSessions.sessions}
          selectedSessionId={app.selectedSessionId}
          onStartSelect={() => app.selectSession(null)}
          onSessionSelect={app.selectSession}
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
            launchStep={app.launchStep}
            plan={app.plan}
            launchConfirmation={app.launchConfirmation}
            confirmedWarnings={app.confirmedWarnings}
            launchReady={app.launchReady}
            launching={app.launching}
            launchImageReady={app.launchImageReady}
            onLaunchConfirmationChange={app.setLaunchConfirmation}
            onConfirmWarning={app.setWarningConfirmation}
            onLaunchStepChange={app.setLaunchStep}
            onStart={app.startRun}
            request={app.request}
            selectedAgent={app.selectedAgent}
            imageLoading={app.imageLoading}
            imageStatus={app.imageStatus}
            imageError={app.imageError}
            planning={app.planning}
            onAgentChange={app.changeAgent}
            onPickFolder={app.pickFolder}
            onRequestChange={app.updateRequest}
            onNetworkChange={app.updateNetworkMode}
            onReviewPlan={app.reviewPlan}
            lastLaunch={app.lastLaunch}
            runStatus={app.runStatus}
            runStatusLoading={app.runStatusLoading}
            runStatusError={app.runStatusError}
            diagnostics={app.diagnostics}
            diagnosticsLoading={app.diagnosticsLoading}
            diagnosticsError={app.diagnosticsError}
            onLoadDiagnostics={app.loadDiagnostics}
          />
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
