# Native GUI Apple HIG audit

Last reviewed: 2026-07-13

This is the current design contract for the shipped RunHaven desktop GUI. It
replaces the historical 2026-07-03 audit. Every GUI change must update the
relevant row below after checking the current official Apple Human Interface
Guidelines (HIG). `runhaven-design-spec.md` owns the canonical brand, visual,
voice, and component direction; this audit records what the current product
actually ships.

Status meanings:

- **Aligned**: source and focused automated checks match the cited guidance.
- **Fixed**: corrected in the 2026-07-13 pass.
- **Intentional exception**: RunHaven keeps a documented security or Tauri
  constraint instead of the default HIG pattern.
- **Open medium finding / user decision required**: source proves a current
  mismatch, but the required brand asset needs explicit user direction.
- **Live gap**: source checks are not enough. A packaged macOS check remains.

## Sources

Official Apple sources reviewed for this pass:

- [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos)
- [Windows](https://developer.apple.com/design/human-interface-guidelines/windows),
  [sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars),
  [menus](https://developer.apple.com/design/human-interface-guidelines/menus),
  [toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars),
  and [settings](https://developer.apple.com/design/human-interface-guidelines/settings)
- [Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons),
  [pop-up buttons](https://developer.apple.com/design/human-interface-guidelines/pop-up-buttons),
  [toggles](https://developer.apple.com/design/human-interface-guidelines/toggles),
  [text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields),
  [text views](https://developer.apple.com/design/human-interface-guidelines/text-views),
  [lists and tables](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables),
  [alerts](https://developer.apple.com/design/human-interface-guidelines/alerts),
  [feedback](https://developer.apple.com/design/human-interface-guidelines/feedback),
  [progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators),
  and [disclosure controls](https://developer.apple.com/design/human-interface-guidelines/disclosure-controls)
- [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility),
  [typography](https://developer.apple.com/design/human-interface-guidelines/typography),
  [color](https://developer.apple.com/design/human-interface-guidelines/color),
  [materials](https://developer.apple.com/design/human-interface-guidelines/materials),
  [motion](https://developer.apple.com/design/human-interface-guidelines/motion),
  and [focus and selection](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection)
- [Notifications](https://developer.apple.com/design/human-interface-guidelines/notifications),
  [offering help](https://developer.apple.com/design/human-interface-guidelines/offering-help),
  [generative AI](https://developer.apple.com/design/human-interface-guidelines/generative-ai),
  [privacy](https://developer.apple.com/design/human-interface-guidelines/privacy),
  [file management](https://developer.apple.com/design/human-interface-guidelines/file-management),
  and [app icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)

Nontechnical-user sources were used as supporting guidance, not as a
replacement for the HIG:

- [WowTo user-guide guidance](https://wowto.ai/blog/how-to-design-user-friendly-software-user-guides-for-non-technical-users)
- The user-supplied WebSiteMatics excerpt on minimal fields, logical flow,
  clear labels, visual error cues, responsive layout, large controls,
  whitespace, immediate validation, and progress indicators
- [LinkedIn collection on interfaces for nondevelopers](https://www.linkedin.com/top-content/user-experience/user-experience-for-non-technical-users/creating-user-friendly-interfaces-for-non-developers/)
- [Lightweight Solutions on digital literacy](https://lightweightsolutions.co/building-ui-ux-for-non-tech-savvy-users-designing-for-digital-literacy/)
- [IIC Lab on nontechnical users](https://iiclab.com/blogs/designing-tech-interfaces-for-non-tech-users/)
- [AlterSquare on domain-heavy users](https://altersquare.io/blog/designing-software-non-technical-domain-heavy-users)

The WebSiteMatics page itself was not retrievable during the audit. Do not
describe its full page as independently reviewed.

## Complete shipped-surface audit

| Surface | HIG topics checked | Current result |
| --- | --- | --- |
| Main window close, Dock reopen, and app exit | macOS, windows | **Fixed.** Close hides and preserves the existing WebView and live terminal channels. Dock reopen and custom menu commands show and focus it. Only app exit ends app-owned terminals and jobs. |
| Window frame and sidebar material | windows, sidebars, materials | **Intentional exception.** Tauri uses overlay chrome and a WebView toolbar with native traffic lights and sidebar vibrancy. The fallback is opaque. Native AppKit toolbar behavior remains preferable if the shell becomes native. |
| App menu | menus, settings | **Fixed.** Standard app, File, Edit, View, Session, Window, and Help order remains. Settings and Cmd-, were removed because RunHaven has no valid user preference that needs a Settings surface. |
| View menu text size | accessibility, typography, layout, menus | **Fixed.** Increase, Decrease, and Actual Text Size support 80 to 200 percent, persist as a bounded enum, and update terminal canvas text. Structural sidebar width and the identity mark use fixed pixel geometry instead of root-relative units, leaving at least 520 pixels for the main task at the 800-pixel minimum window and 200 percent text without truncating the primary New session label. |
| Dock icon | notifications | **Fixed.** The session-attention count was removed. Apple reserves a numeric app badge for unread notifications, not arbitrary problem counts. Attention remains visible in the app. |
| Sidebar destinations | sidebars, navigation, labels | **Aligned.** New session, History, and System are stable. Running terminals and Saved setups appear only when present. Selection has text and `aria-current`, and the sidebar has a named navigation landmark. |
| Running terminal rows | sidebars, status, accessibility | **Aligned.** Concurrent terminals have distinct assistant, project, time, and duplicate labels. Selecting another destination hides rather than unmounts a terminal. |
| Saved setups | sidebars, file management, feedback | **Fixed.** Missing folders show a plain problem and Choose folder again action. Repair preserves the assistant, replaces the folder, removes the stale pin, applies the replacement, and reports success. |
| New session, assistant step | buttons, pop-up buttons, labels, progressive disclosure | **Fixed.** The dead first-step Cancel was removed. One Continue action advances the task. Readiness and sign-in state stay beside the assistant choice. |
| New session, project step | file management, buttons, forms | **Aligned.** A native folder picker replaces path entry. The protected copy and provider-only network are easiest. Advanced exceptions remain disclosed in context. |
| Launch preparation and image install | progress indicators, feedback, buttons | **Fixed.** Long checks and mutations have a visible activity indicator and specific changing text. Button ellipses identify native confirmation or additional-input steps. |
| Final launch confirmation | alerts, privacy, generative AI | **Fixed and intentional exception.** A Rust-owned native confirmation names file and network access. It now identifies generative AI, possible errors, provider content sharing, and the unknown provider storage/training policy. The per-launch alert is retained because it is the trusted security boundary. |
| History | lists and tables, status, focus | **Fixed.** A semantic table names state and next action without color alone. Verified changes and an unreadable copy with a Check again recovery appear under Needs attention, but only verified changes are called ready. A clean copy stays under Other sessions with its normal status. Rows open with pointer or keyboard. Unsupported output and review actions are absent. |
| Session attention banner | alerts, feedback, offering help | **Fixed.** A failed session keeps the sanitized backend reason, separately states when output was not kept, opens same-page diagnostics, and offers Start new session. Verified changes say Changes ready to review. An unreadable copy says RunHaven could not check it and points to Check again. Clean copies have no alert. |
| Session message | text views, buttons, feedback | **Aligned.** Message is enabled only for an attached live session. Send uses a native confirmation, activity feedback, error alert, and short success status. |
| Session output | disclosure, privacy, feedback | **Fixed.** Output is offered only when process-memory output exists. The UI distinguishes a load error from the documented no-persistence policy. Show and Refresh expose their real action names to sighted and screen-reader users. |
| Session diagnostics and runtime status | disclosure, progress, alerts | **Fixed.** A session that needs attention opens issue details and loads its diagnostics once. Static details are not urgent live regions. Loading, error, recovery, and success states use appropriate status or alert semantics. |
| Worktree review | file management, destructive actions, alerts | **Fixed.** Rust separately reports whether a verified protected copy has changes, is clean, or could not be read. Changed filenames and conflicts appear before Apply or Discard. A clean copy uses neutral No changes to review text and a secondary Delete protected copy action. An unreadable copy offers Check again without claiming changes. Rust revalidates before and after native confirmation. |
| Terminal | accessibility, keyboards, typography, privacy, alerts | **Fixed.** xterm screen-reader mode is on, Shift-Escape leaves the terminal, Cmd-C and Cmd-V are handled, text follows the 80 to 200 percent scale, and the no-filter/no-store boundary is visible. End session uses a Rust-owned native confirmation and revalidates the exact live terminal before ending it. Cancel keeps the terminal open. |
| System | progressive disclosure, feedback, alerts | **Aligned.** Actionable setup problems lead. Technical checks, restart, and old-network cleanup are under Advanced system tools. Lengthy actions show motion plus specific status text. |
| Help | offering help, generative AI, privacy | **Fixed.** Help is task-based and plain. It explains that assistants use generative AI, can make mistakes, need human review, may send read content to providers, and may be stored or trained on according to provider account settings and terms. |
| Shared alerts | alerts, accessibility | **Fixed.** Static callouts have no assertive role. Async success and progress use `status`. Action failures and urgent recovery use `alert`. |
| Shared buttons | buttons, accessibility | **Fixed.** Labels use result verbs. Idle labels use an ellipsis when they open another view, picker, confirmation, or request more input. Busy labels keep progress ellipses. |
| Appearance and contrast | dark mode, color, materials, accessibility | **Aligned in automation.** RunHaven follows the system light or dark appearance, increased contrast, reduced transparency, and reduced motion. Information never relies on color alone. Missing-folder warning text uses one adaptive semantic text role in light and dark appearances, with component and axe coverage. |
| App icon and identity | app icons, branding | **Partially fixed; release review remains.** The user selected a distinct 1024 by 1024 unmasked identity and bundle-icon master at `docs/assets/app-icon.png`. The older transparent logo was removed; the README and single in-app shell lockup now use the selected identity. Tauri's generated desktop PNG, ICNS, and ICO assets live in `src-tauri/icons/`. Layered Icon Composer appearances, direct system-surface QA, and brand-rights clearance remain open release work. |

## Verification and remaining live gaps

Automated coverage includes typed frontend checks, alert-role regressions,
clean, changed, and unreadable protected-copy states, menu contracts,
main-window lifecycle tests, and Playwright at 800 by 560 with 200 percent text
on New session, History, System, and Session. The terminal has a focused test
proving its canvas font changes from 13 to 26 pixels at 200 percent.

The exact unsigned **RunHaven Debug** app built from this tree was also checked
in its packaged WKWebView. New session, History, Help, sidebar collapse and
restore, fullscreen, the View text-size commands through 200 percent, Cmd-N,
idle Cmd-W plus app activation restore, and idle Cmd-Q behaved as designed. A
fresh rebuild also proved that the four formerly false Ready-to-review sessions
now stay in ordinary Ended history and open to the neutral No changes to review
state with optional protected-copy deletion.

The deep design adversarial pass used the fail-closed GUI review preflight. Its
fresh `dev.runhaven.desktop.debug` process showed the 32-pixel unframed identity,
full navigation labels, and “Start with recommended settings.” The recorded
review fingerprint was `15daed25f82c5007bd3e950f4ff2e9ceb7e185a1e8e9eb43d098acb61ab39aa1`.

The exact unsigned **RunHaven 0.7.0** app also bundles the user-selected icon.
Its packaged ICNS matches the repository ICNS byte for byte and contains
all ten 1x and 2x renditions from 16 through 1024 pixels. macOS 26 NSWorkspace
applies one clean system mask, and light and dark background checks show no
clipping, fringe, hard square, or double mask. Direct Dock, Finder, Spotlight,
and About inspection remains release QA rather than verified evidence.

The following still require a packaged macOS review before release:

- VoiceOver speech order and rotor landmarks on every destination, including
  native confirmations and the terminal.
- Cmd-W, Dock reopen, and Cmd-Q while a real terminal is live.
- Active, inactive, fullscreen, increased-contrast, and reduced-transparency
  appearance for the custom overlay toolbar and traffic lights.
- The drag region and a live terminal at 200 percent in the packaged WKWebView.
- Direct Dock, Finder, Spotlight, and About rendering of the selected app icon.
- A task-based review with people who do not use Git, containers, or terminals.

## Required gate for every future GUI change

Before implementation, enumerate every changed window, destination, control,
state, and interaction. Check the current official HIG page for each item. Add
fail-first coverage for a behavior change, implement the smallest native or
existing-component solution, and update this audit. Document any security or
Tauri exception with its consequence. A macOS design specialist must review
the source and the live packaged result when rendering or native behavior
changes. Use `scripts/review-gui.sh` to build and identify that live target;
never reuse a pre-existing RunHaven process as review evidence.
