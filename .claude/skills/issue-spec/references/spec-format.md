# Spec Format Reference

Section-by-section guide for writing lean-spec style issue specs. Based on the [lean-spec SDD methodology](https://github.com/codervisor/lean-spec).

## Frontmatter (file specs only)

When writing specs to files (not GitHub issues), include YAML frontmatter:

```yaml
---
status: draft          # draft | planned | in-progress | complete | archived
created: 2026-04-08
priority: high         # critical | high | medium | low
tags: [core, feat]     # area + type tags
depends_on: []         # list of spec numbers or issue numbers
---
```

For GitHub issues, these map to labels and issue metadata instead.

### Status Lifecycle

```
draft → planned → in-progress → complete
  ↓        ↓          ↓            ↓
  └────────┴──────────┴────────→ archived
```

- **draft**: Spec written but not reviewed. Open questions may remain.
- **planned**: Human has reviewed, decisions made, ready for implementation.
- **in-progress**: Someone is actively working on it.
- **complete**: All plan items done, all tests passing.
- **archived**: No longer relevant (superseded, abandoned, or completed long ago).

**Key rule**: Don't skip `planned`. A spec goes from `draft` to `planned` only after a human reviews it and resolves open questions. This is the primary human-AI alignment gate.

## Sections

### Overview

**Purpose**: Why does this work matter? What problem does it solve?

**Good overview:**
```markdown
## Overview

Sessions in `WAITING_INPUT` state can hang indefinitely if the user
disconnects. This wastes agent capacity and leaves stale sessions in
the dashboard. We need a configurable timeout that transitions idle
sessions to `FAILED` after a period of inactivity.
```

**Bad overview:**
```markdown
## Overview

Add a timeout feature to sessions.
```

The bad version says *what* but not *why*. The AI has no context to make tradeoff decisions during implementation.

**Guidelines:**
- 2-4 sentences. Problem → impact → what we need.
- Reference specific code/behavior when possible.
- Don't describe the solution here — that's Design's job.

### Design

**Purpose**: How should this work? What's the technical approach?

**Write intent, not implementation:**
```markdown
## Design

Each session gets an inactivity timer that resets on any WebSocket
message. When the timer expires:
1. Emit a `SessionTimeoutWarning` event 5 minutes before deadline
2. Transition state to `Failed` with reason `session_timeout`
3. Preserve all session output collected before timeout

The timeout duration is server-configurable via environment variable.
Per-session overrides are out of scope for now.
```

**Guidelines:**
- Describe data flow, state changes, API surface — not line-by-line code.
- Include what's explicitly **out of scope** to prevent scope creep.
- If design is complex, extract subsections into child specs.
- Reference existing architecture: "follows the pattern in `specs/000-architecture.md`"

### Plan

**Purpose**: Concrete deliverables as a checklist. Each item is independently verifiable.

```markdown
## Plan

- [ ] Add `STIGLAB_SESSION_TIMEOUT` env var to server config (default: 30m)
- [ ] Implement per-session inactivity timer in `SessionManager`
- [ ] Add `SessionTimeoutWarning` event type to `stiglab-core`
- [ ] Emit warning event 5 minutes before timeout
- [ ] Transition `WaitingInput → Failed` on timeout expiry
- [ ] Preserve session output on timeout (no data deletion)
- [ ] Add timeout info to `GET /api/sessions/:id` response
```

**Guidelines:**
- Each item starts with a verb: Add, Implement, Update, Remove, Fix.
- Items should be small enough to verify in isolation.
- Order reflects implementation sequence.
- If a plan has more than ~10 items, the spec is too big — split it.

### Test

**Purpose**: How to verify each plan item is done correctly.

```markdown
## Test

- [ ] Unit test: config parses valid duration strings, rejects invalid
- [ ] Unit test: timer resets on WebSocket message
- [ ] Integration test: session transitions to Failed after timeout
- [ ] Integration test: warning event emitted 5 minutes before timeout
- [ ] Integration test: session output preserved after timeout
- [ ] Manual: dashboard shows timeout state and warning indicator
```

**Guidelines:**
- Each test item maps to one or more plan items.
- Specify test type: unit, integration, manual, type check, lint.
- Include negative cases: "rejects invalid", "does not delete".
- For manual tests, describe what to check — not exact click paths.

### Notes

**Purpose**: Context, tradeoffs, references, and anything that doesn't fit elsewhere.

```markdown
## Notes

- Considered per-session timeout overrides via API, but deferred to keep
  scope small. Can add in a follow-up spec.
- The timeout timer approach uses `tokio::time::sleep` per session.
  At 100+ concurrent sessions this may need optimization (timer wheel).
- Related: Crawlab uses a similar heartbeat-based timeout for spider tasks.
```

**Guidelines:**
- Tradeoffs considered and why you chose this approach.
- Performance or scalability concerns for future reference.
- Links to related specs, issues, external resources.
- Keep it brief — notes are context, not a second design section.

### Alignment (extension for human-AI collaboration)

**Purpose**: Explicit partition of work between human and AI.

```markdown
## Alignment

### Human decides
- [ ] Timeout default value (proposed: 30m)
- [ ] Whether to show UI warning (toast vs. banner)
- [ ] Behavior on network partition (agent disconnects mid-timeout)

### AI implements
- [ ] Config parsing and validation
- [ ] Timer logic in SessionManager
- [ ] Event types and emission
- [ ] State transition + database update
- [ ] Unit and integration tests per Test section

### Open questions
> Should timed-out sessions be retryable, or must the user create a new task?
> Impact: changes whether final state is `Failed` or `Pending`.

> What happens to the agent process when a session times out?
> Impact: affects data preservation and distributed consistency.
```

**Guidelines:**
- Every Plan item maps to exactly one of: "Human decides" or "AI implements."
- Human items are decisions/tradeoffs. AI items are execution.
- Open questions block implementation — they must be resolved before status moves to `planned`.
- Once resolved, move the answer to "Human decides" as a recorded decision.

## Context Economy Rules

From lean-spec's core principle — smaller specs produce better results:

| Spec size | Action |
|-----------|--------|
| < 500 tokens | Good for bug fixes and small changes |
| 500-2000 tokens | Standard spec — covers most features |
| > 2000 tokens | Split into parent + children |

**How to split:**
1. Create an umbrella issue (parent) with Overview + high-level Plan.
2. Create child issues, one per independent concern.
3. Each child has its own Design, Plan, Test sections.
4. Children reference the parent. Parent lists all children.

**Example split:**
```
spec(server): session lifecycle improvements  (parent)
├── spec(core): session timeout mechanism     (child)
├── spec(core): session retry on failure      (child)
└── spec(ui): timeout warning indicator       (child)
```
