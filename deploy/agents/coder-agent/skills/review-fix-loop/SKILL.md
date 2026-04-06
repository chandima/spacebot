---
name: review-fix-loop
description: Use this skill when addressing review findings from the reviewer-agent. Maps every finding to a specific code change and verification evidence, ensuring systematic closure with minimal back-and-forth.
---

# Review Fix Loop

Close review rounds systematically by mapping every finding to a code change and proof. No finding is left ambiguous.

## When to Activate

- After receiving P1/P2 findings from reviewer-agent
- When the orchestrator routes review feedback to you for fixing
- When re-submitting after a failed review round

## Execution Loop

1. **Parse** — categorize findings by severity (P1 / P2 / P3) and type (AUTO-FIX / ASK / INVESTIGATE)
2. **Build closure table** — one row per finding, planned change identified BEFORE editing
3. **Implement** — fix the smallest coherent batch (don't mix unrelated fixes)
4. **Verify narrowly** — run targeted tests for each fix
5. **Update closure table** — record verification command and result
6. **Verify broadly** — run full test suite and lint

## Closure Table

Build this before making any changes:

```
| # | Finding | Severity | Type | File(s) | Planned Fix | Verify Command | Result |
|---|---------|----------|------|---------|-------------|----------------|--------|
| 1 | [desc]  | P1       | AUTO | [path]  | [what]      | [cmd]          |        |
| 2 | [desc]  | P2       | ASK  | [path]  | [pending]   | [cmd]          |        |
```

Fill the Result column AFTER each fix is verified.

## Re-Run Control

- If the same verification command fails twice, STOP rerunning it
- Isolate a smaller reproduction (single test, specific input)
- Identify the smallest likely cause
- Fix that specific cause
- Re-run narrow verification before broad checks

## Verification Ladder

Work from narrow to broad:

1. **Narrow** — unit test for the specific function/module touched
2. **Medium** — compile, lint, type-check for the touched files
3. **Broad** — full test suite, project-level checks

## Handling ASK Items

- Do NOT guess at ASK findings — batch them and route to the orchestrator
- Include your analysis and recommendation with each ASK item
- Wait for the human decision before implementing

## Handling INVESTIGATE Items

- Spend no more than 2 worker turns investigating
- If root cause is clear → fix it and add to closure table
- If root cause is unclear → escalate to orchestrator with your analysis

## Handoff

When submitting the fixed code back to the orchestrator:

```
## Review Fix Report
**Findings addressed:** N of M
**Closure table:** [include completed table]
**Verification evidence:** [commands run and results]
**Open items:** [any ASK/INVESTIGATE items pending human input]
**Residual risk:** [anything that might need follow-up]
```
