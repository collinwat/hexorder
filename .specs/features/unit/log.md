# Feature Log: unit

## Status: complete

## Decision Log

### 2026-02-09 — Tool mode design
**Context**: M3 introduces unit placement and movement. Need to decide tool mode organization.
**Decision**: Three modes — Select / Paint / Place. Movement is folded into Select mode.
**Rationale**: Fewest new modes. Select naturally handles "click to interact" (select unit, then click destination to move). Dedicated Move mode adds UI complexity without UX benefit.
**Alternatives rejected**: Four modes (Select/Paint/Place/Move) — unnecessary complexity for M3.

### 2026-02-09 — Unit visual representation
**Context**: Units need to be visually distinct from hex tiles on the grid.
**Decision**: Colored cylinders (radius 0.3, half_height 0.2) at Y=0.25 above the tile.
**Rationale**: Follows wargame token/puck convention. Simple geometry, one shared mesh, per-type materials. Visually distinct from flat hex tiles.
**Alternatives rejected**: Spheres (less wargame-like), capsules (more organic than military).

### 2026-02-09 — Unit deletion method
**Context**: Users need to remove placed units during design sessions.
**Decision**: Delete button in the inspector panel when a unit is selected.
**Rationale**: Discoverable, consistent with inspector workflow. Low implementation cost.
**Alternatives rejected**: Delete key only (less discoverable), both key+button (over-engineering for M3).

### 2026-02-09 — Enum definition sharing
**Context**: Both CellTypeRegistry and UnitTypeRegistry need enum definitions for Enum properties.
**Decision**: Duplicate enum_definitions in both registries for M3.
**Rationale**: Avoids breaking changes to CellTypeRegistry. Acceptable duplication at current scale. Flagged for consolidation in a future milestone.
**Alternatives rejected**: Extract to standalone EnumRegistry resource (cleaner but breaking change).

## Test Results

| Date | Command | Result | Notes |
|------|---------|--------|-------|
| 2026-02-09 | `cargo test --lib unit` | 9 passed | All unit-level tests pass |
| 2026-02-09 | `cargo test` | 71 passed | Full suite including 4 new integration tests |
| 2026-02-09 | `cargo clippy -- -D warnings` | clean | Zero warnings |
| 2026-02-09 | unwrap audit | clean | No raw unwrap() in production code |
| 2026-02-09 | unsafe audit | clean | No unsafe code anywhere |

## Blockers

| Blocker | Waiting On | Raised | Resolved |
|---------|-----------|--------|----------|

## Status Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-02-09 | speccing | Initial spec created from M3 plan |
| 2026-02-09 | in-progress | Implementation started: contracts, game_system, unit plugin, editor UI |
| 2026-02-09 | complete | All 13 success criteria met. 71 tests, clippy clean, audit clean. |
