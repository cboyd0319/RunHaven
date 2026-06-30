import {
  CircleAlert,
  ClipboardCheck,
  FolderOpen,
  ShieldCheck,
} from "lucide-react"

import type {
  AgentProfile,
  ImageStatusResponse,
  RunPlanRequest,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
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

import { networkHelp, networkLabels } from "./types"

type LaunchPlanCardProps = {
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
  onReviewPlan: () => Promise<void>
}

export function LaunchPlanCard({
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
  onReviewPlan,
}: LaunchPlanCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Choose agent
        </CardTitle>
        <CardDescription>
          Safe defaults are already selected.
        </CardDescription>
        <CardAction>
          <Select value={request.agent} onValueChange={onAgentChange}>
            <SelectTrigger className="w-[180px]">
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
        </CardAction>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault()
            void onReviewPlan()
          }}
        >
          <div className="grid gap-3 sm:grid-cols-2 2xl:grid-cols-3">
            {agents.map((agent) => (
              <button
                type="button"
                key={agent.name}
                className="rounded-lg border bg-background p-4 text-left transition hover:bg-muted/60"
                onClick={() => onAgentChange(agent.name)}
              >
                <div className="mb-3 flex items-center justify-between gap-3">
                  <div className="font-medium">{agent.name}</div>
                  <Badge variant="outline">
                    {agent.name === request.agent ? "Selected" : "Ready"}
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  {agent.description}
                </p>
              </button>
            ))}
          </div>

          {selectedAgent ? (
            <p className="text-sm text-muted-foreground">
              {selectedAgent.description}
            </p>
          ) : null}

          <ImageReadiness
            imageLoading={imageLoading}
            imageStatus={imageStatus}
            imageError={imageError}
          />

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

          <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
            <Checkbox
              checked={request.readOnlyWorkspace}
              onCheckedChange={(checked) =>
                onRequestChange({ readOnlyWorkspace: checked === true })
              }
            />
            <span>Read-only project folder</span>
          </label>

          <Button type="submit" disabled={!request.workspacePath || planning}>
            <ClipboardCheck />
            {planning ? "Reviewing..." : "Review plan"}
          </Button>
        </form>
      </CardContent>
    </Card>
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
        {imageStatus.image.ready ? "Image ready" : "Image needs rebuild"}
      </AlertTitle>
      <AlertDescription>
        {imageStatus.image.image}
        {imageStatus.image.fixCommand ? `, ${imageStatus.image.fixCommand}` : ""}
      </AlertDescription>
    </Alert>
  )
}
