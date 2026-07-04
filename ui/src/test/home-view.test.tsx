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
    // The four Home panels are present; the empty recent list is a compact row.
    expect(html).toContain("Agent setup");
    expect(html).toContain("Secure boundary");
    expect(html).toContain("Recent activity");
    expect(html).toContain("No runs yet.");
    // The wizard is never inline on Home; it lives in the closed sheet.
    expect(html).not.toContain("Choose agent");
    expect(html).not.toContain("Finish setup");
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
    expect(html).toContain("Ready");
    expect(html).toContain("Open Agents");
    // The one-agent-ready preview drives the summary line.
    expect(html).toContain("1 ready");
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
    expect(html.split("Codex - failing").length - 1).toBe(1);
  });

  it("never leaks raw paths, containers, or env values into the Home panels", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({
          sessions: [
            managedSession({
              id: "leak",
              title: "Codex - review",
              lifecycle: "failed",
              project: "/Users/c/private/leak",
              containerName: "runhaven-codex-hidden-home",
              detailCommand: "runhaven runs show --env TOKEN=value",
            }),
          ],
        })}
      />
    );

    expect(html).toContain("Codex - review");
    expect(html).not.toContain("/Users/c/private");
    expect(html).not.toContain("runhaven-codex-hidden-home");
    expect(html).not.toContain("TOKEN=value");
  });
});
