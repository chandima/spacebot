---
name: code-quality
description: Use this skill alongside adversarial review to evaluate code structure, maintainability, and engineering standards. Checks SOLID principles, file organization, naming, error handling, and documentation quality.
---

# Code Quality Review

Evaluate the code for structural quality, maintainability, and adherence to engineering standards. This runs alongside or after adversarial review.

## Evaluation Dimensions

### 1. Code Structure

- **Single Responsibility:** Does each function/method do one thing? Each file have one purpose?
- **Open/Closed:** Can behavior be extended without modifying existing code?
- **Interface Segregation:** Are interfaces/traits focused? No "god interfaces"?
- **Dependency Inversion:** Do modules depend on abstractions, not concretions?

### 2. Naming

- Are names descriptive and unambiguous?
- Do function names describe what they return or do?
- Are boolean variables phrased as questions (`is_valid`, `has_permission`)?
- Are abbreviations avoided? (`queue` not `q`, `message` not `msg`)
- Are constants named in SCREAMING_SNAKE_CASE?

### 3. Error Handling

- Are errors propagated with context? (`map_err`, `.context()`)?
- Are error types specific? (Not just `String` or generic `Error`)
- Are error paths tested?
- Are recoverable vs. unrecoverable errors distinguished?

### 4. File Organization

- Does each file have a clear, single responsibility?
- Are files growing too large? (Flag files > 500 lines)
- Do new files follow existing project patterns?
- Are test files colocated or in a parallel test directory (matching project convention)?

### 5. Documentation

- Do public APIs have doc comments?
- Are complex algorithms explained?
- Are "why" comments present where the code is non-obvious?
- Are comments accurate (not stale from previous iterations)?

### 6. Testing Quality

- Do tests verify behavior, not implementation?
- Are test names descriptive (`test_user_creation_fails_with_duplicate_email`)?
- Are edge cases covered?
- Are tests independent (no shared mutable state between tests)?
- Is test setup minimal and focused?

## Output Format

```
## Code Quality Assessment

### Strengths
- [What's done well — acknowledge good patterns]

### Issues
- [P2] [Category]: [Description] | File: [path:line] | Fix: [suggestion]
- [P3] [Category]: [Description] | File: [path:line] | Fix: [suggestion]

### Metrics
- New files: [count] | Avg size: [lines]
- Test coverage: [if measurable]
- Error handling: [complete/partial/missing]

### Assessment: [GOOD / ACCEPTABLE / NEEDS_WORK]
```

## Guidelines

- Code quality issues are P2 or P3, never P1. P1 is reserved for correctness and security (adversarial review).
- Focus on patterns, not instances. If naming is inconsistent, mention the pattern once.
- Acknowledge strengths. One sentence about what's well-done reinforces good habits.
- Be specific. "Code could be cleaner" is useless. "Function X (42 lines) should be split: parsing (lines 5-20) and validation (lines 21-42) are independent concerns" is useful.
