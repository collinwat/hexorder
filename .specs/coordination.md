# Hexorder Coordination

## Active Milestone: M3 — "Things Live in the World"

## Active Features

| Feature | Owner | Status | Dependencies | Notes |
|---------|-------|--------|--------------|-------|
| hex_grid | — | complete (M1) | none | Unchanged for M3. Grid rendering, tile selection, hover feedback. |
| camera | — | complete (M1) | none | Unchanged for M3. Orthographic top-down, pan + zoom. |
| game_system | — | complete (M3) | none | M3: added UnitTypeRegistry, ActiveUnitType, SelectedUnit, 3 starter unit types. |
| cell | — | complete (M2) | hex_grid contract, game_system contract, editor_ui contract | Unchanged for M3. Cell painting + visual sync. |
| unit | — | complete (M3) | hex_grid contract, game_system contract, editor_ui contract | M3: Unit placement, movement, visual sync, deletion. 9 unit tests + 4 integration tests. |
| editor_ui | — | complete (M3) | hex_grid contract, game_system contract | M3: Place tool, unit palette, unit type editor, unit inspector. |

Status values: `speccing` | `in-progress` | `testing` | `blocked` | `complete` | `retiring`

## Plugin Load Order
Declared in `main.rs`. Update this when adding a new plugin.
1. DefaultPlugins (built-in)
2. HexGridPlugin
3. CameraPlugin
4. GameSystemPlugin (must be before CellPlugin, UnitPlugin, and EditorUiPlugin)
5. CellPlugin
6. UnitPlugin
7. EditorUiPlugin

## Pending Contract Changes

| Contract | Proposed By | Change Description | Affected Features | Status |
|----------|-------------|-------------------|-------------------|--------|
| game_system | game_system | NEW — GameSystem, PropertyType, PropertyDefinition, PropertyValue, EnumDefinition, CellType, CellTypeRegistry, CellData, ActiveCellType | cell, editor_ui | done |
| terrain | M2 | RETIRED — TerrainType, Terrain, TerrainPalette, TerrainEntry, ActiveTerrain removed | — | done |
| editor_ui | — | Unchanged — EditorTool stays as-is | cell | done |
| hex_grid | — | Unchanged | cell, editor_ui | done |
| game_system | M3 | ADD — UnitType, UnitTypeRegistry, UnitData, UnitInstance, ActiveUnitType, SelectedUnit, UnitPlacedEvent | unit, editor_ui | done |
| editor_ui | M3 | EVOLVE — EditorTool gains Place variant | unit, cell | done |

Status: `proposed` | `approved` | `implementing` | `done`

## Cross-Cutting Concerns
- **3D rendering**: Application uses Camera3d with orthographic projection, locked top-down
- **Hex coordinate system**: All features using hex positions must use `HexPosition` from `contracts::hex_grid`
- **Input separation**: Left-click for selection/painting, middle-click for camera pan, scroll for zoom. bevy_egui consumes input when mouse is over UI panels (via `egui_wants_any_pointer_input` run condition).
- **Game System**: M2 introduces the Game System container (id + version). All design definitions (cell types, future unit types) live inside the Game System.
- **Property system**: Entity-agnostic. PropertyDefinition and PropertyValue are reused across cell types (M2), unit types (M3), and any future entity types.
- **Terminology**: Hex tiles on the board are "cells." Game entities on tiles are "units." Their types and properties are defined by the Game System. "Terrain" terminology is retired. "Vertex" terminology is retired (vertex refers to hex corners in grid geometry).
- **Unit placement**: Units are separate entities from hex tiles. They share HexPosition for grid location. Multiple units can occupy the same tile (no stacking rules in M3).
- **Enum definition duplication**: Both CellTypeRegistry and UnitTypeRegistry have their own enum_definitions. Flagged for future consolidation into a standalone resource.
- **Serialization**: Not needed for M3; all state is ephemeral
- **Editor tool mode**: `EditorTool` resource (owned by editor_ui) must be checked by cell before painting.
- **Module privacy enforcement**: Feature sub-modules are `mod` (private). Contract boundary violations are compile errors + enforced by `architecture_tests::feature_modules_are_private`.

## Feature Dependency Graph (M3)
```
game_system (contract) ──→ cell
game_system (contract) ──→ unit
game_system (contract) ──→ editor_ui
hex_grid (contract)    ──→ cell
hex_grid (contract)    ──→ unit
hex_grid (contract)    ──→ editor_ui
editor_ui (contract)   ──→ cell
editor_ui (contract)   ──→ unit

camera: independent
hex_grid: independent (M1, unchanged)
game_system: independent (provides registries)
cell: depends on hex_grid + game_system + editor_ui contracts
unit: depends on hex_grid + game_system + editor_ui contracts
editor_ui: depends on hex_grid + game_system contracts
```

## Merge Lock

> Only one merge to `main` at a time. See `docs/git-guide.md` → Merge Lock Protocol for full rules.

| Branch | Version | Claimed By | Status |
|--------|---------|------------|--------|
| — | — | — | — |

Status values: `merging` | `done`

Rules:
- Before merging, check this table. If any row is `merging`, wait.
- Claim your row before starting the Pre-Merge Checklist.
- Release (mark `done`) after the tag is created and verified.
- Do not clear another session's `merging` row without investigation.

## Integration Test Checkpoints

| Date | Features Tested | Result | Notes |
|------|----------------|--------|-------|
| 2026-02-08 | all M1 | FAIL | Constitution audit found 5 cross-feature internal imports. Promoted to contracts. |
| 2026-02-08 | all M1 | PASS | Re-audit: 0 violations, 44 tests pass, clippy clean. Module privacy enforced. |
| 2026-02-09 | all M2 | PASS | Full 9-point audit: 48 tests pass, clippy clean, no unwrap/unsafe in prod, all pub types Debug, no boundary violations, contracts spec-code parity fixed (terrain.md marked retired, editor_ui refs updated terrain→cell). |
| 2026-02-09 | all M2 (final) | PASS | M2 Checkpoint audit: 53 tests pass (added 4 integration tests + 1 architecture test), clippy clean, all 9 constitution checks pass. M2 complete. |
| 2026-02-09 | all M3 | PASS | 71 tests pass (9 unit tests, 5 game_system unit tests, 5 editor_ui tests, 4 integration tests added for M3), clippy clean, no unwrap/unsafe in prod, boundary tests pass. |
| 2026-02-10 | all M3 (final) | PASS | M3 Checkpoint audit: 71 tests, clippy clean, all 9 constitution checks pass. M3 complete. |
| 2026-02-10 | all M3 (polish) | PASS | Post-M3 polish audit: 71 tests, clippy clean, all 9 constitution checks pass. Ring border overlays for hover/selection, click/Escape deselect, camera pan rework, view shortcuts, resize compensation, TileBaseMaterial + PaintPreview contracts added. Specs, logs, and contract docs updated. |

## Known Blockers
- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/bevy-guide.md` and `docs/bevy-egui-guide.md`.
- hexx 0.22 API is documented in `docs/bevy-guide.md`.
