# web-testing

L2 AI-driven web UI testing skill for Stiglab. Uses agent-browser CLI to
interactively explore the running UI, triage L1 test failures, and validate
new features on PRs.

## When to use

- After L1 (deterministic) tests fail — to triage and diagnose the root cause
- On PRs introducing new UI features — to exploratory-test unstable surfaces
- When asked to verify a specific user flow end-to-end
- When asked to check visual or interactive regressions

## Prerequisites

- Dev server running: `pnpm dev` (or set `STIGLAB_TEST_URL`)
- Chrome installed: `npx agent-browser install`

## How to use agent-browser

agent-browser is a CLI tool. Run commands via shell:

```bash
# Navigate to a page
npx agent-browser open "http://localhost:5173/"

# Get page structure (accessibility tree)
npx agent-browser snapshot

# Get interactive elements with refs (@e1, @e2, etc.)
npx agent-browser snapshot -i

# Interact with elements
npx agent-browser click "@e1"
npx agent-browser fill "@e2" "search text"

# Wait for content
npx agent-browser wait --text "Dashboard" --timeout 10000

# Take screenshot for visual check
npx agent-browser screenshot --output /tmp/test-screenshot.png

# Get page info
npx agent-browser get title
npx agent-browser get url
npx agent-browser get text
```

## Testing workflow

### Triage L1 failure

1. Read the failing test to understand what it expected
2. Open the relevant page with `agent-browser open`
3. Take a snapshot to see the actual page state
4. Compare expected vs actual — identify the discrepancy
5. Check the DOM structure, network state, or console errors
6. Report findings with concrete evidence (snapshots, screenshots)

### Exploratory test for new PR features

1. Read the PR diff to understand what changed
2. Open the affected pages with `agent-browser open`
3. Navigate through the new/changed UI flows
4. Verify: correct rendering, proper navigation, error states, empty states
5. Test edge cases: rapid clicks, missing data, invalid routes
6. Report any issues found with reproduction steps

### Smoke verification

1. Open each main route: `/`, `/sessions`, `/nodes`
2. Verify page loads with expected headings and structure
3. Check navigation between pages works
4. Verify theme toggle works
5. Check responsive layout indicators

## Stiglab UI routes

| Route | Page | Key elements |
|-------|------|-------------|
| `/` | Dashboard | Overview stats, Recent Sessions table |
| `/sessions` | Sessions | All Sessions table with state badges |
| `/sessions/:id` | Session Detail | Session metadata, output log stream |
| `/nodes` | Nodes | Registered Nodes table with status badges |

## Reporting format

When reporting test results, use this format:

```
## Test Results: [scope]

**Status**: PASS / FAIL / PARTIAL

### Checks performed
- [ ] Page loads correctly
- [ ] Data renders as expected
- [ ] Navigation works
- [ ] Error states handled

### Issues found
- [description + evidence]

### Screenshots
- [paths to any screenshots taken]
```
