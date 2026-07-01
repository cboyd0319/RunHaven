# Native GUI design resources

Reference material for the planned Apple-HIG-aligned visual redesign of the
RunHaven desktop app. The current GUI work is a functional prototype with
targeted clarity fixes; the comprehensive native-macOS visual pass will draw on
the sources below.

Scope note: this is the deferred "update all design" pass. Keep the existing
typed Tauri/Rust boundary and the security model. A visual redesign changes
look and copy, not the trust boundary.

## Design direction (current, from current-state.md)

Dark by default, logo-informed, restrained, native feeling. Dark navy base,
subtle glass-like panels, blue primary actions, teal/green ready states, amber
warnings, red only for blocked or destructive states. No corny safety copy.
Canonical logo: `docs/assets/logo.png`.

## Apple references (public)

- Human Interface Guidelines:
  https://developer.apple.com/design/human-interface-guidelines/
- Design principles:
  https://developer.apple.com/design/human-interface-guidelines/design-principles
- Designing for macOS:
  https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- SF Symbols:
  https://developer.apple.com/sf-symbols/
- Icon Composer:
  https://developer.apple.com/icon-composer/

## Local design tools and assets (maintainer)

The maintainer has these installed or downloaded locally. Machine-local paths
are deliberately not committed here (`scripts/check-docs-local-paths.sh` guards
against local-path references); the exact paths are recorded in AgentMemory for
session continuity.

- SF Symbols app (installed) for the icon set that pairs with macOS.
- Icon Composer (installed) for the app icon.
- Apple macOS 27 UI Kit (Sketch file, downloaded locally). Before citing
  specific components in repo docs, import or summarize the relevant pieces into
  `docs/` first, then cite the repo copy, per the repo's no-local-path rule.

## How to use these in the redesign pass

- Treat the HIG and "Designing for macOS" as the primary bar for spacing,
  typography, control choice, and native behavior.
- Prefer native or SF-Symbols-aligned iconography over bespoke icons.
- Keep the RunHaven design direction (dark navy, blue primary, restrained) as
  the palette, and use the HIG for layout, control, and interaction patterns.
- Any component adapted from the macOS UI Kit should be summarized into `docs/`
  (not linked by local path) before it drives repo docs or code comments.
