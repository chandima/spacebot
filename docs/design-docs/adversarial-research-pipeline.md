# Adversarial Research Pipeline

A structured multi-agent research pipeline that decomposes research tasks into parallel workers, applies adversarial quality review, and delivers formatted output. Modeled after the adversarial coding pipeline but adapted for knowledge work.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ DEFAULT-AGENT (Orchestrator)                                │
│ Skill: adversarial-research-pipeline                        │
│ Phases: INTAKE → PLAN → RESEARCH → CHALLENGE → WRITE → DELIVER │
└────────┬──────────────┬──────────────┬──────────────────────┘
         │              │              │
  ARCHITECT-AGENT  WORKERS (parallel)  REVIEWER-AGENT
  (Research Planner) (Data Gatherers)  (Adversarial Critic)
                                       
  Skills:             MCP Tools:       Skills:
  - research-planner  - searxng        - research-critic
                      - linkedin       
                      - fetcher-mcp    
                      - arxiv-mcp      
                      - pdf-reader-mcp 
                      - paper-search   
                      - rss-feeds      
                      - youtube        
```

## Pipeline Phases

### Phase 0: INTAKE
Channel branches to classify the research request:
- **Type**: newsletter-analysis, competitive-intel, literature-review, report-generation, general-research
- **Sources**: Which MCP tools and data sources to use
- **Output format**: slack-summary, gist, google-doc, google-slides, google-sheets
- **Depth level**: quick-scan, standard, deep-dive

### Phase 1: PLAN [HITL Gate]
Default-agent delegates to architect-agent (research-planner skill) to produce a structured research plan with:
- Multi-perspective discovery (STORM-inspired — 2-4 expert viewpoints)
- Parallelizable research threads with specific MCP tool assignments
- Output format recommendation
- Explicit out-of-scope boundaries

User approves, adjusts, or rejects the plan before proceeding.

### Phase 2: RESEARCH (Parallel Workers)
One worker per research thread. Each worker:
- Uses assigned MCP tools to gather information
- Adopts a specific expert perspective
- Returns structured findings: key findings, direct quotes, source URLs, gaps

### Phase 3: CHALLENGE (Adversarial Quality Gate)
Default-agent synthesizes worker findings into a draft, then delegates to reviewer-agent (research-critic skill) for adversarial review:

1. **Fact Verification** — Are claims supported by cited sources?
2. **Perspective Gap Analysis** — What viewpoints are missing?
3. **Bias Detection** — Confirmation bias, selection bias, survivorship bias?
4. **Source Quality Assessment** — Primary vs secondary, recency, diversity

Verdict: PASS → proceed | NEEDS_REVISION → fix and re-review | NEEDS_MORE_RESEARCH → spawn additional workers

Max 2 revision cycles (convergence detection stops the loop if same findings repeat).

### Phase 4: WRITE
Worker compiles final output in the specified format:
- **Slack summary**: ≤4000 chars, key takeaways, source count
- **Gist**: Full markdown report via `gh gist create` (github-ops skill)
- **Google Doc/Slides/Sheets**: Via google-workspace MCP on google-agent

### Phase 5: DELIVER
Send output to user, include links to Gist/Doc/Slides, save research findings as memories.

## Depth Levels

| Level | Review Cycles | Use Case |
|-------|---------------|----------|
| **quick-scan** | 0 (skip review) | Simple lookups, single-source summaries |
| **standard** | 1 cycle | Multi-source research, reports |
| **deep-dive** | 2 cycles | Literature reviews, competitive intel, proposals |

## Research Design Influences

- **Stanford STORM** (28K⭐): Multi-perspective pre-writing — simulating expert viewpoints to improve depth/breadth
- **GPT Researcher** (26K⭐): Parallel subtopic research with review/revise adversarial loops
- **MAD Framework** (EMNLP 2024): Multi-Agent Debate — adversarial positions break "Degeneration-of-Thought" (single LLM can't self-correct once confident)
- **Superpowers** (obra/superpowers): Context isolation — subagents get precisely crafted context, never full session history

## MCP Servers Used

| Server | Purpose | Agents |
|--------|---------|--------|
| `searxng` | Web search | All agents |
| `linkedin` | LinkedIn posts, articles, profiles | All agents |
| `fetcher` | Web page → clean markdown (Playwright) | All agents |
| `pdf-reader` | PDF text extraction | All agents |
| `arxiv` | arXiv paper search & analysis | google-agent |
| `paper-search` | Multi-source academic search (8 databases) | google-agent |
| `rss-feeds` | RSS/newsletter monitoring | default-agent |
| `youtube` | YouTube video search & transcripts | google-agent |
| `google-workspace` | Google Drive, Docs, Slides, Sheets | google-agent |

## Skills

| Skill | Agent | Purpose |
|-------|-------|---------|
| `adversarial-research-pipeline` | default-agent | Pipeline orchestration (INTAKE → DELIVER) |
| `research-planner` | architect-agent | Research decomposition, perspective discovery |
| `research-critic` | reviewer-agent | Fact verification, bias detection, gap analysis |

## Example Workflows

### LinkedIn Newsletter Analysis
```
User: "Read all articles on TechEd LinkedIn newsletter, summarize each"

INTAKE: newsletter-analysis, LinkedIn source, standard depth
PLAN: architect decomposes → 3 threads (article list, per-article summaries, context research)
RESEARCH: 3 workers in parallel using linkedin + fetcher MCPs
CHALLENGE: reviewer checks summary accuracy, identifies missing perspectives
WRITE: Slack summary + Google Doc
DELIVER: Slack message with summary + Doc link
```

### Competitive Intelligence
```
User: "Research how Anthropic, OpenAI, and Google approach AI safety"

INTAKE: competitive-intel, web + papers, deep-dive depth
PLAN: architect discovers 4 perspectives (researcher, regulator, ethicist, customer)
RESEARCH: 4 workers — one per company + one for regulatory context
CHALLENGE: 2 review cycles — reviewer checks for bias toward any one company
WRITE: Google Doc with executive summary + comparison table
DELIVER: Slack summary + Doc link
```

### Academic Literature Review
```
User: "What's the latest research on multi-agent debate for improving LLM reasoning?"

INTAKE: literature-review, papers + web, standard depth
PLAN: architect decomposes into theory, benchmarks, practical applications
RESEARCH: workers use arxiv + paper-search MCPs via google-agent
CHALLENGE: reviewer checks source recency, methodology quality
WRITE: Gist with full citations
DELIVER: Slack summary + Gist link
```
