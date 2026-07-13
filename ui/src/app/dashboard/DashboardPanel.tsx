import type { ReactNode } from "react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"

// Shared chrome for History sections. One panel shell keeps borders, headers,
// and count badges consistent.
export function DashboardPanel({
  title,
  count,
  description,
  children,
  className,
}: {
  title: string
  // A count badge next to the title. Omit to hide it.
  count?: number
  description?: string
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
    </Card>
  )
}
