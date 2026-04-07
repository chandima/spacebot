# AGENTS.md

Implementation guide for coding agents working on Spacebot. Read `RUST_STYLE_GUIDE.md` before writing any code.

## What Spacebot Is

A Rust agentic system where every LLM process has a dedicated role and delegation is the only way work gets done. It replaces the monolithic session model (one LLM thread doing conversation + thinking + tool execution + memory retrieval + compaction) with specialized processes that only do one thing.

Single binary. No server dependencies. Runs on tokio. All data lives in embedded databases in a local data directory.

**Stack:** Rust (edition 2024), tokio, Rig (v0.30.0, agentic loop framework), SQLite (sqlx), LanceDB (embedded vector + FTS), redb (embedded key-value).

## JavaScript Tooling (Critical)

- For UI work in `spacebot/interface/`, use `bun` for all JS/TS package management and scripts.
- **NEVER** use `npm`, `pnpm`, or `yarn` in this repo unless the user explicitly asks for one.
- Standard commands:
  - `bun install`
  - `bun run dev`
  - `bun run build`
  - `bun run test`
  - `bunx <tool>` (instead of `npx <tool>`)

## Migration Safety

- **NEVER edit an existing migration file in place** once it has been committed or applied in any environment.
- Treat migration files as immutable; modifying historical migrations causes checksum mismatches and can block startup.
- For schema changes, always create a new migration with a new timestamp/version.

## Delivery Gates (Mandatory)

Run these checks in this order for code changes before pushing or updating a PR:

1. `just preflight` — validate git/remote/auth state and avoid push-loop churn.
2. `just gate-pr` — enforce formatting, compile checks, migration safety, lib tests, and integration test compile.

If `just` is unavailable, run the equivalent scripts directly in the same order: `./scripts/preflight.sh` then `./scripts/gate-pr.sh`.

Additional rules:

- If the same command fails twice in one session, stop rerunning it blindly. Capture root cause and switch strategy.
- For every external review finding marked P1/P2, add a targeted verification command in the final handoff.
- For changes in async/stateful paths (worker lifecycle, cancellation, retrigger, recall cache behavior), include explicit race/terminal-state reasoning in the PR summary and run targeted tests in addition to `just gate-pr`.
- Do not push if any gate is red.

## Release Build & Deploy

This is an M1 Mac with 8GB RAM. Builds are resource-constrained. Follow these steps:

**Prerequisites:**
- `sccache` must be installed (`brew install sccache`)
- `RUSTC_WRAPPER` must be set: `export RUSTC_WRAPPER="$(which sccache)"` (in `~/.zshrc`)
- LTO is disabled in `Cargo.toml` (`lto = false`) to reduce build time and RAM usage

**Deploy command (preferred):**
```bash
just deploy
```

This single command builds a release binary, ad-hoc codesigns it (macOS), copies it to `~/.local/bin/spacebot`, and restarts the launchd service. Always use this instead of manual steps.

**Manual deploy (if `just` is unavailable):**
```bash
export RUSTC_WRAPPER="$(which sccache)"
cargo build --release
cp target/release/spacebot ~/.local/bin/spacebot
codesign -s - -f ~/.local/bin/spacebot   # Required — sccache builds invalidate code signature
~/.local/bin/spacebot restart
```

**⚠️ Never skip the `codesign` step on macOS.** Without it, macOS will SIGKILL the binary on launch with `Code Signature Invalid`, causing a crash loop under launchd.

**Build times (baseline):**
- Cold build (no sccache cache): ~25 min
- Incremental (single file change, warm cache): should be significantly faster
- The binary is at `~/.local/bin/spacebot` (not `/usr/local/bin` — no sudo needed)

**Verify sccache is working:**
```bash
sccache --show-stats  # Should show non-zero compile requests and cache hits after a build
```

## Daemon Management

Spacebot runs as a daemon managed by macOS launchd. The binary includes built-in subcommands for lifecycle management.

**CLI subcommands:**
```bash
~/.local/bin/spacebot start    # Start the daemon (writes PID to ~/.spacebot/spacebot.pid)
~/.local/bin/spacebot stop     # Stop the daemon gracefully
~/.local/bin/spacebot restart  # Stop + start (preferred way to restart after deploy)
~/.local/bin/spacebot status   # Check if the daemon is running
```

**Key paths:**
| Path | Purpose |
|------|---------|
| `~/.local/bin/spacebot` | Installed binary |
| `~/.spacebot/config.toml` | Running configuration (agent definitions, tokens, bindings) |
| `~/.spacebot/spacebot.pid` | PID file for the running daemon |
| `~/.spacebot/spacebot.sock` | Unix socket for IPC / CLI commands |
| `~/.spacebot/logs/spacebot.log.YYYY-MM-DD` | Daily-rotated structured logs |
| `~/.spacebot/agents/{agent-id}/data/spacebot.db` | Per-agent SQLite database |
| `~/.spacebot/agents/{agent-id}/identity/` | Per-agent SOUL.md, IDENTITY.md, ROLE.md |
| `~/Library/LaunchAgents/com.spacebot.agent.plist` | launchd service definition |

