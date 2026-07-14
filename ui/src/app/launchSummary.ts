import type { RunPlanRequest } from "@/commands/runhaven";

export function projectName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  const name = (parts.at(-1) ?? path)
    .replace(
      /[\u0000-\u001f\u007f-\u009f\u00ad\u034f\u061c\u180e\u200b-\u200f\u2028-\u202e\u2060-\u2069\ufeff]/gu,
      "",
    )
    .trim();
  return name || "Project";
}

export function projectAccessLabel(request: RunPlanRequest): string {
  if (request.worktree) {
    return "Protected copy";
  }
  return request.readOnlyWorkspace
    ? "Read-only original folder"
    : "Original folder with changes";
}
