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
  it("offers one plain cleanup task without terminal-only instructions", () => {
    const markup = renderToStaticMarkup(<MaintenanceView setupReady />)
    expect(markup).toContain("Clean up stale networks")
    expect(markup).toContain("Check for stale networks")
    // Image rebuild moved to the Agents destination; a note links there.
    expect(markup).not.toContain("Rebuild agent image")
    expect(markup).not.toContain("runhaven state reset")
    expect(markup).not.toContain("Yes, remove these stale networks")
  })

  it("explains why cleanup is unavailable before setup is ready", () => {
    const markup = renderToStaticMarkup(<MaintenanceView setupReady={false} />)

    expect(markup).toContain("Finish setup before checking for stale networks")
    expect(markup).toContain("disabled")
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
