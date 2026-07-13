import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { SidebarTrigger, useSidebar } from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

import {
  sessionAllowsRunControl,
  sessionBadgeVariant,
  sessionNeedsAttention,
} from "./ManagedSessionCard"
import type { RunControlAction } from "./types"

type AppHeaderProps = {
  title?: string
  session?: ManagedSessionSummary | null
  controlBusy?: boolean
  onControl?: (action: RunControlAction) => Promise<void>
}

export function AppHeader({
  title,
  session = null,
  controlBusy = false,
  onControl,
}: AppHeaderProps) {
  const { state: sidebarState } = useSidebar()
  const canStop = session !== null && sessionAllowsRunControl(session)

  return (
    // data-tauri-drag-region: with the overlay title bar the header doubles as
    // the draggable top strip (drag starts only when the header itself is the
    // mousedown target, so the buttons inside keep working normally).
    <header
      data-tauri-drag-region
      className={cn(
        "flex h-14 shrink-0 items-center gap-3 border-b px-4",
        // Collapsed icon rail (~48px) is narrower than the macOS traffic
        // lights (~70px), which then spill over the header's leading edge;
        // inset the sidebar toggle so it stays visible right of the lights.
        sidebarState === "collapsed" && "pl-8",
      )}
    >
      <SidebarTrigger variant="outline" />
      {title ? (
        <div className="flex min-w-0 items-center gap-2">
          <h1 className="truncate text-lg font-semibold">{title}</h1>
          {session ? (
            <Badge variant={sessionBadgeVariant(session)}>
              {sessionNeedsAttention(session) ? "Needs you" : session.statusLabel}
            </Badge>
          ) : null}
        </div>
      ) : null}
      {canStop && onControl ? (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="ml-auto"
          aria-label="Stop session"
          disabled={controlBusy}
          onClick={() => void onControl("stop")}
        >
          Stop
        </Button>
      ) : null}
    </header>
  )
}
