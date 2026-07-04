import { describe, expect, it } from "vitest";

import { authDetailSections, authStatusBadgeVariant } from "../app/authDetails";
import { authStatusLabel } from "../app/labels";
import type { AuthExplainResponse } from "../commands/runhaven";

function response(overrides: Partial<AuthExplainResponse> = {}): AuthExplainResponse {
  return {
    name: "claude",
    status: "api-key-broker",
    runtime: "macOS 26+ with Apple container only",
    supportedAuth: ["API key through the host-side broker"],
    hostKeeps: ["the host environment variable value you name"],
    guestReceives: ["a placeholder API key value"],
    providerHosts: ["api.anthropic.com"],
    currentSafePath: "Use the API key broker for a headless run.",
    notes: ["A note."],
    ...overrides,
  };
}

describe("authStatusLabel", () => {
  it("maps known statuses to beginner-friendly text", () => {
    expect(authStatusLabel("api-key-broker")).toBe("API key broker");
    expect(authStatusLabel("design-only")).toBe("Not ready");
    expect(authStatusLabel("n/a")).toBe("No sign-in needed");
  });

  it("treats missing status as unknown", () => {
    expect(authStatusLabel("-")).toBe("Unknown");
    expect(authStatusLabel("")).toBe("Unknown");
  });

  it("guards and title-cases an unrecognized status", () => {
    expect(authStatusLabel("custom")).toBe("Custom");
  });
});

describe("authStatusBadgeVariant", () => {
  it("marks brokered and ready states as success", () => {
    expect(authStatusBadgeVariant("api-key-broker")).toBe("success");
    expect(authStatusBadgeVariant("keychain-oauth")).toBe("success");
    expect(authStatusBadgeVariant("n/a")).toBe("success");
  });

  it("warns on design-only and defaults to secondary", () => {
    expect(authStatusBadgeVariant("design-only")).toBe("warning");
    expect(authStatusBadgeVariant("unknown")).toBe("secondary");
  });
});

describe("authDetailSections", () => {
  it("builds labeled lists in a stable order", () => {
    const sections = authDetailSections(response());
    expect(sections.map((section) => section.label)).toEqual([
      "Supported sign-in",
      "What this Mac keeps",
      "What the agent gets",
      "Provider hosts",
      "Notes",
    ]);
    expect(sections[3].items).toEqual(["api.anthropic.com"]);
  });

  it("drops sections that have no items", () => {
    const sections = authDetailSections(
      response({ providerHosts: [], notes: [] }),
    );
    expect(sections.map((section) => section.label)).toEqual([
      "Supported sign-in",
      "What this Mac keeps",
      "What the agent gets",
    ]);
  });
});
