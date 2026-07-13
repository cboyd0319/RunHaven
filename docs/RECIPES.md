# RunHaven Task Recipes

These examples build on the full [usage guide](USAGE.md).

Review a project without writing to it:

```bash
runhaven run claude --read-only-workspace
```

Run local checks without internet egress:

```bash
runhaven run shell --network internal -- /bin/bash -lc "cargo test --locked"
```

Talk only to a bundled provider profile:

```bash
runhaven run codex --network provider
```

Use an isolated git worktree and inspect changes before merging:

```bash
runhaven run claude --worktree
runhaven runs list
runhaven runs recover RUN_ID
runhaven runs diff RUN_ID
```

Reset one project/profile agent home volume:

```bash
runhaven state reset claude --workspace .
runhaven state reset claude --workspace . --yes
```
