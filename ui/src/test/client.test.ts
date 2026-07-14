// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// The invoke choke point and the dev-log sinks are the only Tauri surfaces the
// `call` helper touches. Mock the sinks so we can assert which level a traced
// invoke lands on, and the folder-picker plugin so importing client.ts is inert.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-log", () => ({
  debug: vi.fn(() => Promise.resolve()),
  error: vi.fn(() => Promise.resolve()),
  info: vi.fn(() => Promise.resolve()),
  trace: vi.fn(() => Promise.resolve()),
  warn: vi.fn(() => Promise.resolve())
}));

const { invoke } = await import("@tauri-apps/api/core");
const log = await import("@tauri-apps/plugin-log");
const { call, createSupportReport, performSetupRecovery } = await import("../commands/client");

const invokeMock = vi.mocked(invoke);

type TauriWindow = { __TAURI_INTERNALS__?: unknown };

// Drive the two performance.now() reads inside `call` (start, then end) so the
// measured duration is deterministic instead of wall-clock.
function stubDuration(durationMs: number): void {
  vi.spyOn(performance, "now").mockReturnValueOnce(0).mockReturnValueOnce(durationMs);
}

beforeEach(() => {
  (window as unknown as TauriWindow).__TAURI_INTERNALS__ = {};
  vi.clearAllMocks();
});

afterEach(() => {
  delete (window as unknown as TauriWindow).__TAURI_INTERNALS__;
  vi.restoreAllMocks();
});

describe("call invoke tracing", () => {
  it("creates support through one argument-free Rust-owned command", async () => {
    invokeMock.mockResolvedValueOnce({
      fileName: "RunHaven-0.7.0-support.txt",
      issueOpened: true,
      finderRevealed: true
    });

    await createSupportReport();

    expect(invokeMock).toHaveBeenCalledWith("create_support_report", {});
  });

  it("sends only the opaque setup recovery action id", async () => {
    invokeMock.mockResolvedValueOnce({
      actionId: "start-container-system",
      status: "succeeded",
      label: "Container service started",
      detail: "RunHaven started the local container service."
    });

    await performSetupRecovery("start-container-system");

    expect(invokeMock).toHaveBeenCalledWith("perform_setup_recovery", {
      request: { actionId: "start-container-system" }
    });
    expect(JSON.stringify(invokeMock.mock.calls[0]?.[1])).not.toContain("args");
    expect(JSON.stringify(invokeMock.mock.calls[0]?.[1])).not.toContain("executable");
  });

  it("keeps the Apple installer target entirely behind the Rust action id", async () => {
    invokeMock.mockResolvedValueOnce({
      actionId: "open-apple-container-installer",
      status: "succeeded",
      label: "Apple installer download opened",
      detail: "Your browser is downloading Apple's installer."
    });

    await performSetupRecovery("open-apple-container-installer");

    expect(invokeMock).toHaveBeenCalledWith("perform_setup_recovery", {
      request: { actionId: "open-apple-container-installer" }
    });
    const payload = JSON.stringify(invokeMock.mock.calls[0]?.[1]);
    expect(payload).not.toContain("github.com");
    expect(payload).not.toContain("/usr/bin/open");
    expect(payload).not.toContain(".pkg");
  });

  it("traces a slow successful invoke at warn so it is greppable", async () => {
    invokeMock.mockResolvedValueOnce("result");
    stubDuration(1500);

    const result = await call<string>("plan_run", {}, () => "fallback");

    expect(result).toBe("result");
    expect(log.warn).toHaveBeenCalledTimes(1);
    expect(log.warn).toHaveBeenCalledWith(
      expect.stringContaining("invoke plan_run: SLOW ok in 1500.0ms")
    );
    expect(log.debug).not.toHaveBeenCalled();
    expect(log.error).not.toHaveBeenCalled();
  });

  it("traces a fast successful invoke at debug", async () => {
    invokeMock.mockResolvedValueOnce("result");
    stubDuration(500);

    const result = await call<string>("plan_run", {}, () => "fallback");

    expect(result).toBe("result");
    expect(log.debug).toHaveBeenCalledTimes(1);
    expect(log.debug).toHaveBeenCalledWith(
      expect.stringContaining("invoke plan_run: ok in 500.0ms")
    );
    expect(log.warn).not.toHaveBeenCalled();
    expect(log.error).not.toHaveBeenCalled();
  });

  it("keeps the 1000ms threshold strict: exactly at the bound stays debug", async () => {
    invokeMock.mockResolvedValueOnce("result");
    stubDuration(1000);

    await call<string>("plan_run", {}, () => "fallback");

    expect(log.debug).toHaveBeenCalledWith(
      expect.stringContaining("invoke plan_run: ok in 1000.0ms")
    );
    expect(log.warn).not.toHaveBeenCalled();
  });

  it("leaves the failure path at error even when the invoke was slow", async () => {
    invokeMock.mockRejectedValueOnce(new Error("boom"));
    stubDuration(2000);

    await expect(call<string>("plan_run", {}, () => "fallback")).rejects.toThrow("boom");

    expect(log.error).toHaveBeenCalledTimes(1);
    expect(log.error).toHaveBeenCalledWith(
      expect.stringContaining("invoke plan_run: failed in 2000.0ms")
    );
    expect(log.warn).not.toHaveBeenCalled();
    expect(log.debug).not.toHaveBeenCalled();
  });
});
