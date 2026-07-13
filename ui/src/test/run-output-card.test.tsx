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
    warnings: [
      "The assistant can show private project information. Review this text before you share it.",
    ],
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
    expect(html).toContain("Session output");
    expect(html).toContain("Captured");
    expect(html).toContain("capture-timestamp");
    expect(html).toContain("Lines shown");
    expect(html).toContain("The assistant can show private project information");
    expect(html).toContain("entry line one");
    expect(html).toContain("Technical details");
    expect(html).toContain('aria-label="Session output"')
    expect(html.indexOf("entry line one")).toBeLessThan(
      html.indexOf("Technical details")
    );
    expect(html).not.toContain("/Users/");
  });

  it("renders every warning, including the managed-session redaction notice", () => {
    const html = renderToStaticMarkup(
      <RunOutputCard
        logLoading={false}
        logError=""
        logSnapshot={snapshot({
          warnings: [
            "The assistant can show private project information. Review this text before you share it.",
            "Known secrets and control characters were removed automatically. Review before sharing."
          ]
        })}
        onLoad={async () => undefined}
      />
    );

    expect(html).toContain(
      "The assistant can show private project information. Review this text before you share it.",
    );
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

    expect(unacknowledged).toContain("Show output…");
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
    expect(acknowledged).toContain("Refresh output");
    expect(acknowledged).not.toContain("Refresh output…");
    expect(acknowledged).not.toContain('aria-label="Show latest output…"');
    expect(acknowledged).not.toContain('disabled=""');
    expect(acknowledged).not.toContain("run-output-ack-hint");
  });

  it("removes the inert action when past output is no longer available", () => {
    const html = renderToStaticMarkup(
      <RunOutputCard
        logLoading={false}
        logError="RunHaven keeps session output only until the app closes. Output for this run is no longer available."
        logSnapshot={null}
        onLoad={async () => undefined}
      />
    );

    expect(html).toContain("Output unavailable");
    expect(html).not.toContain("Show output");
    expect(html).not.toContain("Show latest output");
  });
});
