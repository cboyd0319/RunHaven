import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
  rebuildAgentImage,
} from "../commands/runhaven";
import { MaintenanceCard } from "../app/MaintenanceCard";
import { maintenanceClients } from "../app/useMaintenance";

describe("MaintenanceCard", () => {
  it("offers the image rebuild and network cleanup actions with agent choices", () => {
    const markup = renderToStaticMarkup(
      <MaintenanceCard agents={["codex", "claude"]} />
    );
    expect(markup).toContain("Rebuild agent image");
    expect(markup).toContain("Clean up stale networks");
    expect(markup).toContain(">codex<");
    expect(markup).toContain(">claude<");
    // The rebuild button stays disabled until the user confirms.
    expect(markup).toContain("disabled");
  });

  it("binds each maintenance action to its typed client", () => {
    expect(maintenanceClients.rebuildAgentImage).toBe(rebuildAgentImage);
    expect(maintenanceClients.getImageBuildStatus).toBe(getImageBuildStatus);
    expect(maintenanceClients.getNetworkPrunePreview).toBe(getNetworkPrunePreview);
    expect(maintenanceClients.pruneManagedNetworks).toBe(pruneManagedNetworks);
  });

  it("keeps preview fallbacks to sanitized counts only", async () => {
    const preview = await getNetworkPrunePreview();
    expect(preview).toEqual({ count: 0 });
    const outcome = await pruneManagedNetworks();
    expect(Object.keys(outcome).sort()).toEqual(["failed", "removed"]);
  });
});
