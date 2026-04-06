---
name: office-hours
description: Use this skill when starting a new coding task or feature request. Frames the problem using six forcing questions before any design or implementation begins. Produces a problem statement that feeds into architecture design.
---

# Office Hours — Problem Framing

Before designing or building anything, work through these six forcing questions. Do not skip any. Write your answers as a structured problem statement.

## The Six Forcing Questions

### 1. What problem are we actually solving?

Strip away assumptions and restate in one sentence. Not "build feature X" but "users need to Y because Z."

Ask yourself: If we could wave a magic wand, what would be different? That's the real problem.

### 2. Who is affected and how?

Map the stakeholders:
- Who uses this directly?
- Who is affected indirectly?
- What are their constraints (time, skill, environment)?
- What do they currently do instead?

### 3. What does success look like?

Define measurable outcomes, not activities:
- Bad: "Implement caching layer"
- Good: "API response time drops from 2s to 200ms for repeated queries"

What would make this a 10-star solution? (Then cut to what's achievable.)

### 4. What are the failure modes?

List what can go wrong:
- What if the input is malformed?
- What if the dependency is unavailable?
- What's the blast radius of each failure?
- What's the recovery path?

### 5. What's the simplest version that delivers value?

Find the MVP:
- What's the smallest change that proves the concept?
- What can be deferred to a follow-up?
- What's the 80/20 — the 20% of work that delivers 80% of value?

### 6. What are we explicitly NOT doing?

Define boundaries:
- What features are out of scope?
- What systems are we not touching?
- What edge cases are we explicitly not handling in v1?

## Output Format

Produce a structured problem statement:

```markdown
## Problem Statement
[One sentence]

## Stakeholders
[Who and how affected]

## Success Criteria
[Measurable outcomes]

## Failure Modes
[What can go wrong, ordered by blast radius]

## MVP Scope
[Simplest version that delivers value]

## Out of Scope
[What we're explicitly not doing]
```

This feeds into the architecture-lock skill for design document production.
