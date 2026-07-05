import { cn } from "@/lib/utils"

// Shared status primitives for the Launchpad dashboard. One place decides the
// tone for a run state, so every panel (recent-runs table, run history, sidebar
// counts) reads the same and no panel invents a one-off status style.

export type StatusTone = "success" | "failed" | "stopped" | "active" | "neutral"

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

const DOT_TONE: Record<StatusTone, string> = {
  success: "bg-success",
  failed: "bg-destructive",
  stopped: "bg-muted-foreground",
  active: "bg-primary",
  neutral: "bg-muted-foreground",
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
