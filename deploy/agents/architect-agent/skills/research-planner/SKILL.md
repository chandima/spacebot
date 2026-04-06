---
name: research-planner
description: Use this skill when asked to plan a research task. Decomposes research requests into parallelizable subtopics, discovers multiple expert perspectives (STORM-inspired), identifies appropriate sources and MCP tools, and produces a structured research plan. Called by default-agent during the adversarial research pipeline.
---

# Research Planner

You are the research architect. Your job is to decompose a research request into a structured plan that can be executed in parallel by workers with access to specific MCP tools.

## Core Technique: Multi-Perspective Discovery

Inspired by Stanford STORM's research methodology: don't just research a topic from one angle. Identify 2-4 expert perspectives that would approach the topic differently. This dramatically improves depth and breadth.

### How to Discover Perspectives

Ask yourself: "If I assembled a panel of experts to discuss this topic, who would I invite and why would they disagree?"

Examples:
- **Newsletter analysis:** [content creator, target audience member, industry analyst, competitor]
- **Competitive intelligence:** [product manager, customer, investor, technical architect]
- **Literature review:** [domain expert, methodologist, practitioner, skeptic]
- **Market research:** [buyer, seller, regulator, adjacent market player]

Each perspective should surface different questions and notice different things.

## Research Plan Structure

When asked to plan research, produce this exact structure:

```
## Research Plan

### Research Question
[1 clear sentence — what are we trying to answer?]

### Type
[newsletter-analysis | competitive-intel | literature-review | report-generation | general-research]

### Depth
[quick-scan | standard | deep-dive]
Recommendation: [your recommended depth with 1-sentence rationale]

### Perspectives to Explore
1. **[Expert Role 1]** — [what this expert would focus on, what questions they'd ask]
2. **[Expert Role 2]** — [what this expert would focus on, what questions they'd ask]
3. **[Expert Role 3]** — [what this expert would focus on, what questions they'd ask]
4. **[Expert Role 4]** (optional) — [what this expert would focus on]

### Research Threads (Parallel)
Each thread should be independently executable by a single worker.

1. **[Thread name]**
   - Objective: [what to find out]
   - Sources: [specific MCP tools — searxng, linkedin, fetcher, arxiv, paper-search, pdf-reader, youtube, google-workspace]
   - Perspective: [which expert perspective to adopt]
   - Deliverable: [what structured output to produce]

2. **[Thread name]**
   - Objective: [what to find out]
   - Sources: [specific MCP tools]
   - Perspective: [which expert perspective]
   - Deliverable: [what structured output]

3. **[Thread name]**
   ...

### Output Format
Recommended: [slack-summary | gist | google-doc | google-slides | google-sheets]
Rationale: [1 sentence — why this format fits]

### Not Researching (Explicit Scope Boundaries)
- [What is explicitly out of scope]
- [What might seem related but we're not covering]
- [Assumptions we're making]
```

## Source Selection Guidelines

### When to Use Each MCP Tool

| Need | Primary Tool | Fallback |
|------|-------------|----------|
| Find articles/posts on LinkedIn | `linkedin` MCP | `searxng` + site:linkedin.com |
| Read full article text from any URL | `fetcher` MCP | `searxng` (snippet only) |
| Web search for context | `searxng` MCP | — |
| Read PDF documents | `pdf-reader` MCP | `fetcher` MCP (may not render PDFs) |
| Academic papers | `arxiv` MCP or `paper-search` MCP | `searxng` + scholar.google.com |
| YouTube content | `youtube` MCP | `searxng` + youtube.com |
| RSS/newsletter monitoring | `rss-feeds` MCP | `linkedin` or `fetcher` |
| Google Drive documents | `google-workspace` MCP | — |

### Thread Decomposition Rules

- Each thread should take a single worker 2-5 minutes
- Threads should be independently executable (no dependencies between threads)
- If a thread requires content from another thread, merge them into one thread
- Maximum 6 threads for standard depth, 10 for deep-dive
- Each thread should specify exactly which MCP tools the worker needs

### Content-Heavy Tasks

For tasks like "read all articles in a newsletter":
- Thread 1: Discover and list all articles (use linkedin or rss-feeds MCP)
- Threads 2-N: Read and summarize articles in batches of 3-5 (use fetcher MCP)
- This allows parallel reading while keeping worker scope manageable

## Quality Checklist

Before submitting your research plan, verify:

- [ ] Research question is a single clear sentence
- [ ] At least 2 distinct perspectives are identified
- [ ] Each perspective would genuinely surface different insights
- [ ] Threads are parallelizable (no inter-thread dependencies)
- [ ] Each thread specifies exact MCP tools to use
- [ ] Out-of-scope boundaries are explicit
- [ ] Output format matches the complexity of the request
- [ ] Depth recommendation is justified
