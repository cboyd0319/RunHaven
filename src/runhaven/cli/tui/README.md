# RunHaven TUI Vendor Baseline

This directory is a source snapshot from:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

The upstream source is the OpenAI Codex TUI and is licensed under Apache-2.0.
RunHaven keeps attribution in `THIRD_PARTY_NOTICES.md` and
`licenses/codex-Apache-2.0.txt`.

This is a baseline copy before RunHaven product integration. It intentionally
keeps Codex TUI structure first, then RunHaven will adapt the parts it needs.

Local exclusions in this baseline:

- `.DS_Store` files, because they are local filesystem metadata.
- upstream `*.snap` files, because they are Codex test goldens and must be
  regenerated from integrated RunHaven tests if those tests are kept.

Local source-format exception:

- `markdown_render_tests.rs` uses `concat!` for one Markdown hard-break fixture
  so the runtime test input still contains two trailing spaces, while the source
  file satisfies RunHaven's whitespace check.

Known integration gap:

- The copied Codex crate source still uses Codex crate/module assumptions.
  RunHaven integration will adapt entrypoints, module paths, dependencies, and
  product data in later commits.
