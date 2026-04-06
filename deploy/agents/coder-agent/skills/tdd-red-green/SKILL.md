---
name: tdd-red-green
description: Use this skill when implementing code. Enforces strict test-driven development with the RED-GREEN-REFACTOR cycle. No production code without a failing test first. Includes anti-pattern detection and rationalization blocking.
---

# TDD Red-Green-Refactor

## The Iron Law

**No production code without a failing test first.** This is non-negotiable.

## The Cycle

### RED: Write a Failing Test

1. Write one minimal test that captures a single behavior from the design document.
2. Run the test suite. The new test MUST fail.
3. Verify the failure reason is correct (not a typo, import error, or build failure).
4. If the test passes immediately, the test is wrong — it proves nothing. Delete it and rethink.

### GREEN: Make It Pass

1. Write the **simplest** code that makes the failing test pass.
2. Do not add features beyond what the test requires.
3. Run the full test suite. ALL tests must pass (new and existing).
4. If existing tests break, either fix them or reconsider your approach.

### REFACTOR: Clean Up

1. Remove duplication.
2. Improve names (variables, functions, types).
3. Extract abstractions where patterns emerge.
4. Run all tests after each refactor step — they must stay green.
5. Commit the refactor separately from the implementation.

## Anti-Patterns to Reject

### Writing Tests After Code
Tests written after code are verification, not specification. They test what the code does, not what it should do. They miss edge cases because you're adapting tests to match your implementation.

### Testing Mock Behavior
If your test only verifies that a mock was called with certain arguments, you're testing the wiring, not the behavior. Test the observable outcome instead.

### Adding Test-Only Methods
If you need to add public methods solely for testing, the design is wrong. Refactor to make the relevant state observable through the public API.

### Keeping Pre-Test Code
"I already wrote the implementation, I'll just add tests around it." No. Delete the code, write the test, watch it fail, then rewrite the implementation. Tests that pass immediately prove nothing.

## Rationalizations to Block

| Rationalization | Response |
|----------------|----------|
| "Too simple to test" | The test takes 30 seconds to write. Write it. |
| "I'll test after" | Tests pass immediately, proving nothing. |
| "Already manually tested" | Ad-hoc ≠ systematic. Write the test. |
| "Keep the code, write tests" | You'll adapt tests to bugs. Start fresh. |
| "This is just a prototype" | Prototypes become production. Test now. |

## Commit Strategy

Each RED-GREEN-REFACTOR cycle produces one or two commits:
1. `test: add failing test for [behavior]` (RED + GREEN together is fine)
2. `refactor: [what was cleaned up]` (only if refactoring happened)

Keep commits small and focused. Each commit should be independently revertible.
