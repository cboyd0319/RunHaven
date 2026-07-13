// Worktree review contracts for finished sessions that saved an isolated git
// copy. File entries are bounded safe repo-relative paths; the backend never
// sends a host path.
import type { ManagedSessionSummary } from "./types";

export type WorktreeFileChange = {
  path: string;
  kind: "added" | "changed" | "removed";
};

export type WorktreeReviewResponse = {
  runId: string;
  branch: string;
  statusSummary: string;
  worktreeChanges: number;
  sourceChanges: number;
  clean: boolean;
  fileChanges: WorktreeFileChange[];
  fileChangesTruncated: boolean;
  sourceFileChanges: WorktreeFileChange[];
  sourceFileChangesTruncated: boolean;
  reviewId: string;
  availableActions: string[];
};

export type WorktreeActionResult = {
  runId: string;
  action: string;
  branch: string;
  status: string;
  nextSteps: string[];
};

export type WorktreeActionResponse = {
  session: ManagedSessionSummary;
  result: WorktreeActionResult;
};
