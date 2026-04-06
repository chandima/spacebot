---
name: agent-capabilities
description: |
  Reference map of all sub-agent skills, MCP tools, and capabilities.
  Use this to decide which agent to delegate to for any task — not just coding.
  Triggers: any delegation decision, "who can do X", routing a non-coding task,
  creating gists, inspecting websites, running security scans, research tasks.
---

# Agent Capabilities Map

Use this reference when deciding which agent to delegate a task to via `send_agent_message`. Every agent has specialized skills and MCP tools — route work to the agent best equipped for it.

## Agent Inventory

### architect-agent (GPT-5.4)

**Role:** Problem framing, scope review, architecture lock, reference app inspection.

| Capability | Skill/Tool | When to Use |
|-----------|-----------|-------------|
| Problem framing | `office-hours` | Starting any new feature or project — 6 forcing questions |
| Design documents | `architecture-lock` | Producing locked design docs with data flow, state machines, test matrices |
| Website inspection | `chrome-devtools` (MCP) | Inspecting reference URLs, analyzing page structure, taking screenshots |
| Accessibility audit | `a11y-debugging` | Evaluating semantic HTML, ARIA, focus states, contrast |
| Web search | `searxng` (MCP) | Searching the web for research, examples, documentation |

**Best for:** "Look at this site and tell me how it's built", "Design a system that does X", "Frame this problem before we build it."

---

### coder-agent (GPT-5.3-Codex)

**Role:** TDD implementation, GitHub operations, coding in project directories via OpenCode workers.

| Capability | Skill/Tool | When to Use |
|-----------|-----------|-------------|
| Test-driven development | `tdd-red-green` | Writing failing tests first, then making them pass |
| Dependency injection | `di-patterns` | Structuring code for testability |
| Review feedback triage | `fix-first` | Processing reviewer findings (AUTO-FIX / ASK / INVESTIGATE) |
| GitHub operations | `github-ops` | PRs, branches, commits, issues, **gists**, code search, releases, actions |
| Website verification | `chrome-devtools` (MCP) | Verifying UI output after implementation |
| Accessibility check | `a11y-debugging` | Checking a11y of implemented UI |
| Web search | `searxng` (MCP) | Looking up documentation, examples |

**GitHub-ops details (via scripts):**

| Domain | Script | Key Operations |
|--------|--------|---------------|
| Gists | `gists.sh` | `create`, `view`, `edit`, `delete`, `fork`, `list` |
| Repos | `repos.sh` | `view`, `clone`, `fork`, `create`, `contents` |
| PRs | `prs.sh` | `list`, `view`, `create`, `merge`, `diff`, `checks` |
| Issues | `issues.sh` | `list`, `view`, `create`, `close`, `comment` |
| Actions | `actions.sh` | `runs`, `jobs`, `logs`, `artifacts`, `rerun` |
| Releases | `releases.sh` | `list`, `view`, `create`, `download` |
| Search | `search.sh` | `repos`, `code`, `issues`, `prs`, `users`, `commits` |

**Gist workflow:** The coder can create a gist, clone it to a working directory, develop in it, commit, and push — treating a gist as a mini-repo.

**Best for:** "Create a gist with X", "Implement this feature", "Open a PR", "Write tests for X", "Clone this and work on it."

---

### reviewer-agent (Kimi-K2.5)

**Role:** Three-phase adversarial review, security scanning, production hardening.

| Capability | Skill/Tool | When to Use |
|-----------|-----------|-------------|
| Spec compliance | `spec-compliance` | Verifying implementation matches design doc |
| Adversarial review | `adversarial-review` | Attack surface analysis, race conditions, edge cases |
| QA verification | `qa-verification` | Running tests, verifying behavior, regression checks |
| Code quality | `code-quality` | SOLID principles, naming, structure, documentation |
| Security scanning | `security-auditor` | Trivy + Semgrep scans, CRITICAL/HIGH/MEDIUM findings |
| Resilience audit | `production-hardening` | Missing retries, timeouts, circuit breakers |
| GitHub operations | `github-ops` | Review PR diffs, check CI, post review comments |
| Website verification | `chrome-devtools` (MCP) | Visual/behavioral verification of web output |
| Accessibility audit | `a11y-debugging` | Full a11y audit of web interfaces |
| Web search | `searxng` (MCP) | Looking up security advisories, best practices |

