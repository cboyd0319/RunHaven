import type { ComponentProps } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunStatusCard } from "../app/RunStatusCard";
import type { RunControlAction } from "../app/types";

const emptyConfirmations: Record<RunControlAction, boolean> = {
  stop: false,
  kill: false,
  repair: false,
  cleanup: false
};

const confirmedControls: Record<RunControlAction, boolean> = {
  stop: true,
  kill: true,
  repair: true,
  cleanup: true
};

function renderCard(
  patch: Partial<ComponentProps<typeof RunStatusCard>> = {}
) {
  return renderToStaticMarkup(
    <RunStatusCard
      title="Session controls"
      controlTarget="session"
      showStatus={false}
      controlsAvailable={true}
      cleanupAvailable={false}
      runStatus={null}
      runStatusLoading={false}
      runStatusError=""
      controlBusy={false}
      controlError=""
      controlMessage=""
      controlConfirmations={emptyConfirmations}
      onConfirmationChange={() => undefined}
      onControl={async () => undefined}
      {...patch}
    />
  );
}

function disabledControlCount(html: string): number {
  return html.match(/disabled=""/g)?.length ?? 0;
}

describe("RunStatusCard", () => {
  it("shows cleanup only when cleanup is available", () => {
    const unavailable = renderCard({
      controlsAvailable: false,
      cleanupAvailable: false,
      controlConfirmations: confirmedControls
    });

    expect(unavailable).not.toContain("Delete worktree");
    expect(unavailable).not.toContain("Confirm deleting");
    expect(unavailable).not.toContain("Stop run");

    const available = renderCard({
      controlsAvailable: false,
      cleanupAvailable: true
    });

    expect(available).toContain("Delete worktree");
    expect(available).toContain("Confirm deleting this session");
    expect(available).not.toContain("Stop run");
    expect(available).not.toContain("Hard stop");
    expect(available).not.toContain("Repair session record");
  });

  it("leaves destructive controls disabled after confirmations reset", () => {
    const beforeAttempt = renderCard({
      cleanupAvailable: true,
      controlConfirmations: confirmedControls
    });

    expect(beforeAttempt).toContain("Stop run");
    expect(beforeAttempt).toContain("Hard stop");
    expect(beforeAttempt).toContain("Repair session record");
    expect(beforeAttempt).toContain("Delete worktree");
    expect(disabledControlCount(beforeAttempt)).toBe(0);

    const afterAttempt = renderCard({
      cleanupAvailable: true,
      controlMessage: "Stop requested for run-123.",
      controlConfirmations: emptyConfirmations
    });

    expect(afterAttempt).toContain("Stop requested for run-123.");
    expect(disabledControlCount(afterAttempt)).toBe(4);
  });
});
