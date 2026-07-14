import {
  apiKeyBrokerEnvForAgent,
  authScopeHelp,
  supportsApiKeyBroker,
  type AgentProfile,
  type ImageStatusResponse,
  type RunPlanRequest,
} from "@/commands/runhaven"
import { CircleAlert, ShieldCheck } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { ActivityIndicator } from "@/components/ui/activity-indicator"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"

import { authScopeLabels } from "./labels"
import { AgentSignIn } from "./AgentSignIn"
import { agentLabel } from "./sessionIdentity"

export function AuthScopePicker({
  value,
  onChange,
}: {
  value: RunPlanRequest["authScope"]
  onChange: (value: RunPlanRequest["authScope"]) => void
}) {
  return (
    <fieldset className="grid gap-2 border-0 p-0">
      <legend className="mb-2 text-sm font-medium">Sign-in sharing</legend>
      <div className="grid gap-2 sm:grid-cols-2">
        {(["agent", "project"] as const).map((scope) => (
          <label
            key={scope}
            className="flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm"
          >
            <input
              type="radio"
              name="auth-scope"
              checked={value === scope}
              onChange={() => onChange(scope)}
            />
            <span>
              <span className="block font-medium">
                {authScopeLabels[scope]}
              </span>
              <span className="text-muted-foreground">
                {authScopeHelp(scope)}
              </span>
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
  const envName = apiKeyBrokerEnvForAgent(agent)
  if (!envName || !supportsApiKeyBroker(agent)) {
    return null
  }
  return (
    <label className="flex items-start gap-3 rounded-lg bg-muted/50 p-3 text-sm">
      <Checkbox
        checked={value !== null}
        onCheckedChange={(checked) =>
          onChange(checked === true ? envName : null)
        }
        aria-label={`Use ${envName} from this Mac`}
      />
      <span className="flex flex-col gap-1">
        <span className="font-medium">Use an API key from this Mac</span>
        <span className="text-muted-foreground">
          RunHaven keeps <code className="font-mono">{envName}</code> on this
          Mac and gives the assistant temporary access for this session. This
          option uses Assistant services only.
        </span>
      </span>
    </label>
  )
}

export function ImageReadiness({
  imageLoading,
  imageStatus,
  imageError,
  installing = false,
  onInstall,
}: {
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  installing?: boolean
  onInstall?: () => void
}) {
  if (imageLoading) {
    return (
      <p
        role="status"
        className="flex items-center gap-2 text-base text-muted-foreground"
      >
        <ActivityIndicator />
        Checking assistant setup…
      </p>
    )
  }
  if (imageError) {
    return (
      <Alert variant="destructive" role="alert">
        <CircleAlert />
        <AlertTitle>Couldn’t check assistant setup</AlertTitle>
        <AlertDescription>{imageError}</AlertDescription>
      </Alert>
    )
  }
  if (!imageStatus) {
    return null
  }
  if (imageStatus.image.ready) {
    return (
      <p
        role="status"
        className="flex items-center gap-2 text-base text-muted-foreground"
      >
        <ShieldCheck className="size-4 text-success" aria-hidden />
        Assistant ready
      </p>
    )
  }
  return (
    <Alert>
      <ShieldCheck />
      <AlertTitle>Assistant setup needed</AlertTitle>
      <AlertDescription>
        <span className="flex flex-col items-start gap-2">
          <span>
            {onInstall
              ? "Install this assistant before starting a session."
              : "This assistant must be installed before continuing."}
          </span>
          {onInstall ? (
            <Button type="button" disabled={installing} onClick={onInstall}>
              {installing ? <ActivityIndicator /> : null}
              {installing
                ? `Installing ${agentLabel(imageStatus.agent)}…`
                : `Install ${agentLabel(imageStatus.agent)}…`}
            </Button>
          ) : null}
        </span>
      </AlertDescription>
    </Alert>
  )
}

export function AssistantPreparation({
  agent,
  imageLoading,
  imageStatus,
  imageError,
  installing,
  onInstall,
  needsSignIn,
  onSignedIn,
}: {
  agent: AgentProfile | undefined
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  installing: boolean
  onInstall?: () => void
  needsSignIn: boolean
  onSignedIn: () => void
}) {
  return (
    <div className="flex flex-col gap-3">
      <ImageReadiness
        imageLoading={imageLoading}
        imageStatus={imageStatus}
        imageError={imageError}
        installing={installing}
        onInstall={imageStatus && !imageStatus.image.ready ? onInstall : undefined}
      />
      {needsSignIn ? <AgentSignIn agent={agent} onSignedIn={onSignedIn} /> : null}
    </div>
  )
}
