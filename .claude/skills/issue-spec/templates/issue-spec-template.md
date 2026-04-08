<!-- issue-spec template — lean-spec style formal specification -->
<!-- Title format: spec(<area>): <short proposition in imperative mood> -->
<!-- Labels: spec, <type: feat|fix|refactor|perf>, <area> -->

## Definitions

<!-- Define domain terms, types, and concepts used in this spec.
     Every non-obvious term below should be defined here or reference
     where it's defined in the codebase. -->

- **Term**: Definition referencing `crate::module::Type` where applicable

## Preconditions

<!-- What must be true in the codebase RIGHT NOW for this spec to be
     implementable. Each should be verifiable with a grep, file read,
     or test run. -->

- [ ] <!-- Precondition 1: verifiable assertion about current codebase state -->
- [ ] <!-- Precondition 2 -->

## Postconditions

<!-- What must be true AFTER implementation. These are your acceptance
     criteria as testable propositions. Number them for cross-reference. -->

### P1: <!-- Short name -->
<!-- Testable proposition describing what holds after implementation -->

### P2: <!-- Short name -->
<!-- Testable proposition -->

## Invariants

<!-- Properties that must hold AT ALL TIMES — before, during, and after.
     These are the strongest claims. Omit for minimal-depth specs. -->

- **I1: <!-- Name -->** — <!-- Property that must never be violated -->
- **I2: <!-- Name -->** — <!-- Property that must never be violated -->

## Interface Contracts

<!-- New or modified public APIs: functions, endpoints, messages, env vars.
     Specify types, errors, side effects. Omit for minimal-depth specs. -->

### <!-- New/Modified: API name -->
```
<!-- Type signature, endpoint spec, or message schema -->
```

## Dependencies

<!-- Related specs, issues, and PRs -->

- **Requires**: <!-- #N or none -->
- **Blocked by**: <!-- #N or none -->
- **Enables**: <!-- #N or none -->
- **Related**: <!-- specs/NNN-*.md, docs, etc. -->

## Human-AI Alignment

### Human decides
<!-- Architectural choices, tradeoffs, UX, security model, scope -->
- [ ] <!-- Decision that requires human judgment -->

### AI implements
<!-- Concrete tasks executable given the spec and human decisions above -->
- [ ] <!-- Implementation task matching a postcondition -->

### Joint verification
<!-- Items that both human and AI should review together -->
- [ ] <!-- Behavior or property that needs collaborative review -->

## Verification

<!-- How each postcondition and invariant is proven to hold -->

| Proposition | Method | Location |
|-------------|--------|----------|
| P1 | <!-- unit test / integration test / type check / manual --> | <!-- file path --> |
| P2 | <!-- method --> | <!-- file path --> |
| I1 | <!-- method --> | <!-- file path --> |

## Open Questions

<!-- Unresolved decisions that block implementation.
     Remove this section if there are none. -->

> **DECISION NEEDED:** <!-- Question -->
>
> Impact: <!-- Which postconditions/invariants are affected -->
