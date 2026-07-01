import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  discardManagedSessionWorktree,
  keepManagedSessionWorktree,
  mergeManagedSessionWorktree,
  recoverManagedSessionWorktree,
  type ManagedSessionSummary,
  type WorktreeActionResult,
  type WorktreeReviewResponse,
} from "../commands/runhaven";
import { WorktreeReviewCard } from "../app/WorktreeReviewCard";
import { worktreeActionClients } from "../app/useWorktreeReview";
import {
  sessionAllowsWorktreeReview,
  worktreeActionRemovesWorktree,
  worktreeActionSuccessMessage,
  type WorktreeAction,
} from "../app/worktreeReview";

function session(patch: Partial<ManagedSessionSummary> = {}): ManagedSessionSummary {
  return {
    id: "run-123",
    runId: "run-123",
    agent: "codex",
    session: "project",
    title: "Codex - project",
    project: "project",
    status: "finished",
    statusLabel: "Finished",
    authStatus: "api-key-broker",
    authScope: "project",
    network: "provider",
    startedAt: "2026-06-30T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
    lifecycle: "finished",
    input: { attached: false, status: "detached", detail: "Session ended." },
    nextAction: { kind: "monitor", label: "Monitor session", detail: "Tracking." },
    events: [],
    ...patch,
  };
}

function review(patch: Partial<WorktreeReviewResponse> = {}): WorktreeReviewResponse {
  return {
    runId: "run-123",
    branch: "runhaven/codex-project",
    statusSummary: "2 files changed in the isolated copy.",
    worktreeChanges: 2,
    sourceChanges: 0,
    clean: false,
    availableActions: ["merge", "keep", "recover", "discard"],
    ...patch,
  };
}

const emptyConfirmations: Record<WorktreeAction, boolean> = {
  keep: false,
  merge: false,
  recover: false,
  discard: false,
};

function renderCard(patch: Partial<Parameters<typeof WorktreeReviewCard>[0]> = {}) {
  return renderToStaticMarkup(
    <WorktreeReviewCard
      review={review()}
      loading={false}
      loadError=""
      confirmations={emptyConfirmations}
      busy={false}
      message=""
      nextSteps={[]}
      actionError=""
      onConfirmationChange={() => undefined}
      onAction={() => undefined}
      {...patch}
    />,
  );
}

function disabledButtonCount(html: string): number {
  return html.match(/disabled=""/g)?.length ?? 0;
}

describe("worktree review gating", () => {
  it("shows the review only for an available worktree cleanup target", () => {
    const eligible = session({
      cleanup: {
        available: true,
        target: "worktree",
        label: "Review worktree",
        detail: "Choose what to do with the isolated copy.",
      },
    });
    expect(sessionAllowsWorktreeReview(eligible)).toBe(true);
  });

  it("hides the review when cleanup is unavailable, missing, or a different target", () => {
    expect(sessionAllowsWorktreeReview(session())).toBe(false);
    expect(
      sessionAllowsWorktreeReview(
        session({
          cleanup: {
            available: false,
            target: "worktree",
            label: "Review worktree",
            detail: "Not available yet.",
          },
        }),
      ),
    ).toBe(false);
    expect(
      sessionAllowsWorktreeReview(
        session({
          cleanup: {
            available: true,
            target: "container",
            label: "Clean up",
            detail: "Not a worktree.",
          },
        }),
      ),
    ).toBe(false);
  });

  it("marks only merge and discard as removing the copy", () => {
    expect(worktreeActionRemovesWorktree("merge")).toBe(true);
    expect(worktreeActionRemovesWorktree("discard")).toBe(true);
    expect(worktreeActionRemovesWorktree("keep")).toBe(false);
    expect(worktreeActionRemovesWorktree("recover")).toBe(false);
  });

  it("maps each action to its own typed client call", () => {
    expect(worktreeActionClients.keep).toBe(keepManagedSessionWorktree);
    expect(worktreeActionClients.merge).toBe(mergeManagedSessionWorktree);
    expect(worktreeActionClients.recover).toBe(recoverManagedSessionWorktree);
    expect(worktreeActionClients.discard).toBe(discardManagedSessionWorktree);
  });

  it("writes a plain success message for each action", () => {
    const base: WorktreeActionResult = {
      runId: "run-123",
      action: "keep",
      branch: "runhaven/codex-project",
      status: "kept",
      nextSteps: [],
    };
    expect(worktreeActionSuccessMessage({ ...base, action: "merge" })).toContain("Merged");
    expect(worktreeActionSuccessMessage({ ...base, action: "keep" })).toContain("Kept");
    expect(worktreeActionSuccessMessage({ ...base, action: "recover" })).toContain("manual steps");
    expect(worktreeActionSuccessMessage({ ...base, action: "discard" })).toContain("Discarded");
  });
});

describe("WorktreeReviewCard", () => {
  it("renders only safe review fields with no host paths", () => {
    const html = renderCard();

    expect(html).toContain("Review the isolated copy");
    expect(html).toContain("Branch: runhaven/codex-project");
    expect(html).toContain("2 files changed in the isolated copy.");
    expect(html).toContain("Changes in the copy");
    expect(html).toContain("Changes in your project");
    expect(html).toContain("Has changes");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/workspace");
    expect(html).not.toContain("state_volume");
  });

  it("keeps every action button disabled until its box is confirmed", () => {
    const unconfirmed = renderCard();
    expect(disabledButtonCount(unconfirmed)).toBe(4);

    const oneConfirmed = renderCard({
      confirmations: { ...emptyConfirmations, merge: true },
    });
    expect(disabledButtonCount(oneConfirmed)).toBe(3);
  });

  it("shows only the actions the backend allows", () => {
    const html = renderCard({
      review: review({ availableActions: ["keep", "recover"] }),
    });

    expect(html).toContain("Keep for later");
    expect(html).toContain("Recover by hand");
    expect(html).not.toContain("Merge into project");
    expect(html).not.toContain("Discard changes");
  });

  it("surfaces recover next steps as an ordered list", () => {
    const html = renderCard({
      message: "Here are the manual steps to recover the isolated copy.",
      nextSteps: [
        "Open the isolated copy in your terminal.",
        "Review the changes before you merge or discard them.",
      ],
    });

    expect(html).toContain("Here are the manual steps to recover the isolated copy.");
    expect(html).toContain("Open the isolated copy in your terminal.");
    expect(html).toContain("Review the changes before you merge or discard them.");
    expect(html).toContain("<ol");
  });

  it("reports a load error and a failed action clearly", () => {
    const loadFailed = renderCard({ review: null, loadError: "Copy details unavailable." });
    expect(loadFailed).toContain("Copy details unavailable");
    expect(loadFailed).not.toContain("Branch:");

    const actionFailed = renderCard({ actionError: "The repo was not clean." });
    expect(actionFailed).toContain("That action did not finish");
    expect(actionFailed).toContain("The repo was not clean.");
  });
});
