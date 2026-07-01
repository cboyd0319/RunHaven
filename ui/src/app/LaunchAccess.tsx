import {
  apiKeyBrokerEnvPlaceholder,
  authScopeHelp,
  authScopeLabels,
  supportsApiKeyBroker,
  type AgentProfile,
  type AuthStatusResponse,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { CircleAlert, ShieldCheck } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"

export function AgentAccessSummary({
  agent,
  authStatus,
  authScope,
}: {
  agent: AgentProfile | undefined
  authStatus: AuthStatusResponse | null
  authScope: RunPlanRequest["authScope"]
}) {
  if (!agent) {
    return null
  }
  const profileStatus = authProfileStatus(agent, authStatus)
  return (
    <div className="grid gap-3 rounded-lg border p-3 text-sm sm:grid-cols-4">
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
      <div>
        <div className="text-muted-foreground">Auth status</div>
        <Badge variant={authStatusBadgeVariant(profileStatus)}>
          {authStatusLabel(profileStatus)}
        </Badge>
      </div>
      <div>
        <div className="text-muted-foreground">Login state</div>
        <div className="font-medium">{authScopeLabels[authScope]}</div>
      </div>
    </div>
  )
}

export function AuthScopePicker({
  value,
  onChange,
}: {
  value: RunPlanRequest["authScope"]
  onChange: (value: RunPlanRequest["authScope"]) => void
}) {
  return (
    <fieldset className="rounded-lg border p-3">
      <legend className="px-1 text-sm font-medium">Login state</legend>
      <div className="grid gap-2 sm:grid-cols-2">
        {(["agent", "project"] as const).map((scope) => (
          <label
            key={scope}
            className="flex items-start gap-2 rounded-lg border p-3 text-sm"
          >
            <input
              type="radio"
              name="auth-scope"
              checked={value === scope}
              onChange={() => onChange(scope)}
            />
            <span>
              <span className="block font-medium">{authScopeLabels[scope]}</span>
              <span className="text-muted-foreground">{authScopeHelp(scope)}</span>
            </span>
          </label>
        ))}
      </div>
    </fieldset>
  )
}

export function ApiKeyBrokerField({
  agent,
  value,
  onChange,
}: {
  agent: AgentProfile | undefined
  value: string | null
  onChange: (value: string | null) => void
}) {
  if (!supportsApiKeyBroker(agent)) {
    return null
  }
  return (
    <label className="flex flex-col gap-1.5 text-sm font-medium">
      API key variable
      <Input
        value={value ?? ""}
        onChange={(event) => onChange(event.currentTarget.value.trim() || null)}
        placeholder={apiKeyBrokerEnvPlaceholder(agent)}
        aria-label="API key broker environment variable"
      />
      <span className="text-sm font-normal text-muted-foreground">
        Enter the host variable name. RunHaven keeps the key on this Mac and
        gives the session a placeholder.
      </span>
    </label>
  )
}

export function ImageReadiness({
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

function authProfileStatus(
  agent: AgentProfile,
  authStatus: AuthStatusResponse | null,
): string {
  return authStatus?.profiles.find((profile) => profile.name === agent.name)?.status ?? "-"
}

function authStatusLabel(status: string): string {
  switch (status.trim().toLowerCase()) {
    case "api-key-broker":
    case "brokered":
      return "API key broker"
    case "ready":
    case "available":
      return "Ready"
    case "n/a":
      return "Not needed"
    case "-":
      return "Unknown"
    case "design-only":
      return "Not ready"
    default:
      return status
  }
}

function authStatusBadgeVariant(
  status: string,
): "destructive" | "secondary" | "success" | "warning" {
  switch (status.trim().toLowerCase()) {
    case "api-key-broker":
    case "brokered":
    case "ready":
    case "available":
    case "n/a":
      return "success"
    case "design-only":
      return "warning"
    default:
      return "secondary"
  }
}
