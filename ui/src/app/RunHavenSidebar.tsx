import {
  Activity,
  Bot,
  CircleX,
  GitBranch,
  House,
  KeyRound,
  Terminal,
  TriangleAlert,
  Wrench,
  type LucideIcon,
} from "lucide-react"

import logo from "@/assets/logo.png"
import {
  Sidebar,
  SidebarContent,
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
import { cn } from "@/lib/utils"
import type { ManagedSessionSummary } from "@/commands/runhaven"

import { safeBackendLabel } from "./labels"
import { groupSessions, type NavDestination } from "./navigation"
import { useSessionLabels } from "./sessionLabels"
import { sessionDisplayTitle } from "./sessionIdentity"
import {
  sessionAttention,
  type SessionAttention,
  type SessionAttentionKind,
} from "./sessionAttention"

// Sidebar taxonomy per docs/design/native-gui-navigation-spec.md section 2:
// a fixed Home row first, dynamic Active and Finished session groups (hidden
// when empty), and a fixed System group last. The sidebar is navigation only.
export function RunHavenSidebar({
  sessions,
  navigation,
  onNavigate,
}: {
  sessions: ManagedSessionSummary[]
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
}) {
  const groups = groupSessions(sessions)

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center gap-3 px-2 py-2">
          <img
            src={logo}
            alt=""
            className="size-12 rounded-xl border bg-background object-cover"
          />
          <div className="min-w-0 group-data-[collapsible=icon]:hidden">
            <div className="font-medium">RunHaven</div>
            <div className="text-xs text-muted-foreground">
              Agent launcher
            </div>
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
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SessionGroup
          label="Active"
          sessions={groups.active}
          navigation={navigation}
          onNavigate={onNavigate}
        />
        <SessionGroup
          label="Finished"
          sessions={groups.finished}
          navigation={navigation}
          onNavigate={onNavigate}
        />
        <SidebarGroup>
          <SidebarGroupLabel className="tracking-[0.2em] uppercase">
            System
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <DestinationRow
                label="Agents"
                Icon={Bot}
                active={navigation.kind === "agents"}
                onSelect={() => onNavigate({ kind: "agents" })}
              />
              <DestinationRow
                label="Diagnostics"
                Icon={Activity}
                active={navigation.kind === "diagnostics"}
                onSelect={() => onNavigate({ kind: "diagnostics" })}
              />
              <DestinationRow
                label="Maintenance"
                Icon={Wrench}
                active={navigation.kind === "maintenance"}
                onSelect={() => onNavigate({ kind: "maintenance" })}
              />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
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

function SessionGroup({
  label,
  sessions,
  navigation,
  onNavigate,
}: {
  label: string
  sessions: ManagedSessionSummary[]
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
}) {
  if (sessions.length === 0) {
    return null
  }
  return (
    <SidebarGroup aria-label={`${label} sessions`}>
      <SidebarGroupLabel className="tracking-[0.2em] uppercase">
        {label}
      </SidebarGroupLabel>
      <SidebarGroupContent>
        <SidebarMenu>
          {sessions.map((session) => (
            <SessionRow
              key={session.id}
              session={session}
              selected={
                navigation.kind === "session" && navigation.id === session.id
              }
              onSelect={() => onNavigate({ kind: "session", id: session.id })}
            />
          ))}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}

function SessionRow({
  session,
  selected,
  onSelect,
}: {
  session: ManagedSessionSummary
  selected: boolean
  onSelect: () => void
}) {
  const attention = sessionAttention(session)
  const { labels } = useSessionLabels()
  const title = sessionDisplayTitle(session, labels)

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        aria-current={selected ? "page" : undefined}
        className="h-auto items-start gap-3 rounded-xl border border-transparent py-3 pr-10 data-[active=true]:border-sidebar-primary/25 data-[active=true]:bg-sidebar-accent/70"
        tooltip={title}
        type="button"
        isActive={selected}
        onClick={onSelect}
      >
        <Terminal />
        {/* Spec section 2 row anatomy: primary label, one status line, and the
            trailing attention badge. Auth, scope, network, and input detail
            live in the session view and its Details disclosure, not the row. */}
        <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
          <div className="truncate text-sm font-medium">{title}</div>
          <div className="truncate text-[11px] leading-4 text-muted-foreground">
            {attention
              ? attention.label
              : safeBackendLabel(session.statusLabel, "Unknown")}
          </div>
        </div>
      </SidebarMenuButton>
      {attention ? (
        // The visible status line above names the state, so the dot itself is
        // decoration for assistive tech.
        <SidebarMenuBadge aria-hidden>
          <AttentionDot attention={attention} />
        </SidebarMenuBadge>
      ) : null}
    </SidebarMenuItem>
  )
}

// One symbol-in-dot badge per row: a distinct symbol per state plus the status
// text in the row, never color alone and never a count.
const ATTENTION_ICONS: Record<SessionAttentionKind, LucideIcon> = {
  failed: CircleX,
  "needs-sign-in": KeyRound,
  "not-responding": TriangleAlert,
  "ready-to-review": GitBranch,
}

function AttentionDot({ attention }: { attention: SessionAttention }) {
  const Icon = ATTENTION_ICONS[attention.kind]
  return (
    <span
      data-attention={attention.kind}
      className={cn(
        "flex size-4 items-center justify-center rounded-full",
        attention.tone === "failed"
          ? "bg-destructive text-white"
          : "bg-amber-500 text-amber-950",
      )}
    >
      <Icon className="size-3" />
    </span>
  )
}
