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
- **Fixed**: corrected in the current tree with focused coverage.
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
  [VoiceOver](https://developer.apple.com/design/human-interface-guidelines/voiceover),
  [typography](https://developer.apple.com/design/human-interface-guidelines/typography),
  [layout](https://developer.apple.com/design/human-interface-guidelines/layout),
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
| Window frame and sidebar material | windows, sidebars, materials | **Intentional exception.** Tauri supplies native traffic lights and its native sidebar effect. Main content and the custom WebView header remain opaque. RunHaven does not claim that header as a native Liquid Glass toolbar. The fallback is opaque, and a narrow AppKit toolbar bridge remains preferable if evidence justifies it. |
| App menu | menus, settings | **Fixed.** Standard app, File, Edit, View, Session, Window, and Help order remains. Settings and Cmd-, were removed because RunHaven has no valid user preference that needs a Settings surface. |
| View menu text size | accessibility, typography, layout, menus | **Fixed.** Increase, Decrease, and Actual Text Size support 80 to 200 percent, persist as a bounded enum, and update terminal canvas text. Structural sidebar width and the identity mark use fixed pixel geometry instead of root-relative units, leaving at least 520 pixels for the main task at the 800-pixel minimum window and 200 percent text without truncating the primary New session label. |
| Dock icon | notifications | **Fixed.** The session-attention count was removed. Apple reserves a numeric app badge for unread notifications, not arbitrary problem counts. Attention remains visible in the app. |
| Sidebar destinations | sidebars, navigation, labels | **Aligned.** New session, History, and System are stable. Running terminals and Saved setups appear only when present. Selection has text and `aria-current`, and the sidebar has a named navigation landmark. |
| Running terminal rows | sidebars, status, accessibility | **Aligned.** Concurrent terminals have distinct assistant, project, time, and duplicate labels. A row selects that exact live state in the terminal workspace. Selecting another destination hides rather than unmounts it. |
| Saved setups | sidebars, file management, feedback | **Fixed.** Missing folders show a plain problem and Choose folder again action. Repair preserves the assistant, replaces the folder, removes the stale pin, applies the replacement, and reports success. |
| New session sequence | layout, windows, progress indicators, focus, accessibility | **Fixed.** New session is the empty and preflight state of one terminal workspace. A semantic ordered list presents Choose assistant, Choose project, and Review and start. Wide ready windows center a compact progress rail beside the current grouped task. Narrow windows and 200-percent text stack the same DOM order without overflow. Primary task text uses 15-point-equivalent body, 13-point support, and 17-point headings. Explicit step changes focus the task; background refresh does not. |
| New session, assistant step | buttons, pop-up buttons, labels, progressive disclosure | **Fixed.** One Continue action advances the task. Readiness and sign-in remain beside the assistant choice, while ordinary ready state is quieter than the primary action. |
| New session, project step | file management, buttons, forms | **Aligned.** A native folder picker replaces path entry. The protected copy and provider-only network are easiest. Advanced exceptions remain disclosed in context. Continue opens Review and never plans or starts a run. |
| New session, review step | layout, privacy, buttons, accessibility | **Fixed.** A typed-request summary names assistant, project display name, project access, network, sign-in sharing, and optional Mac API-key access without exposing the host path. Start session is the only action that plans a run. Canceling native confirmation returns to the focused Review step with choices intact. Save setup is secondary and explains that advanced choices are not stored. |
| Launch preparation and image install | progress indicators, feedback, buttons | **Fixed.** Long checks and mutations have a visible activity indicator and specific changing text. Button ellipses identify native confirmation or additional-input steps. |
| Final launch confirmation | alerts, privacy, generative AI | **Fixed and intentional exception.** WebView Review is explanatory; the Rust-owned native confirmation remains authoritative and names file and network access. Its generative-AI review instruction follows effective access, so a read-only session does not claim that the assistant can change files. The per-launch alert remains the trusted security boundary. |
| History | lists and tables, status, focus | **Fixed.** A semantic table names state and next action without color alone. Verified changes and an unreadable copy with a Check again recovery appear under Needs attention, but only verified changes are called ready. A clean copy stays under Other sessions with its normal status. Rows open with pointer or keyboard. Unsupported output and review actions are absent. |
| Session attention banner | alerts, feedback, offering help | **Fixed.** A failed session keeps the sanitized backend reason, separately states when output was not kept, opens same-page diagnostics, and offers Start new session. Verified changes say Changes ready to review. An unreadable copy says RunHaven could not check it and points to Check again. Clean copies have no alert. |
| Session message | text views, buttons, feedback | **Aligned.** Message is enabled only for an attached live session. Send uses a native confirmation, activity feedback, error alert, and short success status. |
| Session output | disclosure, privacy, feedback | **Fixed.** Output is offered only when process-memory output exists. The UI distinguishes a load error from the documented no-persistence policy. Show and Refresh expose their real action names to sighted and screen-reader users. |
| Session diagnostics and runtime status | disclosure, progress, alerts | **Fixed.** A session that needs attention opens issue details and loads its diagnostics once. Static details are not urgent live regions. Loading, error, recovery, and success states use appropriate status or alert semantics. |
| Worktree review | file management, destructive actions, alerts | **Fixed.** Rust separately reports whether a verified protected copy has changes, is clean, or could not be read. Changed filenames and conflicts appear before Apply or Discard. A clean copy uses neutral No changes to review text and a secondary Delete protected copy action. An unreadable copy offers Check again without claiming changes. Rust revalidates before and after native confirmation. |
| Terminal workspace | windows, navigation, accessibility, keyboards, typography, privacy, alerts | **Fixed.** Confirmed setup becomes the live terminal in the same content pane with no intermediate page or click. The header uses Rust-owned assistant and project identity instead of a generic terminal title. xterm screen-reader mode is on, Shift-Escape leaves it, text scales from 80 to 200 percent, and the privacy boundary is visible. End session revalidates the exact terminal through native confirmation. Cancel keeps it open. |
| System | progressive disclosure, feedback, alerts | **Aligned.** Actionable setup problems lead. Technical checks, restart, and old-network cleanup are under Advanced system tools. Lengthy actions show motion plus specific status text. |
| Help | offering help, generative AI, privacy | **Fixed.** Help is task-based and plain. It explains that assistants use generative AI, can make mistakes, need human review, may send read content to providers, and may be stored or trained on according to provider account settings and terms. |
| Get Support | offering help, alerts, privacy, file management | **Source and functional smoke aligned; packaged design review remains open.** Help and the Help menu share one action. Creating, cancellation, success, browser-only recovery, and independent Finder/GitHub failures have distinct feedback. A Rust-owned native confirmation explains the private text file and public attachment boundary before collection. The report stays allowlisted and local until the person attaches it. Automated and packaged no-upload smokes passed; the current native confirmation, partial failures, and VoiceOver announcement order still need a packaged design walkthrough. |
| Shared alerts | alerts, accessibility | **Fixed.** Static callouts have no assertive role. Async success and progress use `status`. Action failures and urgent recovery use `alert`. |
| Shared buttons | buttons, accessibility | **Fixed.** Labels use result verbs. Idle labels use an ellipsis when they open another view, picker, confirmation, or request more input. Busy labels keep progress ellipses. |
| Appearance and contrast | dark mode, color, materials, accessibility | **Fixed.** RunHaven follows system appearance. The dark ladder keeps the navy visibly blue and separates the sidebar, opaque work surface, grouped content, and controls without near-black planes. Reduced-transparency fallbacks use the same raised hierarchy. Semantic text and destructive actions pass the light and dark axe matrix. Exact packaged review confirmed the lighter hierarchy at actual size and 200-percent text. |
| App icon and identity | app icons, branding | **Fixed in the current flattened pipeline.** The user selected the brighter cube-over-three-layers artwork. Its 1254-pixel source was normalized once into the 1024 by 1024 unmasked, opaque sRGBA master at `docs/assets/app-icon.png`; the pinned Tauri 2.11.5 tool regenerated every tracked bundle rendition and the 256-pixel WebView lockup. Source and exact packaged reviews found no blocker, high, or medium defect. The silhouette plus teal base remain recognizable at small sizes, and live Finder rendering applies one clean system mask. Layered Icon Composer appearances, remaining system-surface QA, and brand-rights clearance remain open. |

## Verification and remaining live gaps

Automated coverage includes the three-step and same-pane terminal workspace contracts, typed frontend
checks, alert-role regressions,
clean, changed, and unreadable protected-copy states, menu contracts,
main-window lifecycle tests, and Playwright at 800 by 560 with 200 percent text
on New session, History, System, and Session. Review has focused responsive source,
unit, default-size Playwright, and packaged 200-percent coverage. The terminal has a focused test
proving its canvas font changes from 13 to 26 pixels at 200 percent.

Before the three-step New session slice, an exact unsigned **RunHaven Debug** app
was checked in its packaged WKWebView. New session, History, Help, sidebar collapse and
restore, fullscreen, the View text-size commands through 200 percent, Cmd-N,
idle Cmd-W plus app activation restore, and idle Cmd-Q behaved as designed. A
fresh rebuild also proved that the four formerly false Ready-to-review sessions
now stay in ordinary Ended history and open to the neutral No changes to review
state with optional protected-copy deletion. This historical evidence does not
prove the current New session rendering.

The deep design adversarial pass used the fail-closed GUI review preflight. Its
fresh `dev.runhaven.desktop.debug` process showed the 32-pixel unframed identity,
full navigation labels, and “Start with recommended settings.” The recorded
review fingerprint was `15daed25f82c5007bd3e950f4ff2e9ceb7e185a1e8e9eb43d098acb61ab39aa1`.

The three-step slice received source review and two exact packaged reviews. The
final target was **RunHaven Debug**, build
`19abf6e8-dirty-20260713T233412Z`, executable SHA-256
`6465d59c587e1fcedc87573b2e44d0a0d238f743c9787921f7eb7d975e2ed2c5`, and
review fingerprint
`2de7e9191704ca85de8fb764c3ed26bacd295da4cc76825eb4a53eed44c42176`.
Live review verified the three-step hierarchy, one current grouped task, quiet
readiness, path-free summaries, Edit and Back continuity, dark appearance, and
no visible horizontal overflow. A read-only Codex confirmation named read-only
folder access and said “Review its output.” without file-change wording. Cancel
started no session, retained every choice, and returned accessibility focus to
the Review and start step. No blocker, high, medium, or low finding remained.
The current terminal-workspace and appearance package was **RunHaven Debug**,
build `19abf6e8-dirty-20260714T004923Z`, executable SHA-256
`81b93e0cdaa388d9d385cc7ce039e75b26068f025126a436edcea408624afba6`, and
review fingerprint
`cd08fd51e7fdccbc801f16998b57112cc8bb0a406aa1c0c1307b9a01264e97af`.
Live review verified the raised navy hierarchy, distinct sidebar and grouped
surfaces, actual-size setup and Review compositions, and semantic contrast. At
200 percent, the 15/13/17 roles visibly doubled, the layout stacked, actions
remained reachable by vertical scrolling, and no horizontal overflow appeared.
Accessibility inspection exposed named navigation, heading hierarchy, progress,
controls, typed Review rows, path-free assistant/project identity, explicit-step
focus, and forward Tab order. The reviewer did not cross native confirmation to
start an assistant, so the same-pane terminal transition, backend terminal
identity, and Running-row keyboard handoff remain source- and Playwright-backed.
Exact 800 by 560 live resizing also remains automation-backed. No blocker,
high, or medium finding remained. Older fingerprints above are historical.

The exact unsigned **RunHaven Debug 0.7.0** app, build
`19abf6e8-dirty-20260714T011936Z`, packaged icon SHA-256
`a8737dd060c1fb4513e0ead3999641e5af38d87dd6a3b174f1b96ac8c5baa6c7`, and
review fingerprint
`8fef34bd677d33df202adc9b36c2b50463758c9717f0bfbe9fbb5909369e6b33`
passed live icon review. Its packaged ICNS matches the repository ICNS byte for
byte and contains all ten 1x and 2x renditions from 16 through 1024 pixels.
The 32-pixel dark-sidebar lockup, dark Finder list and icon views, Finder Get
Info header, and large native preview keep the cube, all three layers, and teal
base distinct. Finder applies one clean system mask with no hard square, double
rounding, crop, fringe, or external halo. Dock was auto-hidden, Spotlight could
not be inspected after the UI service returned error `-10005`, the app has no
About item, and changing system light appearance was outside the nonmutating
review. Those surfaces remain explicit release QA.

The following still require a packaged macOS review before release:

- VoiceOver speech order and rotor landmarks on every destination, including
  native confirmations and the terminal.
- Cmd-W, Dock reopen, and Cmd-Q while a real terminal is live.
- Active, inactive, fullscreen, increased-contrast, and reduced-transparency
  appearance for the custom overlay toolbar and traffic lights.
- The drag region and a live terminal at 200 percent in the packaged WKWebView.
- Get Support's native confirmation, partial Finder/GitHub failures, and VoiceOver announcement order in the current packaged WKWebView. The packaged no-upload smoke proves the functional boundary, not the full design walkthrough.
- Direct Dock, Spotlight, About, and system light-appearance rendering of the selected app icon.
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
