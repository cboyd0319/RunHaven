import { CircleAlert, ShieldCheck } from "lucide-react"
import { useCallback, useState } from "react"

import { getAuthExplain, type AuthExplainResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

import {
  authDetailSections,
  authStatusBadgeVariant,
  authStatusLabel,
} from "./authDetails"
import { errorMessage } from "./types"

// On-demand, read-only auth details for the selected agent. State is local and
// resets when the parent remounts this with a new agent key. The command is
// read-only: it never sends input and shows no secrets, tokens, or host paths.
export function AgentAuthDetailsCard({ agentName }: { agentName: string }) {
  const [details, setDetails] = useState<AuthExplainResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")

  const load = useCallback(async () => {
    setLoading(true)
    setError("")
    try {
      setDetails(await getAuthExplain(agentName))
    } catch (cause) {
      setDetails(null)
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }, [agentName])

  return (
    <section className="rounded-lg border p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h3 className="text-sm font-medium">Auth details</h3>
          <p className="text-sm text-muted-foreground">
            See how this agent signs in and what stays on this Mac. No secrets are
            shown.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => void load()}
        >
          <ShieldCheck className="size-4" />
          {loading ? "Loading..." : details ? "Refresh" : "Show auth details"}
        </Button>
      </div>
      {error ? (
        <Alert variant="destructive" className="mt-3">
          <CircleAlert />
          <AlertTitle>Auth details unavailable</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ) : null}
      {details ? (
        <div className="mt-3 flex flex-col gap-3 text-sm">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-muted-foreground">Auth status</span>
            <Badge variant={authStatusBadgeVariant(details.status)}>
              {authStatusLabel(details.status)}
            </Badge>
          </div>
          {authDetailSections(details).map((section) => (
            <div key={section.label}>
              <div className="font-medium">{section.label}</div>
              <ul className="ml-4 list-disc text-muted-foreground">
                {section.items.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
          ))}
          {details.currentSafePath ? (
            <p className="text-muted-foreground">{details.currentSafePath}</p>
          ) : null}
        </div>
      ) : null}
    </section>
  )
}
