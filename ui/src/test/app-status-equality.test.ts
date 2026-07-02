import { describe, expect, it } from "vitest"

import {
  sameDashboardStatus,
  sameImageStatus,
  sameRunStatus,
} from "../app/appStatusEquality"
import type {
  AgentProfile,
  DashboardStatus,
  ImageStatusResponse,
  RunStatusResponse,
  RunSummary,
} from "../commands/runhaven"

function runSummary(patch: Partial<RunSummary> = {}): RunSummary {
  return {
    runId: "run-1",
    profile: "codex",
    workspace: "/workspace",
    network: "provider",
    status: "running",
    timestamp: "2026-07-01T02:00:00Z",
    authScope: "project",
    session: "default",
    ...patch,
  }
}

function agent(patch: Partial<AgentProfile> = {}): AgentProfile {
  return {
    name: "codex",
    description: "Codex agent",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex", "run"],
    providerHosts: ["api.openai.com"],
    signIn: "api-key",
    broker: "codex-api-key",
    supportsInAppSignIn: true,
    ...patch,
  }
}

function dashboard(patch: Partial<DashboardStatus> = {}): DashboardStatus {
  return {
    setup: {
      ok: true,
      blockerCount: 0,
      checks: [{ name: "container", ok: true, detail: "ready", remedy: "" }],
    },
    agents: [agent()],
    activeRuns: [runSummary()],
    recentRuns: [runSummary({ runId: "run-0", status: "finished" })],
    warnings: ["one warning"],
    appVersion: "0.5.0",
    ...patch,
  }
}

function imageStatus(patch: Partial<ImageStatusResponse> = {}): ImageStatusResponse {
  return {
    agent: "codex",
    image: {
      agent: "codex",
      image: "runhaven/codex:0.1.0",
      status: "ok",
      ready: true,
      expectedSourceDigest: "sha256:expected",
      localSourceDigest: "sha256:expected",
      fixCommand: null,
    },
    builder: {
      status: "running",
      detail: "builder ready",
      image: "runhaven/builder:0.1.0",
      cpus: "2",
      memory: "2048 MiB",
      rosetta: false,
      startedDate: "2026-07-01T01:00:00Z",
      ipv4Address: "192.168.64.2/24",
      warning: null,
    },
    ...patch,
  }
}

function runStatus(patch: Partial<RunStatusResponse> = {}): RunStatusResponse {
  return {
    run: {
      runId: "run-1",
      profile: "codex",
      workspace: "/workspace",
      networkMode: "provider",
      status: "running",
      timestamp: "2026-07-01T02:00:00Z",
      authScope: "project",
      session: "default",
      containerName: "runhaven-codex-default",
    },
    container: {
      state: "running",
      image: "runhaven/codex:0.1.0",
      startedAt: "2026-07-01T02:00:03Z",
      resources: { cpus: "4", memoryBytes: 4 * 1024 ** 3 },
      networks: [
        {
          network: "runhaven-provider",
          hostname: "runhaven-codex-default",
          ipv4Address: "192.168.64.20/24",
          ipv4Gateway: "192.168.64.1",
          ipv6Address: null,
        },
      ],
    },
    ...patch,
  }
}

