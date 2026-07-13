import type { ComponentProps } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunStatusCard } from "../app/RunStatusCard";

function renderCard(
  patch: Partial<ComponentProps<typeof RunStatusCard>> = {}
) {
  return renderToStaticMarkup(
    <RunStatusCard
      title="Session controls"
      showStatus={false}
      controlsAvailable={true}
      cleanupAvailable={false}
      runStatus={null}
      runStatusLoading={false}
      runStatusError=""
      controlBusy={false}
      controlError=""
      controlMessage=""
      onControl={async () => undefined}
      {...patch}
    />
  );
}

describe("RunStatusCard", () => {
  it("shows cleanup only when cleanup is available", () => {
    const unavailable = renderCard({
      controlsAvailable: false,
      cleanupAvailable: false
    });

    expect(unavailable).not.toContain("Clean up run");
    expect(unavailable).not.toContain("Confirm cleaning up");
    expect(unavailable).not.toContain("Stop run");

    const available = renderCard({
      controlsAvailable: false,
      cleanupAvailable: true
    });

    expect(available).toContain("Clean up run");
    expect(available).toContain("RunHaven asks for confirmation in a Mac dialog");
    expect(available).not.toContain("Stop run");
    expect(available).not.toContain("Hard stop");
    expect(available).not.toContain("Repair session record");
  });

  it("keeps recovery actions secondary and relies on native confirmation", () => {
    const beforeAttempt = renderCard({ cleanupAvailable: true });

    expect(beforeAttempt).toContain("Stop run");
    expect(beforeAttempt).toContain("Hard stop");
    expect(beforeAttempt).toContain("Repair run record");
    expect(beforeAttempt).toContain("Clean up run");
    expect(beforeAttempt).toContain("More actions");
    expect(beforeAttempt).not.toContain("confirmation required");
    expect(beforeAttempt).not.toContain('type="checkbox"');

    const afterAttempt = renderCard({
      cleanupAvailable: true,
      controlMessage: "Stop requested for run-123."
    });

    expect(afterAttempt).toContain("Stop requested for run-123.");
    expect(afterAttempt).not.toContain('disabled=""');
  });
});