**Best for:** "Review this code", "Run a security audit", "Check if this is production-ready", "Audit accessibility."

---

### slack-agent (GPT-5-mini)

**Role:** Enterprise Slack search, conversation history, user lookup, channel monitoring.

| Capability | Skill/Tool | When to Use |
|-----------|-----------|-------------|
| Slack search | `enterprise-slack` (MCP) | Searching messages, channels, threads |
| User lookup | `enterprise-slack` (MCP) | Finding Slack users and their activity |
| Channel history | `enterprise-slack` (MCP) | Reading conversation history from channels |

**Best for:** "Search Slack for X", "What did Y say about Z", "Find messages about this topic."

---

### notebooklm-agent (GPT-5-mini)

**Role:** Research, knowledge management, content generation via Google NotebookLM.

| Capability | Skill/Tool | When to Use |
|-----------|-----------|-------------|
| Notebook management | `notebooklm` (MCP) | Create/list/manage notebooks |
| Source management | `notebooklm` (MCP) | Add URLs, text, Drive files as sources |
| Content generation | `notebooklm` (MCP) | Generate audio, video, slides, infographics |
| Note management | `notebooklm` (MCP) | Create/update/delete notes in notebooks |
| Research queries | `notebooklm` (MCP) | Ask questions against notebook sources |

**Best for:** "Research this topic", "Create a podcast about X", "Summarize these sources."

---

## Common Delegation Patterns

### "Build something that looks like [URL]"
1. → **architect-agent**: Inspect URL with `chrome-devtools`, take screenshots, analyze structure. Produce design doc.
2. → **coder-agent**: Implement based on design doc. Use `chrome-devtools` to verify visual output.
3. → **reviewer-agent**: Review implementation, run security + a11y audits.

### "Create a gist/document/proposal about X"
1. → **architect-agent** (if research needed): Inspect references, frame the problem.
2. → **coder-agent**: Create gist via `github-ops gists.sh create`, clone to `~/Documents/Projects/`, develop content as a coding effort in that directory.
3. → **reviewer-agent** (if review needed): Review the document quality/accuracy.

### "Search Slack for X and summarize"
1. → **slack-agent**: Search with enterprise Slack MCP.
2. → Branch (yourself): Synthesize and summarize results.

### "Research X and create a notebook"
1. → **notebooklm-agent**: Create notebook, add sources, generate content.

### "Is this code production-ready?"
1. → **reviewer-agent**: Run `security-auditor` + `production-hardening` + full review phases.

### "Open a PR / create an issue / check CI"
1. → **coder-agent**: Use `github-ops` for any GitHub operation.

## Project Bootstrapping

When a task requires creating a new project or artifact:

**Default projects location:** `~/Documents/Projects/`

**Gist-as-project workflow:**
1. Coder creates the gist via `github-ops gists.sh create`
2. Coder clones it: `gh gist clone <gist-id> ~/Documents/Projects/<project-name>`
3. Coder spawns OpenCode worker in that directory to develop content
4. Changes are committed and pushed back to the gist
5. Gist URL is reported back to the user

**Repo-as-project workflow:**
1. Coder creates repo via `github-ops repos.sh create`
2. Coder clones it to `~/Documents/Projects/`
3. Full adversarial pipeline runs against that directory

## CLI Tool Paths

All agents run under launchd. These absolute paths are available:

| Tool | Path |
|------|------|
| `gh` | `/opt/homebrew/bin/gh` |
| `git` | `/opt/homebrew/bin/git` |
| `node` | `/opt/homebrew/bin/node` |
| `npx` | `/opt/homebrew/bin/npx` |
| `trivy` | `/opt/homebrew/bin/trivy` |
| `semgrep` | `/opt/homebrew/bin/semgrep` |
