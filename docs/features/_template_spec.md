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
