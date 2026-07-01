// Worktree review contracts for finished sessions that saved an isolated git
// copy. The responses carry only branch names, change counts, a status summary,
// and path-free manual step labels; the backend never sends a host path.
import type { ManagedSessionSummary } from "./types";

export type WorktreeReviewResponse = {
  runId: string;
  branch: string;
  statusSummary: string;
  worktreeChanges: number;
  sourceChanges: number;
  clean: boolean;
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
