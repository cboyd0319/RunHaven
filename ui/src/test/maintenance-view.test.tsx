import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import {
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
  rebuildAgentImage,
} from "../commands/runhaven"
import { MaintenanceView } from "../app/views/MaintenanceView"
import { maintenanceClients } from "../app/useMaintenance"

describe("MaintenanceView", () => {
  it("offers network cleanup and points at Agents and the CLI for the rest", () => {
    const markup = renderToStaticMarkup(<MaintenanceView />)
    expect(markup).toContain("Clean up stale networks")
    expect(markup).toContain("Check for stale networks")
    // Image rebuild moved to the Agents destination; a note links there.
    expect(markup).not.toContain("Rebuild agent image")
    expect(markup).toContain("open Agents in the sidebar")
    // Saved-state reset stays CLI-only, named plainly with the command.
    expect(markup).toContain("runhaven state reset")
  })

  it("binds each maintenance action to its typed client", () => {
    expect(maintenanceClients.rebuildAgentImage).toBe(rebuildAgentImage)
    expect(maintenanceClients.getImageBuildStatus).toBe(getImageBuildStatus)
    expect(maintenanceClients.getNetworkPrunePreview).toBe(getNetworkPrunePreview)
    expect(maintenanceClients.pruneManagedNetworks).toBe(pruneManagedNetworks)
  })

  it("keeps preview fallbacks to sanitized counts only", async () => {
    // The preview starts with one stale network so the confirm-gated cleanup
    // flow is reachable, then drops to zero after a prune.
    const preview = await getNetworkPrunePreview()
    expect(preview).toEqual({ count: 1 })
    const outcome = await pruneManagedNetworks()
    expect(outcome).toEqual({ removed: 1, failed: 0 })
    expect(await getNetworkPrunePreview()).toEqual({ count: 0 })
  })
})
