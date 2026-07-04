import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

// The diagnostic log forwards every console call into the local log file
// (ui/src/commands/logging.ts), and docs/DESKTOP_APP.md promises that file
// never records tokens or other secrets. That promise is only real if app
// code cannot casually console-log objects that might carry sensitive values.
// This guard keeps production source console-free: logging.ts is the single
// reviewed place allowed to touch the console (it wraps it), and everything
// else must let errors flow through the typed command layer or React, both of
// which log bounded, payload-free messages.
const CONSOLE_ALLOWED_FILES = new Set(["commands/logging.ts"]);

type SourceEntry = { path: string; source: string };

function sourceRoot(): string {
  return fileURLToPath(new URL("../", import.meta.url));
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

function consoleCallLines(source: string): number[] {
  const lines: number[] = [];
  source.split("\n").forEach((line, index) => {
    const trimmed = line.trimStart();
    // Skip comments; catch real calls like console.log(, console.error(.
    if (trimmed.startsWith("//") || trimmed.startsWith("*")) {
      return;
    }
    if (/\bconsole\.\w+\(/.test(line)) {
      lines.push(index + 1);
    }
  });
  return lines;
}

describe("no-console source guard", () => {
  it("keeps production source console-free outside the reviewed logging module", () => {
    const offenders: string[] = [];
    for (const entry of productionSourceEntries()) {
      if (CONSOLE_ALLOWED_FILES.has(entry.path)) {
        continue;
      }
      for (const line of consoleCallLines(entry.source)) {
        offenders.push(`${entry.path}:${line}`);
      }
    }
    expect(
      offenders,
      "direct console calls are forwarded verbatim into the local log file; " +
        "route errors through the typed command layer instead, or extend the " +
        "reviewed logging module"
    ).toEqual([]);
  });
});
