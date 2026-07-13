import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const FORBIDDEN_HOST_MARKERS = [
  "app_server",
  "filesystem_rpc",
  "filesystem",
  "mcp",
  "shell",
  "exec_server",
  "command_exec",
  "execute_command",
  "attach_session",
  "resize_session",
  "codex",
  "login",
  "logout",
  "set_auth",
  "update_auth",
  "write_auth",
  "clear_auth",
  "delete_auth"
];
const EXPECTED_TAURI_MODULES = [
  "@tauri-apps/api/core",
  "@tauri-apps/api/event",
  "@tauri-apps/plugin-dialog",
  "@tauri-apps/plugin-log"
];
// The reviewed Tauri touchpoints: typed invoke plus the folder picker in
// client.ts, the write-only dev log forwarding in logging.ts (log plugin's
// single `log` command; name/duration/outcome only, never payloads or the
// terminal stream), the listen-only menu event subscription in menu_events.ts,
// and the terminal Channel in terminal.ts. terminal.ts routes through
// client.ts's `call` helper, so `invoke` itself stays centralized in client.ts.
const EXPECTED_TAURI_FILES = [
  "commands/client.ts",
  "commands/logging.ts",
  "commands/menu_events.ts",
  "commands/terminal.ts"
];
const EXPECTED_INVOKED_COMMANDS = [
  "get_dashboard_status",
  "perform_setup_recovery",
  "get_image_status",
  "rebuild_agent_image",
  "get_image_build_status",
  "get_network_prune_preview",
  "prune_managed_networks",
  "get_run_status",
  "get_log_snapshot",
  "get_managed_session_log_snapshot",
  "get_managed_sessions",
  "get_managed_session_status",
  "get_managed_session_diagnostics",
  "plan_run",
  "launch_run",
  "get_launch_status",
  "stop_run",
  "stop_managed_session",
  "kill_run",
  "kill_managed_session",
  "repair_run",
  "repair_managed_session",
  "cleanup_managed_session",
  "get_managed_session_worktree",
  "keep_managed_session_worktree",
  "merge_managed_session_worktree",
  "recover_managed_session_worktree",
  "discard_managed_session_worktree",
  "write_session_input",
  "get_egress_log",
  "get_auth_log",
  "get_auth_status",
  "get_auth_explain",
  "get_why_network",
  "get_why_state",
  "get_why_host",
  "get_why_workspace",
  "set_attention_badge",
  "sync_menu_state",
  "open_session_window",
  "get_session_window_target",
  "open_terminal_session",
  "open_agent_sign_in_terminal",
  "open_agent_run_terminal",
  "write_terminal_input",
  "resize_terminal",
  "close_terminal_session"
];

type SourceEntry = {
  path: string;
  source: string;
};

function normalize(value: string): string {
  return value.toLowerCase().replaceAll("-", "_");
}

function sourceRoot(): string {
  return fileURLToPath(new URL("../", import.meta.url));
}

function readCommandClientSource(): string {
  return readFileSync(
    fileURLToPath(new URL("../commands/client.ts", import.meta.url)),
    "utf8"
  );
}

// The full WebView command surface: client.ts owns the shared `call`/`invoke`
// helper and most wrappers; terminal.ts and worktree.ts own
// their domain wrappers, which route through the same `call`. Auditing all
// four keeps every host-reaching command in the allow-list even though the
// surface is split by ownership.
function readCommandSurfaceSource(): string {
  return [
    readCommandClientSource(),
    readFileSync(
      fileURLToPath(new URL("../commands/terminal.ts", import.meta.url)),
      "utf8"
    ),
    readFileSync(
      fileURLToPath(new URL("../commands/worktree.ts", import.meta.url)),
      "utf8"
    )
  ].join("\n");
}

function productionSourceEntries(root = sourceRoot()): SourceEntry[] {
  const entries: SourceEntry[] = [];

  function visit(directory: string): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (entry.name !== "test") {
          visit(join(directory, entry.name));
        }
        continue;
      }
      if (!entry.isFile() || !/\.(ts|tsx)$/.test(entry.name)) {
        continue;
      }
      const path = join(directory, entry.name);
      entries.push({
        path: relative(root, path).split(sep).join("/"),
        source: readFileSync(path, "utf8")
      });
    }
  }

  visit(root);
  return entries.sort((left, right) => left.path.localeCompare(right.path));
}

function tauriModuleReferences(source: string): string[] {
  return [...source.matchAll(/@tauri-apps\/[A-Za-z0-9_@./-]+/g)].map((match) => match[0]);
}

function invokeCallCount(source: string): number {
  return source.match(/\binvoke(?:<[^>]+>)?\s*\(/g)?.length ?? 0;
}

function commandCallSiteCount(source: string): number {
  return Math.max(0, (source.match(/\bcall(?:<[^>]+>)?\s*\(/g)?.length ?? 0) - 1);
}

function invokedCommands(source: string): string[] {
  return [...source.matchAll(/call\(\s*"([^"]+)"/g)].map((match) => match[1]);
}

describe("frontend command surface guard", () => {
  it("keeps the WebView on typed invoke plus the reviewed folder picker", () => {
    const entries = productionSourceEntries();
    const source = readCommandClientSource();
    const tauriFiles = entries
      .filter((entry) => tauriModuleReferences(entry.source).length > 0)
      .map((entry) => entry.path);
    const tauriModules = entries.flatMap((entry) => tauriModuleReferences(entry.source));
    const invokeFiles = entries
      .filter((entry) => invokeCallCount(entry.source) > 0)
      .map((entry) => entry.path);

    expect(tauriFiles).toEqual(EXPECTED_TAURI_FILES);
    expect([...new Set(tauriModules)].sort()).toEqual([...EXPECTED_TAURI_MODULES].sort());
    expect(invokeFiles).toEqual(["commands/client.ts"]);
    expect(invokeCallCount(source)).toBe(1);
  });

  it("fails if the WebView command map grows forbidden host-reaching commands", () => {
    const source = readCommandSurfaceSource();
    const commands = invokedCommands(source);

    expect(commands).toHaveLength(commandCallSiteCount(source));
    expect([...commands].sort()).toEqual([...EXPECTED_INVOKED_COMMANDS].sort());

    for (const command of commands) {
      const normalized = normalize(command);
      for (const marker of FORBIDDEN_HOST_MARKERS) {
        expect(normalized).not.toContain(marker);
      }
    }
  });
});
