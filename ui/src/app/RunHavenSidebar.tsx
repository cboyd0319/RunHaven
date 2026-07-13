import {
  Activity,
  Bot,
  CheckCircle2,
  CircleAlert,
  CirclePlay,
  House,
  Settings,
  Terminal,
  Wrench,
  type LucideIcon,
} from "lucide-react"

import logo from "@/assets/logo.png"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar"
import type { ManagedSessionSummary } from "@/commands/runhaven"

import { groupSessions, RUN_TERMINAL_NAV, type NavDestination } from "./navigation"
import { sessionAttention } from "./sessionAttention"

// Fixed destinations first, then three filtered Run history shortcuts and the
// app version. Individual runs stay out of the sidebar so it remains stable.
export function RunHavenSidebar({
  sessions,
  navigation,
  onNavigate,
  onOpenSettings,
  runTerminalActive,
  appVersion,
}: {
  sessions: ManagedSessionSummary[]
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
  onOpenSettings: () => void
  // True while an interactive run terminal is live. It is not a managed session,
  // so it gets its own fixed row: without it, navigating away from a live run
  // would leave no way back (audit RT-1/RT-2).
  runTerminalActive: boolean
  appVersion: string
}) {
  const groups = groupSessions(sessions)
  const needsReview = sessions.filter(
    (session) => sessionAttention(session) !== null,
  ).length

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        {/* Overlay title bar (NAT-1): the macOS traffic lights float over this
            top-left strip, so it insets the logo below them and doubles as a
            draggable region (data-tauri-drag-region -> the window-chrome
            capability's start-dragging permission). */}
        <div data-tauri-drag-region className="h-7 shrink-0" />
        <div className="flex items-center gap-3 px-2 py-2">
          <img
            src={logo}
            alt=""
            className="size-12 rounded-xl border bg-background object-cover"
          />
          <div className="min-w-0 group-data-[collapsible=icon]:hidden">
            <div className="font-medium">RunHaven</div>
            <div className="text-xs text-muted-foreground">Agent launcher</div>
          </div>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <DestinationRow
                label="Home"
                Icon={House}
                active={navigation.kind === "home"}
                onSelect={() => onNavigate({ kind: "home" })}
              />
              <DestinationRow
                label="Agents"
                Icon={Bot}
                active={navigation.kind === "agents"}
                onSelect={() => onNavigate({ kind: "agents" })}
              />
              <DestinationRow
                label="System Check"
                Icon={Activity}
                active={navigation.kind === "diagnostics"}
                onSelect={() => onNavigate({ kind: "diagnostics" })}
              />
              <DestinationRow
                label="Cleanup"
                Icon={Wrench}
                active={navigation.kind === "maintenance"}
                onSelect={() => onNavigate({ kind: "maintenance" })}
              />
              {runTerminalActive ? (
                <DestinationRow
                  label="Run terminal"
                  Icon={Terminal}
                  active={navigation.kind === "run-terminal"}
                  onSelect={() => onNavigate(RUN_TERMINAL_NAV)}
                />
              ) : null}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        {/* The three counts are independent status flags, not a partition: a
            finished run that is ready to review is intentionally in both "Needs
            you" (attention) and "Past" (lifecycle), matching the Home
            panels, so they can sum to more than the session total. */}
        <SidebarGroup>
          <SidebarGroupLabel className="tracking-[0.2em] uppercase">
            Runs
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <CountRow
                label="Needs you"
                Icon={CircleAlert}
                iconClassName="text-destructive"
                count={needsReview}
                onSelect={() => onNavigate({ kind: "runs", filter: "needs-you" })}
              />
              <CountRow
                label="Running"
                Icon={CirclePlay}
                iconClassName="text-primary"
                count={groups.active.length}
                onSelect={() => onNavigate({ kind: "runs", filter: "running" })}
              />
              <CountRow
                label="Past"
                Icon={CheckCircle2}
                iconClassName="text-success"
                count={groups.finished.length}
                onSelect={() => onNavigate({ kind: "runs", filter: "past" })}
              />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              tooltip="Settings"
              type="button"
              onClick={onOpenSettings}
            >
              <Settings />
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <div className="px-3 py-1 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
          {`RunHaven ${appVersion}`.trim()}
        </div>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
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

// A run-pipeline summary row: a tone-colored icon, a label, and a fixed count
// badge. The count is not the only cue; the label names the group in text.
function CountRow({
  label,
  Icon,
  iconClassName,
  count,
  onSelect,
}: {
  label: string
  Icon: LucideIcon
  iconClassName: string
  count: number
  onSelect: () => void
}) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton tooltip={`${label}: ${count}`} type="button" onClick={onSelect}>
        <Icon className={iconClassName} />
        <span>{label}</span>
      </SidebarMenuButton>
      <SidebarMenuBadge className="tabular-nums">{count}</SidebarMenuBadge>
    </SidebarMenuItem>
  )
}
