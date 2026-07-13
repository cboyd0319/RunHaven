import type { AgentProfile } from "@/commands/runhaven"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
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

type SettingsDialogProps = {
  settings: SettingsState
  agents: AgentProfile[]
}

// Select items cannot carry an empty value, so "let RunHaven choose" rides a
// sentinel that maps back to null before anything is stored. Agent names come
// from the Rust dashboard and never collide with it.
const CHOOSE_FOR_ME = "runhaven-choose-for-me"

// Settings for this Mac (spec 3.9, shipped lean as a dialog in the main
// window instead of a separate settings window; the gap tracker's B3 section
// records why). Opens from the sidebar or RunHaven > Settings... (Cmd-,).
// Everything here is a convenience: no security default and no confirmation
// step can ever be changed from Settings.
export function SettingsDialog({ settings, agents }: SettingsDialogProps) {
  return (
    <Dialog open={settings.open} onOpenChange={settings.setOpen}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Small choices for how RunHaven behaves. Safety confirmations can
            never be turned off here.
          </DialogDescription>
        </DialogHeader>
        <SettingsForm settings={settings} agents={agents} />
      </DialogContent>
    </Dialog>
  )
}

// The dialog body, exported so tests can render it without the dialog portal
// (which does not exist under static markup).
export function SettingsForm({ settings, agents }: SettingsDialogProps) {
  return (
    <div className="flex flex-col gap-5">
      <label className="flex items-start gap-2 text-sm font-medium">
        <Checkbox
          checked={settings.dockBadge}
          onCheckedChange={(checked) => settings.setDockBadge(checked === true)}
        />
        <span className="flex flex-col gap-1">
          Show a number on the app icon
          <span className="font-normal text-muted-foreground">
            The number counts sessions that need you. Turning it off clears the
            number.
          </span>
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
        <span className="text-sm font-normal text-muted-foreground">
          The agent picked first when you start a new run. You can still change
          it for each run.
        </span>
      </label>
      <p className="text-sm text-muted-foreground">
        RunHaven follows your Mac&apos;s light or dark appearance.
      </p>
    </div>
  )
}
