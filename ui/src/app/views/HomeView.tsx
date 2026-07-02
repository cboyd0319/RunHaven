import { CheckCircle2 } from "lucide-react"

import type { LaunchRunResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { LastLaunchCard } from "../LastLaunchCard"
import { LaunchPlanCard } from "../LaunchPlanCard"
import { LaunchStatusCard } from "../LaunchStatusCard"
import { PlanReviewCard } from "../PlanReviewCard"
import { RunStatusCard } from "../RunStatusCard"
import type { MainViewProps } from "./props"
import { OutputView } from "./SessionView"

// Home hosts the start-a-run wizard inline for now (the modal sheet is a later
// slice) plus the last-launch notice or failed-launch fallback controls.
export function HomeView(props: MainViewProps) {
  return (
    <>
      <LaunchView {...props} />
      {props.lastLaunch ? (
        props.lastLaunchSessionId ? (
          <LaunchedRunNotice
            sessionId={props.lastLaunchSessionId}
            onSelect={props.onSelectSession}
          />
        ) : (
          <RunView {...props} lastLaunch={props.lastLaunch} />
        )
      ) : null}
    </>
  )
}

function LaunchView(props: MainViewProps) {
  if (props.launchStep === "review" && props.plan) {
    return (
      <PlanReviewCard
        plan={props.plan}
        launchConfirmation={props.launchConfirmation}
        confirmedWarnings={props.confirmedWarnings}
        launchReady={props.launchReady}
        launching={props.launching}
        setupOk={props.setupReady}
        launchImageReady={props.launchImageReady}
        onLaunchConfirmationChange={props.onLaunchConfirmationChange}
        onConfirmWarning={props.onConfirmWarning}
        onBack={() => props.onLaunchStepChange("project")}
        onStart={props.onStart}
      />
    )
  }
  return (
    <LaunchPlanCard
      step={props.launchStep === "review" ? "project" : props.launchStep}
      agents={props.dashboard?.agents ?? []}
      request={props.request}
      selectedAgent={props.selectedAgent}
      authStatus={props.authStatus}
      imageLoading={props.imageLoading}
      imageStatus={props.imageStatus}
      imageError={props.imageError}
      planning={props.planning}
      onAgentChange={props.onAgentChange}
      onPickFolder={props.onPickFolder}
      onRequestChange={props.onRequestChange}
      onNetworkChange={props.onNetworkChange}
      onBack={() => props.onLaunchStepChange("agent")}
      onContinue={() => props.onLaunchStepChange("project")}
      onReviewPlan={props.onReviewPlan}
      onSignedIn={props.onSignedIn}
    />
  )
}

// After a launch becomes a managed session, the live status, controls, and
// output live only in that session's view. Home shows a single confirmation
// that points at the sidebar, so one run is never controllable from two places
// at once.
function LaunchedRunNotice({
  sessionId,
  onSelect,
}: {
  sessionId: string
  onSelect: (sessionId: string) => void
}) {
  return (
    <Alert role="status">
      <CheckCircle2 />
      <AlertTitle>Run started - find it in your sidebar</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <span>Open the session to watch its status and controls.</span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => onSelect(sessionId)}
        >
          Open session
        </Button>
      </AlertDescription>
    </Alert>
  )
}

// Fallback control surface for a launched run that never registered as a managed
// session (for example a failed launch), so stop and repair stay reachable.
function RunView(props: MainViewProps & { lastLaunch: LaunchRunResponse }) {
  return (
    <>
      <LastLaunchCard lastLaunch={props.lastLaunch} />
      <LaunchStatusCard
        status={props.launchStatus}
        loading={props.launchStatusLoading}
        error={props.launchStatusError}
      />
      <RunStatusCard
        runStatus={props.runStatus}
        runStatusLoading={props.runStatusLoading}
        runStatusError={props.runStatusError}
        controlBusy={props.controlBusy}
        controlError={props.controlError}
        controlMessage={props.controlMessage}
        controlConfirmations={props.controlConfirmations}
        onConfirmationChange={props.onControlConfirmationChange}
        onControl={props.onControl}
      />
      <OutputView runOutput={props.runOutput} />
    </>
  )
}
