import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunOutputCard } from "../app/RunOutputCard";
import type { LogSnapshotResponse, SessionLogEntry } from "../commands/runhaven";

function snapshot(patch: Partial<LogSnapshotResponse> = {}): LogSnapshotResponse {
  return {
    runId: "run-123",
    capturedAt: "legacy-cursor",
    cursor: "entry-cursor",
    requestedLines: 200,
    entries: [
      {
        source: "container-stdio",
        timestamp: "entry-timestamp",
        text: "entry line one\nentry line two\n",
        lineCount: 2
      }
    ],
    text: "legacy line one\nlegacy line two\n",
    returnedLines: 99,
    truncated: false,
    source: "legacy-source",
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
    expect(html).toContain("entry-cursor");
    expect(html).not.toContain("legacy-cursor");
    expect(html).toContain("Returned lines");
    expect(html).not.toContain("99");
    expect(html).toContain("Raw container output can contain secrets");
    expect(html).toContain("entry line one");
    expect(html).not.toContain("legacy line one");
    expect(html).not.toContain("/Users/");
  });

  it("prefers session-owned log entries when present", () => {
    const sessionEntry: SessionLogEntry = {
      sessionId: "session-123",
      runId: "run-123",
      source: "container-stdio",
      cursor: "session-cursor",
      timestamp: "session-timestamp",
      text: "session entry line\n",
      lineCount: 1,
      redaction: {
        status: "notApplied",
        message: "Raw output is not automatically redacted."
      }
    };
    const html = renderToStaticMarkup(
      <RunOutputCard
        logAcknowledged={true}
        logLoading={false}
        logError=""
        logSnapshot={snapshot({
          sessionEntries: [sessionEntry]
        })}
        liveOutputAvailable={true}
        liveOutputEnabled={false}
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
        onLiveOutputChange={() => undefined}
      />
    );

    expect(html).toContain("session-cursor");
    expect(html).toContain("session entry line");
    expect(html).toContain("Raw output is not automatically redacted.");
    expect(html).not.toContain("entry-cursor");
    expect(html).not.toContain("entry line one");
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
  });
});