**API (when running):**
- Base URL: `http://127.0.0.1:19898/api/`
- Health: `GET /api/health` → `{"status":"ok"}`
- Agents: `GET /api/agents` — list all agents with config
- Messaging: `GET /api/messaging/status` — adapter status per platform
- MCP: `GET /api/mcp/status` — MCP server status per agent
- Cron: `GET /api/agents/cron?agent_id=X` — list cron jobs with execution stats

**Troubleshooting a stuck daemon:**
```bash
# If `spacebot restart` fails:
~/.local/bin/spacebot stop
ps aux | grep spacebot           # Find lingering PIDs
kill -9 <PID>                    # Force kill
rm -f ~/.spacebot/spacebot.pid ~/.spacebot/spacebot.sock
~/.local/bin/spacebot start
```

**After deploying code changes, always restart the daemon.** The running binary at `~/.local/bin/spacebot` is the OLD build until you copy the new binary and restart.

## Config Backup & Restore

Agent configs, identity files, skills, and the launchd plist are version-controlled in `deploy/`. This is the canonical backup — restore from here if the machine is wiped.

**What's backed up:**

| Path | Contents |
|------|----------|
| `deploy/config.toml.template` | Full config with secrets replaced by `${PLACEHOLDER}` vars |
| `deploy/agents/{agent-id}/` | IDENTITY.md, ROLE.md, SOUL.md per agent |
| `deploy/agents/{agent-id}/skills/` | All agent skills (SKILL.md + scripts + configs) |
| `deploy/com.spacebot.agent.plist` | launchd service definition (GH_TOKEN redacted) |

**Secrets handling:** All secrets in `deploy/` are redacted to `${VAR}` placeholders. Never commit real tokens. The placeholder variables are:
- `${SLACK_BOT_TOKEN}`, `${SLACK_APP_TOKEN}`, `${SLACK_BROWSER_TOKEN}`, `${SLACK_BROWSER_COOKIE}`
- `${GH_TOKEN}`
- `${GOOGLE_OAUTH_CLIENT_ID}`, `${GOOGLE_OAUTH_CLIENT_SECRET}`
- Provider API keys: `${ANTHROPIC_API_KEY}`, `${OPENAI_API_KEY}`, etc.

**Updating the backup after config changes:**
```bash
# Copy live config (redact secrets manually or via script)
cp ~/.spacebot/config.toml deploy/config.toml.template
# Edit to replace secrets with ${PLACEHOLDERS}

# Copy agent identity/skill files
cp -r ~/.spacebot/agents/*/identity/* deploy/agents/*/  # identity files
cp -r ~/.spacebot/agents/*/skills/* deploy/agents/*/skills/  # skills

# Commit (force-add needed — agents/ is gitignored)
git add -f deploy/ && git commit -m "deploy: update config backup"
```

**Restoring to a new machine:**
```bash
# 1. Copy identity files
for agent in deploy/agents/*/; do
  id=$(basename "$agent")
  mkdir -p ~/.spacebot/agents/$id/identity ~/.spacebot/agents/$id/skills
  cp "$agent"/*.md ~/.spacebot/agents/$id/identity/ 2>/dev/null
  cp -r "$agent"/skills/* ~/.spacebot/agents/$id/skills/ 2>/dev/null
done

# 2. Copy config template and fill in secrets
cp deploy/config.toml.template ~/.spacebot/config.toml
# Edit ~/.spacebot/config.toml — replace all ${PLACEHOLDERS} with real values

# 3. Install launchd plist
cp deploy/com.spacebot.agent.plist ~/Library/LaunchAgents/
# Edit to set real GH_TOKEN

# 4. Start
~/.local/bin/spacebot start
```

**What is NOT backed up to git** (use Time Machine for these):
- `~/.spacebot/agents/*/data/` — SQLite databases (conversations, memories, cron)
- `~/.spacebot/agents/*/data/lance/` — LanceDB vector embeddings
- `~/.spacebot/data/` — redb key-value store (secrets, settings)

## Deployed Agent Topology

The deployed instance (`~/.spacebot/config.toml`) defines a multi-agent system with a hub-and-spoke topology. The `default-agent` orchestrator delegates to specialist agents via hierarchical two-way links.

**Communication graph:**
```
                    ┌──────────────┐
                    │ default-agent│ ← Slack Bot adapter (Socket Mode)
                    │ (Orchestrator)│ ← 10 cron jobs for Slack monitoring
                    └──────┬───────┘
            ┌──────┬───────┼───────┬──────────┐
            ▼      ▼       ▼       ▼          ▼
      ┌─────────┐┌──────┐┌────────┐┌────────┐┌───────────┐
      │ slack   ││archi-││ coder  ││reviewer││notebooklm││google │
      │ agent   ││tect  ││ agent  ││ agent  ││  agent   ││ agent │
      └─────────┘└──────┘└────────┘└────────┘└───────────┘└───────┘
      Enterprise  Design  TDD impl  Code      NotebookLM   Google
      Slack MCP   + scope  (OpenCode) review    research   Workspace
                                                          + YouTube
```

