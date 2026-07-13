import { useEffect, useRef } from "react"
import { CircleAlert } from "lucide-react"

import { isTerminalState } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

import { LaunchPlanCard } from "../LaunchPlanCard"
import { PlanReviewCard } from "../PlanReviewCard"
import { SetupCard } from "../SetupCard"
import { useImageRebuild } from "../useMaintenance"
import type { MainViewProps } from "./props"

// New Session is a first-class page. It captures intent in three small steps;
// the trusted Rust dialog still owns the final launch decision.
export function NewSessionView(props: MainViewProps) {
  const flow = props.launchFlow
  const imageBuild = useImageRebuild()
  const buildState = imageBuild.status?.state ?? ""
  const refreshedBuild = useRef("")

  useEffect(() => {
    const buildId = imageBuild.status?.buildId ?? ""
    if (
      buildId &&
      buildId !== refreshedBuild.current &&
      isTerminalState(buildState)
    ) {
      refreshedBuild.current = buildId
      props.onRefresh()
    }
  }, [buildState, imageBuild.status?.buildId, props.onRefresh])

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-4">
      {flow.error ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Could not continue</AlertTitle>
          <AlertDescription>{flow.error}</AlertDescription>
        </Alert>
      ) : null}
      {imageBuild.error ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Could not prepare this agent</AlertTitle>
          <AlertDescription>{imageBuild.error}</AlertDescription>
        </Alert>
      ) : null}
      {imageBuild.status ? (
        <Alert role="status">
          <AlertTitle>{imageBuild.status.label}</AlertTitle>
          <AlertDescription>{imageBuild.status.detail}</AlertDescription>
        </Alert>
      ) : null}
      {flow.launchStep === "agent" && !props.setupReady ? (
        <SetupCard
          loading={props.loading}
          dashboard={props.dashboard}
          onCheckAgain={props.onRefresh}
          onOpenDiagnostics={() => props.onNavigate({ kind: "diagnostics" })}
        />
      ) : null}
      {flow.launchStep === "review" && flow.plan ? (
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
      ) : (
        <LaunchPlanCard
          step={flow.launchStep === "review" ? "project" : flow.launchStep}
          agents={props.dashboard?.agents ?? []}
          agentReadiness={props.dashboard?.agentReadiness ?? []}
          request={flow.request}
          selectedAgent={props.selectedAgent}
          imageLoading={props.imageLoading}
          imageStatus={props.imageStatus}
          imageError={props.imageError}
          imageInstalling={imageBuild.busy || Boolean(buildState && !isTerminalState(buildState))}
          planning={flow.planning}
          setupOk={props.setupReady}
          onAgentChange={flow.changeAgent}
          onPickFolder={flow.pickFolder}
          onRequestChange={flow.updateRequest}
          onNetworkChange={flow.updateNetworkMode}
          onInstallImage={() => {
            if (props.selectedAgent) {
              void imageBuild.rebuild(props.selectedAgent.name)
            }
          }}
          onCancel={() => {
            flow.setOpen(false)
            props.onNavigate({ kind: "home" })
          }}
          onBack={() => flow.setLaunchStep("agent")}
          onContinue={() => flow.setLaunchStep("project")}
          onReviewPlan={flow.reviewPlan}
          onSignedIn={props.onSignedIn}
        />
      )}
    </div>
  )
}
