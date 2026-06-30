import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunOutputCard } from "../app/RunOutputCard";
import type { LogSnapshotResponse } from "../commands/runhaven";

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
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
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
});