### LLM Provider & Model Reference

Three LLM providers are configured. **All GPT models must come from `openai-chatgpt`**. Do not use generic model names — use the exact model IDs listed below.

| Provider | Auth | Purpose | Available Models |
|----------|------|---------|-----------------|
| `github-copilot/*` | GitHub Copilot device flow | Claude Opus for default-agent orchestrator | `claude-opus-4.6` |
| `openai-chatgpt/*` | OpenAI Codex CLI OAuth | All GPT/Codex models | See list below |
| `litellm/*` | API key (LiteLLM Gateway) | GLM model for adversarial review | `glm-5` |

**`openai-chatgpt` models (exhaustive list — do not invent model names):**

| Model ID | Description |
|----------|-------------|
| `gpt-5.4` | Latest frontier agentic coding model |
| `gpt-5.4-mini` | Smaller frontier agentic coding model |
| `gpt-5.3-codex` | Frontier Codex-optimized agentic coding model |
| `gpt-5.2-codex` | Frontier agentic coding model |
| `gpt-5.2` | Optimized for professional work and long-running agents |
| `gpt-5.1-codex-max` | Codex-optimized model for deep and fast reasoning |
| `gpt-5.1-codex-mini` | Optimized for codex — cheaper, faster, but less capable |

**Routing rules:**
- `github-copilot/claude-opus-4.6` is **only** used for the `default-agent` channel (orchestrator reasoning)
- `litellm/glm-5` is **only** used for `reviewer-agent` channel and worker (adversarial review)
- **Every other model reference** must use `openai-chatgpt/<model-id>` with an exact model ID from the table above
- Never use `github-copilot/gpt-*` — GitHub Copilot does not provide GPT models
- Never use bare model names like `gpt-5-mini` — always prefix with the provider

### Agent Definitions

| Agent ID | Display Name | Models (ch/wk/br) | Skills | MCP Servers |
|----------|-------------|-------------------|--------|-------------|
| `default-agent` | Spacebot | github-copilot/claude-opus-4.6 / openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini | adversarial-coding-pipeline, adversarial-research-pipeline, agent-capabilities, github-ops | searxng, microsoft, enterprise-slack, fetcher, pdf-reader, rss-feeds |
| `slack-agent` | Slack | openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini | — | enterprise-slack, searxng, microsoft |
| `architect-agent` | Architect | openai-chatgpt/gpt-5.4 / openai-chatgpt/gpt-5.4 / openai-chatgpt/gpt-5.4-mini | office-hours, architecture-lock, research-planner, pr-slicer, context7-docs, github-ops, chrome-devtools, a11y-debugging | searxng, microsoft, chrome-devtools, fetcher, pdf-reader |
| `coder-agent` | Coder | openai-chatgpt/gpt-5.3-codex / openai-chatgpt/gpt-5.3-codex / openai-chatgpt/gpt-5.4-mini | tdd-red-green, di-patterns, fix-first, review-fix-loop, browser-testing, dev-preview, context7-docs, github-ops, chrome-devtools, a11y-debugging | searxng, microsoft, chrome-devtools |
| `reviewer-agent` | Reviewer | litellm/glm-5 / litellm/glm-5 / openai-chatgpt/gpt-5.4-mini | spec-compliance, adversarial-review, research-critic, code-quality, qa-verification, simplicity-review, production-hardening, security-auditor, context7-docs, github-ops, chrome-devtools, a11y-debugging | searxng, microsoft, chrome-devtools, fetcher, pdf-reader |
| `notebooklm-agent` | NotebookLM | openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini | — | notebooklm, searxng, microsoft |
| `google-agent` | Google | openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini / openai-chatgpt/gpt-5.4-mini | — | google-workspace, youtube, searxng, microsoft, fetcher, pdf-reader, arxiv, paper-search |

### Slack Integration Architecture

Spacebot connects to Slack through **two independent mechanisms**:

1. **Slack Bot adapter** (`[messaging.slack]` in config) — Socket Mode bot using `xoxb-` bot token and `xapp-` app token. Bound to `default-agent` via `[[bindings]]`. Handles inbound/outbound messaging (DMs, channel mentions). This is the conversational interface.

2. **Enterprise Slack MCP** (`enterprise-slack` MCP server) — The `slack-mcp-server` binary using `xoxc-`/`xoxd-` browser tokens for read-only enterprise workspace access. Connected **only on `slack-agent`** (explicitly disabled on all other agents). Provides `conversations_search_messages`, `users_list`, `conversations_history`, `files_list`, etc. This is the data ingestion/search interface.

