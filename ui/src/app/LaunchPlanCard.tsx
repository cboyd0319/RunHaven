import {
  supportsApiKeyBroker,
  type AgentProfile,
  type AgentReadiness,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { CircleAlert, FolderOpen } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
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
  AuthScopePicker,
  ImageReadiness,
} from "./LaunchAccess"
import { networkLabels } from "./labels"
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
  onCancel: () => void
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
  onCancel,
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
  const selectedImageReady = imageStatus?.image.ready === true
  const selectedAgentNeedsSignIn = needsSignIn(selectedReadiness)
  const selectedAgentNeedsPreparation =
    imageLoading ||
    Boolean(imageError) ||
    (imageStatus !== null && !selectedImageReady) ||
    selectedAgentNeedsSignIn
  const projectAccess = request.worktree
    ? "protected"
    : request.readOnlyWorkspace
      ? "read-only"
      : "original"
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {step === "agent"
            ? "Choose assistant"
            : "Choose project"}
        </CardTitle>
        <CardDescription>
          {step === "agent"
            ? "Pick the coding assistant you want to use."
            : selectedAgent
              ? `Pick the project folder ${agentLabel(selectedAgent.name)} can use.`
              : "Pick the project folder this assistant can use."}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault()
            if (step === "agent") {
              onContinue()
            } else {
              void onStartSession()
            }
          }}
        >
          {step === "agent" ? (
            <>
              <label className="flex flex-col gap-1.5 text-sm font-medium">
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
                imageStatus={imageStatus}
                imageError={imageError}
                installing={imageInstalling}
                onInstall={
                  imageStatus && !imageStatus.image.ready
                    ? onInstallImage
                    : undefined
                }
              />
              {selectedReadiness ? (
                <div className="flex items-center justify-between gap-3 rounded-lg border p-3 text-sm">
                  <span>Sign-in</span>
                  <Badge variant={needsSignIn(selectedReadiness) ? "warning" : "success"}>
                    {selectedReadiness.label}
                  </Badge>
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
                <div className="flex flex-col gap-3">
                  <ImageReadiness
                    imageLoading={imageLoading}
                    imageStatus={imageStatus}
                    imageError={imageError}
                    installing={imageInstalling}
                    onInstall={
                      imageStatus && !imageStatus.image.ready
                        ? onInstallImage
                        : undefined
                    }
                  />
                  {selectedAgentNeedsSignIn ? (
                    <AgentSignIn agent={selectedAgent} onSignedIn={onSignedIn} />
                  ) : null}
                </div>
              ) : null}
              <div className="rounded-lg border p-4">
                <div className="flex flex-wrap items-center gap-3">
                  <FolderOpen className="size-5 text-muted-foreground" aria-hidden />
                  <div className="min-w-0 flex-1">
                    <div className="text-sm font-medium">
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
                    Choose folder
                  </Button>
                </div>
              </div>

              <details className="rounded-lg border p-3">
                <summary className="cursor-pointer text-sm font-medium">
                  Advanced options
                </summary>
                <div className="mt-3 flex flex-col gap-4">
                  <fieldset className="rounded-lg border p-3">
                    <legend className="px-1 text-sm font-medium">Project access</legend>
                    <div className="grid gap-2">
                      {([
                        {
                          value: "protected",
                          label: "Protected copy (recommended)",
                          detail: "Keep changes separate until you review them.",
                          patch: { worktree: true, readOnlyWorkspace: false },
                        },
                        {
                          value: "read-only",
                          label: "Read-only original folder",
                          detail: "The assistant can view your files but can’t change them.",
                          patch: { worktree: false, readOnlyWorkspace: true },
                        },
                        {
                          value: "original",
                          label: "Original folder with changes (less safe)",
                          detail: "The assistant changes your original files directly.",
                          patch: { worktree: false, readOnlyWorkspace: false },
                        },
                      ] as const).map((choice) => (
                        <label
                          key={choice.value}
                          className="flex items-start gap-2 rounded-lg border p-3 text-sm"
                        >
                          <input
                            type="radio"
                            name="project-access"
                            checked={projectAccess === choice.value}
                            onChange={() => onRequestChange(choice.patch)}
                          />
                          <span>
                            <span className="block font-medium">{choice.label}</span>
                            <span className="text-muted-foreground">{choice.detail}</span>
                          </span>
                        </label>
                      ))}
                    </div>
                  </fieldset>

                  <AuthScopePicker
                    value={request.authScope}
                    onChange={(authScope) => onRequestChange({ authScope })}
                  />

                  <fieldset className="@container rounded-lg border p-3">
                    <legend className="px-1 text-sm font-medium">Network access</legend>
                    <div className="grid gap-2 @[42rem]:grid-cols-3">
                      {(["provider", "internal", "internet"] as const).map((mode) => (
                        <label
                          key={mode}
                          className="flex items-start gap-2 rounded-lg border p-3 text-sm"
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
                      ))}
                    </div>
                    {offlineAgentNotice ? (
                      <Alert variant="warning" className="mt-3">
                        <CircleAlert />
                        <AlertTitle>This assistant will be offline</AlertTitle>
                        <AlertDescription>{offlineAgentNotice}</AlertDescription>
                      </Alert>
                    ) : null}
                    {brokerBypassNotice ? (
                      <Alert className="mt-3">
                        <CircleAlert />
                        <AlertTitle>Sign-in method changes</AlertTitle>
                        <AlertDescription>{brokerBypassNotice}</AlertDescription>
                      </Alert>
                    ) : null}
                  </fieldset>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      CPU cores
                      <Input
                        inputMode="decimal"
                        pattern="[1-9][0-9]*(\.[0-9]+)?"
                        required
                        title="Enter a CPU count such as 4 or 2.5."
                        value={request.cpus}
                        onChange={(event) =>
                          onRequestChange({ cpus: event.currentTarget.value })
                        }
                      />
                    </label>
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      Memory
                      <Input
                        inputMode="text"
                        pattern="[1-9][0-9]*[KkMmGgTtPp]?"
                        required
                        title="Enter memory such as 4g for 4 GB or 4096m."
                        value={request.memory}
                        onChange={(event) =>
                          onRequestChange({ memory: event.currentTarget.value })
                        }
                      />
                    </label>
                  </div>

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

          {/* Cancel leads the first step, Back leads later steps, and the
              primary action trails. */}
          <div className="flex justify-between gap-3">
            {step === "agent" ? (
              <Button type="button" variant="outline" onClick={onCancel}>
                Cancel
              </Button>
            ) : (
              <Button type="button" variant="outline" onClick={onBack}>
                Back
              </Button>
            )}
            <Button
              type="submit"
              disabled={
                step === "project"
                  ? !request.workspacePath ||
                    planning ||
                    imageLoading ||
                    (imageStatus !== null && !selectedImageReady) ||
                    selectedAgentNeedsSignIn
                  : !request.agent || !setupOk || imageLoading || !selectedImageReady
              }
            >
              {step === "agent"
                ? "Continue"
                : planning
                  ? "Preparing…"
                  : "Start session"}
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

function projectName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean)
  return parts.at(-1) ?? path
}
