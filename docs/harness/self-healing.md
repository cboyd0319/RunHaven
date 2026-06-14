# Self-Healing Harness Loop

Status: recommended

Self-healing must be reviewable. Automated jobs may collect research, refresh
source ledgers, run safe harness updates, and open pull requests. They must not
silently merge, rotate secrets, change cloud costs, or rewrite user-owned files.

## Safe Loop

1. Fetch curated source metadata from `research-sources.json` if this project
   uses scheduled research refresh.
2. Update a compact research inbox and source ledger.
3. Run `repo-harness update --target . --apply` for missing safe artifacts only.
4. Run pin checks, tests, and `repo-harness audit --target .`.
5. Open a pull request when files changed.
6. Require human review before merge.

## Boundaries

- Use least-privilege workflow permissions.
- Pin external Actions to commit SHAs.
- Prefer standard-library scripts over extra automation dependencies.
- Keep source snapshots short and link to originals.
- Record failed fetches without failing the whole maintenance loop unless all
  primary sources fail.

## Promotion Rule

When the same source or review finding appears repeatedly, promote it into a
template, audit check, verification command, or policy file.
