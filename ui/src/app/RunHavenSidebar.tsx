import {
  CircleGauge,
  History,
  House,
  Terminal,
  type LucideIcon,
} from "lucide-react"

import logo from "@/assets/logo.png"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { RUN_TERMINAL_NAV, type NavDestination } from "./navigation"

// Individual runs stay out of the sidebar so navigation remains stable.
export function RunHavenSidebar({
  navigation,
  onNavigate,
  runTerminalActive,
  appVersion,
}: {
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
  // True while an interactive run terminal is live. It is not a managed session,
  // so it gets its own fixed row: without it, navigating away from a live run
  // would leave no way back (audit RT-1/RT-2).
  runTerminalActive: boolean
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
            src={logo}
            alt=""
            className="size-12 rounded-xl border bg-background object-cover"
          />
          <div className="min-w-0 group-data-[collapsible=icon]:hidden">
            <div className="font-medium">RunHaven</div>
            <div className="text-xs text-muted-foreground">Safe coding sessions</div>
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
                onSelect={() => navigate({ kind: "home" })}
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
              {runTerminalActive ? (
                <DestinationRow
                  label="Session terminal"
                  Icon={Terminal}
                  active={navigation.kind === "run-terminal"}
                  onSelect={() => navigate(RUN_TERMINAL_NAV)}
                />
              ) : null}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <div className="px-3 py-1 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
          {`RunHaven ${appVersion}`.trim()}
        </div>
      </SidebarFooter>
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
