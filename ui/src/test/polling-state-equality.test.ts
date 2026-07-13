import { describe, expect, it } from "vitest"

import { sameManagedSessionList } from "../app/sessionList"
import { sameLaunchStatus } from "../app/useLaunchStatus"
import { sameManagedSessionDiagnostics } from "../app/useManagedSessionDiagnostics"
import { sameManagedSessionStatus } from "../app/useManagedSessionStatus"
import type {
  LaunchStatusResponse,
  ManagedSessionDiagnosticsResponse,
  ManagedSessionStatusResponse,
  ManagedSessionSummary,
} from "../commands/runhaven"

function session(patch: Partial<ManagedSessionSummary> = {}): ManagedSessionSummary {
  return {
    id: "run-123",
    runId: "run-123",
    agent: "codex",
    session: "project",
    title: "Codex - project",
    project: "project",
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready.",
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [
      {
        source: "run",
        label: "Run record",
        detail: "Running",
        timestamp: "2026-07-01T02:00:00Z",
      },
    ],
    ...patch,
  }
}

function sessionStatus(
  patch: Partial<ManagedSessionStatusResponse> = {},
): ManagedSessionStatusResponse {
  return {
    session: session(),
    container: {
      state: "running",
      image: "runhaven/codex:0.7.0",
      startedAt: "2026-07-01T02:00:03Z",
      resources: {
        cpus: "4",
        memoryBytes: 4 * 1024 ** 3,
      },
      networks: [
        {
          network: "runhaven-provider-preview",
          hostname: "runhaven-codex-project",
          ipv4Address: "192.168.64.20/24",
          ipv4Gateway: "192.168.64.1",
          ipv6Address: null,
        },
      ],
    },
    statusUnavailable: null,
    recovery: null,
    ...patch,
  }
}

function sessionDiagnostics(
  patch: Partial<ManagedSessionDiagnosticsResponse> = {},
): ManagedSessionDiagnosticsResponse {
  return {
    session: session(),
    authStatus: {
      name: "codex",
      status: "api-key-broker",
    },
    authLogEntries: [
      {
        timestamp: "2026-07-01T03:00:01Z",
        profile: "codex",
        broker: "codex-api-key",
        decision: "allowed",
        method: "POST",
        path: "/v1/responses",
        upstreamStatus: 200,
        count: 1,
        reason: "ok",
        runId: "run-123",
      },
    ],
    egressLogEntries: [
      {
        timestamp: "2026-07-01T03:00:02Z",
        profile: "codex",
        decision: "allowed",
        host: "api.openai.com",
        port: 443,
        count: 1,
        reason: "provider-host",
        matchedRule: "api.openai.com",
        runId: "run-123",
      },
    ],
    ...patch,
  }
}

describe("polling state equality", () => {
  it("suppresses no-op managed-session list updates", () => {
    const current = { sessions: [session()] }

    expect(sameManagedSessionList(current, { sessions: [session()] })).toBe(true)
    expect(
      sameManagedSessionList(current, {
        sessions: [
          session({
            input: {
              attached: false,
              status: "detached",
              detail: "Input is not attached for this session.",
            },
          }),
        ],
      }),
    ).toBe(false)
  })

  it("suppresses no-op launch status updates", () => {
    const current: LaunchStatusResponse = {
      runId: "launch-card-run",
      state: "starting",
      label: "Starting",
      detail: "Waiting for the managed session.",
      updatedAt: "2026-07-01T00:00:00Z",
    }

    expect(sameLaunchStatus(current, { ...current })).toBe(true)
    expect(
      sameLaunchStatus(current, {
        ...current,
        updatedAt: "2026-07-01T00:00:01Z",
      }),
    ).toBe(false)
  })

  it("suppresses no-op selected-session status updates", () => {
    const current = sessionStatus({
      container: null,
      statusUnavailable: "Container status is unavailable. The run may have stopped.",
      recovery: {
        status: "attention",
        title: "Runtime status unavailable",
        detail:
          "RunHaven can still see this session record, but Apple container status is unavailable.",
        remedy: "Refresh sessions or open diagnostics before taking action.",
        repairAvailable: false,
      },
    })

    expect(sameManagedSessionStatus(current, { ...current })).toBe(true)
    expect(
      sameManagedSessionStatus(current, {
        ...current,
        recovery: {
          ...current.recovery!,
          remedy: "Open diagnostics before taking action.",
        },
      }),
    ).toBe(false)
  })

  it("suppresses no-op session diagnostics updates", () => {
    const current = sessionDiagnostics()

    expect(sameManagedSessionDiagnostics(current, sessionDiagnostics())).toBe(true)
    expect(
      sameManagedSessionDiagnostics(current, {
        ...sessionDiagnostics(),
        authLogEntries: [
          {
            ...current.authLogEntries[0]!,
            count: 2,
          },
        ],
      }),
    ).toBe(false)
  })
})
