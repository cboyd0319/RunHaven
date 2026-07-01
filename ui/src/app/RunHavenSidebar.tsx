import { AppWindow, Terminal, type LucideIcon } from "lucide-react"

import logo from "@/assets/logo.png"
import { Badge } from "@/components/ui/badge"
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
import type { ManagedSessionSummary } from "@/commands/runhaven"

import {
  authScopeLabel,
  authStatusLabel,
  networkLabel,
  safeBackendLabel,
} from "./labels"
import {
  agentLabel,
  sessionIdentityLabel,
  sessionInputLabel,
} from "./sessionIdentity"

type SidebarNavItem = {
  label: string
  Icon: LucideIcon
  onClick?: () => void
}

export function RunHavenSidebar({
  sessions,
  selectedSessionId,
  onStartSelect,
  onSessionSelect,
}: {
  sessions: ManagedSessionSummary[]
  selectedSessionId: string | null
  onStartSelect: () => void
  onSessionSelect: (sessionId: string) => void
}) {
  const navItems: SidebarNavItem[] = [
    { label: "Start", Icon: AppWindow, onClick: onStartSelect },
  ]

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
          <SidebarGroupLabel>Workspace</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map(({ label, Icon, onClick }) => (
                <SidebarMenuItem key={label}>
                  <SidebarMenuButton
                    isActive={label === "Start" && selectedSessionId === null}
                    tooltip={label}
                    type="button"
                    onClick={onClick}
                  >
                    <Icon />
                    <span>{label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        {sessions.length > 0 ? (
          <SidebarGroup>
            <SidebarGroupLabel>Sessions ({sessions.length})</SidebarGroupLabel>
            <SidebarGroupContent>
              <div className="flex flex-col gap-4">
                {groupSessionsByAgent(sessions).map(([agent, agentSessions]) => (
                  <section key={agent} aria-label={`${agentLabel(agent)} sessions`}>
                    <div className="mb-2 flex items-center justify-between px-2 group-data-[collapsible=icon]:hidden">
                      <div className="text-[11px] font-medium tracking-[0.2em] text-muted-foreground uppercase">
                        {agentLabel(agent)}
                      </div>
                      <SidebarSessionBadge>{String(agentSessions.length)}</SidebarSessionBadge>
                    </div>
                    <SidebarMenu>
                      {agentSessions.map((session) => {
                        const selected = selectedSessionId === session.id

                        return (
                          <SidebarMenuItem key={session.id}>
                            <SidebarMenuButton
                              aria-current={selected ? "page" : undefined}
                              className="h-auto items-start gap-3 rounded-xl border border-transparent py-3 pr-24 data-[active=true]:border-sidebar-primary/25 data-[active=true]:bg-sidebar-accent/70"
                              tooltip={session.title}
                              type="button"
                              isActive={selected}
                              onClick={() => onSessionSelect(session.id)}
                            >
                              <Terminal />
                              <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                                <div className="truncate text-sm font-medium">
                                  {session.title}
                                </div>
                                <div className="truncate text-[11px] text-muted-foreground">
                                  {sessionIdentityLabel(session)}
                                </div>
                                <div className="mt-1 text-[11px] leading-4 text-muted-foreground">
                                  {authStatusLabel(session.authStatus)}
                                  {" • "}
                                  {authScopeLabel(session.authScope)}
                                </div>
                                <div className="text-[11px] leading-4 text-muted-foreground">
                                  {networkLabel(session.network)}
                                  {" • "}
                                  {sessionInputLabel(session)}
                                </div>
                              </div>
                            </SidebarMenuButton>
                            <SidebarMenuBadge className="max-w-20 truncate">
                              {safeBackendLabel(session.statusLabel, "Unknown")}
                            </SidebarMenuBadge>
                          </SidebarMenuItem>
                        )
                      })}
                    </SidebarMenu>
                  </section>
                ))}
              </div>
            </SidebarGroupContent>
          </SidebarGroup>
        ) : null}
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}

function SidebarSessionBadge({ children }: { children: string }) {
  return (
    <Badge
      variant="outline"
      className="h-auto rounded-md px-1.5 py-0 text-[10px] leading-4 whitespace-normal"
    >
      {children}
    </Badge>
  )
}

function groupSessionsByAgent(
  sessions: ManagedSessionSummary[],
): Array<[agent: string, sessions: ManagedSessionSummary[]]> {
  const groups = new Map<string, ManagedSessionSummary[]>()
  for (const session of sessions) {
    const agentSessions = groups.get(session.agent) ?? []
    agentSessions.push(session)
    groups.set(session.agent, agentSessions)
  }
  return [...groups.entries()].sort(([left], [right]) => {
    const labelOrder = agentLabel(left).localeCompare(agentLabel(right), undefined, {
      sensitivity: "base",
    })
    return labelOrder || left.localeCompare(right)
  })
}
