---
name: adversarial-research-pipeline
description: Use this skill when asked to research a topic, analyze articles/newsletters, write a report, do competitive intelligence, or synthesize information from multiple sources. Orchestrates a structured research pipeline across architect-agent (planning), parallel workers (data gathering), and reviewer-agent (adversarial quality gate) using send_agent_message delegation. IMPORTANT — this is a CHANNEL-LEVEL orchestration skill. Execute these phases yourself using send_agent_message and branch. Do NOT spawn a single worker to run the entire pipeline.
---

# Adversarial Research Pipeline

**⚠️ CHANNEL-LEVEL SKILL.** Execute this pipeline yourself (the channel). Use `branch` for thinking, `send_agent_message` for delegation to specialists. Do NOT spawn a single worker to handle the entire task — workers lack `send_agent_message` and cannot orchestrate.

## Pipeline Overview

```
INTAKE → PLAN [HITL] → RESEARCH (parallel) → CHALLENGE (adversarial) → WRITE → DELIVER
```

Six phases. User approves the plan at Phase 1, then the pipeline runs autonomously.

## Agents & Roles

| Agent | Role | Key Skills |
|-------|------|------------|
| **default-agent** (you) | Orchestrator — intake, dispatch, synthesis, delivery | this skill |
| **architect-agent** | Research planner — decompose topic, discover perspectives | research-planner |
| **reviewer-agent** | Adversarial critic — fact-check, find gaps, detect bias | research-critic |
| **google-agent** | Research specialist — academic papers, Google Workspace output | arxiv, paper-search, google-workspace |

## Depth Levels

| Level | When to Use | Review Cycles | Typical Time |
|-------|------------|---------------|-------------|
| **quick-scan** | Simple lookups, single-source summaries | 0 (skip review) | 2-5 min |
| **standard** | Multi-source research, reports | 1 cycle | 10-20 min |
| **deep-dive** | Competitive intel, literature reviews, proposals | 2 cycles | 20-40 min |

Default to **standard** unless the user specifies otherwise or the request is clearly a quick lookup.

## Phase 0: INTAKE

Branch to understand the research request. Classify and extract:

```
Research Type: [newsletter-analysis | competitive-intel | literature-review | report-generation | general-research]
Sources: [LinkedIn, web, papers, YouTube, specific URLs, ...]
Output Format: [slack-summary | gist | google-doc | google-slides | google-sheets]
Depth: [quick-scan | standard | deep-dive]
Key Question: [1 sentence — what are we trying to answer?]
```

### Source-to-Tool Mapping

| Source Type | MCP Tool | Agent |
|-------------|----------|-------|
| LinkedIn articles/posts | `linkedin` MCP → `fetcher` MCP (for full article text) | any agent |
| Web pages, blogs | `searxng` MCP (search) → `fetcher` MCP (read) | any agent |
| PDFs, whitepapers | `pdf-reader` MCP | any agent |
| Academic papers | `arxiv` MCP, `paper-search` MCP | google-agent |
| YouTube videos | `youtube` MCP | google-agent |
| RSS feeds | `rss-feeds` MCP | default-agent |
| Google Drive files | `google-workspace` MCP | google-agent |

### Quick-Scan Shortcut

If depth = quick-scan AND sources ≤ 2 AND output = slack-summary:
- Skip Phases 1, 3 (no planning, no adversarial review)
- Branch to do the research directly
- Deliver the summary
- Done

## Phase 1: PLAN [HITL Gate]

Delegate to architect-agent with the research-planner skill:

```
send_agent_message to architect-agent:

"Use your research-planner skill.

Research request: [user's full request]
Research type: [classified type from intake]
Key question: [1 sentence]

Produce a structured research plan with:
- Perspectives to explore (2-4 expert viewpoints)
- Subtopics decomposed for parallel research
- Sources to consult per subtopic
- Output format recommendation
- Explicit out-of-scope boundaries"
```

When architect responds with the plan, present it to the user:

```
📋 **Research Plan**

**Question:** [key question]
**Type:** [research type]
**Depth:** [level]

**Perspectives:**
1. [Expert viewpoint 1]
2. [Expert viewpoint 2]
3. [Expert viewpoint 3]

**Research threads (parallel):**
1. [Subtopic 1] — sources: [tools/sources]
2. [Subtopic 2] — sources: [tools/sources]
3. [Subtopic 3] — sources: [tools/sources]

**Output:** [format]
**Not researching:** [explicit exclusions]

Approve, adjust, or reject?
```

**HITL gate:** Wait for user approval. Do NOT proceed without it.

Once approved, scope is locked. Proceed immediately — do not stop or ask again.

## Phase 2: RESEARCH (Parallel Workers)

Spawn one worker per research thread from the plan. Each worker gets:
- Its specific subtopic
- Which MCP tools to use
- What to extract and how to structure findings
- The perspective it should adopt (from the plan)

### Worker Prompt Template

```
Research subtopic: [subtopic]
Perspective: [expert viewpoint to adopt]
Sources to use: [specific MCP tools]

Instructions:
1. Use [MCP tool] to search/fetch the relevant content
2. Extract: key findings, direct quotes with attribution, source URLs
3. Adopt the perspective of [expert viewpoint] — what would this expert notice?
4. Structure your findings as:

SUBTOPIC: [name]
PERSPECTIVE: [viewpoint]
KEY FINDINGS:
- [finding 1] (source: [URL/title])
- [finding 2] (source: [URL/title])
DIRECT QUOTES:
- "[quote]" — [source]
GAPS: [what couldn't be found or needs deeper investigation]
```

