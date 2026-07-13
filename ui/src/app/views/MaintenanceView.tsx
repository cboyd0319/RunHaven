import { CheckCircle2, CircleAlert, Network } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { useNetworkCleanup } from "../useMaintenance"

// The Maintenance destination: host upkeep that changes things, kept separate
// from read-only Diagnostics. Network cleanup runs through the run-control
// command, which requires a native confirmation in Rust before any change.
// Image rebuild lives in the Agents view. State prune stays CLI-only for
// safety and is not offered here. See docs/DESKTOP_APP.md.
export function MaintenanceView({ setupReady }: { setupReady: boolean }) {
  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Cleanup
        </CardTitle>
        <CardDescription>
          Remove RunHaven items that are no longer being used.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <NetworkCleanupSection setupReady={setupReady} />
      </CardContent>
    </Card>
  )
}

function NetworkCleanupSection({ setupReady }: { setupReady: boolean }) {
  const {
    count,
    loading,
    loadError,
    busy,
    actionError,
    result,
    loadPreview,
    prune,
  } = useNetworkCleanup()

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Network className="size-4" aria-hidden />
        Clean up stale networks
      </div>
      <p className="text-sm text-muted-foreground">
        RunHaven removes only its own leftover networks. Networks still used by a
        running session are skipped.
      </p>
      <div>
        <Button
          type="button"
          disabled={!setupReady || loading}
          onClick={() => void loadPreview()}
        >
          {loading ? "Checking..." : "Check for stale networks"}
        </Button>
      </div>
      {!setupReady ? (
        <p className="text-sm text-muted-foreground">
          Finish setup before checking for stale networks.
        </p>
      ) : null}
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
      {count === 0 ? (
        <p className="text-sm text-muted-foreground">
          Nothing to clean up right now.
        </p>
      ) : null}
      {count !== null && count > 0 ? (
        <div className="flex flex-wrap items-center gap-3">
          <Button
            type="button"
            variant="outline"
            disabled={!setupReady || busy}
            onClick={() => void prune()}
          >
            {busy ? "Removing..." : "Remove stale networks"}
          </Button>
          <span className="text-xs text-muted-foreground">
            RunHaven asks for confirmation in a Mac dialog.
          </span>
        </div>
      ) : null}
      {actionError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Cleanup failed</AlertTitle>
          <AlertDescription>{actionError}</AlertDescription>
        </Alert>
      ) : null}
      {result ? (
        <Alert variant={result.failed > 0 ? "destructive" : "default"}>
          {result.failed > 0 ? <CircleAlert /> : <CheckCircle2 />}
          <AlertTitle>
            {result.failed > 0
              ? "Cleanup needs attention"
              : "Cleanup finished"}
          </AlertTitle>
          <AlertDescription>
            {result.failed > 0
              ? `Removed ${result.removed}. Could not remove ${result.failed}.`
              : `Removed ${result.removed}.`}
          </AlertDescription>
        </Alert>
      ) : null}
    </section>
  )
}
