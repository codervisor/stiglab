# web-testing

L2 AI-driven web UI testing skill for Stiglab. Uses agent-browser CLI to
verify the running UI against the L2 test spec, triage L1 failures, and
validate new PR features.

## Spec-driven testing

**Always start by reading the L2 spec:**

```
packages/stiglab-ui/tests/l2/spec.toml
```

This spec defines routes, flows, and invariants that must hold. Your job is
to execute each applicable spec item using agent-browser and report results.

## When to use

- **PR opened/updated**: Run all route and invariant checks. Run flows if
  the diff touches navigation or layout. Also inspect the diff for changes
  not covered by the spec.
- **L1 failure triage**: Run the spec items related to the failing test to
  diagnose root cause.
- **Manual invocation**: Run the full spec.

## Prerequisites

- Dev server running: `pnpm dev` (or set `STIGLAB_TEST_URL`)
- Chrome installed: `npx agent-browser install`

## Execution procedure

### 1. Read the spec

Parse `tests/l2/spec.toml`. Each section is a test item:
- `[route.*]` — open the path, check `must_contain` / `must_not_contain`
- `[flow.*]` — execute the steps sequentially with agent-browser
- `[invariant.*]` — verify the property holds across all routes

### 2. Execute with agent-browser

```bash
# Navigate
npx agent-browser open "http://localhost:5173/"

# Check page content
npx agent-browser snapshot

# Interactive elements (for flows)
npx agent-browser snapshot -i

# Interact
npx agent-browser click "@e1"
npx agent-browser fill "@e2" "value"

# Verify
npx agent-browser get title
npx agent-browser get url
npx agent-browser get text

# Check for JS errors
npx agent-browser evaluate "window.__stiglab_errors || []"

# Screenshot evidence
npx agent-browser screenshot --output /tmp/l2-<name>.png

# Clean up
npx agent-browser close
```

### 3. On PRs, also check the diff

After running the spec, read the PR diff. If it touches UI files not covered
by any spec item, do exploratory checks on those areas. Report any gaps.

### 4. Report results

Use this structured format for every run:

```markdown
## L2 Test Results

**Trigger**: PR #N / L1 triage / manual
**Status**: PASS / FAIL / PARTIAL

### Route checks
| Route | Status | Notes |
|-------|--------|-------|
| / | PASS | All expected content present |
| /sessions | PASS | |
| /nodes | FAIL | "undefined" found in snapshot |

### Flow checks
| Flow | Status | Notes |
|------|--------|-------|
| sidebar_navigation | PASS | All nav transitions verified |
| theme_toggle | PASS | |

### Invariant checks
| Invariant | Status | Notes |
|-----------|--------|-------|
| no_console_errors | PASS | |
| graceful_api_failure | PASS | Shows empty states |

### Diff coverage
- [x] All changed files covered by spec
- [ ] `NewComponent.tsx` — not in spec, exploratory check: PASS

### Issues found
- (none)
```
