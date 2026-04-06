---
name: pr-slicer
description: Use this skill when a coding task produces a large change set that should be split into smaller, independently reviewable slices. Activated when the architect's task breakdown exceeds 10 files or 400 changed lines, or when explicitly asked to slice a PR.
---

# PR Slicer

Break large changes into small, independently verifiable slices that reduce review latency and rework risk.

## When to Activate

- The architect's task breakdown touches more than 10 files
- A subtask implementation exceeds ~400 changed lines
- The orchestrator asks you to "split this" or "make this smaller"
- A review round reveals the change set is too large to review effectively

## Slice Budgets

- Target ≤ 400 changed lines per slice
- Target ≤ 10 changed files per slice
- Target 1–4 commits per slice
- Each slice must be behaviorally coherent and independently verifiable

## Slicing Order

1. **Prerequisites first** — shared types, interfaces, utility functions
2. **Mechanical refactors next** — renames, moves, extractions (no behavior change)
3. **Behavior changes after prerequisites** — new features, logic changes
4. **UI/docs/polish last** — styling, documentation, cleanup

## Slice Packet

For each slice, define:

```
## Slice N: [title]
**Goal:** [what this slice accomplishes independently]
**Files:** [list of owned files]
**Out of scope:** [what this slice explicitly does NOT touch]
**Risk:** low / medium / high
**Depends on:** [slice N-1] or [none]
**Verification:**
  - [command 1] → expected result
  - [command 2] → expected result
**Rollback:** [how to undo if this slice causes problems]
```

## Hard Rules

- Never mix refactor and behavior changes in one slice unless unavoidable
- Never touch unrelated subsystems in one slice
- Never create cross-slice hidden dependencies — if slice B depends on slice A, state it
- Each slice must leave the codebase in a working state (tests pass, builds succeed)

## Verification Per Slice

1. Run narrow checks first — tests for the specific behavior touched
2. Run broader project checks — build, lint, type check
3. Record exact commands and outcomes

## Handoff

When presenting slices to the orchestrator:
- Numbered slice list with order and purpose
- Per-slice file ownership (no overlapping files between slices when possible)
- Per-slice verification evidence
- Dependency graph between slices
- Residual risk and follow-up items
