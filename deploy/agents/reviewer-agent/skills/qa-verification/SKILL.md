---
name: qa-verification
description: Use this skill for Phase 3 of code review. Runs tests, verifies behavior matches expectations, checks edge cases against the design document, and confirms no regressions. The final gate before approval.
---

# QA Verification (Phase 3)

Verify that the implementation actually works. Reading code catches logic errors; running code catches integration errors. Both are necessary.

## Verification Steps

### 1. Full Test Suite

Run the complete test suite:
```bash
# Rust
cargo test --workspace

# TypeScript
bun test

# Python
pytest
```

**Requirements:**
- All tests must pass (zero failures).
- No new warnings introduced.
- Test output must be clean (no error messages in stdout/stderr from passing tests).

### 2. Test Quality Audit

Review the tests themselves:
- Do tests verify **behavior** (what the function does) or **implementation** (how it does it)?
- Are assertions on observable outcomes, not internal state?
- Do test names describe the scenario and expected outcome?
- Are negative tests present (what should NOT happen)?

### 3. Design Document Edge Cases

For each edge case identified in the architect-agent's design document:
- [ ] Is there a test covering this edge case?
- [ ] Does the test verify the correct behavior?
- [ ] If no test exists, flag as SPEC_GAP

### 4. Regression Check

- Run `git diff` to identify changed files.
- For each changed file, check if existing tests still pass.
- Look for tests that were modified — verify modifications are intentional, not workarounds.
- Check if any test was deleted — flag deleted tests for review.

### 5. Error Path Verification

For each error path in the implementation:
- Is there a test that triggers the error condition?
- Does the error produce the correct error type/message?
- Are resources cleaned up on error?
- Does the error propagate correctly to the caller?

### 6. Integration Smoke Test (if applicable)

If the change involves multiple components:
- Do components interact correctly?
- Are boundaries respected (no direct access to internal state)?
- Do mock boundaries match production boundaries?

## Output Format

```
## QA Verification Report

### Test Suite
- Total: [N] tests
- Passed: [N]
- Failed: [N] (list failures)
- Warnings: [N] (list new warnings)

### Test Quality
- [GOOD/CONCERN] Behavior testing: [details]
- [GOOD/CONCERN] Assertion quality: [details]
- [GOOD/CONCERN] Negative tests: [details]

### Edge Case Coverage
- [COVERED] Edge case 1: [test name]
- [MISSING] Edge case 2: [no test found]

### Regressions
- [NONE/FOUND] [details]

### Error Paths
- [COVERED/MISSING] Error path 1: [details]

### Verdict: [PASS / FAIL / NEEDS_TESTS]
```

If verdict is FAIL or NEEDS_TESTS: list each issue with specific details. The coder-agent must address these before the review can be approved.
