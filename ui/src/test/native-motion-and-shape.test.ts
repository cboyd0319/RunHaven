import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

// Guards the macOS-native polish for audit findings NAT-3 (button shape/press)
// and NAT-7 (reduced-motion). These are CSS/token contracts, so assert on the
// primitive source rather than rendered output.
const testDir = dirname(fileURLToPath(import.meta.url));
const read = (relativePath: string) =>
  readFileSync(join(testDir, relativePath), "utf8");

describe("NAT-3 button shape and press feedback", () => {
  const button = read("../components/ui/button.tsx");

  it("uses a macOS rounded-rect base, not a full pill", () => {
    expect(button).not.toContain("rounded-full");
    expect(button).toContain("rounded-md");
  });

  it("tints on press instead of physically nudging down", () => {
    expect(button).not.toContain("translate-y-px");
    expect(button).toContain("active:not-aria-[haspopup]:brightness-95");
  });

  it("keeps the focus ring intact", () => {
    expect(button).toContain("focus-visible:ring-ring/50");
  });
});

describe("NAT-7 native overlay motion", () => {
  it("adds a global prefers-reduced-motion guard that shortens motion", () => {
    const css = read("../index.css");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    // Collapses the tw-animate-css enter/exit utilities and every transition.
    expect(css).toMatch(/\.animate-in[\s\S]*animation-duration:\s*1ms\s*!important/);
    expect(css).toMatch(/transition-duration:\s*1ms\s*!important/);
  });

  it("uses ease-out timing on the overlay primitives", () => {
    for (const file of [
      "../components/ui/dialog.tsx",
      "../components/ui/sheet.tsx",
      "../components/ui/alert-dialog.tsx",
      "../components/ui/select.tsx",
    ]) {
      expect(read(file)).toContain("ease-out");
    }
    // The sheet's generic web easing is gone.
    expect(read("../components/ui/sheet.tsx")).not.toContain("ease-in-out");
  });
});
