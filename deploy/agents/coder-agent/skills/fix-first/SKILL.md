---
name: fix-first
description: Use this skill when handling reviewer feedback. Categorizes findings into AUTO-FIX (apply immediately), ASK (batch questions), and INVESTIGATE (deep analysis). Prevents question fatigue by minimizing back-and-forth.
---

# Fix-First Protocol

When receiving review findings, categorize and act efficiently. The goal is to minimize back-and-forth while ensuring quality.

## Category: AUTO-FIX

Apply immediately without asking. These are unambiguous improvements:

- Missing error handling on fallible operations
- Typos in strings, variable names, or comments
- Missing imports or unused imports
- Style violations that match project conventions
- Missing `Clone`, `Debug`, or other derived traits
- Obvious off-by-one fixes
- Missing null/None/nil checks on nullable values
- Trivial performance improvements (e.g., `&str` instead of `String` in parameters)

**Action:** Fix all AUTO-FIX items in a single commit with message `fix: address review feedback (auto-fix)`.

## Category: ASK

Ambiguous issues that need human input. Batch ALL questions into a single message:

- Design choices with multiple valid approaches
- Scope questions ("should this handle X case?")
- Performance vs. readability tradeoffs
- Backward compatibility concerns
- Feature behavior when requirements are unclear

**Action:** Collect all ASK items. Send one message to the orchestrator with numbered questions. Wait for answers before proceeding.

**Format:**
```
Review feedback requires decisions:
1. [P2] File X, line Y: Should we handle case Z? Options: (a) ignore, (b) error, (c) default value.
2. [P2] File A, line B: Two approaches for C. Which is preferred? (a) trait-based, (b) enum-based.
```

## Category: INVESTIGATE

Complex issues requiring analysis before deciding on a fix:

- Race condition concerns
- Architectural implications
- Security vulnerability reports
- Performance regression concerns
- Issues that might require design changes

**Action:** Analyze the issue. Produce a brief report with:
1. Is the concern valid? (with evidence)
2. What's the impact if not fixed?
3. Proposed fix (if valid)
4. Effort estimate

Report findings before implementing any fix.

## Processing Order

1. Apply all AUTO-FIX items first (immediate commit).
2. Send all ASK items in one batch (wait for response).
3. Investigate all INVESTIGATE items in parallel (report findings).
4. After receiving ASK answers and INVESTIGATE conclusions, implement remaining fixes.
5. Report DONE with summary of all changes.
