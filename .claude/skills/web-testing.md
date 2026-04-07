# web-testing

L2 AI-driven web UI testing for Stiglab. Runs on PRs only — validates new
and changed UI surfaces using agent-browser against the L2 spec. Regression
is L1's job (deterministic tests). L2 findings get crystallized into L1.

## Spec

```
packages/stiglab-ui/tests/l2/spec.toml
```

Defines routes, flows, and invariants. Each item has `related_paths` — only
run items whose related paths overlap with the PR diff.

## When this runs

- **PR (automatic via CI)**: Diff-scoped spec checks + exploratory on uncovered changes
- **L1 triage (automatic via CI)**: Diagnose why an L1 test failed
- **Manual**: When asked to test specific UI behavior

## Prerequisites

- Dev server or preview server running
- Chrome installed: `npx agent-browser install`

## Execution

### 1. Scope to the PR diff

Read the diff. Match changed files against `related_paths` in the spec.
Only run matching spec items — skip unrelated ones. This keeps L2 cheap.

### 2. Run matching spec items with agent-browser

```bash
npx agent-browser open "http://localhost:5173<path>"
npx agent-browser snapshot            # check must_contain / must_not_contain
npx agent-browser snapshot -i         # for flow steps
npx agent-browser click "@e1"         # interact
npx agent-browser get url             # verify navigation
npx agent-browser evaluate "..."      # check JS errors
npx agent-browser screenshot --output /tmp/l2-<name>.png
npx agent-browser close
```

### 3. Check diff for uncovered changes

If the diff touches UI files not matched by any spec item, do a quick
exploratory check (open the page, snapshot, look for obvious issues).

### 4. Crystallize findings into L1

**This is the key output of L2.** When you validate behavior or find a bug:

- **Validated behavior** → write a new L1 test in `tests/smoke/` or `tests/e2e/`
  that encodes the expected behavior as a deterministic assertion.
- **Bug found** → report it, and once fixed, write an L1 regression test.
- **New spec item needed** → add it to `spec.toml` with `related_paths`.

Example crystallization:
```
L2 finding: "Theme toggle switches class on <html> element"
→ New L1 e2e test: tests/e2e/regression.test.ts
   it("theme toggle switches html class", () => {
     browser.open("/");
     browser.click("@theme-toggle");
     expect(browser.evaluate("document.documentElement.classList")).toContain("dark");
   });
```

### 5. Report

```markdown
## L2 Test Results — PR #N

**Scope**: 3/4 routes, 1/3 flows (diff-matched)
**Status**: PASS / FAIL / PARTIAL

### Checks run
| Spec item | Status | Notes |
|-----------|--------|-------|
| route.dashboard | PASS | |
| flow.theme_toggle | PASS | |

### Skipped (not in diff)
- route.nodes, flow.session_click_through

### Uncovered diff
- `NewComponent.tsx` — exploratory: PASS

### Crystallized
- Added: tests/smoke/new-component.test.tsx (2 tests)

### Issues
- (none)
```

## L1 triage mode

When triggered by L1 failure:

1. Read the failing test file to understand the assertion
2. Open the relevant page with agent-browser
3. Snapshot to see actual state vs expected
4. Diagnose: real regression, environment issue, or flaky test
5. If real regression → report with evidence + suggested fix
6. Once fixed → the existing L1 test serves as the regression guard
