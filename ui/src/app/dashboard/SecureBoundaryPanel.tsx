import { Folder, Globe, Lock, Shield, ShieldCheck, type LucideIcon } from "lucide-react"

import { DashboardPanel, PanelFooterLink } from "./DashboardPanel"

// Home "Secure boundary": a plain-language summary of RunHaven's default safety
// posture, so the secure path is visible before a run, not buried in docs. These
// are the product's standing guarantees (AGENTS.md safety boundary), not
// per-run state: an agent sees only the chosen project folder, signs in through
// an isolated broker/keychain, reaches only provider hosts by default, and never
// gets the host home directory. The footer opens the full model in Diagnostics.
const BOUNDARY: { icon: LucideIcon; label: string; value: string }[] = [
  { icon: Folder, label: "Workspace", value: "Project folder only" },
  { icon: Lock, label: "Credentials", value: "Isolated sign-in" },
  { icon: Globe, label: "Network", value: "Provider allowlist" },
  { icon: Shield, label: "Host access", value: "Home not mounted" },
]

export function SecureBoundaryPanel({ onViewModel }: { onViewModel: () => void }) {
  return (
    <DashboardPanel
      title="Secure boundary"
      footer={
        <PanelFooterLink
          icon={ShieldCheck}
          label="View security model"
          onClick={onViewModel}
        />
      }
    >
      <ul className="flex flex-col text-sm">
        {BOUNDARY.map((item) => (
          <li
            key={item.label}
            className="flex items-center gap-3 border-t py-2.5 first:border-t-0"
          >
            <item.icon aria-hidden className="size-4 shrink-0 text-muted-foreground" />
            <span className="text-foreground">{item.label}</span>
            <span className="ml-auto text-right text-muted-foreground">{item.value}</span>
          </li>
        ))}
      </ul>
    </DashboardPanel>
  )
}
