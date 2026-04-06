---
name: context7-docs
description: Use this skill to fetch up-to-date library and framework documentation via Context7. Use for SST.dev, Astro.js, Alpine.js, Tailwind CSS, TypeScript, Terraform, GitHub Actions, React, or any npm/PyPI library. Always use before external web search when you need API references or framework patterns.
---

# Context7 Docs — Library Documentation

Fetch up-to-date library documentation via the Context7 MCP server. Use this when you need to research any library, framework, or tool's API before writing code.

## When to Use

- Before implementing with a framework you need current API reference for
- When the coding task references libraries like SST, Astro, Alpine, Tailwind, Terraform, etc.
- When you need to verify correct API usage, method signatures, or configuration options
- Always use BEFORE searching the web — Context7 is faster and more accurate

## Do NOT Use For

- Internal/proprietary API documentation
- General web search unrelated to libraries
- Documentation the user already provided inline

## Quick Reference

```bash
# Search for a library's Context7 ID
./scripts/docs.sh search <library>

# Fetch documentation (resolves ID automatically)
./scripts/docs.sh docs <library> [topic] [--tokens N]

# Examples
./scripts/docs.sh search "sst.dev"
./scripts/docs.sh docs astro "content collections"
./scripts/docs.sh docs tailwindcss "grid layout"
./scripts/docs.sh docs alpinejs "x-data directives"
./scripts/docs.sh docs terraform "aws provider"
```

## Common Libraries

| Library | Search term | Example topics |
|---------|------------|----------------|
| SST.dev | `sst` | constructs, live dev, permissions |
| Astro.js | `astro` | islands, content collections, SSR, routing |
| Alpine.js | `alpinejs` | x-data, x-bind, x-on, reactivity |
| Tailwind CSS | `tailwindcss` | utilities, responsive, grid, flex |
| TypeScript | `typescript` | generics, utility types, config |
| Terraform | `terraform` | providers, state, modules |
| GitHub Actions | `github-actions` | workflows, matrix, secrets |

## Workflow

1. **Search** — find the Context7 library ID: `./scripts/docs.sh search <name>`
2. **Fetch** — get docs with optional topic filter: `./scripts/docs.sh docs <name> [topic]`
3. **Use** — apply the documentation to your implementation

## Tips

- Use specific topic filters to reduce context size (e.g., `docs astro routing` not just `docs astro`)
- Use `--tokens N` to control response size when context budget is tight
- If a library isn't found, try alternative names (e.g., `nextjs` vs `next.js`)
- For Terraform providers, search for the specific provider (e.g., `terraform-aws`)

## Prerequisites

- Node.js 18+ and `npx` available
- The `docs.sh` script handles Context7 MCP resolution automatically
- Falls back to Context7 public API when local server isn't configured
