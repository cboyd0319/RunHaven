# macOS 27 UI Kit extraction notes

Design-system facts extracted from the official Apple macOS UI kit for the
RunHaven desktop redesign. Source: Apple Design Resources - macOS UI Kit
(Sketch), https://developer.apple.com/design/resources/ ("macOS 27" kit
cover). All values below were read directly from the kit file's layer and
style data. Each section names the kit page or artboard the values came from.
Values are points at 1x; treat them as CSS px.

Two limits of the source file, verified by scanning every page and shared
style: the kit contains no blur radii at all (materials are flat translucent
fills; Sketch background-blur parameters are absent), and window, alert, and
menu-container corner rounding is not stored as geometry (those masters are
square rectangles with styles). Encoded radii anchors are listed in section 3.
The kit also ships no badge or status-dot component; see section 5.

## 1. Color tokens (dark palette)

From the document color library (kit page "Colors") plus the named layer
styles on "Windows", "Sidebars", "Materials", "Menus", and "Titlebars and
Toolbars".

| Token | Kit name | Value |
| --- | --- | --- |
| Window background | Window Backgrounds/Dark Background | `#1E1E1E` |
| Sidebar material (active window) | Sidebar/Background/Dark/Active | `rgba(12,12,12,0.85)` |
| Sidebar material (inactive window) | Sidebar/Background/Dark/Inactive | `rgba(46,46,46,0.85)` |
| Grouped content container | Fills/Dark/4 Quatenary (Group Boxes/Dark, Forms) | `rgba(255,255,255,0.03)` over window bg |
| Label primary | Labels/Dark/1 Primary | `rgba(255,255,255,1.0)` |
| Label secondary | Labels/Dark/2 Secondary | `rgba(255,255,255,0.55)` |
| Label tertiary | Labels/Dark/3 Tertiary | `rgba(255,255,255,0.25)` |
| Label quaternary | Labels/Dark/4 Quaternary | `rgba(255,255,255,0.10)` |
| Vibrant label primary (on sidebar) | Labels/Dark Vibrant/1 Primary | `rgba(255,255,255,0.96)` |
| Vibrant label secondary | Labels/Dark Vibrant/2 Secondary | `#8A8A8A` (plus-lighter blend) |
| Vibrant label tertiary | Labels/Dark Vibrant/3 Tertiary | `#4D4D4D` (plus-lighter blend) |
| Fill primary .. quinary | Fills/Dark/1-5 | white at 10% / 8% / 5% / 3% / 2% |
| Separator (menu) | Menus/Dark Separator rectangle | `rgba(255,255,255,0.08)` |
| Separator (toolbar) | Titlebars and Toolbars Separator | `rgba(255,255,255,0.05)` |
| Separator (light mode, for reference) | Separators/Light | `rgba(60,60,67,0.29)`; no dark swatch exists |
| Accent blue (dark) | System Colors/Dark/8 Blue | `#0091FF` |
| Accent blue (light, focus ring) | System Colors/Light/8 Blue | `#0088FF` |
| systemGreen (dark) | System Colors/Dark/4 Green | `#30D158` |
| systemTeal (dark) | System Colors/Dark/6 Teal | `#00D2E0` |
| systemOrange (dark) | System Colors/Dark/2 Orange | `#FF9230` |
| systemRed (dark) | System Colors/Dark/1 Red | `#FF4245` |
| systemYellow (dark) | System Colors/Dark/3 Yellow | `#FFD600` |
| systemGray (dark) | Grays/Dark/Grey | `#98989D` |
| Traffic lights | Titlebars/Dark/Window Controls/Standard/Active | close `#FF5C60`, minimize `#FAC800`, zoom `#35C759`; inactive all `rgba(255,255,255,0.17)` |

Dark also ships a "plus lighter" vibrant variant of every system color for
text over materials (blue `#0A99FF`, green `#3BDB63`, red `#FF4747`, etc.).

## 2. Typography

Shared text styles from the document library (used across all pages). Font is
SF Pro. Every style has tracking 0 as authored. Each style exists as
Default/Emphasized plus Tight and Loose leading variants; the table shows the
default line height with tight/loose in parentheses. Authored text color is
the light-mode label `rgba(0,0,0,0.85)`; dark mode swaps to the Labels/Dark
values in section 1.

