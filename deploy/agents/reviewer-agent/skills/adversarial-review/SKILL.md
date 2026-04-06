---
name: adversarial-review
description: Use this skill for Phase 2 of code review. Fresh context, no checklist bias. Think like an attacker and chaos engineer. Finds edge cases, race conditions, security holes, and resource leaks that structured review misses.
---

# Adversarial Review (Phase 2)

Forget the checklist. Forget the spec. Think like an attacker and a chaos engineer. Your job is to break this code.

## Mindset

You are two people:
1. **The Attacker:** What input would I craft to exploit this? Where can I bypass validation? Can I escalate privileges? Can I exfiltrate data?
2. **The Chaos Engineer:** What happens when the network drops mid-operation? When the disk fills up? When two requests hit the same resource simultaneously? When the system has been running for 30 days and memory has leaked?

## Attack Vectors

### Input Attacks
- Empty strings, null values, maximum-length inputs
- Unicode edge cases (zero-width characters, RTL override, emoji in identifiers)
- Injection: SQL, command, path traversal, template injection
- Type confusion: string where number expected, array where object expected
- Boundary values: 0, -1, MAX_INT, NaN, Infinity

### Concurrency Attacks
- Race conditions: two requests modifying the same resource
- Lock ordering: deadlock potential in multi-lock scenarios
- TOCTOU: check-then-act without atomicity
- Stale reads: cache returning outdated data during updates

### Resource Attacks
- Unbounded growth: collections, caches, log files, connection pools
- Missing timeouts: HTTP requests, database queries, file I/O
- Handle leaks: file descriptors, database connections, channels
- Memory leaks: circular references, forgotten subscriptions, growing buffers

### Error Handling Attacks
- Silent failures: errors caught and discarded
- Incorrect error propagation: wrapping loses context
- Missing cleanup: resources not released on error paths
- Inconsistent state: partial operations without rollback

### Security Attacks
- Authentication bypass: missing auth checks on endpoints
- Authorization holes: checking role but not resource ownership
- Data exposure: sensitive fields in logs, error messages, or API responses
- Trust boundaries: user input used unsanitized in system operations

## Pattern Detection

Flag these patterns with specific file:line references:

| Pattern | Risk | Example |
|---------|------|---------|
| String concatenation in SQL | Injection | `format!("SELECT * FROM users WHERE id = '{}'", id)` |
| User input in system commands | Command injection | `Command::new("sh").arg("-c").arg(user_input)` |
| `let _ =` on Result | Silent failure | `let _ = file.write_all(data);` |
| Unbounded `Vec::push` in loop | Memory exhaustion | Loop pushing without size check |
| Missing timeout on async | Hang forever | `client.get(url).send().await` without timeout |
| Shared mutable state without lock | Race condition | `Arc<RefCell<T>>` across threads |

## Output Format

```
## Adversarial Findings

- [P1] [Finding title]
  File: [path:line]
  Attack: [how to exploit]
  Impact: [what happens]
  Fix: [suggested remediation]

- [P2] [Finding title]
  File: [path:line]
  Risk: [what could go wrong]
  Fix: [suggested remediation]

- [P3] [Finding title]
  File: [path:line]
  Note: [suggestion]
```

## Severity Guide

- **P1 (Must Fix):** Exploitable in production. Data loss, security breach, or crash.
- **P2 (Should Fix):** Likely to cause issues under load, edge cases, or extended operation.
- **P3 (Suggestion):** Defensive improvement. Not currently exploitable but reduces attack surface.

Zero false positives. If you're not confident, don't report it. Noise destroys trust.
