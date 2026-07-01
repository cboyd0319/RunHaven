import { RefreshCw } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { SidebarTrigger } from "@/components/ui/sidebar"

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

  return (
    <header className="flex h-14 shrink-0 items-center gap-3 border-b px-4">
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
