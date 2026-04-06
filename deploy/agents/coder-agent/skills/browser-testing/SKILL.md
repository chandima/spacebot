---
name: browser-testing
description: Use this skill when a coding task affects the UI and you need to verify rendered output using Spacebot's built-in browser automation tools. Spawn a browser-enabled worker to navigate, snapshot, and verify the accessibility tree.
---

# Browser Testing

Use Spacebot's built-in browser automation tools to verify UI changes during the TDD cycle. The browser tools use Chrome's native CDP Accessibility API to extract the ARIA tree — this gives you structured, LLM-readable output without needing screenshots.

## When to Use

Activate browser testing when a subtask:
- Changes HTML/template output
- Modifies CSS layout or styling
- Alters component rendering logic
- Adds or changes interactive elements (forms, buttons, links)
- Affects navigation or routing

Do NOT use browser testing for:
- Backend-only changes
- API endpoint modifications
- Database schema changes
- Configuration changes
- Test-only changes

## How to Use

### 1. Spawn a Browser-Enabled Worker

Request the orchestrator to spawn a worker with browser tools enabled:
```
Spawn a task worker with browser tools to verify the UI changes for [subtask].
```

### 2. Launch and Navigate

```
Use browser_launch to start a headless browser.
Navigate to [URL] using browser_navigate.
```

### 3. Snapshot the Accessibility Tree

```
Use browser_snapshot to get the ARIA accessibility tree.
```

The snapshot returns structured text showing all elements, their roles, names, and interactive indices. This is more useful than a screenshot for verifying:
- Element presence and ordering
- Text content and labels
- Interactive element availability
- Form structure and labels
- Navigation structure

### 4. Interact and Verify

For interactive testing:
```
Use browser_click [index] to click elements.
Use browser_type [index] [text] to fill form fields.
Use browser_snapshot to verify the result after interaction.
```

### 5. Close

Always close the browser when done:
```
Use browser_close to shut down the browser process.
```

## Memory Constraints

This machine has 8GB RAM. Headless Chromium uses ~100-200MB per page.

**Rules:**
- Test one page at a time. Don't open multiple tabs for parallel testing.
- Close the browser immediately after verification. Don't leave it running.
- If the app requires a dev server, ensure it's running before launching the browser.
- Prefer accessibility tree snapshots over screenshots — they use less memory and are more useful for LLM analysis.

## What to Report

After browser verification, include in your status report:
- Elements verified (what was checked in the ARIA tree)
- Interaction results (if forms/buttons were tested)
- Issues found (missing elements, wrong labels, broken interactions)
- Accessibility concerns (missing ARIA labels, wrong roles)

## Integration with TDD

Browser testing supplements, not replaces, unit and integration tests:
1. **RED phase:** Write component/E2E tests that verify behavior
2. **GREEN phase:** Make tests pass, then browser-verify the rendered output
3. **Report:** Include both test results AND browser verification in your status
