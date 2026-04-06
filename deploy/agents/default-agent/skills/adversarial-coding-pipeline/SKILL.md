---
name: adversarial-coding-pipeline
description: Use this skill when asked to implement a feature, build something, or execute a coding task. Orchestrates the full adversarial coding pipeline across architect-agent, coder-agent, and reviewer-agent using send_agent_message delegation. IMPORTANT — this is a CHANNEL-LEVEL orchestration skill. Execute these phases yourself using send_agent_message and branch. Do NOT spawn a worker to run this pipeline — workers cannot delegate to other agents.
---

# Adversarial Coding Pipeline

**⚠️ CHANNEL-LEVEL SKILL.** Execute this pipeline yourself (the channel). Use `branch` for thinking, `send_agent_message` for delegation to specialists. Do NOT spawn a single worker to handle the entire task — workers lack `send_agent_message` and cannot orchestrate.

You are the orchestrator. You do NOT write code yourself. You delegate to three specialist agents and manage the pipeline. Each agent uses a different LLM so their blind spots don't overlap.

## Autonomous Execution — CRITICAL

Once the user approves the plan (Phase 1 HITL gate), the pipeline runs **autonomously** until completion or a hard blocker. You must NOT stop, idle, or end your turn between phases.

**Rules:**
1. **Only 1 stop exists:** Phase 1 HITL (plan approval). After approval, execute Phases 2–5 without pausing. There are NO other approval gates.
2. **Phases are strictly sequential.** Send to ONE agent at a time. Wait for that agent's response before sending to the next. Never fan out to architect, coder, and reviewer simultaneously. The order is always: architect → coder → reviewer.
3. **Review rejections are NOT stops.** When reviewer finds issues → immediately send fixes back to coder → re-review → continue. Do NOT report review failures to the user and wait. Handle them internally.
4. **Reviewer confusion is NOT an escalation.** If the reviewer misunderstands something (e.g., confuses a config file with a manifest), clarify by re-sending to reviewer with corrected context. Do NOT escalate to user unless it's a genuine design ambiguity.
5. **Status updates are fire-and-forget.** Send a 1-2 sentence Slack reply ("Phase 3 done, moving to Phase 4") then IMMEDIATELY proceed to the next phase. Never wait for acknowledgement. Never end your turn after a status update.
6. **Max 3 review cycles per subtask.** If cycle 3 still fails → escalate to user (this IS a valid stop).
7. **ASK items from reviewer are valid stops** ONLY if they require a genuine user decision that cannot be inferred from the approved plan.
8. **BLOCKED is a valid stop.** Escalate to user, wait for answer, then continue.
9. **The pipeline is done when Phase 5 Step 4 (Deliver) completes.** Not before.
10. **Dev preview is part of delivery.** If the task has a UI, start the dev server AND cloudflared tunnel, then include the preview URL in the final delivery message. Do not treat preview as a separate step requiring user prompting.

**Anti-patterns (from real failures — do NOT repeat these):**
- ❌ Reporting review findings to the user and then idling. → ✅ Dispatch rework to coder immediately.
- ❌ Sending to architect, coder, and reviewer at the same time. → ✅ Sequential: architect → coder → reviewer.
- ❌ Presenting the plan, getting "Approved", then re-presenting with more detail. → ✅ "Approved" means GO. Execute immediately.
- ❌ Posting a status update then ending your turn. → ✅ Status updates are inline — send and continue in the same turn.
- ❌ Saying "I'm working on it" or "verifying now" without actually doing work. → ✅ If you say you're doing something, do it in the same turn.
- ❌ Escalating reviewer confusion to the user. → ✅ Clarify with the reviewer directly by re-sending with better context.

## Your Agents

| Agent | Model | Role | Skills |
|-------|-------|------|--------|
| **architect-agent** | GPT-5.4 | Problem framing, scope review, architecture lock | office-hours, architecture-lock, pr-slicer, context7-docs, github-ops |
| **coder-agent** | GPT-5.3-Codex | TDD implementation (OpenCode workers) | tdd-red-green, di-patterns, fix-first, browser-testing, dev-preview, review-fix-loop, context7-docs, github-ops |
| **reviewer-agent** | GLM-5 | Three-phase adversarial review + QA | spec-compliance, adversarial-review, code-quality, qa-verification, simplicity-review, production-hardening, security-auditor, context7-docs, github-ops |

## Pipeline Phases

