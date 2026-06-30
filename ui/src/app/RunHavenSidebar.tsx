import {
  AppWindow,
  Box,
  ShieldCheck,
  Terminal,
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
import type { ManagedSessionSummary } from "@/commands/runhaven"

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
            <SidebarGroupLabel>Active sessions</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {sessions.map((session) => (
                  <SidebarMenuItem key={session.id}>
                    <SidebarMenuButton
                      tooltip={session.title}
                      type="button"
                      isActive={selectedSessionId === session.id}
                      onClick={() => onSessionSelect(session.id)}
                    >
                      <Terminal />
                      <span>{session.title}</span>
                    </SidebarMenuButton>
                    <SidebarMenuBadge>{session.statusLabel}</SidebarMenuBadge>
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
