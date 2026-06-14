# Evaluator Rubric

Use this after non-trivial agent work or before accepting a large change.

Score each category from 0 to 2.

| Category | 0 | 1 | 2 |
| --- | --- | --- | --- |
| Correctness | Target behavior is missing or contradicted | Partially works with known gaps | Matches requested behavior |
| Verification | No relevant check ran | Some checks ran but coverage or evidence is incomplete | Required checks ran with evidence |
| Scope Discipline | Changed unrelated behavior or files | Mostly scoped with minor drift | Stayed within the selected objective |
| Reliability | Fails restart, repeat run, or common platform path | Works in one path with stated risk | Survives restart and expected platform paths |
| Maintainability | Hard to understand or extends drift | Understandable but needs cleanup | Fits project patterns and reduces confusion |
| Handoff Readiness | Next session must infer state | Handoff exists but has gaps | State, blockers, files, and next step are clear |

## Verdict

- Accept: total score is 10 or higher and no category is 0.
- Revise: score is 7 to 9, or one category is 0 with a clear fix.
- Block: score is below 7, verification is absent, or correctness is unproven.

## Follow-Up

Record any rubric weakness here when human judgment and the rubric disagree.
Update this file before using the same evaluation again.
