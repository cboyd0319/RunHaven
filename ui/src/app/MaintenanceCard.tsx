import { useState } from "react"
import { CheckCircle2, CircleAlert, Hammer, Network } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { useImageRebuild, useNetworkCleanup } from "./useMaintenance"

export type MaintenanceCardProps = {
  agents: string[]
}

// In-app recovery and housekeeping. Image rebuild and network cleanup both run
// through the run-control commands, which require a native confirmation in Rust
// before any change. State prune stays CLI-only for safety and is not offered
// here. See docs/DESKTOP_APP.md.
export function MaintenanceCard({ agents }: MaintenanceCardProps) {
  return (
    <Card aria-live="polite">
      {/* The "Maintenance" accordion header already names this card, so the
          header carries only the description instead of repeating the title. */}
      <CardHeader>
        <CardDescription>
          Rebuild an agent image or clean up stale RunHaven networks. Each action
          asks you to confirm before anything changes.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <ImageRebuildSection agents={agents} />
        <NetworkCleanupSection />
      </CardContent>
    </Card>
  )
}

function ImageRebuildSection({ agents }: { agents: string[] }) {
  const [agent, setAgent] = useState(agents[0] ?? "")
  const { confirm, setConfirm, busy, error, status, rebuild } = useImageRebuild()

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Hammer className="size-4" aria-hidden />
        Rebuild agent image
      </div>
      <p className="text-sm text-muted-foreground">
        Use this if an agent image is missing or out of date. RunHaven rebuilds it
        from the bundled template. Running sessions keep working until you launch a
        new one.
      </p>
      <label className="flex flex-col gap-1 text-sm">
        <span>Agent</span>
        <select
          className="rounded-md border bg-background p-2"
          value={agent}
          onChange={(event) => setAgent(event.target.value)}
          aria-label="Agent to rebuild"
        >
          {agents.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      </label>
      <label className="flex items-start gap-2 text-sm">
        <Checkbox
          checked={confirm}
          onCheckedChange={(checked) => setConfirm(checked === true)}
          aria-label="Confirm agent image rebuild"
        />
        <span>Yes, rebuild this agent image.</span>
      </label>
      <div>
        <Button
          type="button"
          disabled={busy || !confirm || !agent}
          onClick={() => void rebuild(agent)}
        >
          {busy ? "Starting..." : "Rebuild image"}
        </Button>
      </div>
      {error ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Rebuild failed to start</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ) : null}
      {status ? (
        <Alert>
          <CheckCircle2 />
          <AlertTitle>
            {status.label} ({status.agent})
          </AlertTitle>
          <AlertDescription className="flex flex-col gap-2">
            <span>{status.detail}</span>
            {status.outputTail ? (
              <pre className="max-h-48 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap">
                {status.outputTail}
              </pre>
            ) : null}
          </AlertDescription>
        </Alert>
      ) : null}
    </section>
  )
}

function NetworkCleanupSection() {
  const {
    count,
    loading,
    loadError,
    confirm,
    setConfirm,
    busy,
    actionError,
    result,
    loadPreview,
    prune,
  } = useNetworkCleanup()

  return (
    <section className="flex flex-col gap-3 border-t pt-6">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Network className="size-4" aria-hidden />
        Clean up stale networks
      </div>
      <p className="text-sm text-muted-foreground">
        RunHaven removes only its own leftover networks. Networks still used by a
        running session are skipped.
      </p>
      <div>
        <Button type="button" variant="outline" disabled={loading} onClick={() => void loadPreview()}>
          {loading ? "Checking..." : "Check for stale networks"}
        </Button>
      </div>
      {loadError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Could not check networks</AlertTitle>
          <AlertDescription>{loadError}</AlertDescription>
        </Alert>
      ) : null}
      {count !== null ? (
        <p className="text-sm">
          Stale RunHaven networks found: <strong>{count}</strong>
        </p>
      ) : null}
      {count !== null && count > 0 ? (
        <>
          <label className="flex items-start gap-2 text-sm">
            <Checkbox
              checked={confirm}
              onCheckedChange={(checked) => setConfirm(checked === true)}
              aria-label="Confirm network cleanup"
            />
            <span>Yes, remove these stale networks.</span>
          </label>
          <div>
            <Button type="button" disabled={busy || !confirm} onClick={() => void prune()}>
              {busy ? "Removing..." : "Remove stale networks"}
            </Button>
          </div>
        </>
      ) : null}
      {actionError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Cleanup failed</AlertTitle>
          <AlertDescription>{actionError}</AlertDescription>
        </Alert>
      ) : null}
      {result ? (
        <Alert>
          <CheckCircle2 />
          <AlertTitle>Cleanup finished</AlertTitle>
          <AlertDescription>
            Removed {result.removed}. Skipped {result.failed} still in use.
          </AlertDescription>
        </Alert>
      ) : null}
    </section>
  )
}
