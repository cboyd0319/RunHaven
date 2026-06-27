# Third-Party Notices

RunHaven includes third-party code. This file records the attribution that the
upstream licenses require.

## openai/codex (Apache-2.0)

RunHaven vendors source code from [openai/codex](https://github.com/openai/codex),
specifically the `codex-rs/tui/src/` source tree, under
`src/runhaven/cli/tui/`. That code is licensed under the Apache License,
Version 2.0. The full license text is in
[`licenses/codex-Apache-2.0.txt`](licenses/codex-Apache-2.0.txt).

The upstream `NOTICE` file, carried forward verbatim:

```
OpenAI Codex
Copyright 2025 OpenAI

This project includes code derived from [Ratatui](https://github.com/ratatui/ratatui), licensed under the MIT license.
Copyright (c) 2016-2022 Florian Dehau
Copyright (c) 2023-2025 The Ratatui Developers
```

### Vendored snapshot

The current baseline copies `codex-rs/tui/src/` into
`src/runhaven/cli/tui/`, excluding only local metadata files and upstream
snapshot goldens:

- `.DS_Store` files are not source.
- Upstream `*.snap` files are Codex test goldens, not runtime code. RunHaven
  will regenerate snapshots from integrated RunHaven tests if it keeps those
  test surfaces.

RunHaven-specific product integration, culling decisions, and local
modifications are tracked in `docs/plans/tui-codex-vendor-reset.md` and
`src/runhaven/cli/tui/README.md`.

## ferrif-zmachine (MIT)

RunHaven plans to keep the hidden TUI Zork easter egg attributed if the engine
is reintroduced after the Codex TUI vendor reset. The earlier prototype used and
adapted the Z-machine engine from
[moosepod/ferrif-zmachine](https://github.com/moosepod/ferrif-zmachine). The
upstream source is licensed under the MIT license, copyright 2022 Matthew
Christensen. The full license text is in
[`licenses/ferrif-zmachine-MIT.txt`](licenses/ferrif-zmachine-MIT.txt).

Earlier prototype revision:
`e9a4149817ddfb11c5599dcd161cf3952924cc59`.

The current Codex TUI vendor baseline does not include the earlier local
Ferrif-derived engine files. If the Zork easter egg is rebuilt, the active
engine source and modifications must be listed here again.

## historicalsource/zork1 (MIT)

RunHaven vendors the open-source Zork I source collection from
[historicalsource/zork1](https://github.com/historicalsource/zork1) for the
hidden TUI easter egg. The repository is licensed under the MIT license,
copyright 2025 Microsoft. The full license text is in
[`licenses/zork1-MIT.txt`](licenses/zork1-MIT.txt).

Source revision vendored:
`97b7b3d68c075dd9af7da499c3e9690ada3471fd`.

RunHaven copies the full upstream repository contents under `third_party/zork1/`,
including the ZIL source files and the compiled Z-machine story file
`third_party/zork1/COMPILED/zork1.z3`. `historicalsource` is credited as the
upstream host of the open-source collection. RunHaven does not claim ownership of
the Zork name or any Infocom trademarks.