**How Slack monitoring works:** Cron jobs on `default-agent` fire on schedule → branch context thinks about what to search → `send_agent_message` delegates to `slack-agent` → `slack-agent` uses Enterprise Slack MCP tools to search/scan → results flow back to `default-agent` → findings delivered to Slack via the Bot adapter.

### MCP Servers

Defined at the defaults level in config, overridden per-agent:

| MCP Server | Command | Enabled On | Purpose |
|------------|---------|-----------|---------|
| `enterprise-slack` | `/usr/local/bin/slack-mcp-server` | `slack-agent` only | Enterprise Slack workspace search (browser tokens) |
| `searxng` | `mcp-searxng` | All agents | Web search via SearXNG |
| `microsoft` | `npx @softeria/ms-365-mcp-server` | All agents | Microsoft 365 (calendar, mail, contacts, files) |
| `notebooklm` | `uvx notebooklm-mcp-cli notebooklm-mcp` | `notebooklm-agent` only | Google NotebookLM |
| `google-workspace` | `workspace-mcp --single-user --tools drive docs slides sheets gmail calendar` | `google-agent` only | Google Drive, Docs, Slides, Sheets, Gmail, Calendar |
| `youtube` | `youtube-mcp` | `google-agent` only | YouTube Data API, transcript extraction, yt-dlp fallback |
| `linkedin` | `linkedin-scraper-mcp --log-level WARNING` | All agents (enabled by default) | LinkedIn profiles, companies, posts, jobs, messaging, search |
| `fetcher` | `npx fetcher-mcp` | All agents | Web page → clean Markdown via Playwright |
| `pdf-reader` | `npx @sylphx/pdf-reader-mcp` | All agents | PDF text extraction |
| `arxiv` | `uvx arxiv-mcp-server` | `google-agent` only | arXiv paper search & analysis |
| `paper-search` | `npx paper-search-mcp-nodejs` | `google-agent` only | Multi-source academic search (arXiv, PubMed, Semantic Scholar, Google Scholar, etc.) |
| `rss-feeds` | `npx rss-feeds-mcp` | `default-agent` only | RSS/newsletter monitoring for cron-based research |

### Cron Jobs

Cron jobs are stored in SQLite (`cron_jobs` table) per agent and loaded at startup. They drive the periodic Slack monitoring. All 10 cron jobs are on `default-agent` and deliver results to Slack DMs.

The cron system has a **reconciliation loop** that runs every 5 minutes to catch jobs that were saved to the database but not registered with the in-memory scheduler (e.g. due to interrupted tool calls between the `save()` and `register()` await points).

### LinkedIn Integration Use Cases

The LinkedIn MCP integration (via `linkedin-scraper-mcp`) enables professional network research and monitoring workflows:

**Research & Intelligence:**
- Competitive analysis: "Find all ML engineers at Anthropic with Rust experience"
- Market research: "What are the top companies hiring for Rust positions in Phoenix?"
- Company monitoring: "Track new hires at competitor companies and alert me weekly"

**Content Monitoring:**
- Newsletter tracking: "Find posts from AI safety newsletters in the last 24 hours"
- Thought leadership: "Get recent posts from OpenAI's LinkedIn page"
- Hashtag tracking: "Monitor #rustlang posts from top influencers"

**Automated Workflows (via Cron):**
```toml
# Example: Daily LinkedIn newsletter digest
[[cron_job]]
prompt = """
Search LinkedIn for posts from AI safety newsletters in the last 24 hours.
Extract article titles, URLs, and summaries. Send digest to Slack DM.
"""
interval = "0 9 * * *"  # Daily at 9am
notify_channel_id = "C0AR3MA4L5C"
```

**Authentication:** One-time browser login creates a persistent session at `~/.linkedin-mcp/profile`. Session lasts 30-90 days and survives restarts.

**Rate limits:** LinkedIn has undocumented limits. The MCP server uses Patchright (anti-detection Playwright fork) with human-like delays to avoid detection.

**See:** `docs/design-docs/linkedin-integration.md` for full documentation.

## Nix Flake Workflow

### Frontend Dependencies

When updating frontend dependencies in `interface/`:

1. **Update deps:** Modify `interface/package.json` or `interface/bun.lock` as needed
2. **Update the Nix hash:** Run `just update-frontend-hash`
   - This builds the `frontend-updater` package with `fakeHash`
   - Extracts the new hash from the build output
   - Updates `nix/default.nix` automatically
3. **Verify:** Run `nix build .#frontend` to confirm the build works
4. **Commit:** Include both the dependency changes and the hash update in the same PR

**Note:** The `just update-frontend-hash` command uses the `fakeHash` pattern (standard Nix practice) where the build intentionally fails to reveal the correct hash, which is then extracted and applied automatically.

### Nix Flake Inputs

To update all Nix flake inputs (nixpkgs, crane, etc.) and regenerate `flake.lock`:

