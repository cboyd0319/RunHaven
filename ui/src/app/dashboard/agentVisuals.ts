import {
  Bot,
  Hexagon,
  Sparkles,
  Terminal,
  Triangle,
  type LucideIcon,
} from "lucide-react"

// Product-safe provider glyphs. The design doc allows brand-like symbols but
// warns to confirm licensing; we use neutral lucide glyphs tinted per provider
// instead of vendor logos, so nothing ships an unlicensed mark. The tint is
// icon-sized only (the doc forbids brand-colored card backgrounds).
type AgentVisual = {
  Icon: LucideIcon
  // A token-independent accent for the icon only. Kept subtle; the card surface
  // stays neutral.
  tint: string
}

const AGENT_VISUALS: Record<string, AgentVisual> = {
  antigravity: { Icon: Triangle, tint: "text-violet-400" },
  claude: { Icon: Sparkles, tint: "text-orange-400" },
  codex: { Icon: Hexagon, tint: "text-sky-300" },
  copilot: { Icon: Bot, tint: "text-blue-400" },
  shell: { Icon: Terminal, tint: "text-emerald-400" },
}

const FALLBACK: AgentVisual = { Icon: Bot, tint: "text-muted-foreground" }

export function agentVisual(agentName: string): AgentVisual {
  return AGENT_VISUALS[agentName.trim().toLowerCase()] ?? FALLBACK
}
