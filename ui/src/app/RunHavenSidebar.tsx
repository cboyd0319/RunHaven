import { useEffect } from "react"
import {
  History,
  House,
  Settings,
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
  SidebarRail,
  useSidebar,
} from "@/components/ui/sidebar"
import type { AgentProfile } from "@/commands/runhaven"

import { RUN_TERMINAL_NAV, type NavDestination } from "./navigation"
import { SettingsForm } from "./SettingsForm"
import type { SettingsState } from "./useSettings"

// Individual runs stay out of the sidebar so navigation remains stable.
export function RunHavenSidebar({
  navigation,
  onNavigate,
  onOpenSettings,
  settings,
  agents = [],
  runTerminalActive,
  appVersion,
}: {
  navigation: NavDestination
  onNavigate: (destination: NavDestination) => void
  onOpenSettings: () => void
  settings?: SettingsState
  agents?: AgentProfile[]
  // True while an interactive run terminal is live. It is not a managed session,
  // so it gets its own fixed row: without it, navigating away from a live run
  // would leave no way back (audit RT-1/RT-2).
  runTerminalActive: boolean
  appVersion: string
}) {
  const { setOpen: setSidebarOpen } = useSidebar()

  useEffect(() => {
    if (settings?.open) {
      setSidebarOpen(true)
    }
  }, [settings?.open, setSidebarOpen])

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
                label="History"
                Icon={History}
                active={navigation.kind === "runs"}
                onSelect={() => onNavigate({ kind: "runs" })}
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
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              tooltip="Settings"
              type="button"
              aria-expanded={settings?.open}
              onClick={() => {
                const opening = !settings?.open
                if (opening) {
                  setSidebarOpen(true)
                }
                if (settings) {
                  settings.setOpen(opening)
                } else {
                  onOpenSettings()
                }
              }}
            >
              <Settings />
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        {settings?.open ? (
          <div className="mx-2 rounded-lg border bg-background p-3 group-data-[collapsible=icon]:hidden">
            <div className="mb-3 text-sm font-semibold">Settings</div>
            <SettingsForm settings={settings} agents={agents} compact />
            <button
              type="button"
              className="mt-3 text-sm font-medium text-foreground underline underline-offset-4 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
              onClick={() => onNavigate({ kind: "diagnostics" })}
            >
              System details
            </button>
          </div>
        ) : null}
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
