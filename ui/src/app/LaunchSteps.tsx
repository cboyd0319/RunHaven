import { cn } from "@/lib/utils"

import type { LaunchStep } from "./types"

const steps: Array<{ id: LaunchStep; title: string; description: string }> = [
  { id: "agent", title: "1. Agent", description: "Choose who runs" },
  { id: "project", title: "2. Project", description: "Pick the folder" },
  { id: "review", title: "3. Review", description: "Check access" },
]

export function LaunchSteps({ step }: { step: LaunchStep }) {
  return (
    <ol className="grid list-none gap-2 rounded-lg border p-3 text-sm sm:grid-cols-3">
      {steps.map((item) => {
        const active = item.id === step
        return (
          <li
            key={item.id}
            aria-current={active ? "step" : undefined}
            className={cn(
              "rounded-md p-2",
              active ? "bg-accent text-accent-foreground" : "text-muted-foreground",
            )}
          >
            <div className="font-medium">{item.title}</div>
            <div className={cn("text-muted-foreground", active && "text-accent-foreground")}>
              {item.description}
            </div>
          </li>
        )
      })}
    </ol>
  )
}