describe("sameDashboardStatus", () => {
  it("treats a fully populated identical copy as unchanged", () => {
    expect(sameDashboardStatus(dashboard(), dashboard())).toBe(true)
  })

  it("treats two empty dashboards as unchanged", () => {
    const empty = dashboard({
      setup: { ok: true, blockerCount: 0, checks: [] },
      agents: [],
      activeRuns: [],
      recentRuns: [],
      warnings: [],
    })
    expect(sameDashboardStatus(empty, structuredClone(empty))).toBe(true)
  })

  it("detects scalar setup field changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        setup: { ...base.setup, ok: false },
      }),
    ).toBe(false)
    expect(
      sameDashboardStatus(base, {
        ...base,
        setup: { ...base.setup, blockerCount: 1 },
      }),
    ).toBe(false)
  })

  it("detects setup check length and field changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        setup: { ...base.setup, checks: [] },
      }),
    ).toBe(false)
    expect(
      sameDashboardStatus(base, {
        ...base,
        setup: {
          ...base.setup,
          checks: [{ ...base.setup.checks[0]!, detail: "changed" }],
        },
      }),
    ).toBe(false)
  })

  it("detects agent list length changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, { ...base, agents: [agent(), agent()] }),
    ).toBe(false)
  })

  it("detects agent scalar field changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        agents: [agent({ broker: "other-broker" })],
      }),
    ).toBe(false)
  })

  it("detects agent defaultCommand element and length changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        agents: [agent({ defaultCommand: ["codex", "resume"] })],
      }),
    ).toBe(false)
    expect(
      sameDashboardStatus(base, {
        ...base,
        agents: [agent({ defaultCommand: ["codex"] })],
      }),
    ).toBe(false)
  })

  it("detects agent providerHosts changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        agents: [agent({ providerHosts: ["api.openai.com", "extra.example"] })],
      }),
    ).toBe(false)
  })

  it("detects active and recent run field changes", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, {
        ...base,
        activeRuns: [runSummary({ status: "stopping" })],
      }),
    ).toBe(false)
    expect(
      sameDashboardStatus(base, {
        ...base,
        recentRuns: [runSummary({ runId: "run-0", status: "failed" })],
      }),
    ).toBe(false)
  })

  it("detects warnings length and content changes", () => {
    const base = dashboard()
    expect(sameDashboardStatus(base, { ...base, warnings: [] })).toBe(false)
    expect(
      sameDashboardStatus(base, { ...base, warnings: ["different warning"] }),
    ).toBe(false)
  })

  it("detects an app version change", () => {
    const base = dashboard()
    expect(
      sameDashboardStatus(base, { ...base, appVersion: "0.6.0" }),
    ).toBe(false)
  })
})

describe("sameImageStatus", () => {
  it("treats an identical copy as unchanged", () => {
    expect(sameImageStatus(imageStatus(), imageStatus())).toBe(true)
  })

  it("distinguishes a null localSourceDigest from a value", () => {
    const base = imageStatus()
    expect(
      sameImageStatus(base, {
        ...base,
        image: { ...base.image, localSourceDigest: null },
      }),
    ).toBe(false)
  })

  it("detects readiness and fix-command changes", () => {
    const base = imageStatus()
    expect(
      sameImageStatus(base, {
        ...base,
        image: { ...base.image, ready: false },
      }),
    ).toBe(false)
    expect(
      sameImageStatus(base, {
        ...base,
        image: { ...base.image, fixCommand: "runhaven images build codex" },
      }),
    ).toBe(false)
  })

  it("detects nullable builder field changes", () => {
    const base = imageStatus()
    expect(
      sameImageStatus(base, {
        ...base,
        builder: { ...base.builder, rosetta: null },
      }),
    ).toBe(false)
    expect(
      sameImageStatus(base, {
        ...base,
        builder: { ...base.builder, warning: "low disk" },
      }),
    ).toBe(false)
  })
})

describe("sameRunStatus", () => {
  it("treats an identical copy as unchanged", () => {
    expect(sameRunStatus(runStatus(), runStatus())).toBe(true)
  })

  it("detects run metadata changes", () => {
    const base = runStatus()
    expect(
      sameRunStatus(base, { ...base, run: { ...base.run, session: "other" } }),
    ).toBe(false)
  })

  it("detects container resource changes", () => {
    const base = runStatus()
    expect(
      sameRunStatus(base, {
        ...base,
        container: {
          ...base.container,
          resources: { ...base.container.resources, memoryBytes: 8 * 1024 ** 3 },
        },
      }),
    ).toBe(false)
  })

  it("detects network list length changes", () => {
    const base = runStatus()
    expect(
      sameRunStatus(base, {
        ...base,
        container: { ...base.container, networks: [] },
      }),
    ).toBe(false)
  })

  it("distinguishes a null ipv6 address from a value", () => {
    const base = runStatus()
    expect(
      sameRunStatus(base, {
        ...base,
        container: {
          ...base.container,
          networks: [{ ...base.container.networks[0]!, ipv6Address: "fd00::1" }],
        },
      }),
    ).toBe(false)
  })

  it("treats network reordering as a change", () => {
    const first = runStatus().container.networks[0]!
    const second = {
      ...first,
      network: "runhaven-internal",
      ipv4Address: "10.0.0.5/24",
    }
    const ordered = runStatus({
      container: { ...runStatus().container, networks: [first, second] },
    })
    const swapped = runStatus({
      container: { ...runStatus().container, networks: [second, first] },
    })
    expect(sameRunStatus(ordered, swapped)).toBe(false)
  })
})