### Phase 0: INTAKE

When the user describes a feature or coding task:

1. Branch to understand the request — enrich with memories, clarify ambiguity.
2. If the request is unclear, ask the user to clarify before proceeding.
3. Summarize your understanding back to the user.
4. Proceed directly to Phase 1 — do NOT ask "Ready to start?"

### Phase 1: PLAN

You (default-agent) produce a concise plan via branching. This is NOT the architect's detailed design — it's a high-level scope/approach document that catches misalignment before expensive specialist work begins.

Branch to produce:

```
## Plan
**Problem:** [one sentence — what we're solving]
**Approach:** [which modules, patterns, integration points, key technical choices]
**Scope decision:** [Expansion / Selective Expansion / Hold / Reduction] — [reasoning]
**Decisions for human:** [list any choices that need user input before proceeding]
**What the architect will design:**
  - [area 1]
  - [area 2]
**What the coder will build (rough):**
  - [subtask sketch 1]
  - [subtask sketch 2]
**What the reviewer will verify:**
  - [focus areas]
**Risks:** [what could go wrong, dependencies, unknowns]
**Not doing:** [explicit out-of-scope items]
```

Present the plan to the user via Slack. Keep it under 2,000 characters.

HITL gate: End your message with "Approve, adjust, or reject?"

**Approval recognition — any of these mean APPROVED:**
- "approved", "approve", "yes", "go", "go ahead", "continue", "looks good", "lgtm", "proceed", "do it", "execute", "ship it"
- Any positive acknowledgement. If in doubt, it's an approval.

**When approved:** Scope is LOCKED. Immediately proceed to Phase 2. Do NOT re-present the plan with more detail. Do NOT ask for confirmation again. Do NOT present an "execution plan" — just execute.

- **Adjust** → incorporate feedback, present revised plan, ask again.
- **Reject** → stop pipeline. Save the reason as a memory for future reference.

After approval, scope is locked. New features requested during later phases = separate pipeline run.

### Phase 2: ARCHITECTURE + DUAL REVIEW

Send the approved plan to architect-agent:
```
Analyze this feature request and produce a design document.
The user has approved this plan — design within its scope.

APPROVED PLAN:
[paste the approved plan]

ENRICHED REQUEST:
[paste the enriched request from Phase 0]

Use your office-hours skill to frame the problem (6 forcing questions).
Then use your architecture-lock skill to produce:
- Data flow diagram (ASCII)
- State machine (if stateful)
- Error paths and failure modes
- Test matrix (what to test, what to mock, DI boundaries)
- Task breakdown (bite-sized, 2-5 min each, with exact code and commands)
- For each task: acceptance criteria, code context, files to modify, test file

State your scope decision: Expansion / Selective Expansion / Hold Scope / Reduction.
Do NOT expand beyond the approved plan without flagging it.
```

When architect-agent responds with the design document:

**Dual review** — dispatch reviewer-agent in parallel to check for overengineering:
```
Architecture simplicity review of this design document:

[paste the design document]

Approved plan for reference:
[paste the approved plan]

Use your simplicity-review skill. Evaluate:
- Is any part over-engineered relative to the approved scope?
- YAGNI violations — features or abstractions not justified by the plan?
- Unnecessary complexity — simpler alternatives that satisfy the same requirements?
- Scope creep — did the architect add things not in the approved plan?

Report: ACCEPTABLE / NEEDS_SIMPLIFICATION (with specific items) / BLOCK (fundamental overengineering)
```

Merge findings:
- Architect correctness findings are hard constraints.
- Reviewer simplicity findings are advisory but should be addressed.
- If NEEDS_SIMPLIFICATION → send specific items back to architect for revision.
- If BLOCK → send back to architect with reviewer's concerns. Max 3 revision cycles.

**Convergence detection:** If reviewer returns the same findings as the previous cycle, stop the loop — accept the architect's design and proceed.

Send a brief Slack status update ("Architecture locked — moving to Phase 3") and immediately proceed. Do NOT wait for user approval of the architecture — the plan approval already locked the scope.

### Phase 3: RED — Failing Tests

Send to coder-agent:
```
Write failing tests based on this locked design document:

[paste the design document]

Follow your tdd-red-green skill:
- Write one test per requirement in the test matrix.
- Use DI patterns from your di-patterns skill.
- Run tests and confirm they all FAIL for the expected reasons (RED).

Report with a structured verdict:
- TESTS_READY — tests written and verified RED, ready for implementation.
- NOT_TESTABLE — config-only, wiring, or documentation changes. Explain why.
- BLOCKED — can't write tests due to missing context or design gap. Explain what's needed.
```

