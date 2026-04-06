---
name: dev-preview
description: Use this skill when a coding task involves a UI and you need to expose the local dev server via a public URL for review. Launches a Cloudflare quick tunnel to provide a shareable *.trycloudflare.com URL.
---

# Dev Preview — Cloudflare Quick Tunnel

Expose a local dev server via a public URL so the user and reviewers can see the UI in a browser from any device. Uses `cloudflared tunnel` (already installed) with zero configuration.

## When to Use

- A subtask builds or modifies a UI (HTML, components, pages, styles)
- The orchestrator or user wants to see the rendered output
- You need a public URL for browser-testing skill verification
- The user asks to "show me" or "let me see it"

## Do NOT Use For

- Backend-only APIs (use curl/httpie instead)
- Static file generation with no dev server
- Tests that don't need visual verification

## Quick Start

```bash
# 1. Start the dev server (framework-dependent, see detection below)
bun run dev &
DEV_PID=$!

# 2. Wait for the server to be ready
sleep 3

# 3. Launch cloudflared quick tunnel
cloudflared tunnel --url localhost:4321 2>&1 &
TUNNEL_PID=$!

# 4. Wait for the tunnel URL (appears in stderr)
sleep 5

# 5. Extract the URL from cloudflared output
# Look for: "https://<random>.trycloudflare.com"
```

## Framework Detection

Detect the framework from project files to determine the correct dev command and port:

| Indicator | Framework | Dev Command | Default Port |
|-----------|-----------|-------------|-------------|
| `astro.config.*` | Astro | `bun run dev` or `astro dev` | 4321 |
| `vite.config.*` | Vite | `bun run dev` | 5173 |
| `next.config.*` | Next.js | `bun run dev` | 3000 |
| `sst.config.*` | SST | `bun run dev` or `sst dev` | 3000 |
| `svelte.config.*` | SvelteKit | `bun run dev` | 5173 |
| `nuxt.config.*` | Nuxt | `bun run dev` | 3000 |
| None of above | Static | `python3 -m http.server 8000 -d dist` | 8000 |

**Always check `package.json` scripts first** — the `dev` script may override the default port.

**Always use `bun` for JS/TS** unless the project explicitly requires npm (per AGENTS.md rules).

## Step-by-Step Procedure

### 1. Detect and start the dev server

```bash
# Check for framework config files
ls astro.config.* vite.config.* next.config.* 2>/dev/null

# Start dev server in background
bun run dev > /tmp/dev-server.log 2>&1 &
DEV_PID=$!

# Wait for server to be ready (check for "ready" or port binding)
for i in 1 2 3 4 5; do
  sleep 2
  if curl -s -o /dev/null -w "%{http_code}" http://localhost:4321 | grep -q "200\|304"; then
    break
  fi
done
```

### 2. Launch the tunnel

```bash
# cloudflared outputs the URL to STDERR. Redirect both stdout and stderr to the log.
nohup cloudflared tunnel --url http://localhost:4321 &>/tmp/cloudflared.log &
TUNNEL_PID=$!

# Wait for tunnel URL — poll the log file (may take up to 15 seconds)
TUNNEL_URL=""
for i in $(seq 1 15); do
  sleep 2
  TUNNEL_URL=$(grep -oE 'https://[a-z0-9-]+\.trycloudflare\.com' /tmp/cloudflared.log 2>/dev/null | head -1)
  if [ -n "$TUNNEL_URL" ]; then
    break
  fi
done

if [ -z "$TUNNEL_URL" ]; then
  echo "ERROR: Tunnel URL not found after 30 seconds. Check /tmp/cloudflared.log"
  cat /tmp/cloudflared.log
else
  echo "Public URL: $TUNNEL_URL"
fi

# Verify the tunnel is reachable (follow redirects)
sleep 3
HTTP_CODE=$(curl -s -L -o /dev/null -w "%{http_code}" "$TUNNEL_URL")
echo "Tunnel HTTP status: $HTTP_CODE"
```

**CRITICAL:** Do NOT proceed until you have extracted a valid `https://*.trycloudflare.com` URL AND verified it returns HTTP 200. If verification fails, retry the tunnel launch once.

### 3. Report the URL

Use `set_status` to report the tunnel URL so the orchestrator can share it with the user:

```
set_status(kind: "progress", status: "Dev preview available at: <URL>")
```

### 4. Tear down when done

Always clean up before the worker exits:

```bash
kill $TUNNEL_PID 2>/dev/null
kill $DEV_PID 2>/dev/null
```

## Integration with Browser Testing

When both dev-preview and browser-testing are active:

1. Start dev server and tunnel (this skill)
2. Use `browser_launch` + `browser_navigate` to the **tunnel URL** (not localhost)
3. Use `browser_snapshot` to verify the ARIA tree
4. Report both the tunnel URL and browser verification results

## Troubleshooting

- **Port already in use**: Check `lsof -i :<port>` and kill the conflicting process
- **Tunnel fails to start**: Verify `cloudflared` is available: `which cloudflared`
- **No URL in output**: Wait longer (up to 15 seconds) — tunnel establishment can be slow on first run
- **Dev server crashes**: Check `/tmp/dev-server.log` for errors

## Memory Constraints

- Dev server: ~50-150MB depending on framework
- Cloudflared tunnel: ~30MB
- Combined with Spacebot daemon (~200MB) + worker LLM context: fits within 8GB M1
- Always tear down both processes when done — don't leave them running
