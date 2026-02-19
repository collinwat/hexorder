# Plugin Log: Undo/Redo

## Status: in-progress

## Decision Log

### 2026-02-18 — Plugin name and module structure

**Context**: Naming the undo/redo plugin module. **Decision**: Use `undo_redo` as the module name,
`UndoRedoPlugin` as the plugin struct. **Rationale**: Follows existing naming conventions (e.g.,
`rules_engine`, `game_system`). Underscore separator matches Rust module naming conventions.
**Alternatives rejected**: `history` (too vague), `commands` (conflicts with Bevy's Commands type).

### 2026-02-18 — Research context

**Context**: Checked wiki for prior research on undo/redo. **Decision**: No dedicated research
exists. Design Tool Interface Patterns wiki page identifies undo/redo as a fundamental requirement
and its absence as an anti-pattern. Was a deliberate no-go in Cycle 4 (v0.9.0), now being addressed.
**Rationale**: The pitch is well-shaped with clear mitigation strategies for rabbit holes.

### 2026-02-18 — Record-then-undo architecture

**Context**: How should forward mutations and undo interact with Bevy's ECS? **Decision**: Use
record-then-undo pattern. Forward mutations happen inline (plugins mutate state directly), then call
`undo_stack.record(cmd)`. Only undo/redo goes through an exclusive system with `&mut World` access.
**Rationale**: Avoids one-frame delay that would break egui widget reads. The exclusive system only
runs when `pending_undo` or `pending_redo` flags are set. **Alternatives rejected**:
Push-then-execute (one-frame delay), event-based (timing issues with observers).

### 2026-02-18 — Cross-pitch coordination

**Context**: #121 (Editor QoL) is also in 0.10.0, touching editor_ui. **Decision**: No blocking
dependencies. Both touch editor_ui in separate areas. New mutation patterns from #121 (multi-select)
won't have undo support initially — per the pitch's incremental adoption strategy. **Rationale**:
The pitch explicitly states "actions without commands are simply not undoable — no crash, just no
undo."

## Test Results

### 2026-02-18 — Scope 1+2 (contract + plugin + SetPropertyCommand)

- 10 contract tests: stack operations, depth enforcement, redo clearing, round-trip, descriptions
- 6 plugin tests: resource insertion, undo reversal, redo reapplication, shortcut-triggered
  undo/redo, redo cleared on new record
- 258 total tests pass (including all existing tests)
- `cargo clippy --all-targets -- -D warnings`: zero warnings
- `mise check:boundary`: no violations
- `mise check:unwrap`: no violations

### 2026-02-18 — Scope 3 (`SetTerrainCommand` + paint undo)

- 3 new cell tests: paint records undo, paint+undo reverts terrain, no-op paint skips recording
- Modified `paint_cell` observer to capture old state and record `SetTerrainCommand`
- Added no-op paint detection (skip if already same type)
- Integration tests needed `UndoStack` resource added to headless test app
- 261 total tests pass
- `cargo clippy --all-targets -- -D warnings`: zero warnings
- `mise check:boundary`: no violations
- `mise check:unwrap`: no violations

### 2026-02-18 — Scope 4 (`PlaceUnitCommand` + unit placement undo)

- Added `PlaceUnitCommand` to undo_redo contract with full spawn/despawn lifecycle
- Modified `handle_unit_placement` to record placement on undo stack
- 3 new unit tests: placement records undo, place+undo removes, place+undo+redo restores
- Custom `Debug` impl with `finish_non_exhaustive()` to satisfy clippy
- 264 total tests pass
- `cargo clippy --all-targets -- -D warnings`: zero warnings

### 2026-02-18 — Scope 5 (`CompoundCommand`)

- Added `CompoundCommand` to undo_redo contract
- Execute runs all sub-commands in order; undo reverses in reverse order
- 3 new tests: execute, debug format, stack integration
- Updated contract spec doc with all new command types
- 267 total tests pass

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- Rule change commands — incremental adoption in future cycles
- Persistent undo history — pitch no-go
- Branching undo tree — pitch no-go

## Status Updates

| Date       | Status      | Notes                                                      |
| ---------- | ----------- | ---------------------------------------------------------- |
| 2026-02-18 | speccing    | Initial spec created, branch set up, kickoff done          |
| 2026-02-18 | in-progress | Scope 1+2 complete: contract, plugin, 16 tests passing     |
| 2026-02-18 | in-progress | Scope 3 complete: SetTerrainCommand, paint undo, 261 tests |
| 2026-02-18 | in-progress | Scope 4 complete: PlaceUnitCommand, unit undo, 264 tests   |
| 2026-02-18 | in-progress | Scope 5 complete: CompoundCommand, 267 tests               |