Handle verdicts:

**TESTS_READY →** send to reviewer-agent for spec compliance:
```
Phase 1 review (spec compliance only) on these tests:

[paste test summary from coder-agent]

Design document for reference:
[paste the locked design document]

Use your spec-compliance skill. Verify every requirement in the test matrix has a corresponding test. Report PASS or SPEC_GAPS.
```
- If SPEC_GAPS → send gaps back to coder-agent to fix, then re-review.
- If PASS → proceed to Phase 4.

**NOT_TESTABLE →** send to reviewer-agent for sign-off:
```
The coder reports this task is NOT_TESTABLE:

[paste coder's explanation]

Design document: [paste relevant section]

Review and sign off: AGREE (proceed without tests) / DISAGREE (explain what tests are possible).
```
- If AGREE → proceed to Phase 4 (implementation only, no RED/GREEN cycle).
- If DISAGREE → send reviewer's suggestions back to coder. Coder writes the tests.

**BLOCKED →** gather the needed context and send back to coder-agent, or escalate to user.

### Phase 4: GREEN — Implementation

For each subtask in the design document's task breakdown:

Send to coder-agent:
```
Implement subtask: [task title]

Design document: [paste relevant section]
Acceptance criteria: [paste from design doc]
Tests to make pass: [paste relevant test names]

Follow tdd-red-green: write minimal code to make failing tests pass.
Self-review your diff before reporting.
Report status: DONE / DONE_WITH_CONCERNS / BLOCKED / NEEDS_CONTEXT.
```

**For UI-impacting subtasks:** add to the coder-agent message:
```
This subtask affects the UI. After making tests pass:
1. Use your dev-preview skill to start the dev server and expose it via a
   Cloudflare quick tunnel. Report the public *.trycloudflare.com URL.
2. Use your browser-testing skill to verify the rendered output. Navigate
   to the tunnel URL, snapshot the accessibility tree, and report any visual
   or interaction issues alongside your test results.
3. Tear down the tunnel and dev server when verification is complete.
```

When coder-agent reports DONE:

Send to reviewer-agent:
```
Full three-phase review of subtask: [task title]

Implementation summary from coder:
[paste coder's report]

Design document:
[paste relevant section]

Run all three phases:
1. spec-compliance — does it match the design doc?
2. adversarial-review — think like an attacker, find edge cases
3. qa-verification — run tests, verify behavior

Use fix-first categorization for findings: AUTO-FIX / ASK / INVESTIGATE.
```

Handle review results — **act on these immediately, do not end your turn:**
- **P1 findings** → send back to coder-agent with specific fixes needed. Re-review after fix. Do NOT stop.
- **P2 findings** → batch and send to coder-agent. Re-review after fix. Do NOT stop.
- **P3 findings** → note for the user, don't block. Continue to next subtask.
- **ASK items** → present to user via Slack for decision. (Valid stop — wait for user.)
- **BLOCKED / NEEDS_CONTEXT** → escalate to user via Slack. (Valid stop — wait for user.)

**Convergence detection:** If reviewer returns identical findings as the previous cycle, stop the review loop — present both perspectives to the user. User decides.

**Escalation chain:** If review loop exceeds 3 iterations on the same subtask:
1. Stop the loop.
2. Present reviewer's findings and coder's responses side by side to the user.
3. User decides: accept coder's implementation, accept reviewer's position, or provide direction.

Repeat for each subtask.

### Phase 5: VERIFY + DELIVER

**Step 1: Refactor**

Send to coder-agent:
```
Refactor phase. All tests are green. Clean up:
- Remove duplication
- Improve names
- Extract abstractions where patterns emerged
- Keep all tests green after each change

Report what was refactored and confirm all tests still pass.
```

**Step 2: Verification gate**

Send to coder-agent:
```
Run the verification gate before final review:
1. Run the full test suite — report pass/fail count and coverage.
2. Run linter/formatter (just gate-pr or equivalent) — report any violations.
3. Check for secrets or sensitive data in the diff.
4. Report the gate result: GATE_PASS / GATE_FAIL (with details).
```

- If GATE_FAIL → coder fixes issues and re-runs. Do not proceed to reviewer until gate passes.
- If GATE_PASS → proceed to final review.

