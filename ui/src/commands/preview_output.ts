import type { LogSnapshotResponse } from "./types";

export function previewLogSnapshot(
  runId: string,
  requestedLines: number,
  patch: Partial<Pick<LogSnapshotResponse, "cursor" | "capturedAt" | "text">> = {}
): LogSnapshotResponse {
  const text = patch.text ?? "Preview log line one\nPreview log line two\n";
  const capturedAt = patch.capturedAt ?? patch.cursor ?? "preview";
  const cursor = patch.cursor ?? capturedAt;
  const lineCount = text.split("\n").filter(Boolean).length;
  return {
    runId,
    capturedAt,
    cursor,
    requestedLines,
    entries: [
      {
        source: "container-stdio",
        timestamp: capturedAt,
        text,
        lineCount
      }
    ],
    text,
    returnedLines: lineCount,
    truncated: false,
    source: "container-stdio",
    warnings: ["Raw container output can contain secrets or workspace content."]
  };
}
