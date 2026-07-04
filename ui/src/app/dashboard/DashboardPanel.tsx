import type { ReactNode } from "react"
import { ChevronRight, type LucideIcon } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"

// Shared chrome for the Home dashboard panels (Needs review, Recent activity,
// Agent setup, Secure boundary). One panel shell so borders, header, count
// badge, and footer link read identically across the page (design doc: one
// subtle border per surface, no nested card stacks, consistent row actions).
export function DashboardPanel({
  title,
  count,
  description,
  footer,
  children,
  className,
}: {
  title: string
  // A count badge next to the title (e.g. Needs review 3). Omit to hide it.
  count?: number
  description?: string
  // A full-width footer action row (e.g. Open Agents), rendered as a divider row.
  footer?: ReactNode
  children: ReactNode
  className?: string
}) {
  return (
    <Card className={cn("gap-0 py-0", className)}>
      <CardHeader className="gap-1 px-5 pt-5 pb-3">
        <div className="flex items-center gap-2">
          <CardTitle role="heading" aria-level={2} className="text-base font-semibold">
            {title}
          </CardTitle>
          {typeof count === "number" ? (
            <Badge variant="secondary" className="tabular-nums">
              {count}
            </Badge>
          ) : null}
        </div>
        {description ? (
          <p className="text-sm text-muted-foreground">{description}</p>
        ) : null}
      </CardHeader>
      <CardContent className="px-5 pb-2">{children}</CardContent>
      {footer ? (
        <div className="border-t px-2 py-1">{footer}</div>
      ) : null}
    </Card>
  )
}

// A full-width footer link row: an icon, a label, and a trailing chevron. Used
// for "Open Agents", "Open full run history", and "View security model".
export function PanelFooterLink({
  icon: Icon,
  label,
  onClick,
}: {
  icon: LucideIcon
  label: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
    >
      <Icon aria-hidden className="size-4 text-muted-foreground" />
      <span>{label}</span>
      <ChevronRight aria-hidden className="ml-auto size-4 text-muted-foreground" />
    </button>
  )
}
