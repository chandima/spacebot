---
name: spec-compliance
description: Use this skill for Phase 1 of code review. Verifies that the implementation matches the design document exactly — nothing more, nothing less. Must pass before code quality or adversarial review.
---

# Spec Compliance Review (Phase 1)

Verify the implementation matches the architect-agent's design document exactly. This is the first gate — it must pass before any other review phase.

## Core Rule

**Do not trust the coder's report. Read the actual code.**

The coder says "all tests pass" — verify by reading test files. The coder says "implemented per spec" — compare code to spec line by line.

## Checklist

### Requirements Coverage

For every requirement in the design document:
- [ ] Is there code that implements this requirement?
- [ ] Is there a test that verifies this requirement?
- [ ] Does the implementation match the specified approach (not a different approach)?

### Over-Building Detection (YAGNI)

- [ ] Are there any features NOT in the design document?
- [ ] Are there any abstractions that aren't required by the spec?
- [ ] Are there any error handlers for cases the spec doesn't mention?
- [ ] Is there any "future-proofing" code that isn't needed yet?

If over-building detected: flag as P2 with explanation of what should be removed.

### File Organization

- [ ] Do file paths match what the design document specified?
- [ ] Are new files organized according to the project's existing patterns?
- [ ] Are changes contained to the files mentioned in the design document?

### Test Matrix Compliance

Compare against the design document's test matrix:
- [ ] Every component in the test matrix has corresponding tests
- [ ] Mock boundaries match what was specified
- [ ] DI patterns follow the specified interfaces
- [ ] Coverage targets are met (or deviation is justified)

## Output Format

```
## Spec Compliance Review

### Requirements
- [PASS] Requirement 1: [evidence — file:line]
- [FAIL] Requirement 2: [what's missing/wrong — expected vs actual]
- [PASS] Requirement 3: [evidence]

### Over-Building
- [CLEAN] No features beyond spec detected
  OR
- [YAGNI] [description of extra code] — File: [path:line]

### File Organization
- [PASS/FAIL] [details]

### Test Matrix
- [PASS/FAIL] [component]: [details]

### Verdict: [PASS / SPEC_GAPS]
```

If verdict is SPEC_GAPS: list each gap with file:line reference. Do NOT proceed to Phase 2 or Phase 3.
