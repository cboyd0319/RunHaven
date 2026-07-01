import {
  type AgentProfile,
  type AuthStatusResponse,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { ClipboardCheck, FolderOpen } from "lucide-react"
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

import { AgentAuthDetailsCard } from "./AgentAuthDetailsCard"
import { AgentSignIn } from "./AgentSignIn"
import {
  AgentAccessSummary,
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
  request: RunPlanRequest
  selectedAgent: AgentProfile | undefined
  authStatus: AuthStatusResponse | null
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  planning: boolean
  onAgentChange: (agentName: string) => void
  onPickFolder: () => Promise<void>
  onRequestChange: (patch: Partial<RunPlanRequest>) => void
  onNetworkChange: (networkMode: RunPlanRequest["networkMode"]) => void
  onBack: () => void
  onContinue: () => void
  onReviewPlan: () => Promise<void>
  onSignedIn: () => void
}

export function LaunchPlanCard({
  step,
  agents,
  request,
  selectedAgent,
  authStatus,
  imageLoading,
  imageStatus,
  imageError,
  planning,
  onAgentChange,
  onPickFolder,
  onRequestChange,
  onNetworkChange,
  onBack,
  onContinue,
  onReviewPlan,
  onSignedIn,
}: LaunchPlanCardProps) {
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
              />
              <AgentAccessSummary
                agent={selectedAgent}
                authStatus={authStatus}
                authScope={request.authScope}
              />
              <AgentSignIn agent={selectedAgent} onSignedIn={onSignedIn} />
              {selectedAgent ? (
                <AgentAuthDetailsCard
                  key={selectedAgent.name}
                  agentName={selectedAgent.name}
                />
              ) : null}
            </>
          ) : null}

          {step === "project" ? (
            <>
              <label className="flex flex-col gap-1.5 text-sm font-medium">
                Project folder
                <div className="flex gap-2">
                  <Input
                    value={request.workspacePath}
                    onChange={(event) =>
                      onRequestChange({ workspacePath: event.currentTarget.value })
                    }
                    placeholder="/Users/you/project"
                    aria-label="Project folder path"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    onClick={() => void onPickFolder()}
                    aria-label="Choose project folder"
                  >
                    <FolderOpen />
                  </Button>
                </div>
              </label>

              <AuthScopePicker
                value={request.authScope}
                onChange={(authScope) => onRequestChange({ authScope })}
              />

              <details className="rounded-lg border p-3">
                <summary className="cursor-pointer text-sm font-medium">
                  Advanced options
                </summary>
                <div className="mt-3 flex flex-col gap-4">
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
                  </fieldset>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      CPU
                      <Input
                        value={request.cpus}
                        onChange={(event) =>
                          onRequestChange({ cpus: event.currentTarget.value })
                        }
                      />
                    </label>
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      Memory
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

                  <label className="flex items-start gap-3 rounded-lg border p-3 text-sm">
                    <Checkbox
                      checked={request.worktree}
                      onCheckedChange={(checked) =>
                        onRequestChange({ worktree: checked === true })
                      }
                      aria-label="Run in an isolated copy (git worktree)"
                    />
                    <span className="flex flex-col gap-1">
                      <span className="font-medium">
                        Run in an isolated copy (git worktree)
                      </span>
                      <span className="text-muted-foreground">
                        RunHaven makes a separate copy of your project on a new
                        git branch. Your project folder must be a clean git repo.
                        Your original files change only if you merge the copy
                        back later.
                      </span>
                    </span>
                  </label>
                </div>
              </details>

              <WhyBoundaryCard
                agentName={selectedAgent?.name}
                networkMode={request.networkMode}
                workspacePath={request.workspacePath}
                workspaceScope={request.workspaceScope}
                allowSensitiveWorkspace={request.allowSensitiveWorkspace}
              />
            </>
          ) : null}

          <div className={step === "agent" ? "flex justify-end" : "flex justify-between gap-3"}>
            {step === "project" ? (
              <Button type="button" variant="outline" onClick={onBack}>
                Back
              </Button>
            ) : null}
            <Button
              type="submit"
              disabled={
                step === "project"
                  ? !request.workspacePath || planning
                  : !request.agent
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