```bash
just update-flake
```

This runs `nix flake update` and updates all inputs to their latest versions.

## Architecture Overview

Five process types. Every LLM process is a Rig `Agent<SpacebotModel, SpacebotHook>`. They differ in system prompt, tools, history, and hooks.

### Channels

The user-facing LLM process. One per conversation (Telegram DM, Discord thread, etc). Has soul, identity, personality. Talks to the user. Delegates everything else.

A channel does NOT: execute tasks directly, search memories itself, do heavy tool work.

The channel is always responsive — never blocked by work, never frozen by compaction. When it needs to think, it branches. When it needs work done, it spawns a worker. When context gets full, the compactor has already handled it.

**Tools:** reply, branch, spawn_worker, route, cancel, skip, react  
**Context:** Conversation history + compaction summaries + status block  
**History:** Persistent `Vec<Message>`, passed via `agent.prompt().with_history(&mut history)`

### Branches

A fork of the channel's context that goes off to think. Has the channel's full conversation history — same context, same memories, same understanding. Operates independently. The channel never sees the working, only the conclusion.

Creating a branch is `let branch_history = channel_history.clone()`.

The branch result is injected into the channel's history as a distinct message type. Then the branch is deleted. Multiple branches can run concurrently per channel (configurable limit). First done, first incorporated.

**Tools:** memory_recall, memory_save, memory_delete, channel_recall, spacebot_docs, task_create, task_list, task_update, spawn_worker  
**Context:** Clone of channel history at fork time  
**Lifecycle:** Short-lived. Returns a conclusion, then deleted.

### Workers

Independent process that does a job. Gets a specific task, a focused system prompt, and task-appropriate tools. No channel context, no soul, no personality.

Two kinds:
- **Fire-and-forget:** Does a job and returns a result. Memory recall, summarization, one-shot tasks.
- **Interactive:** Long-running, accepts follow-up input from the channel. Coding sessions, complex multi-step tasks.

Workers are pluggable. A worker can be:
- A Rig agent with shell/file tools
- An OpenCode subprocess
- Any external process that accepts a task and reports status

**Tools:** shell, file, set_status (varies by worker type)  
**Context:** Fresh prompt + task description. No channel history.  
**Lifecycle:** Fire-and-forget or long-running. Reports status via `set_status` tool.

### The Compactor

NOT an LLM process. A programmatic monitor per channel. Watches context size and triggers compaction before the channel fills up.

Tiered thresholds:
- **>80%** — background compaction worker (summarize + extract memories)
- **>85%** — aggressive summarization
- **>95%** — emergency truncation (no LLM, just drop oldest turns)

The compaction worker runs alongside the channel without blocking it. Compacted summaries stack at the top of the context window.

### The Cortex

System-level observer. Primary job: generate the **memory bulletin** — a periodically refreshed, LLM-curated summary of the agent's current knowledge. Runs on a configurable interval (default 60 min), uses `memory_recall` to query across multiple dimensions (identity, events, decisions, preferences), synthesizes into a ~500 word briefing cached in `RuntimeConfig::memory_bulletin`. Every channel reads this on every turn via `ArcSwap`.

Also observes system-wide signals for future health monitoring and memory consolidation.

**Tools (bulletin generation):** memory_recall, memory_save  
**Tools (interactive cortex chat):** memory + worker tools, `spacebot_docs`, `config_inspect`, task board tools  
**Tools (future health monitoring):** memory_consolidate, system_monitor  
**Context:** Fresh per bulletin run. No compaction needed.

### Status Injection

Every turn, the channel gets a live status block injected into its context — active workers, recently completed work, branch states. Workers set their own status via `set_status` tool. Short branches are invisible (only appear if running >3s).

### Cron Jobs

Database-stored scheduled tasks. Each cron job has a prompt, interval, delivery target, and optional active hours. When a timer fires, it gets a fresh short-lived channel with full branching and worker capabilities. Multiple cron jobs run independently and concurrently.

## Key Types

```
SpacebotModel          — custom CompletionModel impl, routes through LlmManager
SpacebotHook           — PromptHook impl for channels/branches/workers (status, usage, cancellation)
CortexHook             — PromptHook impl for cortex (system observation)
ProcessType            — enum: Channel, Branch, Worker
ProcessEvent           — tagged enum for inter-process events
Channel (struct)       — owns history, spawns branches, routes to workers
WorkerState            — state machine: Running, WaitingForInput, Done, Failed
Memory                 — content + type + importance + timestamps + source + associations
MemoryType             — enum: Fact, Preference, Decision, Identity, Event, Observation
ChannelId              — Arc<str> type alias
AgentDeps              — dependency bundle (memory_store, llm_manager, tool_server, event_tx)
LlmManager             — holds provider clients, routes by model name
DecryptedSecret        — secret wrapper, redacts in Debug/Display
CronConfig             — prompt + interval + active_hours + notify
```

## Module Map

