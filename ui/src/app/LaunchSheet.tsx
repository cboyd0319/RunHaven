import { CircleAlert, Plus } from "lucide-react"

import type {
  AgentProfile,
  AgentReadiness,
  ImageStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

import { LaunchPlanCard } from "./LaunchPlanCard"
import { PlanReviewCard } from "./PlanReviewCard"
import type { LaunchFlow } from "./useLaunchFlow"
import { useFocusOnChange } from "./useFocusOnChange"

type LaunchSheetProps = {
  flow: LaunchFlow
  agents: AgentProfile[]
  agentReadiness: AgentReadiness[]
  selectedAgent: AgentProfile | undefined
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  setupReady: boolean
  launchReady: boolean
  launchImageReady: boolean
  onSignedIn: () => void
  disabledReason: string | null
}

// The New Run flow as a modal sheet over the app (spec 3.2). The sheet only
// captures intent: the Rust-owned native confirmation dialog inside launch_run
// stays the mutation boundary above it. Escape or Cancel closes with no side
// effects; the entered values stay for the next open. The trigger lives here
// so closing returns focus to the New run button that opened the sheet.
export function LaunchSheet(props: LaunchSheetProps) {
  const { flow, disabledReason } = props
  const stepFocusRef = useFocusOnChange<HTMLDivElement>(flow.launchStep)

  return (
    <Dialog open={flow.sheetOpen} onOpenChange={flow.setOpen}>
      <div>
        <DialogTrigger asChild>
          <Button
            type="button"
            disabled={disabledReason !== null}
            aria-label={disabledReason ?? "New run"}
            title={disabledReason ?? undefined}
          >
            <Plus />
            New run
          </Button>
        </DialogTrigger>
      </div>
      <DialogContent
        className="max-h-[calc(100vh-2rem)] overflow-y-auto sm:max-w-2xl"
        showCloseButton={false}
      >
        <DialogHeader>
          <DialogTitle>New run</DialogTitle>
          <DialogDescription>
            Pick an agent and a project, then review access. Nothing starts
            until you confirm.
          </DialogDescription>
        </DialogHeader>
        <div
          ref={stepFocusRef}
          tabIndex={-1}
          className="flex flex-col gap-4 outline-none"
        >
          {flow.error ? (
            <Alert variant="destructive">
              <CircleAlert />
              <AlertTitle>Action failed</AlertTitle>
              <AlertDescription>{flow.error}</AlertDescription>
            </Alert>
          ) : null}
          <LaunchSheetStep {...props} />
        </div>
      </DialogContent>
    </Dialog>
  )
}

function LaunchSheetStep(props: LaunchSheetProps) {
  const { flow } = props
  // Interactive agent (Claude Code): confirming at review closes this sheet and
  // opens the run terminal as the run-terminal main-content destination (see
  // AppMainContent), so the live session fills the bounded window instead of
  // growing this modal off-screen.
  if (flow.launchStep === "review" && flow.plan) {
    return (
      <PlanReviewCard
        plan={flow.plan}
        confirmedWarnings={flow.confirmedWarnings}
        launchReady={props.launchReady}
        launching={flow.launching}
        setupOk={props.setupReady}
        launchImageReady={props.launchImageReady}
        onConfirmWarning={flow.setWarningConfirmation}
        onBack={() => flow.setLaunchStep("project")}
        onStart={flow.startRun}
      />
    )
  }
  return (
    <LaunchPlanCard
      step={flow.launchStep === "review" ? "project" : flow.launchStep}
      agents={props.agents}
      agentReadiness={props.agentReadiness}
      request={flow.request}
      selectedAgent={props.selectedAgent}
      imageLoading={props.imageLoading}
      imageStatus={props.imageStatus}
      imageError={props.imageError}
      planning={flow.planning}
      setupOk={props.setupReady}
      onAgentChange={flow.changeAgent}
      onPickFolder={flow.pickFolder}
      onRequestChange={flow.updateRequest}
      onNetworkChange={flow.updateNetworkMode}
      onCancel={() => flow.setOpen(false)}
      onBack={() => flow.setLaunchStep("agent")}
      onContinue={() => flow.setLaunchStep("project")}
      onReviewPlan={flow.reviewPlan}
      onSignedIn={props.onSignedIn}
    />
  )
}
