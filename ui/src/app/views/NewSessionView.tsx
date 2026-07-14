import { useEffect, useRef } from "react"
import { CheckCircle2, CircleAlert, Pin } from "lucide-react"

import { isTerminalState } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { LaunchPlanCard } from "../LaunchPlanCard"
import { NewSessionProgress } from "../NewSessionProgress"
import { SetupCard } from "../SetupCard"
import { useImageRebuild } from "../useMaintenance"
import type { MainViewProps } from "./props"

// New session is a first-class page. The three WebView steps explain and retain
// choices, while the trusted Rust dialog still owns the final launch decision.
export function NewSessionView(props: MainViewProps) {
  const flow = props.launchFlow
  const protectedCopyError =
    flow.error && flow.request.worktree && protectedCopyUnavailable(flow.error)
  const imageBuild = useImageRebuild()
  const buildState = imageBuild.status?.state ?? ""
  const refreshedBuild = useRef("")
  const stepRegion = useRef<HTMLElement>(null)
  const previousStep = useRef(flow.launchStep)
  const focusAfterSetupCheck = useRef(false)
  const setupCheckStarted = useRef(false)
  const focusStepHeading = () =>
    stepRegion.current
      ?.querySelector<HTMLElement>('[data-slot="card-title"]')
      ?.focus()

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

  useEffect(() => {
    if (previousStep.current !== flow.launchStep) {
      previousStep.current = flow.launchStep
      focusStepHeading()
    }
  }, [flow.launchStep])

  useEffect(() => {
    if (!focusAfterSetupCheck.current) return
    if (props.loading) {
      setupCheckStarted.current = true
      return
    }
    if (props.setupReady) {
      focusAfterSetupCheck.current = false
      setupCheckStarted.current = false
      focusStepHeading()
    } else if (setupCheckStarted.current) {
      focusAfterSetupCheck.current = false
      setupCheckStarted.current = false
    }
  }, [props.loading, props.setupReady])

  useEffect(() => {
    if (flow.reviewFocusRevision === 0) return
    let mounted = true
    queueMicrotask(() => {
      if (mounted) focusStepHeading()
    })
    return () => {
      mounted = false
    }
  }, [flow.reviewFocusRevision])

  return (
    <div
      className={
        props.setupReady
          ? "@container mx-auto flex min-h-0 w-full max-w-6xl flex-1 flex-col gap-4 @[64rem]:my-auto @[64rem]:flex-none"
          : "mx-auto flex w-full max-w-3xl flex-col gap-4"
      }
    >
      <div className="grid max-w-3xl gap-1">
        <p className="text-base leading-relaxed text-muted-foreground">
          {props.setupReady
            ? "Choose an assistant and project, then check the access it will have. Nothing starts until you review and confirm."
            : "Finish setting up this Mac before you choose an assistant or project."}
        </p>
      </div>
      {flow.error && !protectedCopyError ? (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>Could not continue</AlertTitle>
          <AlertDescription>
            <p>{flow.error}</p>
          </AlertDescription>
        </Alert>
      ) : null}
      {protectedCopyError ? (
        <Alert variant="warning" role="alert">
          <CircleAlert />
          <AlertTitle>Choose how to continue</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-3">
            <p>
              RunHaven could not make a protected copy. Your original files are
              unchanged.
            </p>
            <div className="flex flex-wrap gap-2">
              <Button
                type="button"
                size="sm"
                disabled={flow.planning}
                onClick={() => void flow.retryWithoutProtectedCopy(true)}
              >
                Start without changing files…
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={flow.planning}
                onClick={() => void flow.retryWithoutProtectedCopy(false)}
              >
                Let assistant change original files (less safe)…
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      ) : null}
      {imageBuild.error ? (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>Could not prepare this assistant</AlertTitle>
          <AlertDescription>{imageBuild.error}</AlertDescription>
        </Alert>
      ) : null}
      {imageBuild.status ? (
        <Alert role="status">
          <AlertTitle>{imageBuild.status.label}</AlertTitle>
          <AlertDescription>{imageBuild.status.detail}</AlertDescription>
        </Alert>
      ) : null}
      {props.savedSetups.error ? (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>Could not update saved setups</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>{props.savedSetups.error}</span>
            {props.savedSetups.error.startsWith(
              "Project folder can’t be found",
            ) ? (
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => void props.savedSetups.repairUnavailable()}
              >
                Choose folder again…
              </Button>
            ) : null}
          </AlertDescription>
        </Alert>
      ) : null}
      {props.savedSetups.message ? (
        <Alert role="status">
          <CheckCircle2 />
          <AlertTitle>{props.savedSetups.message}</AlertTitle>
        </Alert>
      ) : null}
      {!props.setupReady ? (
        <SetupCard
          loading={props.loading}
          dashboard={props.dashboard}
          onCheckAgain={() => {
            focusAfterSetupCheck.current = true
            setupCheckStarted.current = false
            props.onRefresh()
          }}
          onOpenDiagnostics={() => props.onNavigate({ kind: "diagnostics" })}
        />
      ) : (
        <div className="grid min-w-0 gap-4 @[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,2fr)] @[64rem]:items-start">
          <NewSessionProgress
            step={flow.launchStep}
            request={flow.request}
            selectedAgent={props.selectedAgent}
            readiness={props.dashboard?.agentReadiness.find(
              (item) => item.name === props.selectedAgent?.name,
            )}
            onEditAgent={() => flow.setLaunchStep("agent")}
            onEditProject={() => flow.setLaunchStep("project")}
          />
          <div className="grid min-w-0 gap-4">
            <section
              ref={stepRegion}
              aria-label={`${stepLabel(flow.launchStep)} step`}
              className="min-w-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
            >
              <LaunchPlanCard
                step={flow.launchStep}
                agents={props.dashboard?.agents ?? []}
                agentReadiness={props.dashboard?.agentReadiness ?? []}
                request={flow.request}
                selectedAgent={props.selectedAgent}
                imageLoading={props.imageLoading}
                imageStatus={props.imageStatus}
                imageError={props.imageError}
                imageInstalling={
                  imageBuild.busy ||
                  Boolean(buildState && !isTerminalState(buildState))
                }
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
                onBack={() => {
                  flow.setLaunchStep(
                    flow.launchStep === "review" ? "project" : "agent",
                  )
                }}
                onContinue={() =>
                  flow.setLaunchStep(
                    flow.launchStep === "agent" ? "project" : "review",
                  )
                }
                onStartSession={flow.startSession}
                onSignedIn={props.onSignedIn}
              />
            </section>
            {flow.launchStep === "review" && flow.request.workspacePath ? (
              <div className="flex flex-wrap items-center justify-between gap-3 border-t pt-4">
                <p className="max-w-xl text-sm leading-relaxed text-muted-foreground">
                  Save the assistant and project with recommended access.
                  Advanced choices apply only to this session.
                </p>
                <Button
                  type="button"
                  variant="outline"
                  disabled={flow.planning}
                  onClick={() =>
                    void props.savedSetups.pinDraft(
                      flow.request.agent,
                      flow.request.workspacePath,
                    )
                  }
                >
                  <Pin />
                  Save setup
                </Button>
              </div>
            ) : null}
          </div>
        </div>
      )}
    </div>
  )
}

function stepLabel(step: MainViewProps["launchFlow"]["launchStep"]): string {
  switch (step) {
    case "agent":
      return "Choose assistant"
    case "project":
      return "Choose project"
    case "review":
      return "Review and start"
  }
}

function protectedCopyUnavailable(error: string): boolean {
  return error.startsWith("RunHaven can't make a protected copy")
}
