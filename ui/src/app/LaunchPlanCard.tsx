import {
  supportsApiKeyBroker,
  type AgentProfile,
  type AgentReadiness,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { CheckCircle2, CircleAlert, FolderOpen } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { ActivityIndicator } from "@/components/ui/activity-indicator"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

import { AgentSignIn } from "./AgentSignIn"
import {
  ApiKeyBrokerField,
  AssistantPreparation,
  AuthScopePicker,
  ImageReadiness,
} from "./LaunchAccess"
import { LaunchReview } from "./LaunchReview"
import { networkLabels } from "./labels"
import { projectName } from "./launchSummary"
import { ResourceLimitsFields } from "./ResourceLimitsFields"
import { agentLabel, orderProductAgents } from "./sessionIdentity"
import { networkHelp } from "./types"
import type { LaunchStep } from "./types"

type LaunchPlanCardProps = {
  step: LaunchStep
  agents: AgentProfile[]
  agentReadiness: AgentReadiness[]
  request: RunPlanRequest
  selectedAgent: AgentProfile | undefined
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  imageInstalling?: boolean
  planning: boolean
  setupOk: boolean
  onAgentChange: (agentName: string) => void
  onPickFolder: () => Promise<void>
  onRequestChange: (patch: Partial<RunPlanRequest>) => void
  onNetworkChange: (networkMode: RunPlanRequest["networkMode"]) => void
  onInstallImage?: () => void
  onBack: () => void
  onContinue: () => void
  onStartSession: () => Promise<void>
  onSignedIn: () => void
}

export function LaunchPlanCard({
  step,
  agents,
  agentReadiness,
  request,
  selectedAgent,
  imageLoading,
  imageStatus,
  imageError,
  imageInstalling = false,
  planning,
  setupOk,
  onAgentChange,
  onPickFolder,
  onRequestChange,
  onNetworkChange,
  onInstallImage,
  onBack,
  onContinue,
  onStartSession,
  onSignedIn,
}: LaunchPlanCardProps) {
  const selectedReadiness = selectedAgent
    ? agentReadiness.find((item) => item.name === selectedAgent.name)
    : undefined
  const alternateNetwork = request.networkMode !== "provider"
  const brokerBypassNotice = !alternateNetwork
    ? null
    : request.networkMode === "internal"
      ? null
      : supportsApiKeyBroker(selectedAgent)
        ? "RunHaven can’t share your saved sign-in with this network choice. The assistant may ask you to sign in again."
        : null
  const offlineAgentNotice =
    request.networkMode === "internal" &&
    (selectedAgent?.providerHosts.length ?? 0) > 0
      ? `${agentLabel(selectedAgent!.name)} can’t connect while Local only is selected. Choose Assistant services only to let it connect safely, or continue if you want it offline.`
      : null
  const selectedImageStatus =
    imageStatus?.agent === selectedAgent?.name ? imageStatus : null
  const selectedImageReady = selectedImageStatus?.image.ready === true
  const selectedAgentNeedsSignIn = needsSignIn(selectedReadiness)
  const selectedAgentNeedsPreparation =
    imageLoading ||
    Boolean(imageError) ||
    (selectedImageStatus !== null && !selectedImageReady) ||
    selectedAgentNeedsSignIn
  const projectAccess = request.worktree
    ? "protected"
    : request.readOnlyWorkspace
      ? "read-only"
      : "original"
  return (
    <Card>
      <CardHeader>
        <CardTitle
          className="text-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          role="heading" aria-level={2} tabIndex={-1}
        >
          {step === "agent"
            ? "Choose assistant"
            : step === "project"
              ? "Choose project"
              : "Review and start"}
        </CardTitle>
        <CardDescription className="text-base">
          {step === "agent"
            ? "Pick the coding assistant you want to use."
            : step === "project"
              ? selectedAgent
                ? `Pick the project folder ${agentLabel(selectedAgent.name)} can use.`
                : "Pick the project folder this assistant can use."
              : "Check what the assistant can access before it starts."}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault()
            if (step === "review") {
              void onStartSession()
            } else {
              onContinue()
            }
          }}
        >
          {step === "agent" ? (
            <>
              <label className="flex flex-col gap-1.5 text-base font-medium">
                Assistant
                <Select value={request.agent} onValueChange={onAgentChange}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select assistant" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {orderProductAgents(agents).map((agent) => (
                        <SelectItem key={agent.name} value={agent.name}>
                          {agentLabel(agent.name)}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </label>

              <ImageReadiness
                imageLoading={imageLoading}
                imageStatus={selectedImageStatus}
                imageError={imageError}
                installing={imageInstalling}
                onInstall={
                  selectedImageStatus && !selectedImageStatus.image.ready
                    ? onInstallImage
                    : undefined
                }
              />
              {selectedReadiness ? (
                <div className="flex items-start gap-2 text-base text-muted-foreground">
                  {needsSignIn(selectedReadiness) ? (
                    <CircleAlert
                      className="mt-0.5 size-4 shrink-0 text-warning"
                      aria-hidden
                    />
                  ) : (
                    <CheckCircle2
                      className="mt-0.5 size-4 shrink-0 text-success"
                      aria-hidden
                    />
                  )}
                  <div className="grid gap-1">
                    <span className="text-foreground">
                      {selectedReadiness.label}
                    </span>
                    {selectedReadiness.category === "sign-in-saved" ? (
                      <p>
                        The assistant may ask you to sign in again if it
                        expired.
                      </p>
                    ) : null}
                  </div>
                </div>
              ) : null}
              {needsSignIn(selectedReadiness) ? (
                <AgentSignIn agent={selectedAgent} onSignedIn={onSignedIn} />
              ) : null}
            </>
          ) : null}

          {step === "project" ? (
            <>
              {selectedAgentNeedsPreparation ? (
                <AssistantPreparation
                  agent={selectedAgent}
                  imageLoading={imageLoading}
                  imageStatus={selectedImageStatus}
                  imageError={imageError}
                  installing={imageInstalling}
                  onInstall={onInstallImage}
                  needsSignIn={selectedAgentNeedsSignIn}
                  onSignedIn={onSignedIn}
                />
              ) : null}
              <div className="rounded-lg bg-muted/50 p-4">
                <div className="flex flex-wrap items-center gap-3">
                  <FolderOpen
                    className="size-5 text-muted-foreground"
                    aria-hidden
                  />
                  <div className="min-w-0 flex-1">
                    <div className="text-base font-medium">
                      {request.workspacePath
                        ? projectName(request.workspacePath)
                        : "No project selected"}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {request.workspacePath
                        ? "This folder is ready to use."
                        : "Choose the folder that contains your project."}
                    </div>
                  </div>
                  <Button
                    type="button"
                    variant={request.workspacePath ? "outline" : "default"}
                    onClick={() => void onPickFolder()}
                  >
                    <FolderOpen />
                    Choose folder…
                  </Button>
                </div>
              </div>

              <details className="border-t pt-3">
                <summary className="cursor-pointer text-sm font-medium">
                  Advanced options
                </summary>
                <div className="mt-3 flex flex-col gap-5">
                  <fieldset className="grid gap-2 border-0 p-0">
                    <legend className="mb-2 text-sm font-medium">
                      Project access
                    </legend>
                    <div className="grid gap-2">
                      {(
                        [
                          {
                            value: "protected",
                            label: "Protected copy (recommended)",
                            detail:
                              "Keep changes separate until you review them.",
                            patch: { worktree: true, readOnlyWorkspace: false },
                          },
                          {
                            value: "read-only",
                            label: "Read-only original folder",
                            detail:
                              "The assistant can view your files but can’t change them.",
                            patch: { worktree: false, readOnlyWorkspace: true },
                          },
                          {
                            value: "original",
                            label: "Original folder with changes (less safe)",
                            detail:
                              "The assistant changes your original files directly.",
                            patch: {
                              worktree: false,
                              readOnlyWorkspace: false,
                            },
                          },
                        ] as const
                      ).map((choice) => (
                        <label
                          key={choice.value}
                          className="flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm"
                        >
                          <input
                            type="radio"
                            name="project-access"
                            checked={projectAccess === choice.value}
                            onChange={() => onRequestChange(choice.patch)}
                          />
                          <span>
                            <span className="block font-medium">
                              {choice.label}
                            </span>
                            <span className="text-muted-foreground">
                              {choice.detail}
                            </span>
                          </span>
                        </label>
                      ))}
                    </div>
                  </fieldset>

                  <AuthScopePicker
                    value={request.authScope}
                    onChange={(authScope) => onRequestChange({ authScope })}
                  />

                  <fieldset className="@container grid gap-2 border-0 p-0">
                    <legend className="mb-2 text-sm font-medium">
                      Network access
                    </legend>
                    <div className="grid gap-2 @[42rem]:grid-cols-3">
                      {(["provider", "internal", "internet"] as const).map(
                        (mode) => (
                          <label
                            key={mode}
                            className="flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm"
                          >
                            <input
                              type="radio"
                              name="network"
                              checked={request.networkMode === mode}
                              onChange={() => onNetworkChange(mode)}
                            />
                            <span>
                              <span className="block font-medium">
                                {networkLabels[mode]}
                              </span>
                              <span className="text-muted-foreground">
                                {networkHelp(mode)}
                              </span>
                            </span>
                          </label>
                        ),
                      )}
                    </div>
                    {offlineAgentNotice ? (
                      <Alert variant="warning" className="mt-3">
                        <CircleAlert />
                        <AlertTitle>This assistant will be offline</AlertTitle>
                        <AlertDescription>
                          {offlineAgentNotice}
                        </AlertDescription>
                      </Alert>
                    ) : null}
                    {brokerBypassNotice ? (
                      <Alert className="mt-3">
                        <CircleAlert />
                        <AlertTitle>Sign-in method changes</AlertTitle>
                        <AlertDescription>
                          {brokerBypassNotice}
                        </AlertDescription>
                      </Alert>
                    ) : null}
                  </fieldset>

                  <ResourceLimitsFields
                    cpus={request.cpus}
                    memory={request.memory}
                    onChange={onRequestChange}
                  />

                  <ApiKeyBrokerField
                    agent={selectedAgent}
                    value={request.apiKeyBrokerEnv}
                    onChange={(apiKeyBrokerEnv) =>
                      onRequestChange({ apiKeyBrokerEnv })
                    }
                  />
                </div>
              </details>
            </>
          ) : null}

          {step === "review" ? (
            <>
              {selectedAgentNeedsPreparation ? (
                <AssistantPreparation
                  agent={selectedAgent}
                  imageLoading={imageLoading}
                  imageStatus={selectedImageStatus}
                  imageError={imageError}
                  installing={imageInstalling}
                  onInstall={onInstallImage}
                  needsSignIn={selectedAgentNeedsSignIn}
                  onSignedIn={onSignedIn}
                />
              ) : null}
              <LaunchReview request={request} selectedAgent={selectedAgent} />
            </>
          ) : null}

          <div
            className={`flex gap-3 ${step === "agent" ? "justify-end" : "justify-between"}`}
          >
            {step !== "agent" ? (
              <Button type="button" variant="outline" onClick={onBack}>
                Back
              </Button>
            ) : null}
            <Button
              type="submit"
              disabled={
                step !== "agent"
                  ? !request.workspacePath ||
                    planning ||
                    selectedAgentNeedsPreparation
                  : !request.agent ||
                    !setupOk ||
                    imageLoading ||
                    !selectedImageReady
              }
            >
              {step === "review" && planning ? <ActivityIndicator /> : null}
              {step === "agent"
                ? "Continue"
                : step === "project"
                  ? "Continue"
                  : planning
                    ? "Preparing…"
                    : "Start session…"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}

function needsSignIn(readiness: AgentReadiness | undefined): boolean {
  return readiness?.category === "needs-sign-in"
}
