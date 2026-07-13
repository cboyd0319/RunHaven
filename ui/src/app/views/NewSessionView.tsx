import { useEffect, useRef } from "react"
import { CheckCircle2, CircleAlert, Pin } from "lucide-react"

import { isTerminalState } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { LaunchPlanCard } from "../LaunchPlanCard"
import { SetupCard } from "../SetupCard"
import { useImageRebuild } from "../useMaintenance"
import type { MainViewProps } from "./props"

// New session is a first-class page. Saved setups start at Project, while opening New
// Session directly adds the optional Agent step. The trusted Rust dialog still
// owns the final launch decision.
export function NewSessionView(props: MainViewProps) {
  const flow = props.launchFlow
  const protectedCopyError =
    flow.error && flow.request.worktree && protectedCopyUnavailable(flow.error)
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
            {props.savedSetups.error.startsWith("Project folder can’t be found") ? (
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
          onCheckAgain={props.onRefresh}
          onOpenDiagnostics={() => props.onNavigate({ kind: "diagnostics" })}
        />
      ) : null}
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
          imageBuild.busy || Boolean(buildState && !isTerminalState(buildState))
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
          flow.setOpen(false)
        }}
        onContinue={() => flow.setLaunchStep("project")}
        onStartSession={flow.startSession}
        onSignedIn={props.onSignedIn}
      />
      {flow.request.workspacePath ? (
        <div>
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
  )
}

function protectedCopyUnavailable(error: string): boolean {
  return error.startsWith("RunHaven can't make a protected copy")
}
