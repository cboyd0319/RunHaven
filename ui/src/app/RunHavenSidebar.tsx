import {
  AppWindow,
  Box,
  ShieldCheck,
  Terminal,
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

export function RunHavenSidebar({
  sessions,
}: {
  sessions: ManagedSessionSummary[]
}) {
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
              {[
                ["Start", AppWindow],
                ["Agents", Terminal],
                ["Security", ShieldCheck],
                ["History", Box],
              ].map(([label, Icon]) => (
                <SidebarMenuItem key={label as string}>
                  <SidebarMenuButton
                    isActive={label === "Start"}
                    tooltip={label as string}
                  >
                    <Icon />
                    <span>{label as string}</span>
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
                    <SidebarMenuButton tooltip={session.title}>
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
