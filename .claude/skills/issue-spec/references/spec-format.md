# Spec Format Reference

This document defines the formal structure of lean-spec style issue specs. Every section maps to a concept from formal verification, adapted for practical software engineering.

## Lean 4 → Issue Spec Mapping

| Lean 4 Concept | Issue Spec Section | Purpose |
|----------------|-------------------|---------|
| `theorem` | **Title** | Names the proposition being specified |
| `hypothesis` | **Preconditions** | What must be true before work begins |
| `goal` | **Postconditions** | What must be true after implementation |
| `invariant` | **Invariants** | Properties preserved throughout |
| `def` | **Definitions** | Terms and types used in the spec |
| `structure` | **Interface Contracts** | Shape of new/modified APIs |
| `proof` | **Verification** | How to prove postconditions hold |
| `import` | **Dependencies** | Other specs/issues this builds on |
| `sorry` | **Open Questions** | Unresolved decisions (must be filled before implementation) |

## Section Details

### Title

Format: `spec(<area>): <proposition in imperative mood>`

The title is a one-line summary of what the spec establishes. Use the `spec` type prefix to distinguish from regular issues.

Examples:
- `spec(core): session state machine rejects invalid transitions`
- `spec(server): WebSocket reconnection preserves session state`
- `spec(ui): dashboard renders node status within 2s of heartbeat`

### Definitions

