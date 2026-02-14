# Hexorder — Feature Guide

## Purpose

Every feature in Hexorder is a Bevy Plugin in its own module under `src/`. Feature documentation
lives in `docs/features/<name>/` with two files: a **spec** (what to build) and a **log** (what
happened). This guide covers the full lifecycle of feature documentation.

## Where Feature Docs Live

```
docs/features/
  <name>/
    spec.md    # What to build — scope, success criteria, dependencies
    log.md     # What happened — decisions, test results, blockers
```

Each feature directory mirrors a plugin module in `src/<name>/`.

## When to Create Feature Docs

- **New feature**: When a shaped pitch is bet on and assigned to a cycle
- **Existing feature, new cycle**: When an existing feature gets new scope in a new cycle, update
  the spec and log rather than creating new files

Feature docs are created during the **orientation phase** of a build cycle, before coding begins.

## The Protocol

### Creating a New Feature

1. Create `docs/features/<name>/spec.md` using the [Spec Template](#spec-template) below
2. Create `docs/features/<name>/log.md` using the [Log Template](#log-template) below
3. Register the feature in `docs/coordination.md`
4. If the feature introduces shared types, use the contract skill to create a contract spec

### During Implementation

1. **Spec first**: Read/update the spec before coding — scope items are discovered as work
   progresses
2. **Log decisions**: Record every meaningful decision with rationale in the log
3. **Track test results**: Record test runs with timestamps in the log
4. **Mark success criteria**: Check off criteria as they pass
5. **Capture deferred items**: Out-of-scope ideas go in the Deferred Items section with a
   corresponding GitHub Issue

### Finishing

1. Update the log status to `complete`
2. Update `docs/coordination.md` status
3. Verify all success criteria are checked off

## File Organization

Feature plugins follow this source layout:

```
src/<feature_name>/
  mod.rs             # Plugin definition
  components.rs      # Feature-local components
  systems.rs         # Systems
  events.rs          # Feature-local events
  tests.rs           # Unit tests (#[cfg(test)])
```

## Spec Template

Use this template when creating a new feature spec at `docs/features/<name>/spec.md`:

```markdown
# Feature: [NAME]

## Summary

[1-2 sentences: what does this feature do for the player/game?]

## Plugin

- Module: `src/[name]/`
- Plugin struct: `[Name]Plugin`
- Schedule: [which schedules does this plugin add systems to?]

## Appetite

- **Size**: [Small Batch (1-2 weeks) | Big Batch (full cycle)]
- **Pitch**: [Link to the pitch Issue, e.g., #XX]

## Dependencies

- **Contracts consumed**: [list contract names from docs/contracts/]
- **Contracts produced**: [list any new contracts this feature introduces]
- **Crate dependencies**: [any new crates needed in Cargo.toml]

## Scope

Scope items are discovered as work progresses. This list grows during implementation.

1. [SCOPE-1] [Clear, testable scope item]
2. [SCOPE-2] [Another scope item]

## Success Criteria

Each criterion maps to a scope item. Mark [x] when passing.

- [ ] [SC-1] [How SCOPE-1 is verified — unit test, visual check, etc.]
- [ ] [SC-2] [How SCOPE-2 is verified]
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [ ] [SC-TEST] `cargo test` passes (all tests, not just this feature's)
- [ ] [SC-BOUNDARY] No imports from other features' internals — all cross-feature types come from
      `crate::contracts::`

## Decomposition (for agent teams)

If this feature warrants parallel work, break it into subtasks here.

| Subtask | Description | Owner | Status |
| ------- | ----------- | ----- | ------ |
|         |             |       |        |

## Constraints

- [Any specific constraints, e.g., "must not allocate per-frame"]

## Open Questions

- [Things not yet decided — resolve these in the log]

## Deferred Items

Items explicitly out of scope for this feature (shaped out by the pitch's No Gos or discovered
during implementation). Each item should be captured as a GitHub Issue (raw idea) for potential
future shaping. Note the issue number next to each item.

- [None yet]
```

## Log Template

Use this template when creating a new feature log at `docs/features/<name>/log.md`:

````markdown
# Feature Log: [NAME]

## Status: [speccing | in-progress | testing | blocked | complete]

## Decision Log

Record every meaningful decision with rationale.

### [DATE] — [Decision Title]

**Context**: [Why did this come up?] **Decision**: [What was decided] **Rationale**: [Why this
choice over alternatives] **Alternatives rejected**: [What else was considered]

## Test Results

Record test runs with timestamps.

### [DATE] — [Test Run Description]

```
[paste cargo test output or summary]
```

**Result**: [pass/fail] **Failures**: [list any failures and root cause if known] **Action**: [what
will be done about failures]

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

Items discovered during implementation that are out of scope. Capture each as a GitHub Issue (raw
idea) for potential future shaping. Use:
`gh issue create --label "status:deferred" --label "type:<type>"`. Note the issue number next to
each transferred item.

- [None yet]

## Status Updates

| Date | Status   | Notes                |
| ---- | -------- | -------------------- |
|      | speccing | Initial spec created |
````
