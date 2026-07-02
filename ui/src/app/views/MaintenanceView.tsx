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
import { Checkbox } from "@/components/ui/checkbox"

import { useNetworkCleanup } from "../useMaintenance"

// The Maintenance destination: host upkeep that changes things, kept separate
// from read-only Diagnostics. Network cleanup runs through the run-control
// command, which requires a native confirmation in Rust before any change.
// Image rebuild lives in the Agents view. State prune stays CLI-only for
// safety and is not offered here. See docs/DESKTOP_APP.md.
export function MaintenanceView() {
  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Maintenance
        </CardTitle>
        <CardDescription>
          Clean up what RunHaven left behind. Each action asks you to confirm
          before anything changes.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <NetworkCleanupSection />
        <div className="flex flex-col gap-2 border-t pt-4 text-sm text-muted-foreground">
          <p>To rebuild an agent image, open Agents in the sidebar.</p>
          <p>
            Resetting an agent&apos;s saved state stays in the terminal on
            purpose, because it can erase a saved sign-in. Run{" "}
            <code className="font-mono">runhaven state reset &lt;agent&gt;</code>{" "}
            in your project folder.
          </p>
        </div>
      </CardContent>
    </Card>
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
        <Button type="button" disabled={loading} onClick={() => void loadPreview()}>
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
      {count === 0 ? (
        <p className="text-sm text-muted-foreground">
          Nothing to clean up right now.
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
            <Button type="button" variant="outline" disabled={busy || !confirm} onClick={() => void prune()}>
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
