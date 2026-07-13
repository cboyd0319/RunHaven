import type { AgentProfile } from "@/commands/runhaven"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

import { agentLabel } from "./sessionIdentity"
import type { SettingsState } from "./useSettings"

type SettingsFormProps = {
  settings: SettingsState
  agents: AgentProfile[]
  compact?: boolean
}

const CHOOSE_FOR_ME = "runhaven-choose-for-me"

// Two convenience choices shown inline in the sidebar. Safety defaults and
// confirmation rules are intentionally not configurable here.
export function SettingsForm({
  settings,
  agents,
  compact = false,
}: SettingsFormProps) {
  return (
    <div className={compact ? "flex flex-col gap-4" : "flex flex-col gap-5"}>
      <label className="flex items-start gap-2 text-sm font-medium">
        <Checkbox
          checked={settings.dockBadge}
          onCheckedChange={(checked) => settings.setDockBadge(checked === true)}
        />
        <span className="flex flex-col gap-1">
          App icon count
          {!compact ? (
            <span className="font-normal text-muted-foreground">
              The number counts sessions that need you. Turning it off clears
              the number.
            </span>
          ) : null}
        </span>
      </label>
      <label className="flex flex-col gap-1.5 text-sm font-medium">
        Start new runs with
        <Select
          value={settings.defaultAgent ?? CHOOSE_FOR_ME}
          onValueChange={(value) =>
            settings.setDefaultAgent(value === CHOOSE_FOR_ME ? null : value)
          }
        >
          <SelectTrigger>
            <SelectValue placeholder="Choose for me" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value={CHOOSE_FOR_ME}>Choose for me</SelectItem>
              {agents.map((agent) => (
                <SelectItem key={agent.name} value={agent.name}>
                  {agentLabel(agent.name)}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        {!compact ? (
          <span className="text-sm font-normal text-muted-foreground">
            The agent picked first when you start a new run. You can still
            change it for each run.
          </span>
        ) : null}
      </label>
      {!compact ? (
        <p className="text-sm text-muted-foreground">
          RunHaven follows your Mac&apos;s light or dark appearance.
        </p>
      ) : null}
    </div>
  )
}