```
src/
├── main.rs             — CLI entry, config loading, startup
├── lib.rs              — re-exports, shared types
├── config.rs           — configuration loading/validation
├── error.rs            — top-level Error enum wrapping domain errors
│
├── llm.rs              → llm/
│   ├── manager.rs      — LlmManager: provider routing, model resolution, fallback chains
│   ├── model.rs        — SpacebotModel: CompletionModel impl
│   ├── routing.rs      — RoutingConfig: process-type defaults, task-type overrides, fallbacks
│   └── providers.rs    — provider client init (Anthropic, OpenAI, etc.)
│
├── agent.rs            → agent/
│   ├── channel.rs      — Channel: user-facing conversation
│   ├── branch.rs       — Branch: fork context, think, return result
│   ├── worker.rs       — Worker: fire-and-forget + interactive management
│   ├── compactor.rs    — Compactor: programmatic context monitor
│   ├── cortex.rs       — Cortex: system-level observer
│   └── status.rs       — StatusBlock: live status snapshot
│
├── hooks.rs            → hooks/
│   ├── spacebot.rs     — SpacebotHook: channels/branches/workers
│   └── cortex.rs       — CortexHook: cortex observation
│
├── tools.rs            → tools/
│   ├── reply.rs        — send message to user (channel only)
│   ├── branch_tool.rs  — fork context and think (channel only)
│   ├── spawn_worker.rs — create new worker (channel + branch)
│   ├── route.rs        — send follow-up to active worker (channel only)
│   ├── cancel.rs       — cancel worker or branch (channel only)
│   ├── skip.rs         — opt out of responding (channel only)
│   ├── react.rs        — add emoji reaction (channel only)
│   ├── memory_save.rs  — write memory to store (branch + cortex + compactor)
│   ├── memory_recall.rs— search + curate memories (branch only)
│   ├── channel_recall.rs— retrieve transcript from any channel (branch only)
│   ├── set_status.rs   — update worker status (workers only)
│   ├── shell.rs        — execute shell commands and subprocesses (task workers)
│   ├── file.rs         — read/write/list files (task workers)
│   ├── browser.rs      — web browsing (task workers)
│   ├── task_create.rs  — create task-board task (branch + cortex chat)
│   ├── task_list.rs    — list task-board tasks (branch + cortex chat)
│   ├── task_update.rs  — update task-board task (branch + cortex chat)
│   ├── spacebot_docs.rs — read embedded Spacebot docs/changelog (branch + cortex chat)
│   ├── config_inspect.rs — inspect live runtime config (cortex chat)
│   └── cron.rs         — cron management (channel only)
│
├── memory.rs           → memory/
│   ├── store.rs        — MemoryStore: CRUD + graph ops (SQLite)
│   ├── types.rs        — Memory, Association, MemoryType, RelationType
│   ├── search.rs       — hybrid search (vector + FTS + RRF + graph traversal)
│   ├── lance.rs        — LanceDB table management, embedding storage
│   ├── embedding.rs    — embedding generation via LlmManager
│   └── maintenance.rs  — decay, prune, merge, reindex
│
├── messaging.rs        → messaging/
│   ├── traits.rs       — Messaging trait + MessagingDyn companion
│   ├── manager.rs      — MessagingManager: start all, fan-in, route outbound
│   ├── discord.rs      — Discord adapter
│   ├── telegram.rs     — Telegram adapter
│   └── webhook.rs      — Webhook receiver (programmatic access)
│
├── conversation.rs     → conversation/
│   ├── history.rs      — conversation persistence (SQLite)
│   └── context.rs      — context assembly (prompt + identity + memories + status)
│
├── cron.rs             → cron/
│   ├── scheduler.rs    — timer management
│   └── store.rs        — cron CRUD (SQLite)
│
├── identity.rs         → identity/
│   └── files.rs        — load SOUL.md, IDENTITY.md, USER.md
│
├── secrets.rs          → secrets/
│   └── store.rs        — encrypted credentials (AES-256-GCM, redb)
│
├── settings.rs         → settings/
│   └── store.rs        — key-value settings (redb)
│
└── db.rs               → db/
    └── migrations.rs   — SQLite migrations
```

Module roots (e.g., `src/memory.rs`) contain `mod` declarations and re-exports. Never create `mod.rs` files.

Tools are organized by function, not by consumer. Which processes get which tools is configured via factory functions in `tools.rs`.

## Three Databases

Each doing what it's best at. No server processes.

**SQLite** (via sqlx) — relational data: conversations, memory graph, cron jobs. Queries with joins, ordering, filtering. Migrations in `migrations/`.

**LanceDB** — vector/search data: embeddings (HNSW), full-text search (Tantivy), hybrid search (RRF). Joined to SQLite on memory ID.

**redb** — key-value config: settings, encrypted secrets. Separate from SQLite so config can be backed up independently.

