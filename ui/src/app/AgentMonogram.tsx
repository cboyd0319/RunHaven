import { cn } from "@/lib/utils"

const MONOGRAMS: Record<string, string> = {
  antigravity: "AG",
  claude: "CL",
  codex: "CO",
  copilot: "CP",
  shell: "SH",
}

// One neutral identity system across launch and history surfaces. The adjacent
// product name supplies the accessible name; this decorative tile stays hidden
// from assistive technology.
export function AgentMonogram({
  agent,
  className,
}: {
  agent: string
  className?: string
}) {
  const normalized = agent.trim().toLowerCase()
  const fallback =
    normalized.match(/[a-z0-9]/g)?.slice(0, 2).join("").toUpperCase() || "AI"
  return (
    <span
      aria-hidden
      className={cn(
        "flex shrink-0 items-center justify-center border bg-background font-heading font-semibold tracking-wide text-muted-foreground",
        className,
      )}
    >
      {MONOGRAMS[normalized] ?? fallback}
    </span>
  )
}
