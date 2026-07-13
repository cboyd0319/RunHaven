import {
  supportsApiKeyBroker,
  type AgentProfile,
  type AgentReadiness,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { CircleAlert, ClipboardCheck, FolderOpen } from "lucide-react"
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
import { Checkbox } from "@/components/ui/checkbox"
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
import { WhyBoundaryCard } from "./WhyBoundaryCard"
import { networkLabels } from "./labels"
import { agentLabel } from "./sessionIdentity"
import { LaunchSteps } from "./LaunchSteps"
import { networkHelp } from "./types"
import type { LaunchStep } from "./types"

type LaunchPlanCardProps = {
  step: Exclude<LaunchStep, "review">
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
  onReviewPlan: () => Promise<void>
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
  onReviewPlan,
  onSignedIn,
}: LaunchPlanCardProps) {
  const selectedReadiness = selectedAgent
    ? agentReadiness.find((item) => item.name === selectedAgent.name)
    : undefined
  const alternateNetwork = request.networkMode !== "provider"
  const brokerBypassNotice = !alternateNetwork
    ? null
    : supportsApiKeyBroker(selectedAgent)
        ? "This choice does not use RunHaven's host API key broker. The agent may ask you to sign in inside the sandbox."
        : null
  const selectedImageReady = imageStatus?.image.ready === true
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {step === "agent" ? "Choose agent" : "Choose project"}
        </CardTitle>
        <CardDescription>
          {step === "agent"
            ? "Pick the agent you want to run."
            : "Pick the project folder the agent can use."}
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
              void onReviewPlan()
            }
          }}
        >
          <LaunchSteps step={step} />

          {step === "agent" ? (
            <>
              <label className="flex flex-col gap-1.5 text-sm font-medium">
                Agent
                <Select value={request.agent} onValueChange={onAgentChange}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select agent" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {agents.map((agent) => (
                        <SelectItem key={agent.name} value={agent.name}>
                          {agentLabel(agent.name)}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
                {selectedAgent ? (
                  <span className="text-sm font-normal text-muted-foreground">
                    {selectedAgent.description}
                  </span>
                ) : null}
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
              <div className="rounded-lg border p-4">
                <div className="flex flex-wrap items-center gap-3">
                  <FolderOpen className="size-5 text-muted-foreground" aria-hidden />
                  <div className="min-w-0 flex-1">
                    <div className="text-sm font-medium">
                      {request.workspacePath
                        ? projectName(request.workspacePath)
                        : "No project selected"}
                    </div>
                    <div className="truncate text-sm text-muted-foreground">
                      {request.workspacePath ||
                        "Choose the folder that contains your project."}
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

              <label className="flex items-start gap-3 rounded-lg border p-3 text-sm">
                <Checkbox
                  checked={request.worktree}
                  onCheckedChange={(checked) =>
                    onRequestChange({ worktree: checked === true })
                  }
                  aria-label="Run in an isolated copy (git worktree)"
                />
                <span className="flex flex-col gap-1">
                  <span className="flex flex-wrap items-center gap-2 font-medium">
                    Protect my project with an isolated copy
                    <Badge variant="secondary">Recommended</Badge>
                  </span>
                  <span className="text-muted-foreground">
                    RunHaven works in a separate copy on a new git branch. Your
                    project folder must be a clean git repo. Your original files
                    change only if you merge the copy later. Turn this off to
                    work directly in your project folder.
                  </span>
                </span>
              </label>

              <details className="rounded-lg border p-3">
                <summary className="cursor-pointer text-sm font-medium">
                  Advanced options
                </summary>
                <div className="mt-3 flex flex-col gap-4">
                  <label className="flex flex-col gap-1.5 text-sm font-medium">
                    Project folder path
                    <Input
                      value={request.workspacePath}
                      onChange={(event) =>
                        onRequestChange({ workspacePath: event.currentTarget.value })
                      }
                      placeholder="/Users/you/project"
                      aria-label="Project folder path"
                    />
                  </label>

                  <AuthScopePicker
                    value={request.authScope}
                    onChange={(authScope) => onRequestChange({ authScope })}
                  />

                  <fieldset className="rounded-lg border p-3">
                    <legend className="px-1 text-sm font-medium">Network access</legend>
                    <div className="grid gap-2 sm:grid-cols-3">
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
                        value={request.cpus}
                        onChange={(event) =>
                          onRequestChange({ cpus: event.currentTarget.value })
                        }
                      />
                    </label>
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      Memory (4g = 4 GB)
                      <Input
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

                  <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
                    <Checkbox
                      checked={request.readOnlyWorkspace}
                      onCheckedChange={(checked) =>
                        onRequestChange({ readOnlyWorkspace: checked === true })
                      }
                    />
                    <span>Read-only project folder</span>
                  </label>

                  <WhyBoundaryCard
                    agentName={selectedAgent?.name}
                    networkMode={request.networkMode}
                    workspacePath={request.workspacePath}
                    workspaceScope={request.workspaceScope}
                    allowSensitiveWorkspace={request.allowSensitiveWorkspace}
                  />
                </div>
              </details>
            </>
          ) : null}

          {/* HIG multi-step sheet chrome: Cancel leads step 1, Back leads
              later steps, the primary action trails. */}
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
                  ? !request.workspacePath || planning
                  : !request.agent || !setupOk || imageLoading || !selectedImageReady
              }
            >
              <ClipboardCheck />
              {step === "agent"
                ? "Continue"
                : planning
                  ? "Reviewing..."
                  : "Review access"}
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
