---
name: architecture-lock
description: Use this skill after problem framing to produce a locked architecture document. Creates data flow diagrams, state machines, error paths, test matrices, and bite-sized task breakdowns. Architecture is locked after human approval.
---

# Architecture Lock

After problem framing is complete, produce a design document that locks the architecture. Once approved by the human, no architectural changes without explicit re-approval.

## Design Document Structure

### 1. Data Flow Diagram (ASCII)

Show how data moves through the system:

```
[Input] → [Validator] → [Processor] → [Store] → [Output]
                ↓              ↓
           [Error Handler] [Event Bus]
```

Include:
- Entry points (APIs, CLI, events)
- Processing steps
- Storage interactions
- Output channels
- Error paths

### 2. State Machine (if stateful)

Define all states and transitions:

```
[Created] → [Validated] → [Processing] → [Complete]
    ↓            ↓              ↓
 [Invalid]   [Rejected]     [Failed]
```

For each transition, specify:
- Trigger condition
- Side effects
- Reversibility (can we go back?)

Terminal states must be explicitly marked.

### 3. Error Paths

For each component:
- What errors can occur?
- How are they detected?
- What's the recovery strategy?
- What's the user-visible impact?

### 4. Test Matrix

| Component | Test Type | What to Mock | DI Boundary | Coverage Target |
|-----------|-----------|-------------|-------------|-----------------|
| Validator | Unit | None | Input trait | All error cases |
| Processor | Unit | Store | Store trait | Happy + error paths |
| API | Integration | None | Full stack | E2E flows |

### 5. Task Breakdown

Break into subtasks (2-5 minutes each):

```
Task 1: [Title]
  Files: [exact paths]
  Code: [exact code to write, not pseudo-code]
  Test: [exact test command]
  Depends on: [none or task N]

Task 2: [Title]
  ...
```

Rules:
- No placeholders ("implement X", "add validation")
- Every task has a verification command
- Dependencies are explicit
- Each task is independently committable

### 6. Security Considerations

- Authentication/authorization requirements
- Input validation boundaries
- Data exposure risks
- Trust boundaries between components

### 7. Performance Considerations

- Expected load characteristics
- Bottleneck analysis
- Caching strategy (if applicable)
- Resource limits

## Lock Protocol

After the design document is produced:
1. Present to human via HITL for approval
2. If approved → architecture is LOCKED
3. Coder-agent implements against this locked spec
4. Reviewer-agent validates against this locked spec
5. Changes to the locked spec require explicit human re-approval

## Scope Decision

Before locking, state your scope decision:
- **Expansion:** Request is too small. Here's what it should include.
- **Selective Expansion:** Adding [X] and [Y] that weren't requested but are necessary.
- **Hold Scope:** Request is well-scoped. Proceeding as-is.
- **Reduction:** Request is too ambitious. Proposing phases: [Phase 1], [Phase 2].
