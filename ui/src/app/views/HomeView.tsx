import { ArrowRight } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { agentVisual } from "../dashboard/agentVisuals"
import { agentLabel } from "../sessionIdentity"
import type { MainViewProps } from "./props"

const PRODUCT_AGENTS = ["claude", "codex", "antigravity", "copilot"] as const

const AGENT_DESCRIPTIONS: Record<(typeof PRODUCT_AGENTS)[number], string> = {
  claude: "Claude Code",
  codex: "OpenAI Codex",
  antigravity: "Google Antigravity",
  copilot: "GitHub Copilot",
}

// Home has one job: help someone start. System setup, sign-in repair, run
// history, and failures appear only after the user chooses an agent or opens
// History/System details. Keeping those operational surfaces one level deeper
// makes the common path clear without hiding the less-common paths.
export function HomeView(props: MainViewProps) {
  function startWithAgent(agent: (typeof PRODUCT_AGENTS)[number]) {
    props.launchFlow.changeAgent(agent)
    props.launchFlow.setLaunchStep("agent")
    props.onNavigate({ kind: "new-session" })
  }

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-5">
      <header className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl leading-tight font-semibold tracking-tight">
          Choose an agent
        </h2>
        <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground">
          Pick one to start a new session. You can review the project and access
          before anything runs.
        </p>
      </header>

      <div className="grid gap-4 md:grid-cols-2">
        {PRODUCT_AGENTS.map((agent) => (
          <AgentLaunchCard
            key={agent}
            agent={agent}
            onStart={() => startWithAgent(agent)}
          />
        ))}
      </div>
    </div>
  )
}

function AgentLaunchCard({
  agent,
  onStart,
}: {
  agent: (typeof PRODUCT_AGENTS)[number]
  onStart: () => void
}) {
  const label = agentLabel(agent)
  const { Icon, tint } = agentVisual(agent)

  return (
    <Card className="min-h-48 justify-between">
      <CardHeader className="gap-3">
        <span className="flex size-11 items-center justify-center rounded-xl bg-muted">
          <Icon aria-hidden className={`size-6 ${tint}`} />
        </span>
        <div className="flex flex-col gap-1">
          <CardTitle className="text-lg tracking-tight">{label}</CardTitle>
          <CardDescription>{AGENT_DESCRIPTIONS[agent]}</CardDescription>
        </div>
      </CardHeader>
      <CardContent>
        <Button
          type="button"
          variant="outline"
          className="w-full justify-between active:scale-[0.99]"
          onClick={onStart}
        >
          Start with {label}
          <ArrowRight aria-hidden />
        </Button>
      </CardContent>
    </Card>
  )
}
