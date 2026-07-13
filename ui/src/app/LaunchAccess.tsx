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
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"

import { authScopeLabels } from "./labels"
import { agentLabel } from "./sessionIdentity"

export function AuthScopePicker({
  value,
  onChange,
}: {
  value: RunPlanRequest["authScope"]
  onChange: (value: RunPlanRequest["authScope"]) => void
}) {
  return (
    <fieldset className="rounded-lg border p-3">
      <legend className="px-1 text-sm font-medium">Sign-in sharing</legend>
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
  const envName = apiKeyBrokerEnvForAgent(agent)
  if (!envName || !supportsApiKeyBroker(agent)) {
    return null
  }
  return (
    <label className="flex items-start gap-3 rounded-lg border p-3 text-sm">
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
          RunHaven keeps <code className="font-mono">{envName}</code> on this Mac
          and gives the assistant temporary access for this session. This option
          uses Assistant services only.
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
      <p className="text-sm text-muted-foreground">
        Checking assistant setup…
      </p>
    )
  }
  if (imageError) {
    return (
      <Alert variant="destructive">
        <CircleAlert />
        <AlertTitle>Couldn’t check assistant setup</AlertTitle>
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
        {imageStatus.image.ready ? "Assistant ready" : "Assistant setup needed"}
      </AlertTitle>
      <AlertDescription>
        {imageStatus.image.ready ? (
          "This assistant is installed."
        ) : (
          <span className="flex flex-col items-start gap-2">
            <span>
              {onInstall
                ? "Install this assistant before starting a session."
                : "This assistant must be installed before continuing."}
            </span>
            {onInstall ? (
              <Button
                type="button"
                disabled={installing}
                onClick={onInstall}
              >
                {installing
                  ? `Installing ${agentLabel(imageStatus.agent)}…`
                  : `Install ${agentLabel(imageStatus.agent)}`}
              </Button>
            ) : null}
          </span>
        )}
      </AlertDescription>
    </Alert>
  )
}