| Style | Size | Line height (tight/loose) | Default weight | Emphasized weight |
| --- | --- | --- | --- | --- |
| 01 LargeTitle | 26 | 32 (30/34) | Regular | Bold |
| 02 Title1 | 22 | 26 (24/28) | Regular | Bold |
| 03 Title2 | 17 | 22 (20/24) | Regular | Bold |
| 04 Title3 | 15 | 20 (18/22) | Regular | Semibold |
| 05 Headline | 13 | 16 (14/18) | Bold | Heavy |
| 06 Body | 13 | 16 (14/18) | Regular | Semibold |
| 07 Callout | 12 | 15 (13/17) | Regular | Semibold |
| 08 Subheadline | 11 | 14 (12/16) | Regular | Semibold |
| 09 Footnote | 10 | 13 (11/15) | Regular | Semibold |
| 10 Caption1 | 10 | 13 (11/15) | Regular | Semibold |
| 11 Caption2 | 10 | 13 (11/15) | Regular | Semibold |

Observed usage in components: sidebar rows and form labels use 06 Body
(13/16); sidebar section headers use 11 Bold at the vibrant tertiary color;
alert title 13 Bold, alert body 13 Regular, alert button label 13 Medium;
window title 15-high text block in the 33pt titlebar.

## 3. Metrics

Control size classes are consistent across "Buttons", "Text Fields", "Pop-up
and Pull-down Buttons", "Search Fields", "Segmented Controls", and
"Progress Indicators" pages: Mini 16, Small 20, Regular 24, Large 28, XL 36
(heights in px).

Controls:

- Push buttons (Buttons page): heights 16/20/24/28/36. Label padding:
  Regular/Large 16px horizontal, Small 10px; label vertically centered
  (13pt label, 4px top offset at Regular). Shape is a capsule; the kit
  applies rounding via style, not stored geometry (switches on the Toggles
  page encode `cornerRadius: 50`, confirming capsule intent).
- Text fields (Text Fields page): default width 120; heights 16/20/24/28/36
  with corner radius exactly height/4: 4/5/6/7/9. Text inset 6px horizontal.
- Pop-up buttons: 120 wide, same heights as buttons.
- Search fields: 120 default; in toolbars 149x24 (compact) and 159x36
  (regular), from the "Windows" page toolbar artboards.
- Switches (Toggles - Switches): 36x16, 44x20, 54x24, 64x28, 80x36; capsule
  (`cornerRadius: 50`).
- Checkboxes: box 18x18 at XL (about 14x14 at Regular), corner radius 5.
- Segmented controls: control heights 16/20/24/28/36; toolbar segment
  buttons 24x20 (medium) and 32x28 (XL).
- Steppers: 84x24 at Regular (Forms page).
- Progress (Progress Indicators page): circular spinners 16 and 32;
  linear bars 140x6 (small) and 140x10 (regular).

Window chrome ("Windows" and "Titlebars and Toolbars" pages, dark artboards):

- Reference window artboards are 840x400; utility panel 280 wide.
- Sidebar pane width 256 (both window artboards and sidebar examples).
- Titlebar heights: title-only 33; unified compact toolbar 40 (24px
  controls, 8px inset); unified toolbar with sidebar 52 (36px controls, 8px
  inset); expanded toolbar 33 titlebar + 44 toolbar row (36px controls).
- Traffic lights: 14px circles with 9px gaps (group 68x14), inset 18px from
  the left edge; panel variant uses 10px circles (group 44x10).
- Scrollbar gutter 12 wide.
- Toolbar separator 3x16; navigation button cluster uses a Liquid Glass
  capsule wrapper.
- Window fill `#1E1E1E`; active window has a 1px outside border
  `rgba(255,255,255,0.20)`; shadows: active no-sidebar
  `0 16px 33px rgba(0,0,0,0.40)` plus `0 0 1px rgba(0,0,0,0.80)`;
  active with-sidebar `0 36px 100px rgba(0,0,0,0.40)` plus the same 1px
  contact shadow; inactive `0 16px 32px rgba(0,0,0,0.20)`.
- Window corner radius is not encoded in the file (masters are square).

Sidebar (Sidebars page, dark artboards):

- Row sizes: Small 24 (16px icon slot), Medium 32 (20px icon slot), Large 40
  (24px icon slot); rows are 240 wide inside the 256 sidebar (8px margins).
