import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { HomeView } from "../app/views/HomeView";

import { dashboard, mainViewProps, managedSession } from "./main-view-props";

describe("HomeView", () => {
  it("shows the one-line health, the New run action, and the first-run note when healthy and empty", () => {
    const html = renderToStaticMarkup(<HomeView {...mainViewProps()} />);

    expect(html).toContain("Your Mac is ready to run agents.");
    expect(html).toContain("New run");
    expect(html).not.toContain('disabled=""');
    expect(html).toContain("RunHaven runs AI coding agents in a sandbox on your Mac.");
    // The wizard is no longer inline on Home; it lives in the closed sheet.
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
    expect(html).not.toContain("Your Mac is ready to run agents.");
  });

  it("lists agent readiness with a link to the Agents view", () => {
    const html = renderToStaticMarkup(<HomeView {...mainViewProps()} />);

    expect(html).toContain("Codex");
    expect(html).toContain("Ready");
    expect(html).toContain("Open Agents");
  });

  it("shows needs-you sessions and recent finished runs as navigable rows", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...mainViewProps({
          sessions: [
            managedSession({
              id: "s1",
              title: "Codex - failing",
              lifecycle: "failed"
            }),
            managedSession({
              id: "s2",
              title: "Codex - done",
              lifecycle: "finished",
              statusLabel: "Finished"
            }),
            managedSession({ id: "s3", title: "Codex - quiet" })
          ]
        })}
      />
    );

    expect(html).toContain("Needs you");
    expect(html).toContain("Codex - failing");
    expect(html).toContain("Recent finished runs");
    expect(html).toContain("Codex - done");
    // A quiet running session appears in neither Home list.
    expect(html).not.toContain("Codex - quiet");
    // A failed run sits in Needs you only: it never repeats in the finished
    // list, so each run has exactly one Home row.
    expect(html.split("Codex - failing").length - 1).toBe(1);
    // With sessions present, the first-run note goes away.
    expect(html).not.toContain("RunHaven runs AI coding agents in a sandbox");
  });
});
