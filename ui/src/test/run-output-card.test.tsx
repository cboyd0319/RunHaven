import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunOutputCard } from "../app/RunOutputCard";
import type { LogSnapshotResponse } from "../commands/runhaven";

function snapshot(patch: Partial<LogSnapshotResponse> = {}): LogSnapshotResponse {
  return {
    runId: "run-123",
    capturedAt: "capture-timestamp",
    requestedLines: 200,
    text: "entry line one\nentry line two\n",
    returnedLines: 2,
    truncated: false,
    source: "container-stdio",
    warnings: ["Raw container output can contain secrets or workspace content."],
    ...patch
  };
}

describe("RunOutputCard", () => {
  it("renders typed snapshot metadata and bounded output", () => {
    const html = renderToStaticMarkup(
      <RunOutputCard
        logAcknowledged={true}
        logLoading={false}
        logError=""
        logSnapshot={snapshot()}
        liveOutputAvailable={false}
        liveOutputEnabled={false}
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
        onLiveOutputChange={() => undefined}
      />
    );

    expect(html).toContain("Source");
    expect(html).toContain("Container output");
    expect(html).toContain("Captured");
    expect(html).toContain("capture-timestamp");
    expect(html).toContain("Returned lines");
    expect(html).toContain("Raw container output can contain secrets");
    expect(html).toContain("entry line one");
    expect(html).not.toContain("/Users/");
  });

  it("renders every warning, including the managed-session redaction notice", () => {
    const html = renderToStaticMarkup(
      <RunOutputCard
        logAcknowledged={true}
        logLoading={false}
        logError=""
        logSnapshot={snapshot({
          warnings: [
            "Raw container output can contain secrets or workspace content.",
            "Known secrets and control characters were removed automatically. Review before sharing."
          ]
        })}
        liveOutputAvailable={true}
        liveOutputEnabled={false}
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
        onLiveOutputChange={() => undefined}
      />
    );

    expect(html).toContain("Raw container output can contain secrets");
    expect(html).toContain("removed automatically");
    expect(html).toContain("entry line one");
  });

  it("gates live output refresh behind raw-output acknowledgement", () => {
    const unacknowledged = renderToStaticMarkup(
      <RunOutputCard
        logAcknowledged={false}
        logLoading={false}
        logError=""
        logSnapshot={null}
        liveOutputAvailable={true}
        liveOutputEnabled={false}
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
        onLiveOutputChange={() => undefined}
      />
    );

    expect(unacknowledged).toContain("Keep output updated");
    expect(unacknowledged).toContain('disabled=""');
    // The disabled load button points at the ack checkbox so the gate reads as
    // intentional, not broken.
    expect(unacknowledged).toContain('aria-describedby="run-output-ack-hint"');
    expect(unacknowledged).toContain(
      "To load output, first check the box above to confirm raw output."
    );

    const acknowledged = renderToStaticMarkup(
      <RunOutputCard
        logAcknowledged={true}
        logLoading={false}
        logError=""
        logSnapshot={snapshot()}
        liveOutputAvailable={true}
        liveOutputEnabled={true}
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
        onLiveOutputChange={() => undefined}
      />
    );

    expect(acknowledged).toContain("Updates every few seconds.");
    expect(acknowledged).not.toContain('disabled=""');
    expect(acknowledged).not.toContain("run-output-ack-hint");
  });
});
