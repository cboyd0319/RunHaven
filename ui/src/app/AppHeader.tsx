import { RefreshCw } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { SidebarTrigger, useSidebar } from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

type AppHeaderProps = {
  title: string
  description: string
  setupKnown: boolean
  setupReady: boolean
  loading: boolean
  onRefresh: () => void
}

export function AppHeader({
  title,
  description,
  setupKnown,
  setupReady,
  loading,
  onRefresh,
}: AppHeaderProps) {
  const status = !setupKnown || loading ? "Checking" : setupReady ? "Ready" : "Blocked"
  const variant = !setupKnown || loading ? "outline" : setupReady ? "success" : "destructive"
  const { state: sidebarState } = useSidebar()

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
      <div className="min-w-0">
        <h1 className="text-lg font-semibold">{title}</h1>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      <div className="ml-auto flex items-center gap-2">
        <Badge variant={variant}>{status}</Badge>
        <Button
          type="button"
          variant="outline"
          size="icon-sm"
          onClick={onRefresh}
          aria-label="Refresh status"
        >
          <RefreshCw />
        </Button>
      </div>
    </header>
  )
}
