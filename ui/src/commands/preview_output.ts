import type { LogSnapshotResponse } from "./types";

export function previewLogSnapshot(
  runId: string,
  requestedLines: number,
  patch: Partial<Pick<LogSnapshotResponse, "capturedAt" | "text">> = {}
): LogSnapshotResponse {
  const text = patch.text ?? "Preview log line one\nPreview log line two\n";
  const capturedAt = patch.capturedAt ?? "preview";
  const lineCount = text.split("\n").filter(Boolean).length;
  return {
    runId,
    capturedAt,
    requestedLines,
    text,
    returnedLines: lineCount,
    truncated: false,
    source: "container-stdio",
    warnings: ["Raw container output can contain secrets or workspace content."]
  };
}