Actual queries live in the modules that use them — `memory/store.rs` has graph queries, `memory/lance.rs` has search, `conversation/history.rs` has conversation queries. The `db/` module is just connection setup and migration running.

## Memory System

Memories are structured objects, not files. Every memory is a row in SQLite with typed metadata and graph connections, paired with a vector embedding in LanceDB.

**Types:** Fact, Preference, Decision, Identity, Event, Observation.

**Graph edges:** RelatedTo, Updates, Contradicts, CausedBy, PartOf. Auto-associated on creation via similarity search. >0.9 similarity marks as `Updates`.

**Three creation paths:**
1. Branch-initiated (during conversation) — branch uses `memory_save` tool
2. Compactor-initiated (during compaction) — extract memories from context being compacted
3. Cortex-initiated (system-level) — consolidation, observations

**Recall flow:** Branch → recall tool → hybrid search (vector + FTS + RRF + graph traversal) → curate → return clean results. The channel never sees raw search results.

**Importance:** Score 0-1. Influenced by explicit importance, access frequency, recency, graph centrality. Identity memories exempt from decay.

**Identity files** (SOUL.md, IDENTITY.md, USER.md) are loaded from disk into system prompts. They are NOT graph memories.

## Rig Integration

Every LLM process is a Rig `Agent`. Key patterns:

**Agent construction:**
```rust
let agent = AgentBuilder::new(model.clone())
    .preamble(&system_prompt)
    .hook(SpacebotHook::new(process_id, process_type, event_tx.clone()))
    .tool_server_handle(tools.clone())
    .default_max_turns(50)
    .build();
```

**History is external**, passed on each call:
```rust
let response = agent.prompt(&user_message)
    .with_history(&mut history)
    .max_turns(5)
    .await?;
```

**Branching is a clone:**
```rust
let branch_history = channel_history.clone();
```

**Custom CompletionModel** — `SpacebotModel` routes through `LlmManager`. We don't use Rig's built-in provider clients.

**PromptHook** — `SpacebotHook` sends `ProcessEvent`s for status reporting, usage tracking, cancellation. Returns `Continue`, `Terminate`, or `Skip`.

**ToolServer topology:**
- Per-channel `ToolServer` (no memory tools, just channel action tools added per turn)
- Per-branch `ToolServer` with memory tools (memory_save, memory_recall, memory_delete), channel recall, docs introspection (`spacebot_docs`), and task-board tools
- Per-worker `ToolServer` with task-specific tools (shell, file)
- Per-cortex `ToolServer` with memory_save

**Max turns:** Rig defaults to 0 (single call). Always set explicitly.
- Workers: `max_turns(50)` — many iterations
- Branches: `max_turns(10)` — a few iterations
- Channels: `max_turns(5)` — typically 1-3 turns

**Error recovery:** Rig returns chat history in `MaxTurnsError` and `PromptCancelled`. Use this for worker timeout, cancellation, budget enforcement.

**We don't use:** Rig's built-in provider clients, RAG/vector store integrations, Agent-as-Tool, Pipeline system.

## Build Order

Phase 1 — Foundation:
1. `error.rs` — top-level Error enum
2. `config.rs` — configuration loading
3. `db/` — SQLite + LanceDB + redb connection setup, migrations
4. `llm/` — SpacebotModel, LlmManager, provider init
5. `main.rs` — startup, config loading, database init

Phase 2 — Memory:
1. `memory/types.rs` — Memory, Association, MemoryType, RelationType
2. `memory/store.rs` — MemoryStore CRUD + graph operations
3. `memory/lance.rs` — LanceDB table management, embedding storage
4. `memory/embedding.rs` — embedding generation
5. `memory/search.rs` — hybrid search (vector + FTS + RRF + graph traversal)
6. `memory/maintenance.rs` — decay, prune, merge, reindex

Phase 3 — Agent Core:
1. `hooks/spacebot.rs` — SpacebotHook (ProcessEvent sending)
2. `agent/status.rs` — StatusBlock
3. `tools/` — implement tools (start with memory_save, memory_recall, set_status)
4. `agent/worker.rs` — Worker lifecycle (fire-and-forget first, interactive later)
5. `agent/branch.rs` — Branch (fork, think, return result)
6. `agent/channel.rs` — Channel (message handling, branching, worker management, status injection)
7. `agent/compactor.rs` — Compactor (threshold monitor, compaction worker spawning)

Phase 4 — System:
1. `identity/` — load identity files
2. `conversation/` — history persistence, context assembly
3. `prompts/` — system prompt files for each process type
4. `agent/cortex.rs` — Cortex
5. `hooks/cortex.rs` — CortexHook
6. `cron/` — scheduler + store

Phase 5 — Messaging:
1. `messaging/traits.rs` — Messaging trait + MessagingDyn
2. `messaging/manager.rs` — MessagingManager (fan-in, routing)
3. `messaging/webhook.rs` — Webhook receiver (for testing + programmatic access)
4. `messaging/telegram.rs` — Telegram
5. `messaging/discord.rs` — Discord

