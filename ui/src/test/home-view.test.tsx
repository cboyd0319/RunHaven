import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { HomeView } from "../app/views/HomeView";

import { dashboard, mainViewProps, managedSession } from "./main-view-props";

describe("HomeView", () => {
  it("shows the readiness line, the New run action, and the dashboard panels when healthy", () => {
    const html = renderToStaticMarkup(<HomeView {...mainViewProps()} />);

    expect(html).toContain("Ready to run");
    expect(html).toContain("New run");
    expect(html).not.toContain('disabled=""');
    // No status read has landed in this fixture, so no freshness line yet.
    expect(html).not.toContain("Last check");
    // The four Home panels are present; the empty recent list is a compact row.
    expect(html).toContain("Agent setup");
    expect(html).toContain("Secure boundary");
    expect(html).toContain("Recent activity");
    expect(html).toContain("No runs yet.");
    // The wizard is never inline on Home; it lives in the closed sheet.
    expect(html).not.toContain("Choose agent");
    expect(html).not.toContain("Finish setup");
  });

  it("shows the last-check freshness line once a status read has landed", () => {
    const html = renderToStaticMarkup(
      <HomeView {...mainViewProps({ lastRefreshedAt: "2026-07-04T18:00:00Z" })} />
    );
    expect(html).toContain("Last check");
  });

  it("leads with the setup blocker and disables New run with a reason while blocked", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({ setupReady: false, dashboard: dashboard(false) })}
      />
    );

    expect(html).toContain("Finish setup");
    expect(html).toContain("Apple container is not installed.");
    expect(html).toContain("Install Apple container.");
    expect(html).toContain("Check again");
    expect(html).toContain('disabled=""');
    expect(html).toContain("Fix the setup problem above before starting a run.");
    // The readiness line reads "Setup needed", not the ready copy.
    expect(html).toContain("Setup needed");
    expect(html).not.toContain("Ready to run");
  });

  it("does not say 'Checking...' forever after a failed first setup read", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({ setupReady: false, loading: false, dashboard: null })}
      />
    );

    // (Static markup escapes the apostrophe, so match on the plain tail.)
    expect(html).toContain("could not read your Mac");
    expect(html).toContain("Try Check again.");
    expect(html).not.toContain("Checking your Mac");
  });

  it("keeps New run enabled during a transient manual refresh when setup is ready", () => {
    const html = renderToStaticMarkup(
      <HomeView {...mainViewProps({ loading: true })} />
    );

    expect(html).not.toContain('disabled=""');
    expect(html).not.toContain("Checking your Mac");
  });

  it("summarizes agent readiness with a link to the Agents view", () => {
    const html = renderToStaticMarkup(<HomeView {...mainViewProps()} />);

    expect(html).toContain("Agent setup");
    expect(html).toContain("Codex");
    expect(html).toContain("Signed in");
    expect(html).toContain("Open Agents");
    // The backend-derived readiness (one signed-in agent) drives the summary.
    expect(html).toContain("1 signed in");
  });

  it("splits attention runs into Needs review and the rest into Recent activity", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({
          sessions: [
            managedSession({ id: "s1", title: "Codex - failing", lifecycle: "failed" }),
            managedSession({
              id: "s2",
              title: "Codex - done",
              lifecycle: "finished",
              statusLabel: "Finished",
            }),
            managedSession({ id: "s3", title: "Codex - quiet" }),
          ],
        })}
      />
    );

    expect(html).toContain("Needs review");
    expect(html).toContain("Codex - failing");
    expect(html).toContain("Recent activity");
    expect(html).toContain("Codex - done");
    // A quiet running session is now reachable from Recent activity (the sidebar
    // no longer lists sessions), so it appears there.
    expect(html).toContain("Codex - quiet");
    // A failed run sits in Needs review only, never repeated in Recent activity.
    // It renders twice WITHIN Needs review (the Run cell + the "Review <title>"
    // button aria-label); a leak into Recent activity would add a third.
    expect(html.split("Codex - failing").length - 1).toBe(2);
  });

  it("never leaks raw paths, containers, env, network, or session ids into the Home panels", () => {
    // Poison every raw ManagedSessionSummary field. One failed session routes to
    // Needs review, one running session to Recent activity, so BOTH table paths
    // are exercised (not just Needs review). None of the raw values may render.
    const poison = {
      project: "/Users/c/private/leak",
      containerName: "runhaven-codex-hidden-home",
      detailCommand: "runhaven runs show --env TOKEN=value",
      network: "provider-secret-net",
      session: "raw-session-8080",
    };
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({
          sessions: [
            managedSession({ id: "nr", title: "Codex - review", lifecycle: "failed", ...poison }),
            managedSession({ id: "ra", title: "Codex - active", ...poison }),
          ],
        })}
      />
    );

    expect(html).toContain("Codex - review");
    expect(html).toContain("Codex - active");
    for (const secret of [
      "/Users/c/private",
      "runhaven-codex-hidden-home",
      "TOKEN=value",
      "provider-secret-net",
      "raw-session-8080",
    ]) {
      expect(html).not.toContain(secret);
    }
  });
});