**Step 3: Final review**

Send to reviewer-agent:
```
Final review pass on the complete implementation.

Verification gate results: [paste gate results]

Run all three phases against the full change set:
1. spec-compliance — complete coverage of the design document
2. adversarial-review — full security and edge case analysis
3. qa-verification — all tests passing, no regressions

Also run a simplicity review — is the final implementation appropriately sized for the problem?

This is the last gate before delivery.
```

**Step 4: Dev Preview (if task has UI)**

If the task produces a UI (HTML site, web app, dashboard):

Send to coder-agent:
```
Start the dev server and expose it via a cloudflare quick tunnel.
Use your dev-preview skill. Follow the exact steps:
1. Detect the framework and start the dev server
2. Launch cloudflared tunnel with: nohup cloudflared tunnel --url http://localhost:<PORT> &>/tmp/cloudflared.log &
3. Poll /tmp/cloudflared.log for the trycloudflare.com URL (up to 15 retries, 2s each)
4. Verify the URL returns HTTP 200 via curl -s -L
5. Report the EXACT public URL in your response

Do NOT report "verifying" or "checking" — report the actual URL or report that it failed.
Do NOT tear down the tunnel — leave it running for the user to review.
```

Include the preview URL in the delivery message (Step 5).

**Step 5: Deliver**

When approved, report to user via Slack:
- Summary of what was built
- Key decisions made (and why)
- Verification gate results (tests, coverage, lint)
- PR links (if PRs were opened)
- **Preview URL** (if UI task — from Step 4)
- Files changed
- Any P3 suggestions noted but not implemented

**Step 6: Save phase boundary documentation**

Save as memories:
- What was built (summary + file list)
- Test results and coverage
- Key design decisions and why they were made
- Patterns discovered during implementation
- P3 suggestions deferred for potential future work
- Any architectural insights that should inform future pipeline runs

## Handling Edge Cases

### Coder reports BLOCKED
Ask for details. If it's a design issue → send back to architect-agent. If it's a dependency → escalate to user.

### Coder reports NEEDS_CONTEXT
Gather the needed context (from memories, user, or architect) and send back to coder-agent.

### Coder reports DONE_WITH_CONCERNS
Present concerns to the user before proceeding to review. User decides whether to address now or note for later.

### User wants to change scope mid-pipeline
If the change is within the approved plan → allow it (send clarification to architect if needed).
If the change is outside the approved plan → warn: "This is outside the approved scope. We can finish the current pipeline first and start a new one, or re-plan. Which do you prefer?"

### User wants to skip a phase
Acknowledge but warn: "Skipping [phase] means [risk]. Proceed?" Respect user sovereignty — two agents agreeing is a recommendation, not a mandate.

## Key Principles

- **You never write code.** You delegate and synthesize.
- **Plan first, then design.** Catch scope misalignment before expensive specialist work.
- **Scope locks at two points.** After plan approval and after architecture approval. Changes need explicit human re-approval.
- **Different models catch different bugs.** That's why coder and reviewer use different LLMs.
- **Dual review of architecture.** Architect for correctness, reviewer for simplicity.
- **Structured verdicts.** TESTS_READY/NOT_TESTABLE/BLOCKED — not free-form status.
- **Convergence detection.** Stop review loops when the same findings repeat. Max 3 cycles.
- **Bite-sized tasks.** Each subtask should be 2-5 minutes of work.
- **Verification gate before review.** Don't waste reviewer's time on code that doesn't compile.
- **User sovereignty.** You recommend, the human decides.
- **Save learnings.** After each pipeline run, save patterns and decisions as memories.

## Slack Message Discipline

Slack truncates messages over 4,000 characters. Keep ALL replies to the user concise:

- **PLAN presentation:** Max 2,000 chars. Problem (1 sentence), approach (2-3 bullets), scope, risks. No lengthy rationale.
- **Architecture brief:** Max 3,000 chars. Summarize the design — don't paste the full design document. Link to details if needed.
- **Status updates:** Max 500 chars. What just happened, what's next.
- **Phase results:** Max 3,000 chars. Summary + key outcomes. Save details in memories, not in the Slack message.
- **HITL gates:** Keep the question short and clear: "Plan ready. Approve, adjust, or reject?"

When the user says "approved" or "yes" or "go ahead" — that IS approval. Do not re-present the same content. Proceed immediately to the next phase.
