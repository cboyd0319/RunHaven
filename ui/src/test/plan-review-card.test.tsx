import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { PlanReviewCard } from "../app/PlanReviewCard";
import type { RunPlanResponse } from "../commands/runhaven";

// The plan carries a state-volume name and container name that the review table
// must not render. The workspace path is intentional (the user picked it).
function plan(patch: Partial<RunPlanResponse> = {}): RunPlanResponse {
  return {
    profile: "codex",
    workspace: "/Users/you/project",
    workspaceScope: "current",
    workspaceScopeNote: null,
    authScope: "project",
    stateVolume: "runhaven-state-secret",
    session: "project",
    containerName: "runhaven-codex-secret",
    networkMode: "provider",
    networkName: "runhaven-provider-preview",
    egressSummary: "Only reviewed provider hosts are reachable.",
    image: "runhaven/codex:0.1.0",
    providerAllowedHosts: ["api.openai.com"],
    access: {
      status: "ready",
      label: "Access looks good",
      detail: "This run uses the secure default access.",
      remedy: "",
      blocking: false,
      apiKeyBrokerEnv: null,
      apiKeyBrokerEnvPresent: null
    },
    warnings: [],
    worktree: false,
    worktreeNote: null,
    ...patch
  };
}

const ready = {
  launchConfirmation: true,
  confirmedWarnings: [] as string[],
  launchReady: true,
  launching: false,
  setupOk: true,
  launchImageReady: true,
  onLaunchConfirmationChange: () => {},
  onConfirmWarning: () => {},
  onBack: () => {},
  onStart: async () => {}
};

describe("PlanReviewCard", () => {
  it("renders a non-blocking review without leaking the state volume", () => {
    const html = renderToStaticMarkup(<PlanReviewCard plan={plan()} {...ready} />);

    expect(html).toContain("Review access");
    expect(html).toContain("This project only");
    expect(html).toContain("Access looks good");
    expect(html).toContain("This run uses the secure default access.");
    expect(html).toContain("Only reviewed provider hosts are reachable.");
    expect(html).toContain("Launch run");
    expect(html).not.toContain("Fix access before launching a run.");
    expect(html).not.toContain("Warnings need confirmation");
    // State volume and container name travel in the plan but never render.
    expect(html).not.toContain("runhaven-state-secret");
    expect(html).not.toContain("runhaven-codex-secret");
  });

  it("flags blocking access and shows its remedy", () => {
    const html = renderToStaticMarkup(
      <PlanReviewCard
        plan={plan({
          access: {
            status: "blocked",
            label: "Access blocked",
            detail: "This run would expose host credentials.",
            remedy: "Choose a project session instead.",
            blocking: true,
            apiKeyBrokerEnv: null,
            apiKeyBrokerEnvPresent: null
          }
        })}
        {...ready}
        launchReady={false}
      />
    );

    expect(html).toContain("Access blocked");
    expect(html).toContain("This run would expose host credentials.");
    expect(html).toContain("Choose a project session instead.");
    expect(html).toContain("Fix access before launching a run.");
  });

  it("requires confirming each warning for a lower-security choice", () => {
    const html = renderToStaticMarkup(
      <PlanReviewCard
        plan={plan({
          warnings: [
            { code: "full-internet", message: "This run can reach the whole internet." },
            { code: "read-write", message: "The agent can change your project files." }
          ]
        })}
        {...ready}
      />
    );

    expect(html).toContain("Warnings need confirmation");
    expect(html).toContain("This run can reach the whole internet.");
    expect(html).toContain("The agent can change your project files.");
    expect(html).toContain('aria-label="Confirm full-internet warning"');
    expect(html).toContain('aria-label="Confirm read-write warning"');
  });

  it("shows the isolated-copy note only when the plan uses a worktree", () => {
    const without = renderToStaticMarkup(<PlanReviewCard plan={plan()} {...ready} />);
    expect(without).not.toContain("Isolated copy (git worktree)");

    const withWorktree = renderToStaticMarkup(
      <PlanReviewCard
        plan={plan({
          worktree: true,
          worktreeNote: "RunHaven will make a separate copy on a new git branch.",
        })}
        {...ready}
      />,
    );
    expect(withWorktree).toContain("Isolated copy (git worktree)");
    expect(withWorktree).toContain(
      "RunHaven will make a separate copy on a new git branch.",
    );
  });

  it("shows a workspace scope note when the plan carries one", () => {
    const html = renderToStaticMarkup(
      <PlanReviewCard
        plan={plan({ workspaceScopeNote: "Using the git root, not just this folder." })}
        {...ready}
      />
    );

    expect(html).toContain("Workspace scope");
    expect(html).toContain("Using the git root, not just this folder.");
  });

  it("explains why launch stays blocked for setup and image gaps", () => {
    const html = renderToStaticMarkup(
      <PlanReviewCard
        plan={plan()}
        {...ready}
        setupOk={false}
        launchImageReady={false}
        launchReady={false}
      />
    );

    expect(html).toContain("Fix setup blockers before launching a run.");
    expect(html).toContain("Fix image readiness before launching a run.");
  });

  it("shows a launching label while the run is starting", () => {
    const html = renderToStaticMarkup(
      <PlanReviewCard plan={plan()} {...ready} launching={true} />
    );

    expect(html).toContain("Launching...");
    expect(html).not.toContain(">Launch run<");
  });
});
