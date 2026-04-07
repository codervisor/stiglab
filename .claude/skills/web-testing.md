# web-testing

L2 AI-driven web UI testing for Stiglab. Runs on PRs — validates new and
changed UI surfaces using agent-browser. Regression is L1's job. L2 findings
get crystallized into L1.

## Inputs

L2 doesn't need a separate spec. It works from:
- **PR diff** — what changed (determines what to test)
- **Existing L1 tests** — what's already crystallized (don't duplicate)
- **agent-browser** — to explore the running UI

## When this runs

- **PR (automatic via CI)**: Read the diff, test affected pages
- **L1 triage (automatic via CI)**: Diagnose why an L1 test failed
- **Manual**: When asked to test specific UI behavior

## Prerequisites

- Dev server or preview server running
- Chrome installed: `npx agent-browser install`

## Execution

### 1. Read the diff

Identify which UI files changed. Map them to affected routes:
- `src/pages/*Page.tsx` → the corresponding route
- `src/components/*` → pages that use that component
- `src/hooks/*`, `src/lib/*` → all pages that import it
- `src/App.tsx`, `src/components/layout/*` → all routes

### 2. Test affected pages with agent-browser

```bash
npx agent-browser open "http://localhost:5173<route>"
npx agent-browser snapshot            # verify page content
npx agent-browser snapshot -i         # get interactive elements
npx agent-browser click "@e1"         # test interactions
npx agent-browser get url             # verify navigation
npx agent-browser evaluate "..."      # check for JS errors
npx agent-browser screenshot --output /tmp/l2-<name>.png
npx agent-browser close
```

For each affected page, check:
- Page loads without errors
- Changed elements render correctly
- No "undefined", "NaN", or uncaught errors in snapshot
- Interactions still work (navigation, clicks, toggles)

### 3. Crystallize findings into L1

**This is the key output of L2.** When you validate new behavior or find a bug:

- **Validated new behavior** → write a new L1 test in `tests/smoke/` or
  `tests/e2e/` that encodes it as a deterministic assertion.
- **Bug found** → report it. Once fixed, write an L1 regression test.

Example:
```
L2 validated: "New status filter dropdown renders 6 options"
→ New L1 test: tests/smoke/session-filter.test.tsx
```

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
6. The existing L1 test already serves as the regression guard once fixed