- Selection highlight: rounded rect, radius 8, `rgba(255,255,255,0.11)`,
  bleeding 4px past the row on each side (248 wide).
- Row layout at Medium: 14px leading inset, 20px icon, 4px gap, 13pt Body
  label at `rgba(255,255,255,0.96)`; trailing detail slot 36 wide (text
  count, not a badge).
- Section headers: 18-20 tall, 11pt Bold at vibrant tertiary `#4C4C4C`
  (plus-lighter).

Radii anchors encoded in the file: selection pills (sidebar and menu items)
8; popover body 20; dock / menu bar 13; text fields height/4; checkbox 5;
capsule controls 50 (max). Alert, window, menu-container, and group-box
rounding is applied by styles and is not stored as numbers.

Focus ring (shared style "Global/Focus Ring"): two outside borders,
`#0088FF` at 25% x 3.5px plus `#0088FF` at 15% x 1px. Dark input focus
("Content Area/Dark/Input Fields/03 - Focus Ring"): `#0091FF` 3.5px outside
plus 1px, whole ring at 50% opacity.

## 4. Dark-mode materials

From the "Materials" shared styles and the sidebar/window styles. The kit
approximates every material as a single translucent fill; no blur values
exist anywhere in the file, so pick backdrop-filter blur strength visually.

| Material | Kit style | Fill |
| --- | --- | --- |
| Ultra thick | Materials/Dark/Ultra Thick | `rgba(44,44,44,0.82)` |
| Thick | Materials/Dark/Thick | `rgba(44,44,44,0.71)` |
| Regular | Materials/Dark/Regular | `rgba(44,44,44,0.61)` |
| Thin | Materials/Dark/Thin | `rgba(41,41,41,0.49)` |
| Ultra thin | Materials/Dark/Ultra Thin | `rgba(41,41,41,0.40)` |
| Sidebar | Sidebar/Background/Dark/Active | `rgba(12,12,12,0.85)` |

Liquid Glass styles (used by alerts, popovers, menus, notifications, toolbar
clusters) are stacks of fills plus edge-highlight shadows rather than blur:

- Menus/popovers ("Liquid Glass/Dark/Menus"): `rgba(90,90,90,0.50)` fill in
  a luminosity-style blend, drop shadow `0 4px 24px rgba(0,0,0,0.30)`, a 1px
  contact ring `rgba(0,0,0,0.80)`, and faint inner top/bottom edge
  highlights.
- Alert/panel glass ("Liquid Glass/Dark/Regular - Large"): layered
  `rgba(26,26,26,0.5..1.0)` fills, drop shadow `0 18px 48px rgba(0,0,0,0.45)`
  and a bright 0.3px rim (`rgba(166,166,166,0.8)`).
- Scroll edge effect (Kit page): "hard" = window color `#1E1E1E` at 55%
  opacity over a blur layer; "soft" = white-to-transparent gradient mask at
  60% over a blur layer. Blur strength again not encoded.

CSS approximation that matches these numbers: sidebar
`background: rgba(12,12,12,0.85); backdrop-filter: blur(30px)
saturate(1.4);` and menu/popover surfaces
`background: rgba(44,44,44,0.61); backdrop-filter: blur(20px);` with a 1px
`rgba(255,255,255,0.10)` inner border and the shadow pairs above. Blur px
values here are visual estimates, not kit data.

## 5. Component inventory mapped to RunHaven surfaces

Kit pages define, per light/dark and per state (idle/clicked/disabled,
active/inactive window):

- Source list sidebar (Sidebars page) -> RunHaven session/nav rail: 256 wide,
  Medium rows 32 tall, radius-8 selection pill at white 11%, 13pt labels at
  white 96%, 11pt bold section headers, trailing text-count slot. Use Large
  rows (40) only if touch-like density is wanted.
- Grouped forms (Forms page, "Forms/Example Forms/Dark") -> settings and
  launch panels: 500-wide window, container fill white 3%, 20px outer
  margin, rows 440 wide and 48 tall (40 for text-only rows), 10px row inset,
  13pt leading label, trailing 24px-tall control (text field, pop-up,
  stepper, radio group, push button).
