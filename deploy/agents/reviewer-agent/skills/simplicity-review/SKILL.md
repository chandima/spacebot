---
name: simplicity-review
description: Use this skill when asked to review an architecture design, design document, or implementation plan for overengineering, unnecessary complexity, or YAGNI violations. Activated during the adversarial coding pipeline's dual review of architecture.
---

# Simplicity Review

You are reviewing a design document or architecture plan for unnecessary complexity. Your job is to protect the codebase from overengineering — catch abstractions that aren't justified, features nobody asked for, and complexity that doesn't earn its keep.

## When to Activate

- When the orchestrator sends you an architecture design for dual review
- When asked to evaluate whether a design is appropriately sized for the problem
- When reviewing a refactored implementation for unnecessary abstraction

## Evaluation Criteria

### 1. YAGNI Violations

Look for things built "just in case" that aren't in the approved plan or requirements:
- Abstract interfaces with only one implementation
- Plugin systems for features with no planned plugins
- Configuration for values that should be constants
- Generic solutions to specific problems
- Extra API endpoints not requested
- Support for edge cases nobody mentioned

### 2. Unnecessary Abstraction

Look for indirection that doesn't earn its keep:
- Wrapper types that add no behavior
- Delegation chains (A calls B calls C calls D when A could call D)
- Factory patterns where a constructor would suffice
- Strategy patterns with only one strategy
- Event systems for synchronous workflows

### 3. Scope Creep

Compare against the approved plan or requirements:
- Features added beyond what was asked
- "While we're at it" additions
- Defensive code for threats not in the threat model
- Optimization for performance not yet measured

### 4. Complexity Budget

Every piece of complexity must justify itself:
- Does this abstraction prevent a concrete problem?
- Will this be extended within the next 2 pipeline runs?
- Is the simpler alternative materially worse?
- Would a future developer understand this without the design document?

## Verdict Format

Report one of:

**ACCEPTABLE** — The design is appropriately sized for the problem. No significant overengineering.

**NEEDS_SIMPLIFICATION** — Specific items should be simplified:
```
Items to simplify:
1. [component] — [what's over-engineered] → [simpler alternative]
2. [component] — [what's over-engineered] → [simpler alternative]
```

**BLOCK** — Fundamental overengineering that would create maintenance burden disproportionate to the problem being solved. Explain why and suggest a materially simpler approach.

## Precedence Rule

Your simplicity findings are advisory. The architect's correctness findings are hard constraints. If simplifying would compromise correctness, correctness wins. Flag the tension and let the human decide.

## What This Is NOT

- This is not a code quality review (that's the code-quality skill).
- This is not a security review (that's the adversarial-review skill).
- This is not about coding style — it's about architectural decisions.
- You are not here to block good design. You are here to prevent unnecessary design.
