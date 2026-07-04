import {
  Activity,
  CheckCircle2,
  CircleAlert,
  CircleDashed,
  Square,
  type LucideIcon,
} from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

// Shared status primitives for the Launchpad dashboard. One place decides the
// tone, icon, and badge variant for a run or agent state, so every panel
// (sidebar rows, agents table, recent-runs table, needs-you cards) reads the
// same and no panel invents a one-off badge style (design doc "Status Badge
// System" + "shared status primitives").

export type StatusTone = "success" | "failed" | "stopped" | "active" | "neutral"

const TONE_ICON: Record<StatusTone, LucideIcon> = {
  success: CheckCircle2,
  failed: CircleAlert,
  stopped: Square,
  active: Activity,
  neutral: CircleDashed,
}

// Badge variant per tone, mapped onto the existing shadcn Badge variants (one
// canonical color source; no parallel token set). "stopped"/"neutral" use the
// muted secondary so gray never competes with real success/failure color.
const TONE_BADGE: Record<StatusTone, "success" | "destructive" | "secondary" | "outline"> = {
  success: "success",
  failed: "destructive",
  stopped: "secondary",
  active: "outline",
  neutral: "secondary",
}

// Icon color per tone. Kept token-based so light/dark both pass contrast.
const TONE_ICON_CLASS: Record<StatusTone, string> = {
  success: "text-success",
  failed: "text-destructive",
  stopped: "text-muted-foreground",
  active: "text-primary",
  neutral: "text-muted-foreground",
}

// Map a backend lifecycle/status word to a tone. Lifecycle is the primary
// signal (the backend's own grouping); the human statusLabel is only a
// fallback. Unknown states are neutral, never guessed into success.
export function runStatusTone(lifecycle: string, statusLabel: string): StatusTone {
  const life = lifecycle.trim().toLowerCase()
  if (life === "failed") return "failed"
  if (life === "ended" || life === "finished" || life === "cleaned-up") return "success"
  if (life === "removed") return "stopped"
  if (life === "starting" || life === "running") return "active"
  if (life === "stop-requested" || life === "kill-requested") return "active"
  // Fall back to the label for records with a thin lifecycle.
  const label = statusLabel.trim().toLowerCase()
  if (label.includes("fail")) return "failed"
  if (label.includes("stop")) return "stopped"
  if (label.includes("end") || label.includes("finish") || label.includes("succeed")) {
    return "success"
  }
  if (label.includes("run") || label.includes("start")) return "active"
  return "neutral"
}

// A small colored status dot with an accessible text label beside it (never
// color alone, per the doc's accessibility rules). The label is a real span so
// screen readers and sighted users read the same word.
export function StatusDot({
  tone,
  label,
  className,
}: {
  tone: StatusTone
  label: string
  className?: string
}) {
  return (
    <span className={cn("inline-flex items-center gap-1.5", className)}>
      <span
        aria-hidden
        className={cn("size-2 shrink-0 rounded-full", DOT_TONE[tone])}
      />
      <span>{label}</span>
    </span>
  )
}

const DOT_TONE: Record<StatusTone, string> = {
  success: "bg-success",
  failed: "bg-destructive",
  stopped: "bg-muted-foreground",
  active: "bg-primary",
  neutral: "bg-muted-foreground",
}

// The leading run icon for a sidebar or table row: a tone-colored glyph that
// pairs with the always-present text label elsewhere in the row.
export function RunStatusIcon({
  tone,
  className,
}: {
  tone: StatusTone
  className?: string
}) {
  const Icon = TONE_ICON[tone]
  return <Icon aria-hidden className={cn("size-4", TONE_ICON_CLASS[tone], className)} />
}

// A fixed-height status chip for tables. Text names the state; color reinforces.
export function StatusBadge({ tone, label }: { tone: StatusTone; label: string }) {
  return (
    <Badge variant={TONE_BADGE[tone]} className="min-w-16">
      {label}
    </Badge>
  )
}
