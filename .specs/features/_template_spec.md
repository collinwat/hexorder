# Feature: [NAME]

## Summary

[1-2 sentences: what does this feature do for the player/game?]

## Plugin

- Module: `src/[name]/`
- Plugin struct: `[Name]Plugin`
- Schedule: [which schedules does this plugin add systems to?]

## Dependencies

- **Contracts consumed**: [list contract names from .specs/contracts/]
- **Contracts produced**: [list any new contracts this feature introduces]
- **Crate dependencies**: [any new crates needed in Cargo.toml]

## Requirements

1. [REQ-1] [Clear, testable requirement]
2. [REQ-2] [Another requirement]

## Success Criteria

Each criterion maps to a requirement. Mark [x] when passing.

- [ ] [SC-1] [How REQ-1 is verified — unit test, visual check, etc.]
- [ ] [SC-2] [How REQ-2 is verified]
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
