# Third-Party Notices

RunHaven includes third-party code. This file records the attribution that the
upstream licenses require.

## openai/codex (Apache-2.0)

RunHaven vendors source code from [openai/codex](https://github.com/openai/codex),
specifically the `codex-rs/tui` pet, rendering, color, and test-backend modules
and the `codex-rs/terminal-detection` crate. That code is licensed under the
Apache License, Version 2.0. The full license text is in
[`licenses/codex-Apache-2.0.txt`](licenses/codex-Apache-2.0.txt).

The upstream `NOTICE` file, carried forward verbatim:

```
OpenAI Codex
Copyright 2025 OpenAI

This project includes code derived from [Ratatui](https://github.com/ratatui/ratatui), licensed under the MIT license.
Copyright (c) 2016-2022 Florian Dehau
Copyright (c) 2023-2025 The Ratatui Developers
```

### Vendored files

The following files were copied into `src/runhaven/cli/tui/codex/` from the
sources listed below:

| RunHaven file | Upstream source |
| --- | --- |
| `src/runhaven/cli/tui/codex/terminal_detection.rs` | `codex-rs/terminal-detection/src/lib.rs` |
| `src/runhaven/cli/tui/codex/image_protocol.rs` | `codex-rs/tui/src/pets/image_protocol.rs` |
| `src/runhaven/cli/tui/codex/sixel.rs` | `codex-rs/tui/src/pets/sixel.rs` |
| `src/runhaven/cli/tui/codex/model.rs` | `codex-rs/tui/src/pets/model.rs` |
| `src/runhaven/cli/tui/codex/frames.rs` | `codex-rs/tui/src/pets/frames.rs` |
| `src/runhaven/cli/tui/codex/catalog.rs` | `codex-rs/tui/src/pets/catalog.rs` |
| `src/runhaven/cli/tui/codex/animation.rs` | `codex-rs/tui/src/pets/ambient.rs` (animation-timing extract only) |
| `src/runhaven/cli/tui/color.rs` | `codex-rs/tui/src/color.rs` |
| `src/runhaven/cli/tui/test_backend.rs` | `codex-rs/tui/src/test_backend.rs` |

### Adapted pet integration

RunHaven's `src/runhaven/cli/tui/pet.rs` is local integration code that follows
the structure and terminal-image behavior of these upstream Codex files:

| RunHaven file | Upstream source used as reference |
| --- | --- |
| `src/runhaven/cli/tui/pet.rs` | `codex-rs/tui/src/pets/ambient.rs` and `codex-rs/tui/src/pets/mod.rs` |

### Modifications

The vendored files were modified by RunHaven on 2026-06-26. The changes are
limited to integration plumbing, not behavior of the copied logic:

- Import paths were adapted to RunHaven's module layout (for example,
  `codex_terminal_detection::` became `super::terminal_detection::`).
- The upstream `tracing` logging call in `terminal_detection.rs` was removed so
  no logging dependency is pulled in.
- TUI and asset-pack couplings that RunHaven does not vendor were removed: the
  `terminal_tests.rs` test-module declaration, the `serial_test` test
  dependency (replaced with a standard-library lock), and the `asset_pack`-based
  test in `model.rs`. A `builtin_spritesheet_path` helper is provided locally to
  keep `model.rs` compiling.
- `animation.rs` is an extract of only the pure frame-selection functions from
  `ambient.rs` (`current_animation_frame`, `frame_at_elapsed`,
  `nanos_to_duration`, and the `AnimationFrameTick` result type). The
  `AmbientPet` struct, `FrameRequester` scheduling, the `PetNotification` state
  machine, and all `crate::tui` / `crate::app_event` / `ratatui` layout
  couplings were excluded. Selected items were widened from `pub(super)` to
  `pub(crate)` so the eventual TUI integration can drive them: `AnimationFrameTick`
  and `current_animation_frame` in `animation.rs`, `Pet::load_with_codex_home`
  and `Pet::frame_cache_key` in `model.rs`, `prepare_png_frames` in `frames.rs`,
  and the frame/spritesheet dimension constants in `catalog.rs`.
- `color.rs` is copied as a small pure helper module; RunHaven clamps `blend`
  alpha values and currently uses it through the TUI theme layer.
- `test_backend.rs` is copied as a test-only VT100 backend and updated for
  ratatui 0.30.2's `Backend::Error` associated type and RunHaven's module path.
