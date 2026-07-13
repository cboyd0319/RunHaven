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
        logLoading={false}
        logError=""
        logSnapshot={snapshot()}
        onLoad={async () => undefined}
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
        logLoading={false}
        logError=""
        logSnapshot={snapshot({
          warnings: [
            "Raw container output can contain secrets or workspace content.",
            "Known secrets and control characters were removed automatically. Review before sharing."
          ]
        })}
        onLoad={async () => undefined}
      />
    );

    expect(html).toContain("Raw container output can contain secrets");
    expect(html).toContain("removed automatically");
    expect(html).toContain("entry line one");
  });

  it("uses one explicit action and leaves the trust warning to the native dialog", () => {
    const unacknowledged = renderToStaticMarkup(
      <RunOutputCard
        logLoading={false}
        logError=""
        logSnapshot={null}
        onLoad={async () => undefined}
      />
    );

    expect(unacknowledged).toContain("Show latest output");
    expect(unacknowledged).not.toContain("Keep output updated");
    expect(unacknowledged).not.toContain("I understand");
    expect(unacknowledged).not.toContain('disabled=""');

    const acknowledged = renderToStaticMarkup(
      <RunOutputCard
        logLoading={false}
        logError=""
        logSnapshot={snapshot()}
        onLoad={async () => undefined}
      />
    );

    expect(acknowledged).not.toContain("Updates every few seconds.");
    expect(acknowledged).not.toContain('disabled=""');
    expect(acknowledged).not.toContain("run-output-ack-hint");
  });
});
