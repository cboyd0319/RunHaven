import { ArrowRight } from "lucide-react"

import { Button } from "@/components/ui/button"

import { AgentMonogram } from "../AgentMonogram"
import { helpNavigation, HOME_NAV } from "../navigation"
import { agentLabel, PRODUCT_AGENTS } from "../sessionIdentity"
import type { MainViewProps } from "./props"

// Home has one job: help someone start. System setup, sign-in repair, run
// history, and failures appear only after the user chooses an agent or opens
// History/System details. Keeping those operational surfaces one level deeper
// makes the common path clear without hiding the less-common paths.
export function HomeView(props: MainViewProps) {
  function startWithAgent(agent: (typeof PRODUCT_AGENTS)[number]) {
    props.launchFlow.changeAgent(agent)
    props.launchFlow.setLaunchStep("project")
    props.onNavigate({ kind: "new-session" })
  }

  return (
    <div className="mx-auto flex w-full max-w-4xl flex-col gap-5 py-1 sm:py-3">
      <header className="flex flex-col gap-1.5">
        <h1 className="font-heading text-3xl leading-none font-semibold tracking-[-0.025em]">
          Start a session
        </h1>
        <div className="max-w-xl text-base leading-relaxed text-muted-foreground">
          <p>Choose the assistant account you already use.</p>
          <p className="flex flex-wrap items-center gap-x-1">
            You can go back without starting anything.
            <Button
              type="button"
              variant="link"
              size="sm"
              className="h-auto px-0"
              onClick={() => props.onNavigate(helpNavigation(HOME_NAV))}
            >
              Not sure?
            </Button>
          </p>
        </div>
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

  return (
    <button
      type="button"
      data-agent-launch={agent}
      className="flex min-h-[7.25rem] items-center gap-4 rounded-xl border bg-card p-5 text-left text-card-foreground transition-colors hover:bg-accent/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:bg-accent"
      onClick={onStart}
    >
      <AgentMonogram agent={agent} className="size-14 rounded-xl text-base" />
      <span className="font-heading text-lg font-semibold tracking-[-0.015em]">
        {label}
      </span>
      <ArrowRight
        aria-hidden
        className="ml-auto size-5 text-muted-foreground"
      />
    </button>
  )
}
