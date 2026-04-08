---
name: issue-spec
description: "Create lean-spec style GitHub issues as formal specifications for human-AI aligned implementation. Use when asked to 'create a spec', 'write a spec issue', 'spec this feature', 'formalize this requirement', 'create an issue spec', or when planning work that needs a rigorous specification before implementation. Produces structured GitHub issues with formal preconditions, postconditions, invariants, and human-AI alignment boundaries."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash(git diff:*), Bash(git log:*), Bash(git show:*), mcp__github__create_issue, mcp__github__list_issues, mcp__github__get_issue, mcp__github__search_issues
---

# Issue Spec

Create lean-spec style GitHub issues that serve as formal specifications for human-AI aligned implementation. Inspired by Lean 4's formal verification approach — every spec is a set of propositions that can be verified.

## Philosophy

Traditional GitHub issues describe *what* to build. Issue specs describe *what must be true* when the work is done. This shift from imperative ("add X") to declarative ("X holds") makes specs:

- **Verifiable**: Each acceptance criterion is a testable proposition
- **Unambiguous**: Formal structure eliminates interpretation gaps between human and AI
- **Composable**: Specs reference each other like theorems reference lemmas
- **Aligned**: Explicit boundaries between human decisions and AI-implementable work

## Setup

| Parameter | Default | Example override |
|-----------|---------|-----------------|
| **Topic** | _(required)_ | `"add session timeout"`, `"fix node heartbeat race condition"` |
| **Scope** | Inferred from topic + codebase | `"only stiglab-core crate"` |
| **Depth** | `standard` | `minimal` (bug fix), `comprehensive` (architecture change) |
| **Labels** | Auto-assigned by type | `"priority: high, area: core"` |
| **Create on GitHub** | `true` (if MCP available) | `false` (output to file instead) |

If the user says "spec session timeout", start immediately — analyze the codebase and draft the spec. Do not ask clarifying questions unless the topic is genuinely ambiguous.

## Workflow

```
1. Understand    Analyze codebase context for the topic
2. Formalize     Structure the spec using lean-spec format
3. Align         Define human/AI boundaries
4. Verify        Self-check the spec for completeness
5. Publish       Create the GitHub issue or output to file
```

### 1. Understand

Before writing anything, build context:

**Identify affected code:**
```
- Grep for related types, functions, modules
- Read key files that will be touched
- Check git log for recent changes in the area
- Look for existing specs in specs/ directory
- Search existing GitHub issues for related work
```

**Determine spec depth:**
- `minimal` — Bug fixes, typos, small config changes. Just precondition/postcondition/verification.
- `standard` — Features, refactors, integrations. Full spec with all sections.
- `comprehensive` — Architecture changes, new subsystems, breaking changes. Full spec + migration plan + rollback strategy.

### 2. Formalize

Read [references/spec-format.md](references/spec-format.md) for the complete format specification.

Draft the spec using the template:

```bash
# Reference the template while drafting
cat {SKILL_DIR}/templates/issue-spec-template.md
```

**Key principles for formalization:**

- **Preconditions**: What must be true in the codebase *before* this work begins. Include version constraints, feature flags, dependencies.
- **Postconditions**: What must be true *after* implementation. These become acceptance criteria. Write as testable propositions.
- **Invariants**: Properties that must hold *throughout* — both during and after implementation. System properties that must not be violated.
- **Interface contracts**: For any new or modified public APIs, specify the contract: input types, output types, error conditions, side effects.

Every proposition should be verifiable — either by a test, a type check, a lint rule, or manual inspection.

### 3. Align

The human-AI alignment section is critical. For each part of the spec, categorize:

**Human decides** (requires judgment, context, or authority):
- Architectural tradeoffs and approach selection
- Scope boundaries and priority
- UX decisions and user-facing behavior
- Security model and trust boundaries
- Whether to proceed at all (go/no-go)

**AI implements** (given the spec, can be executed mechanically):
- Code changes matching the postconditions
- Test creation matching verification criteria
- Documentation updates
- Refactoring within defined boundaries

**Joint verification** (both human and AI check):
- Integration behavior matches expectations
- Performance characteristics are acceptable
- No unintended side effects on adjacent systems

Mark any open questions that block implementation with `DECISION NEEDED:` prefix.

### 4. Verify

Before publishing, self-check the spec:

- [ ] Every postcondition is testable (can write a test or check for it)
- [ ] Preconditions reference actual current state (not assumed state)
- [ ] Invariants are not violated by the proposed postconditions
- [ ] No circular dependencies with other specs/issues
- [ ] Human/AI boundaries are explicit — no ambiguous "figure it out" items in AI scope
- [ ] Scope is bounded — the spec doesn't grow unboundedly
- [ ] At least one verification method exists for each acceptance criterion

### 5. Publish

**If GitHub MCP tools are available:**

Create the issue with appropriate labels and formatting:

```
Title: spec(<area>): <short description>
Labels: spec, <type>, <area>
```

**If GitHub MCP tools are not available:**

Write the spec to a file:

```bash
# Write to specs/ directory
write specs/NNN-<short-description>.md
```

Where `NNN` is the next number in the `specs/` sequence.

**After publishing**, report to the user:
- Issue number/URL (or file path)
- Summary of key propositions
- Any items marked `DECISION NEEDED:`
- Suggested implementation order if multiple specs are related

## Guidance

- **Specs are not implementation plans.** A spec says *what must be true*, not *how to make it true*. Implementation details belong in the PR, not the spec. The exception is when a specific implementation approach is itself a requirement (e.g., "must use streaming, not polling").
- **Start concrete, then generalize.** Write the specific postconditions first, then extract invariants. Don't start with abstract principles.
- **Reference existing code.** Postconditions should reference actual types, functions, and modules — not abstract concepts. Use `file:line` notation.
- **One spec per concern.** If a spec covers two independent changes, split it. Specs should be atomic — implementable and verifiable independently.
- **Version your assumptions.** Preconditions pin the spec to a point in time. If the codebase changes, preconditions may no longer hold, and the spec needs revision.
- **Don't over-formalize.** A bug fix doesn't need invariants and interface contracts. Match depth to complexity.
- **Label open questions explicitly.** Anything that can't be resolved by reading the code needs a `DECISION NEEDED:` marker. These are the human alignment points — places where the AI must stop and ask.
- **Compose specs like theorems.** Complex features should be broken into a chain of specs, where each spec's postconditions become the next spec's preconditions. Reference dependent specs by issue number.

## References

| Reference | When to Read |
|-----------|--------------|
| [references/spec-format.md](references/spec-format.md) | Always — defines the formal structure of every spec |

## Templates

| Template | Purpose |
|----------|---------|
| [templates/issue-spec-template.md](templates/issue-spec-template.md) | Base template for all issue specs — copy and fill in |
