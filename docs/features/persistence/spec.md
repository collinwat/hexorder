# Feature: Persistence

## Summary

Saves and loads game system definitions and board state to `.hexorder` (RON) files. Provides a
launcher screen for creating new projects and opening existing ones, keyboard shortcuts for file
operations, and deferred board reconstruction after load.

## Plugin

- Module: `src/persistence/`
- Plugin struct: `PersistencePlugin`
- Schedule: `Update` (keyboard shortcuts, apply pending board load), observers (save/load/new)

## Dependencies

- **Contracts consumed**: `game_system`, `ontology`, `hex_grid`, `validation`, `persistence`
- **Contracts produced**: `persistence` (shared types in `src/contracts/persistence.rs`)
- **Crate dependencies**: `serde` (serialization), `ron` (file format), `rfd` (native file dialogs)

## Requirements

1. [REQ-1] Save game system definitions + board state to `.hexorder` RON files
2. [REQ-2] Load `.hexorder` files and reconstruct all state (registries + board)
3. [REQ-3] Launcher screen with New/Open buttons on app startup
4. [REQ-4] Keyboard shortcuts: Cmd+S (save), Cmd+Shift+S (save as), Cmd+O (open), Cmd+N (new)
5. [REQ-5] Deferred board load pattern — tiles matched by position, units spawned with core
   components
6. [REQ-6] AppScreen state machine gates all editor systems behind Editor state
7. [REQ-7] File format versioning with forward-compatibility check

## Success Criteria

- [x] [SC-1] Save/load round-trip preserves all fields (unit test)
- [x] [SC-2] Empty board saves and loads successfully (unit test)
- [x] [SC-3] apply_pending_board_load maps tiles and spawns units (unit test)
- [x] [SC-4] File I/O error handling: nonexistent file, malformed RON, unsupported version (unit
      tests)
- [x] [SC-5] Serde round-trip for EntityTypeRegistry, ConceptRegistry, ConstraintExpr (unit tests)
- [x] [SC-6] AppScreen state machine gates editor systems (all existing tests updated)
- [x] [SC-7] Launcher UI registered in EguiPrimaryContextPass with Launcher state gate
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes (all tests, not just this feature's)
- [x] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- Blocking file dialogs (rfd) — acceptable for save/load operations
- MinimalPlugins lacks StatesPlugin — test apps must add it explicitly
- Unit entities spawned without mesh/material — sync systems handle visuals via change detection
