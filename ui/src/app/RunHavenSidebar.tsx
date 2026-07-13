import {
  CircleGauge,
  History,
  Pin,
  SquarePlus,
  Terminal,
  type LucideIcon,
} from "lucide-react"

import appIcon from "@/assets/app-icon.png"
import { Button } from "@/components/ui/button"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { type NavDestination } from "./navigation"
import { agentLabel } from "./sessionIdentity"

type RunningTerminalRow = {
  sessionId: string
  runId?: string
  label?: string
  request?: { plan: { agent: string; workspacePath: string } }
  startedAt?: string
}

type SavedSetupRow = { id: string; name: string; available?: boolean }

// Individual runs stay out of the sidebar so navigation remains stable.
export function RunHavenSidebar({
  navigation,
  onNavigate,
  runTerminals = [],
  savedSetups = [],
  savedSetupsError = "",
  onSelectSavedSetup,
  onUnpinSavedSetup,
  appVersion,
}: {
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
  // True while an interactive run terminal is live. It is not a managed session,
  // so it gets its own fixed row: without it, navigating away from a live run
  // would leave no way back (audit RT-1/RT-2).
  runTerminalActive?: boolean
  runTerminals?: RunningTerminalRow[]
  savedSetups?: SavedSetupRow[]
  savedSetupsError?: string
  onSelectSavedSetup?: (id: string) => void
  onUnpinSavedSetup?: (id: string) => void
  appVersion: string
}) {
  function navigate(destination: NavDestination) {
    onNavigate(destination)
  }

  return (
    <Sidebar collapsible="offcanvas">
      <SidebarHeader>
        {/* Overlay title bar (NAT-1): the macOS traffic lights float over this
            top-left strip, so it insets the logo below them and doubles as a
            draggable region (data-tauri-drag-region -> the window-chrome
            capability's start-dragging permission). */}
        <div data-tauri-drag-region className="h-7 shrink-0" />
        <div className="flex items-center gap-3 px-2 py-2">
          <img
            src={appIcon}
            alt=""
            className="size-[32px] rounded-[7px] object-cover"
          />
          <div className="min-w-0 group-data-[collapsible=icon]:hidden">
            <div className="font-medium">RunHaven</div>
          </div>
        </div>
      </SidebarHeader>
      <nav
        aria-label="RunHaven navigation"
        className="flex min-h-0 flex-1 flex-col"
      >
        <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <DestinationRow
                label="New session"
                Icon={SquarePlus}
                active={navigation.kind === "new-session"}
                onSelect={() => navigate({ kind: "new-session" })}
              />
              <DestinationRow
                label="History"
                Icon={History}
                active={navigation.kind === "runs"}
                onSelect={() => navigate({ kind: "runs" })}
              />
              <DestinationRow
                label="System"
                Icon={CircleGauge}
                active={navigation.kind === "diagnostics"}
                onSelect={() => navigate({ kind: "diagnostics" })}
              />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        {runTerminals.length > 0 ? (
          <SidebarGroup>
            <SidebarGroupLabel>Running</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {runTerminals.map((terminal, index) => {
                  const id = terminal.runId ?? terminal.sessionId
                  const label = terminalLabel(terminal, index, runTerminals)
                  return (
                    <DestinationRow
                      key={terminal.sessionId}
                      label={label}
                      Icon={Terminal}
                      active={
                        navigation.kind === "run-terminal" &&
                        navigation.id === id
                      }
                      onSelect={() => navigate({ kind: "run-terminal", id })}
                    />
                  )
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ) : null}
        {savedSetups.length > 0 ? (
          <SidebarGroup>
            <SidebarGroupLabel>Saved setups</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {savedSetups.map((setup) => (
                  <SidebarMenuItem key={setup.id} className="flex flex-col gap-1">
                    <SidebarMenuButton
                      type="button"
                      tooltip={setup.name}
                      aria-describedby={
                        setup.available === false
                          ? `saved-setup-${setup.id}-status`
                          : undefined
                      }
                      onClick={() =>
                        onSelectSavedSetup
                          ? onSelectSavedSetup(setup.id)
                          : navigate({ kind: "new-session" })
                      }
                    >
                      <Pin />
                      <span>{setup.name}</span>
                    </SidebarMenuButton>
                    {setup.available === false ? (
                      <span
                        id={`saved-setup-${setup.id}-status`}
                        className="px-2 text-xs text-warning-text group-data-[collapsible=icon]:hidden"
                      >
                        Project folder can’t be found
                      </span>
                    ) : null}
                    {onUnpinSavedSetup ? (
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        aria-label={`Unpin ${setup.name}`}
                        className="min-h-8 self-start px-2 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden"
                        onClick={() => onUnpinSavedSetup(setup.id)}
                      >
                        Unpin setup
                      </Button>
                    ) : null}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
              {savedSetupsError ? (
                <p role="alert" className="px-2 py-1 text-xs text-destructive">
                  {savedSetupsError}
                </p>
              ) : null}
            </SidebarGroupContent>
          </SidebarGroup>
        ) : null}
        </SidebarContent>
      </nav>
      <SidebarFooter>
        <div className="px-3 py-1 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
          {`RunHaven ${appVersion}`.trim()}
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}

function terminalLabel(
  terminal: RunningTerminalRow,
  index: number,
  terminals: RunningTerminalRow[],
): string {
  const label = terminalLabelCandidate(terminal)
  const occurrence =
    terminals
      .slice(0, index)
      .filter((candidate) => terminalLabelCandidate(candidate) === label).length + 1
  return occurrence > 1 ? `${label} (${occurrence})` : label
}

function terminalLabelCandidate(terminal: RunningTerminalRow): string {
  const plan = terminal.request?.plan
  const base =
    terminal.label ?? (plan ? `${agentLabel(plan.agent)} · Starting` : "Session starting")
  const started = terminal.startedAt
    ? ` · ${new Intl.DateTimeFormat(undefined, {
        hour: "numeric",
        minute: "2-digit",
        second: "2-digit",
      }).format(new Date(terminal.startedAt))}`
    : ""
  return `${base}${started}`
}

function DestinationRow({
  label,
  Icon,
  active,
  onSelect,
}: {
  label: string
  Icon: LucideIcon
  active: boolean
  onSelect: () => void
}) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        aria-current={active ? "page" : undefined}
        isActive={active}
        tooltip={label}
        type="button"
        onClick={onSelect}
      >
        <Icon />
        <span>{label}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}