### Worker Dispatch Rules

- **LinkedIn content:** Worker uses `linkedin` MCP to find posts/articles → `fetcher` MCP to read full text
- **Web research:** Worker uses `searxng` MCP to search → `fetcher` MCP to read pages
- **Academic papers:** Delegate to google-agent via `send_agent_message` (it has `arxiv` and `paper-search` MCPs)
- **PDF analysis:** Worker uses `pdf-reader` MCP directly
- **YouTube content:** Delegate to google-agent via `send_agent_message` (it has `youtube` MCP)
- **Google Drive content:** Delegate to google-agent via `send_agent_message`

### Collecting Results

As workers complete, collect their structured findings. When all workers are done (or after 5-minute timeout per worker), proceed to Phase 3.

If a worker fails or times out, note the gap and proceed — the reviewer will catch missing coverage.

## Phase 3: CHALLENGE (Adversarial Quality Gate)

**Skip this phase for quick-scan depth.**

### Step 3a: Synthesis Branch

Branch to synthesize all worker findings into a draft:
- Merge findings across subtopics
- Identify themes, patterns, contradictions
- Note source attribution for every claim
- Flag areas where evidence is thin

### Step 3b: Adversarial Review

Delegate to reviewer-agent with the research-critic skill:

```
send_agent_message to reviewer-agent:

"Use your research-critic skill.

Research question: [key question]
Research plan: [the approved plan from Phase 1]
Draft synthesis: [the draft from Step 3a]

Review this research for:
1. Fact verification — are claims supported by cited sources?
2. Perspective gaps — what viewpoints are missing?
3. Bias detection — is the synthesis one-sided or cherry-picked?
4. Source quality — primary vs secondary, recency, authority

Return a structured verdict: PASS | NEEDS_REVISION | NEEDS_MORE_RESEARCH"
```

### Step 3c: Handle Verdict

**PASS** → Proceed to Phase 4

**NEEDS_REVISION** → Branch to address reviewer's specific findings, then re-submit for review. Max 2 revision cycles.

**NEEDS_MORE_RESEARCH** → Spawn additional workers for the missing perspectives/sources identified by reviewer, collect results, re-synthesize, re-review. Max 1 additional research round.

### Convergence Detection

If reviewer returns the same findings as the previous cycle, stop the loop. Accept the current synthesis and proceed to Phase 4 with reviewer's concerns noted in the output.

Max total review cycles: 2 (same as coding pipeline).

## Phase 4: WRITE (Compilation)

Spawn a worker to compile the final output. Format depends on the output format from the plan:

### Slack Summary (≤4000 chars)
```
📊 **[Research Title]**

**Key Question:** [1 sentence]

**Findings:**
• [Key finding 1]
• [Key finding 2]
• [Key finding 3]

**[Per-article/per-source summaries if applicable]**

**Synthesis:** [1-2 paragraph big picture]

**Recommendations:** [if applicable]
• [Recommendation 1]
• [Recommendation 2]

Sources: [count] sources consulted
```

### Gist (Full Report)
Use the `github-ops` skill (via `gh gist create`) to create a markdown gist with:
- Executive summary
- Detailed findings per subtopic
- Source-by-source analysis
- Synthesis and recommendations
- Full source list with URLs

### Google Doc / Slides / Sheets
Delegate to google-agent via `send_agent_message`:
- **Google Doc:** Formatted report with headings, citations, and executive summary
- **Google Slides:** Presentation deck — 1 title slide + 1 slide per key finding + 1 synthesis slide + 1 recommendations slide
- **Google Sheets:** Data tables, comparisons, structured data

## Phase 5: DELIVER

1. Send the output to the user via the appropriate channel
2. If output was a Gist/Doc/Slides, include the link in the Slack message
3. Save research memories:
   ```
   memory_save: {
     type: "research",
     content: "[topic]: [key findings summary]. Sources: [count]. Output: [format + link]",
     importance: 0.7
   }
   ```

## Orchestration Principles

- **You never do the research yourself.** You delegate to workers and specialists.
- **Plan first, then research.** The architect decomposes; workers execute.
- **Scope locks after approval.** The user-approved plan defines what's in and out.
- **Different models catch different things.** Workers gather data; reviewer finds gaps and bias.
- **Parallel by default.** Multiple research threads run simultaneously.
- **Structured findings.** Workers return structured data, not free-form text.
- **Convergence detection.** Stop review loops when findings repeat. Max 2 cycles.
- **Time-bounded.** 5-minute timeout per worker. Don't let one slow source block everything.
- **Save learnings.** After each pipeline run, save key findings as memories for future recall.
- **User sovereignty.** You recommend depth and format; the human decides.

## Example: LinkedIn Newsletter Analysis

User: "Read all articles on TechEd LinkedIn newsletter, summarize each, synthesize the big idea"

```
INTAKE: newsletter-analysis, LinkedIn source, standard depth, slack-summary + google-doc

PLAN: architect decomposes into:
  Thread 1: Fetch newsletter article list (linkedin MCP)
  Thread 2: Read and summarize each article (fetcher MCP)
  Thread 3: Research TechEd concept context (searxng + fetcher)
  Perspectives: [higher-ed admin, instructional designer, enterprise IT]

RESEARCH: 3 workers in parallel → structured findings

CHALLENGE: reviewer checks summaries for accuracy, identifies missing perspectives

WRITE: worker compiles Slack summary + google-agent creates Google Doc

DELIVER: Slack message with summary + Doc link, save research memory
```
