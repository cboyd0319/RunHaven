import {
  apiKeyBrokerEnvPlaceholder,
  supportsApiKeyBroker,
  type AgentProfile,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import {
  CircleAlert,
  ClipboardCheck,
  FolderOpen,
  ShieldCheck,
} from "lucide-react"
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

import { LaunchSteps } from "./LaunchSteps"
import { networkHelp, networkLabels } from "./types"
import type { LaunchStep } from "./types"

type LaunchPlanCardProps = {
  step: Exclude<LaunchStep, "review">
  agents: AgentProfile[]
  request: RunPlanRequest
  selectedAgent: AgentProfile | undefined
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
}

export function LaunchPlanCard({
  step,
  agents,
  request,
  selectedAgent,
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
}: LaunchPlanCardProps) {
  const brokerAvailable = supportsApiKeyBroker(selectedAgent)
  const brokerPlaceholder = apiKeyBrokerEnvPlaceholder(selectedAgent)

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
                          {agent.name}
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
              <AgentAccessSummary agent={selectedAgent} />
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

                  {brokerAvailable ? (
                    <label className="flex flex-col gap-1.5 text-sm font-medium">
                      API key variable
                      <Input
                        value={request.apiKeyBrokerEnv ?? ""}
                        onChange={(event) =>
                          onRequestChange({
                            apiKeyBrokerEnv:
                              event.currentTarget.value.trim() || null,
                          })
                        }
                        placeholder={brokerPlaceholder}
                        aria-label="API key broker environment variable"
                      />
                      <span className="text-sm font-normal text-muted-foreground">
                        Enter the host variable name. RunHaven keeps the key on
                        this Mac and gives the session a placeholder.
                      </span>
                    </label>
                  ) : null}

                  <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
                    <Checkbox
                      checked={request.readOnlyWorkspace}
                      onCheckedChange={(checked) =>
                        onRequestChange({ readOnlyWorkspace: checked === true })
                      }
                    />
                    <span>Read-only project folder</span>
                  </label>
                </div>
              </details>
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

function AgentAccessSummary({
  agent,
}: {
  agent: AgentProfile | undefined
}) {
  if (!agent) {
    return null
  }
  return (
    <div className="grid gap-3 rounded-lg border p-3 text-sm sm:grid-cols-2">
      <div>
        <div className="text-muted-foreground">Sign-in method</div>
        <div className="font-medium">{authMethodLabel(agent.signIn)}</div>
      </div>
      <div>
        <div className="text-muted-foreground">Credential broker</div>
        <Badge variant={agent.broker === "yes" ? "success" : "secondary"}>
          {brokerLabel(agent.broker)}
        </Badge>
      </div>
    </div>
  )
}

function ImageReadiness({
  imageLoading,
  imageStatus,
  imageError,
}: {
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
}) {
  if (imageLoading) {
    return <p className="text-sm text-muted-foreground">Checking image...</p>
  }
  if (imageError) {
    return (
      <Alert variant="destructive">
        <CircleAlert />
        <AlertTitle>Image status unavailable</AlertTitle>
        <AlertDescription>{imageError}</AlertDescription>
      </Alert>
    )
  }
  if (!imageStatus) {
    return null
  }
  return (
    <Alert>
      <ShieldCheck />
      <AlertTitle>
        {imageStatus.image.ready ? "Agent ready" : "Agent setup needed"}
      </AlertTitle>
      <AlertDescription>
        {imageStatus.image.ready
          ? "This agent is installed."
          : imageStatus.image.fixCommand}
      </AlertDescription>
    </Alert>
  )
}

function authMethodLabel(value: string): string {
  if (value === "n/a") {
    return "Not needed"
  }
  return value
}

function brokerLabel(value: string): string {
  if (value === "yes") {
    return "Available"
  }
  if (value === "n/a") {
    return "Not needed"
  }
  return "Not available"
}
