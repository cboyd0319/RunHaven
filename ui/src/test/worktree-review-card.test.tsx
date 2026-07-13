import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  discardManagedSessionWorktree,
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
    statusSummary: "2 file changes are ready to review.",
    worktreeChanges: 2,
    sourceChanges: 0,
    clean: false,
    fileChanges: [
      { path: "src/app.ts", kind: "changed" },
      { path: "src/session.ts", kind: "added" },
    ],
    fileChangesTruncated: false,
    sourceFileChanges: [],
    sourceFileChangesTruncated: false,
    reviewId: "0".repeat(64),
    availableActions: ["merge", "recover", "discard"],
    ...patch,
  };
}

function renderCard(patch: Partial<Parameters<typeof WorktreeReviewCard>[0]> = {}) {
  return renderToStaticMarkup(
    <WorktreeReviewCard
      review={review()}
      loading={false}
      loadError=""
      busy={false}
      message=""
      nextSteps={[]}
      actionError=""
      onAction={() => undefined}
      {...patch}
    />,
  );
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
    expect(worktreeActionRemovesWorktree("recover")).toBe(false);
  });

  it("maps each action to its own typed client call", () => {
    expect(worktreeActionClients.merge).toBe(mergeManagedSessionWorktree);
    expect(worktreeActionClients.recover).toBe(recoverManagedSessionWorktree);
    expect(worktreeActionClients.discard).toBe(discardManagedSessionWorktree);
  });

  it("writes a plain success message for each action", () => {
    const base: WorktreeActionResult = {
      runId: "run-123",
      action: "merge",
      branch: "runhaven/codex-project",
      status: "merged",
      nextSteps: [],
    };
    expect(worktreeActionSuccessMessage({ ...base, action: "merge" })).toBe(
      "Applied the assistant's changes to your project.",
    );
    expect(worktreeActionSuccessMessage({ ...base, action: "recover" })).toBe(
      "Follow these steps to open the protected copy.",
    );
    expect(worktreeActionSuccessMessage({ ...base, action: "discard" })).toBe(
      "Discarded the assistant's changes.",
    );
  });
});

describe("WorktreeReviewCard", () => {
  it("renders only safe review fields with no host paths", () => {
    const html = renderCard();

    expect(html).toContain("Review changes");
    expect(html).toContain("The assistant worked in a protected copy");
    expect(html).toContain("Assistant changes");
    expect(html).toContain("Changed");
    expect(html).toContain("src/app.ts");
    expect(html).toContain("Added");
    expect(html).toContain("src/session.ts");
    expect(html).not.toContain("Has changes");
    expect(html).not.toContain("Changed files");
    expect(html).not.toContain("Other project changes");
    expect(html).not.toContain("<code");
    expect(html).not.toContain("Branch:");
    expect(html).not.toContain("runhaven/codex-project");
    expect(html).not.toContain("Clean");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/workspace");
    expect(html).not.toContain("state_volume");
  });

  it("relies on the Rust-owned native confirmation for every action", () => {
    const html = renderCard();
    expect(html).not.toContain('type="checkbox"');
    expect(html).not.toContain("confirmation required");
    expect(html).not.toContain('disabled=""');
  });

  it("warns when the project changed outside the protected copy", () => {
    const html = renderCard({
      review: review({
        sourceChanges: 1,
        sourceFileChanges: [{ path: "src/local.ts", kind: "changed" }],
      }),
    });

    expect(html).toContain("Your project also changed");
    expect(html).toContain("Changes already in your project");
    expect(html).toContain("src/local.ts");
    expect(html).toContain('disabled=""');
  });

  it("shows only the two routine decisions and keeps recovery contextual", () => {
    const html = renderCard({
      review: review({ availableActions: ["merge", "recover", "discard"] }),
    });

    expect(html).toContain("Apply changes");
    expect(html).toContain("Discard changes");
    expect(html).not.toContain("Review later");
    expect(html).not.toContain("Show recovery steps");
  });

  it("states when the bounded file list is shortened", () => {
    const html = renderCard({
      review: review({
        worktreeChanges: 142,
        fileChangesTruncated: true,
      }),
    });

    expect(html).toContain("Showing 2 of 142 file changes.");
  });

  it("uses plain empty-state copy without count cards", () => {
    const html = renderCard({
      review: review({
        worktreeChanges: 0,
        clean: true,
        fileChanges: [],
      }),
    });

    expect(html).toContain("The assistant didn’t change any files.");
    expect(html).not.toContain("Assistant changes");
  });

  it("surfaces recover next steps as an ordered list", () => {
    const html = renderCard({
      message: "Follow these steps to open the protected copy.",
      nextSteps: [
        "Keep the protected copy so nothing is lost.",
        "Finish or undo other changes in your project.",
        "Return to this review and choose Apply changes again.",
      ],
    });

    expect(html).toContain("Follow these steps to open the protected copy.");
    expect(html).toContain("Keep the protected copy so nothing is lost.");
    expect(html).toContain("Finish or undo other changes in your project.");
    expect(html).toContain("Return to this review and choose Apply changes again.");
    expect(html).toContain("<ol");
  });

  it("reports a load error and a failed action clearly", () => {
    const loadFailed = renderCard({ review: null, loadError: "Copy details unavailable." });
    expect(loadFailed).toContain("Copy details unavailable");
    expect(loadFailed).not.toContain("Branch:");

    const actionFailed = renderCard({ actionError: "The repo was not clean." });
    expect(actionFailed).toContain("Couldn’t update these changes");
    expect(actionFailed).toContain("The repo was not clean.");
    expect(actionFailed).toContain("Show recovery steps");
  });
});