Define terms, types, and concepts used throughout the spec. This prevents ambiguity — every term in the spec should either be:
1. Defined in this section
2. A well-known term in the project (reference where it's defined)
3. A standard programming/domain term

```markdown
## Definitions

- **Session timeout**: Duration after which a session in `WAITING_INPUT` state
  with no user activity transitions to `FAILED`. Configurable via
  `STIGLAB_SESSION_TIMEOUT` (default: 30m).
- **Activity**: Any message received on the session's WebSocket channel,
  including heartbeats.
- **Graceful timeout**: Timeout that sends a warning event 5 minutes before
  transitioning, allowing the UI to prompt the user.
```

### Preconditions

What must be true in the codebase **right now** for this spec to be implementable. These are checked before work begins — if any precondition is false, the spec is blocked.

Write as verifiable propositions:

```markdown
## Preconditions

- [ ] `SessionState` enum exists in `crates/stiglab-core/src/session.rs`
      and includes `WaitingInput` variant
- [ ] Server WebSocket handler in `crates/stiglab-server/src/ws/agent.rs`
      processes `SessionStateChanged` messages
- [ ] No existing timeout mechanism exists (grep for "timeout" in session
      management code returns no results)
- [ ] `tokio::time` is available in workspace dependencies
```

**Verification**: Each precondition should be checkable with a grep, file read, or test run. If a precondition requires human judgment, mark it with `(human-verified)`.

### Postconditions

What must be true **after implementation**. These are the acceptance criteria expressed as testable propositions. Every postcondition must have a corresponding verification method.

```markdown
## Postconditions

### P1: Timeout configuration exists
- `STIGLAB_SESSION_TIMEOUT` env var is read in server config
- Default value is 30 minutes
- Value is validated (must be > 0, must be parseable as duration)

### P2: Sessions timeout after inactivity
- A session in `WaitingInput` state with no activity for `SESSION_TIMEOUT`
  duration transitions to `Failed` with reason `"session_timeout"`
- The transition is recorded in the database with timestamp
- A `SessionStateChanged` event is emitted to connected dashboards

### P3: Activity resets the timeout
- Any WebSocket message on a session's channel resets the timeout timer
- Timer reset is idempotent (multiple resets don't stack or leak)

### P4: Warning before timeout
- 5 minutes before timeout, a `SessionTimeoutWarning` event is emitted
- Warning includes `session_id` and `seconds_remaining`
- Warning is only emitted once per timeout cycle
```

**Rules for postconditions:**
- Number them (P1, P2, ...) for cross-referencing
- Each must be independently testable
- Use concrete values, not vague qualifiers ("fast" → "within 200ms")
- Reference actual types and modules where possible

### Invariants

Properties that must hold **at all times** — before, during, and after implementation. Invariants are the strongest claims and should be verified by both tests and code review.

```markdown
## Invariants

- **I1: State machine validity** — `SessionState` transitions remain valid
  per the state machine in `specs/000-architecture.md`. Timeout introduces
  no new transitions except `WaitingInput → Failed`.
- **I2: No session data loss** — Timeout does not delete session data.
  Output collected before timeout is preserved in the database.
- **I3: Distributed consistency** — If the agent node disconnects before
  receiving the timeout cancellation, the server is the source of truth.
  The session is `Failed` on the server regardless of agent state.
```

### Interface Contracts

For any new or modified public APIs (functions, HTTP endpoints, WebSocket messages, CLI flags), specify the contract.

```markdown
## Interface Contracts

### New: `SessionTimeoutWarning` event
```rust
// In crates/stiglab-core/src/event.rs
pub struct SessionTimeoutWarning {
    pub session_id: String,
    pub seconds_remaining: u32,
    pub timeout_at: DateTime<Utc>,
}
```

### Modified: `POST /api/tasks` response
- **Added field**: `timeout_seconds: Option<u64>` — if set, overrides
  the server default for this session
- **Backward compatible**: field is optional, existing clients unaffected

### New: `STIGLAB_SESSION_TIMEOUT` environment variable
- **Format**: Duration string (`"30m"`, `"1h"`, `"3600s"`)
- **Default**: `"30m"`
- **Validation**: Must parse to positive duration. Server refuses to
  start with invalid value (fail-fast, not silent default).
```

### Dependencies

Other specs, issues, or PRs that this spec depends on or relates to.

```markdown
## Dependencies

- **Requires**: #42 (WebSocket event broadcasting) — P2 and P4 depend on
  the broadcast mechanism specified there
- **Blocked by**: None
- **Enables**: #58 (auto-cleanup of timed-out sessions)
- **Related**: specs/000-architecture.md (state machine definition)
```

### Human-AI Alignment

The most important section for collaborative development. Explicitly partitions the work.

```markdown
## Human-AI Alignment

### Human decides
- [ ] Timeout default value (proposed: 30m — acceptable?)
- [ ] Whether timeout should be per-session configurable via API
      or server-global only
- [ ] Warning UX: toast notification vs. modal vs. inline banner

### AI implements (given decisions above)
- [ ] `SessionTimeout` config struct and env var parsing
- [ ] Timeout tracking in `SessionManager` using `tokio::time::sleep`
- [ ] `SessionTimeoutWarning` event type and emission logic
- [ ] State transition `WaitingInput → Failed` with reason
- [ ] Unit tests for P1-P4
- [ ] Integration test: session times out after configured duration

### Joint verification
- [ ] Timeout behavior under network partition (agent disconnects
      mid-timeout) — review together
- [ ] Performance impact of per-session timers at scale (100+ sessions)
```

### Verification

How each postcondition is proven to hold. Maps directly to test plan.

```markdown
## Verification

| Postcondition | Method | Location |
|--------------|--------|----------|
| P1 | Unit test: config parsing | `crates/stiglab-server/src/config.rs` |
| P1 | Unit test: validation rejects invalid | `crates/stiglab-server/src/config.rs` |
| P2 | Integration test: session timeout flow | `tests/session_timeout.rs` |
| P3 | Unit test: timer reset on activity | `crates/stiglab-server/src/session/timeout.rs` |
| P4 | Integration test: warning event emitted | `tests/session_timeout.rs` |
| I1 | Existing state machine tests pass | `crates/stiglab-core/src/session.rs` |
| I2 | Integration test: output preserved after timeout | `tests/session_timeout.rs` |
```

### Open Questions

Unresolved items that block implementation. These are the `sorry` markers — gaps in the proof that must be filled by a human before the AI can proceed.

```markdown
## Open Questions

> **DECISION NEEDED:** Should timed-out sessions be automatically retryable,
> or must the user explicitly create a new task?
>
> Impact: Changes P2 — if retryable, final state might be `Pending`
> instead of `Failed`.

> **DECISION NEEDED:** What happens to the agent process when a session
> times out? Kill immediately, or send SIGTERM and wait?
>
> Impact: Affects I2 (data preservation) and I3 (distributed consistency).
```

## Spec Depth Guide

### Minimal (bug fixes, config changes)

Include only:
- Title
- Preconditions (brief)
- Postconditions (1-3 propositions)
- Verification (1-2 methods)

### Standard (features, refactors)

Include all sections. This is the default.

### Comprehensive (architecture changes, new subsystems)

Include all sections plus:
- **Migration plan**: How to get from current state to postconditions without downtime
- **Rollback strategy**: How to revert if postconditions can't be achieved
- **Performance budget**: Quantified constraints (latency, memory, throughput)
- **Security considerations**: Threat model changes, new attack surfaces