Phase 6 — Hardening:
1. `secrets/` — encrypted credential storage
2. `settings/` — key-value settings
3. Leak detection (scan tool output via SpacebotHook)
4. Workspace path guards (reject writes to identity/memory paths)
5. Circuit breaker for cron jobs and background tasks

## Anti-Patterns

**Don't block the channel.** The channel never waits on branches, workers, or compaction. If you're writing code where the channel awaits a branch result before responding, the design is wrong.

**Don't dump raw search results into channel context.** Memory recall goes through a branch, which curates. The channel gets clean conclusions, not 50 raw database rows.

**Don't give workers channel context.** Workers get a fresh prompt and a task. If a worker needs conversation context, that's a branch, not a worker.

**Don't make the compactor an LLM process.** The compactor is programmatic — it watches a number (context token count) and spawns workers. The LLM work happens in the compaction worker it spawns.

**Don't store prompts as string constants in Rust.** System prompts live in `prompts/` as markdown files. Load at startup or on demand.

**Don't create `mod.rs` files.** Use `src/memory.rs` as the module root, not `src/memory/mod.rs`.

**Don't silently discard errors.** No `let _ =` on Results. Handle them, log them, or propagate them. The only exception is `.ok()` on channel sends where the receiver may be dropped.

**Don't use `#[async_trait]`.** Use native RPITIT for async traits. Only add a companion `Dyn` trait when you actually need `dyn Trait`.

**Don't create many small files.** Implement functionality in existing files unless it's a new logical component.

**Don't abbreviate variable names.** `queue` not `q`, `message` not `msg`, `channel` not `ch`. Common abbreviations like `config` are fine.

**Don't add new features without updating existing docs.** When a feature change affects user-facing configuration, behaviour, or architecture, update the relevant existing documentation (`README.md`, `docs/`) in the same commit or PR. Don't create new doc files for this — update what's already there.

## Patterns to Implement

These are validated patterns from research (see `docs/research/pattern-analysis.md`). Implement them when building the relevant module.

**Tool nudging / outcome gate:** Workers cannot exit with a text-only response until they signal a terminal outcome via `set_status(kind: "outcome")`. If a worker returns text without an outcome signal, the hook fires `Terminate` and retries with a nudge prompt (up to 2 retries). After retries are exhausted the worker fails with `PromptCancelled`. See `docs/design-docs/tool-nudging.md`.

**Fire-and-forget DB writes:** `tokio::spawn` for conversation history saves, memory writes, worker log persistence. User gets their response immediately.

**Tiered compaction:** >80% background, >85% aggressive, >95% emergency truncation. The compactor uses these thresholds.

**Hybrid search with RRF:** Vector similarity + full-text search, merged via Reciprocal Rank Fusion (`score = sum(1/(60 + rank))`). RRF works on ranks, not raw scores.

**Leak detection:** Regex patterns for API keys, tokens, PEM keys. Scan in `SpacebotHook.on_tool_result()` (after execution) and before outbound HTTP (block exfiltration).

**Workspace path guard:** File tools reject writes to identity/memory paths with an error directing the LLM to the correct tool.

**Circuit breaker:** Auto-disable recurring tasks after 3 consecutive failures. Apply to cron jobs, maintenance workers, cortex routines.

**Config resolution:** `env > DB > default` with per-subsystem `resolve()` methods.

**Error-as-result for tools:** Tool errors are returned as structured results, not panics. The LLM sees the error and can recover.

**Worker state machine:** Validate transitions with `can_transition_to()` using `matches!`. Illegal transitions are runtime errors, not silent bugs.

## Reference Docs

- `README.md` — full architecture design
- `RUST_STYLE_GUIDE.md` — coding conventions (read this first)
- `docs/copilot-setup.md` — Copilot CLI plugins, skills, hooks, and MCP configuration
- `docs/design-docs/agent-factory.md` — agent creation flow, preset archetypes
- `docs/design-docs/multi-agent-communication-graph.md` — inter-agent links and topology
- `docs/design-docs/mcp.md` — MCP client integration (stdio/HTTP, per-agent config)
- `docs/design-docs/named-messaging-adapters.md` — multi-adapter messaging (Slack, Discord, Telegram)
- `docs/design-docs/cron-timezone-and-reliability.md` — cron scheduling and timezone handling
- `docs/design-docs/tool-nudging.md` — worker outcome gate / retry nudging
- `docs/design-docs/sandbox.md` — sandbox implementation (shell/file isolation)
- `docs/design-docs/working-memory.md` — working memory system (situational awareness)
- `docs/design-docs/google-youtube-integration.md` — Google Workspace + YouTube MCP integration
- `docs/design-docs/adversarial-research-pipeline.md` — adversarial research pipeline (STORM-inspired)
- `docs/design-docs/linkedin-integration.md` — LinkedIn MCP integration guide
