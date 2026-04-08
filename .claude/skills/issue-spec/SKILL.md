---
name: issue-spec
description: "Create lean-spec style GitHub issues as specs for human-AI aligned implementation. Use when asked to 'create a spec', 'write a spec issue', 'spec this feature', 'spec this', or when planning work that needs a specification before implementation. Follows the lean-spec SDD methodology: small focused specs (<2000 tokens), intent over implementation, context economy. Produces GitHub issues with Overview, Design, Plan, Test, and Notes sections."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash(git diff:*), Bash(git log:*), Bash(git show:*), mcp__github__create_issue, mcp__github__issue_write, mcp__github__issue_read, mcp__github__list_issues, mcp__github__search_issues, mcp__github__sub_issue_write, mcp__github__get_label
---

# Issue Spec

Create GitHub issues as lean-spec style specifications for human-AI aligned implementation. Follows the [lean-spec](https://github.com/codervisor/lean-spec) SDD methodology.

## Philosophy

Three principles from lean-spec govern every spec:

1. **Context Economy** — Keep specs under 2000 tokens. If a spec grows larger, split it into parent/child issues. Small specs produce better AI output and better human review.
2. **Intent Over Implementation** — Document the *why* and *what*, not the *how*. Implementation details belong in PRs, not specs. The spec captures human intent that isn't in the code.
3. **Living Documents** — Specs evolve. Start as `draft`, refine through `planned`, track through `in-progress`, close at `complete`. Status reflects reality.

## Setup

| Parameter | Default | Example override |
|-----------|---------|-----------------|
| **Topic** | _(required)_ | `"session timeout"`, `"fix heartbeat race"` |
| **Scope** | Inferred from codebase | `"only stiglab-core"` |
| **Priority** | `medium` | `critical`, `high`, `low` |
| **Labels** | Auto from type + area | `"spec, feat, core"` |
| **Parent** | None | `#42` (umbrella issue) |
| **Publish to** | GitHub issue (if MCP available) | `file` (write to specs/ dir) |

If the user says "spec session timeout", start immediately. Do not ask clarifying questions unless the topic is genuinely ambiguous.

## Workflow

```
1. Discover     Check existing specs and codebase context
2. Design       Draft the spec using lean-spec format
3. Align        Partition human decisions vs AI work
4. Validate     Self-check before publishing
5. Publish      Create GitHub issue or write to file
```

### 1. Discover

Before writing anything, understand what exists:

```
- Search existing issues for related/duplicate specs
- Check specs/ directory for related specs
- Grep codebase for types, functions, modules related to topic
- Read key files that will be affected
- Check git log for recent changes in the area
```

If a related spec already exists, reference it as a dependency or parent — don't duplicate.

### 2. Design

Read [references/spec-format.md](references/spec-format.md) for the section-by-section format guide.

Draft the spec using the lean-spec structure:

```markdown
## Overview
Problem statement and motivation. Why does this matter?

## Design
Technical approach: data flow, API changes, architecture decisions.
Keep it high-level — intent, not implementation.

## Plan
- [ ] Checklist of concrete deliverables
- [ ] Each item independently verifiable
- [ ] Order reflects implementation sequence

## Test
- [ ] How to verify each plan item
- [ ] Include: unit tests, integration tests, manual checks

## Notes
Open questions, context, references, tradeoffs considered.
```

**Context economy check**: If the spec body exceeds ~2000 tokens, split it:
- Extract large design sections into child issues
- Use parent/child relationship (umbrella spec + focused children)
- Each child spec should be independently implementable

### 3. Align

Add a **Human-AI Alignment** section (this extends lean-spec for collaborative development):

```markdown
## Alignment

### Human decides
- [ ] Architectural tradeoffs, scope, UX, go/no-go

### AI implements
- [ ] Concrete code tasks tied to Plan items

### Open questions
> Items that block AI implementation until a human decides
```

This section makes the handoff explicit. The AI knows exactly where to stop and ask, and the human knows what decisions are pending.

**Rules for alignment:**
- Every Plan item should map to either "human decides" or "AI implements"
- If an item requires both, split it — the decision part is human, the execution is AI
- Open questions use `>` blockquotes so they're visually distinct
- Once a human answers an open question, move the decision to "Human decides" with the answer

### 4. Validate

Before publishing, self-check:

- [ ] Spec is under ~2000 tokens (context economy)
- [ ] Overview explains *why*, not just *what*
- [ ] Design captures intent, not implementation details
- [ ] Plan items are concrete and independently verifiable
- [ ] Test items map to Plan items
- [ ] Human/AI boundaries are explicit — no "figure it out" items
- [ ] No duplicate of an existing spec/issue
- [ ] Dependencies are referenced (by issue number or spec name)

### 5. Publish

**If GitHub MCP tools are available:**

Create the issue using `mcp__github__issue_write`:

```
Title: spec(<area>): <short description>
Labels: spec, <type>, <area>, priority:<level>
```

Title format follows commit conventions: `spec(core): add session timeout`, `spec(ui): fix node status badge`.

If this is a child spec, link it to the parent using `mcp__github__sub_issue_write`.

**If GitHub MCP tools are not available:**

Write to `specs/NNN-<short-description>.md` where `NNN` is the next number in sequence. Include YAML frontmatter:

```yaml
---
status: draft
created: YYYY-MM-DD
priority: <level>
tags: [<area>, <type>]
depends_on: []
---
```

**After publishing**, report to the user:
- Issue number/URL (or file path)
- Token count estimate (flag if over 2000)
- Any open questions that need human decisions
- Suggested parent/child relationships if splitting was needed

## Spec Relationships

Use two relationship types (from lean-spec):

| Type | When to use | Example |
|------|------------|---------|
| **Parent/Child** | Large feature decomposed into pieces | `spec(server): session management` → children: timeout, retry, cleanup |
| **Depends On** | Spec blocked until another finishes | `spec(ui): timeout warning` depends on `spec(core): timeout events` |

**Decision rule**: Remove the dependency — does the spec still make sense? If no → parent/child. If yes but blocked → depends_on.

## Guidance

- **Small is better.** A 500-token spec that captures intent clearly beats a 3000-token spec that tries to cover everything. Split early.
- **Discover first.** Always check what exists before creating. Duplicate specs create confusion.
- **Status reflects reality.** Don't mark `planned` if decisions are still open. Don't mark `in-progress` until someone is actually working on it.
- **One concern per spec.** If a spec covers two independent changes, split it into two specs with a shared parent.
- **Reference code, not concepts.** Point to actual types, functions, files — not abstract ideas. Use `crates/stiglab-core/src/session.rs` not "the session module."
- **Open questions are alignment points.** These are where AI must stop and ask a human. Make them explicit, specific, and include the impact of each decision.
- **Specs complement, not replace, issues.** Regular bugs and small tasks don't need specs. Use specs when: multiple stakeholders need alignment, intent needs persistence, or the AI needs clear boundaries.

## References

| Reference | When to Read |
|-----------|--------------|
| [references/spec-format.md](references/spec-format.md) | Always — section-by-section guide with examples |

## Templates

| Template | Purpose |
|----------|---------|
| [templates/issue-spec-template.md](templates/issue-spec-template.md) | Copy and fill — covers standard spec depth |
