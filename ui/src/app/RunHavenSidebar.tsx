import {
  AppWindow,
  Box,
  ShieldCheck,
  Terminal,
  type LucideIcon,
} from "lucide-react"

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
  authLabel,
  authScopeLabel,
  networkLabel,
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
    { label: "Agents", Icon: Terminal },
    { label: "Security", Icon: ShieldCheck },
    { label: "History", Icon: Box },
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
            <SidebarGroupLabel>Sessions</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {sessions.map((session) => (
                  <SidebarMenuItem key={session.id}>
                    <SidebarMenuButton
                      className="h-auto items-start gap-3 pr-20 py-3"
                      tooltip={session.title}
                      type="button"
                      isActive={selectedSessionId === session.id}
                      onClick={() => onSessionSelect(session.id)}
                    >
                      <Terminal />
                      <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                        <div className="truncate text-sm font-medium">
                          {session.title}
                        </div>
                        <div className="mt-1 flex flex-wrap gap-1.5 text-[11px]">
                          <SidebarSessionBadge>{session.agent}</SidebarSessionBadge>
                          <SidebarSessionBadge>
                            {authLabel(session.authStatus)}
                          </SidebarSessionBadge>
                          <SidebarSessionBadge>
                            {authScopeLabel(session.authScope)}
                          </SidebarSessionBadge>
                          <SidebarSessionBadge>
                            {networkLabel(session.network)}
                          </SidebarSessionBadge>
                          <SidebarSessionBadge>
                            {sessionInputLabel(session)}
                          </SidebarSessionBadge>
                        </div>
                      </div>
                    </SidebarMenuButton>
                    <SidebarMenuBadge className="max-w-16 truncate">
                      {session.statusLabel}
                    </SidebarMenuBadge>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
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
