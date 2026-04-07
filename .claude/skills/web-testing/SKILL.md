---
name: web-testing
description: "L2 AI-driven web UI testing for Stiglab. Use when testing UI on PRs, triaging L1 test failures, or verifying UI behavior. Triggers include: 'test the UI', 'check the web app', 'triage test failure', 'validate this PR', 'run L2 tests', 'exploratory test'."
allowed-tools: Bash(npx agent-browser:*), Bash(agent-browser:*), Read, Write, Edit, Glob, Grep
---

# Web Testing

L2 AI-driven web UI testing for Stiglab. Validates new and changed UI
surfaces using agent-browser. Regression is L1's job (deterministic tests
in `tests/smoke/` and `tests/e2e/`). L2 findings get crystallized into L1.

## Inputs

L2 works from three things — no separate spec needed:
- **PR diff** (`git diff origin/main...HEAD`) — what changed, what to test
- **Existing L1 tests** (`tests/smoke/`, `tests/e2e/`) — what's already covered
- **agent-browser** — to explore the running UI

## Routes

| Route | Page | Key elements |
|-------|------|-------------|
| `/` | Dashboard | Overview stats, Recent Sessions table |
| `/sessions` | Sessions | All Sessions table with state badges |
| `/sessions/:id` | Session Detail | Session metadata, output log stream |
| `/nodes` | Nodes | Registered Nodes table with status badges |

## Execution

### 1. Read the diff

Map changed files to affected routes:
- `src/pages/*Page.tsx` → the corresponding route
- `src/components/*` → pages that use that component
- `src/hooks/*`, `src/lib/*` → all pages that import it
- `src/App.tsx`, `src/components/layout/*` → all routes

### 2. Test affected pages

For each affected page, use the `agent-browser` skill to:
- Open the route and take a snapshot
- Verify changed elements render correctly
- Check for "undefined", "NaN", or uncaught errors
- Test interactions (navigation, clicks, toggles)

### 3. Crystallize findings into L1

**Key output of L2.** When you validate new behavior or find a bug:

- **Validated new behavior** → write a new L1 test in `tests/smoke/` or
  `tests/e2e/` that encodes it as a deterministic assertion.
- **Bug found** → report it. Once fixed, write an L1 regression test.
- **Update regression contract** → add assertions to `tests/e2e/ui-regression.toml`.

### 4. Report

```markdown
## L2 Test Results — PR #N

**Status**: PASS / FAIL / PARTIAL

### Pages tested
| Route | Status | Notes |
|-------|--------|-------|
| / | PASS | Dashboard renders correctly |
| /sessions | PASS | New filter component works |

### Crystallized into L1
- tests/smoke/session-filter.test.tsx (3 tests)

### Issues found
- (none or description + evidence)
```

## L1 triage mode

When triggered by L1 failure:

1. Read the failing test to understand the expected assertion
2. Open the relevant page with agent-browser
3. Snapshot actual state vs expected
4. Diagnose: real regression, environment issue, or flaky test
5. Report with evidence + suggested fix
