import {
  AppWindow,
  Box,
  KeyRound,
  LockKeyhole,
  Network,
  ShieldCheck,
  Terminal,
} from "lucide-react"

import logo from "@/assets/logo.png"
import { Badge } from "@/components/ui/badge"
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
  SidebarRail,
  SidebarSeparator,
} from "@/components/ui/sidebar"

export function RunHavenSidebar() {
  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center gap-3 px-2 py-2">
          <img
            src={logo}
            alt=""
            className="size-10 rounded-xl border bg-background object-cover"
          />
          <div className="min-w-0 group-data-[collapsible=icon]:hidden">
            <div className="font-medium">RunHaven</div>
            <div className="text-xs text-muted-foreground">
              Safe agent runs
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
        <SidebarSeparator />
        <SidebarGroup>
          <SidebarGroupLabel>Boundary</SidebarGroupLabel>
          <SidebarGroupContent>
            <div className="flex flex-col gap-2 px-2 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
              <div className="flex items-center gap-2">
                <LockKeyhole className="size-3.5" />
                /workspace only
              </div>
              <div className="flex items-center gap-2">
                <Network className="size-3.5" />
                Provider allowlist
              </div>
              <div className="flex items-center gap-2">
                <KeyRound className="size-3.5" />
                Credentials blocked
              </div>
            </div>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <Badge variant="outline" className="mx-2 mb-2">
          Local preview
        </Badge>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
