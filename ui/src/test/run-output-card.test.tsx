import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RunOutputCard } from "../app/RunOutputCard";
import type { LogSnapshotResponse } from "../commands/runhaven";

function snapshot(patch: Partial<LogSnapshotResponse> = {}): LogSnapshotResponse {
  return {
    runId: "run-123",
    capturedAt: "preview-cursor",
    requestedLines: 200,
    text: "line one\nline two\n",
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
        onAcknowledgedChange={() => undefined}
        onLoad={async () => undefined}
      />
    );

    expect(html).toContain("Source");
    expect(html).toContain("Container output");
    expect(html).toContain("Captured");
    expect(html).toContain("preview-cursor");
    expect(html).toContain("Returned lines");
    expect(html).toContain("2");
    expect(html).toContain("Raw container output can contain secrets");
    expect(html).toContain("line one");
    expect(html).not.toContain("/Users/");
  });
});