- Alerts (Alerts page) -> destructive confirmations: 260 wide, 16px padding,
  72px icon, 13 Bold title + 13 Regular body, full-width 228x28 buttons
  stacked with 6px gaps ("Stacked" 260x238) or side-by-side pair
  ("Side by Side" 260x170), on Liquid Glass Regular-Large background.
- Dialogs/panels: "Panels/Dark" = `#1E1E1E`, 1px white 20% border, window
  shadow pair; utility panel 280 wide with 44x10 traffic lights.
- Progress indicators -> build/run status: circular 16 (inline) and 32
  (blocking), linear 140x6; determinate fills use accent blue `#0091FF`.
- Menus and popovers -> context menus, pickers: menu items 24 tall (Regular)
  with radius-8 highlight, container padding 16 horizontal / 6 vertical,
  11-tall separator rows at white 8%; popover body radius 20 with a 46x10.5
  arrow cartouche.
- Notifications page -> toast pattern: 344x78 card, 12px padding, 32px app
  icon, 6-radius thumbnail, title/description at 13pt.
- Badges/status dots: not in the kit. Build RunHaven status dots from the
  System Colors swatches (section 1) at 8-10px, and reuse the sidebar
  trailing-detail slot (36 wide) for counts.
- Also available if needed: combo boxes, sliders, steppers, disclosure
  controls, image wells, color wells, scrollbars (12 wide), tooltips,
  segmented controls, toggles.

## 6. Web translation for ui/src/index.css

Suggested dark-theme custom properties derived from the kit. Per the repo
design direction, RunHaven keeps a navy hue where Apple uses neutral gray:
kit-neutral surfaces below are marked "hue-shift" (move to the existing navy
hue around oklch hue 248 while keeping the kit lightness/alpha); all other
values are kit-exact.

```css
.dark {
  /* Surfaces (hue-shift the neutrals toward navy, keep L and alpha) */
  --background: #1e1e1e;            /* window bg; navy: oklch(0.24 0.015 248) */
  --card: rgba(255, 255, 255, 0.03);      /* grouped form container over bg */
  --popover: rgba(44, 44, 44, 0.61);      /* material regular; + backdrop blur */
  --sidebar: rgba(12, 12, 12, 0.85);      /* sidebar material; + backdrop blur */
  --secondary: rgba(255, 255, 255, 0.07); /* bordered (default) button fill */

  /* Text */
  --foreground: rgba(255, 255, 255, 0.96);
  --muted-foreground: rgba(255, 255, 255, 0.55);
  --sidebar-foreground: rgba(255, 255, 255, 0.96);

  /* Lines and fills */
  --border: rgba(255, 255, 255, 0.08);
  --input: rgba(255, 255, 255, 0.04);     /* text field border */
  --sidebar-accent: rgba(255, 255, 255, 0.11); /* selection pill */

  /* Accents (kit-exact, dark variants) */
  --primary: #0091ff;
  --success: #30d158;                     /* ready: or teal #00d2e0 */
  --warning: #ff9230;
  --destructive: #ff4245;
  --ring: rgba(0, 145, 255, 0.5);         /* 3.5px outside on focus */

  /* Radii: pill 8, panel 20; fields use height/4 */
  --radius: 10px;                          /* --radius-md 8, --radius-2xl 18 */
}

/* Metrics as utility vars */
:root {
  --control-h-sm: 20px;
  --control-h-md: 24px;   /* default buttons, fields, form controls */
  --control-h-lg: 28px;
  --control-h-xl: 36px;   /* toolbar buttons */
  --sidebar-w: 256px;
  --sidebar-row-h: 32px;
  --titlebar-h: 52px;     /* unified toolbar; 40 compact, 33 title-only */
  --form-row-h: 48px;
  --font-size-body: 13px; /* line-height 16px */
  --shadow-window: 0 16px 33px rgba(0, 0, 0, 0.4), 0 0 1px rgba(0, 0, 0, 0.8);
  --shadow-menu: 0 4px 24px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(0, 0, 0, 0.8);
}
```

Application notes: buttons are capsules (`border-radius: 9999px`) at the
heights above with 16px horizontal padding; text fields keep rectangle
corners at height/4 (6px at the 24px default); base type is 13px/16px SF Pro
via the existing `-apple-system` stack; sidebar selection pills use 8px
radius and may bleed 4px wider than the row; pair the translucent sidebar
and popover fills with `backdrop-filter` since the kit supplies opacity but
no blur numbers.
