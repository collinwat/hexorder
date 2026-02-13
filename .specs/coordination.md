# Hexorder Coordination

## Active Milestone: M4 — "Rules Shape the World"

## Active Features

| Feature      | Owner | Status        | Dependencies                                                                     | Notes                                                                                                              |
| ------------ | ----- | ------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| hex_grid     | —     | complete (M4) | validation contract                                                              | M4: move overlay rendering from ValidMoveSet. 4 overlay tests added. 19 tests total.                               |
| camera       | —     | complete (M1) | none                                                                             | Unchanged for M4. Orthographic top-down, pan + zoom.                                                               |
| game_system  | —     | complete (M4) | none                                                                             | M4: EntityType unification complete. EntityTypeRegistry replaces CellType/UnitType registries.                     |
| cell         | —     | complete (M4) | hex_grid contract, game_system contract, editor_ui contract                      | M4: migrated to EntityTypeRegistry/EntityData. 10 tests pass.                                                      |
| unit         | —     | complete (M4) | hex_grid contract, game_system contract, editor_ui contract, validation contract | M4: migrated to EntityTypeRegistry/EntityData. Movement consults ValidMoveSet. 10 tests pass.                      |
| editor_ui    | —     | complete (M4) | hex_grid contract, game_system contract, ontology contract, validation contract  | M4: unified entity editor migrated. Ontology UI panels (concepts, relations, constraints, validation) implemented. |
| ontology     | —     | complete (M4) | game_system contract                                                             | NEW M4: concepts, relations, constraints, auto-generation, schema validation. 7 tests.                             |
| rules_engine | —     | complete (M4) | game_system contract, ontology contract, hex_grid contract, validation contract  | NEW M4: constraint evaluation, ValidMoveSet BFS computation. 8 tests.                                              |

Status values: `speccing` | `in-progress` | `testing` | `blocked` | `complete` | `retiring`

## Plugin Load Order

Declared in `main.rs`. Update this when adding a new plugin.

1. DefaultPlugins (built-in)
2. HexGridPlugin
3. CameraPlugin
4. GameSystemPlugin (must be before CellPlugin, UnitPlugin, OntologyPlugin, and EditorUiPlugin)
5. OntologyPlugin (NEW — must be after GameSystemPlugin, before RulesEnginePlugin)
6. CellPlugin
7. UnitPlugin
8. RulesEnginePlugin (NEW — must be after OntologyPlugin, UnitPlugin)
9. EditorUiPlugin (must be last — reads all resources)

## Pending Contract Changes

| Contract    | Proposed By | Change Description                                                                                                                        | Affected Features                             | Status |
| ----------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------ |
| game_system | M4          | EVOLVE — EntityType, EntityRole, EntityTypeRegistry, EntityData replace CellType/UnitType systems. ActiveBoardType/ActiveTokenType added. | cell, unit, editor_ui, ontology, rules_engine | done   |
| ontology    | M4          | NEW — Concept, ConceptRole, ConceptBinding, Relation, Constraint, ConstraintExpr, registries                                              | rules_engine, editor_ui                       | done   |
| validation  | M4          | NEW — ValidMoveSet, SchemaValidation, SchemaError, ValidationResult                                                                       | hex_grid, unit, editor_ui                     | done   |
| hex_grid    | M4          | EXTEND — MoveOverlay, MoveOverlayState                                                                                                    | hex_grid                                      | done   |
| editor_ui   | M4          | UNCHANGED — EditorTool, PaintPreview stay as-is                                                                                           | cell, unit                                    | done   |

Status: `proposed` | `approved` | `implementing` | `done`

## Cross-Cutting Concerns

- **3D rendering**: Application uses Camera3d with orthographic projection, locked top-down
- **Hex coordinate system**: All features using hex positions must use `HexPosition` from
  `contracts::hex_grid`
- **Input separation**: Left-click for selection/painting, middle-click for camera pan, scroll for
  zoom. bevy_egui consumes input when mouse is over UI panels (via `egui_wants_any_pointer_input`
  run condition).
- **Game System**: The root design artifact. Holds all definitions (entity types, concepts,
  relations, constraints). All design data lives inside the Game System.
- **Property system**: Entity-agnostic. PropertyDefinition and PropertyValue are reused across all
  entity types regardless of role.
- **Terminology (M4)**: Entity types have a role (BoardPosition or Token). Hex tiles on the board
  have EntityData with a BoardPosition-role type. Game pieces on tiles have EntityData with a
  Token-role type plus UnitInstance marker. "Cell" and "unit" are informal shorthand for
  BoardPosition and Token entities respectively. CellType/UnitType terminology is retired in M4
  (unified as EntityType).
- **Entity placement**: Token entities are separate from hex tile entities. They share HexPosition
  for grid location. Multiple tokens can occupy the same tile.
- **Enum definitions**: Consolidated into single EntityTypeRegistry (M4 resolves the M3 duplication
  concern).
- **Serialization**: Not needed for M4; all state is ephemeral.
- **Editor tool mode**: `EditorTool` resource (owned by editor_ui) must be checked by cell and unit
  before painting/placing.
- **Module privacy enforcement**: Feature sub-modules are `mod` (private). Contract boundary
  violations are compile errors + enforced by `architecture_tests::feature_modules_are_private`.
- **Ontology**: Concepts, relations, and constraints are designer-defined abstractions. No hardcoded
  game terms — the tool understands only structural relationships, not domain semantics.
- **Constraint evaluation**: The rules_engine evaluates constraints and produces ValidMoveSet. The
  unit plugin checks ValidMoveSet before allowing moves. If no constraints exist, all moves are
  valid (backward compatible with M3).
- **Move overlays**: Separate lightweight entities above hex tiles, managed by hex_grid. Do not
  modify tile materials or interfere with cell visual sync.

## Feature Dependency Graph (M4)

```
game_system (contract) ──→ cell
game_system (contract) ──→ unit
game_system (contract) ──→ ontology
game_system (contract) ──→ rules_engine
game_system (contract) ──→ editor_ui
hex_grid (contract)    ──→ cell
hex_grid (contract)    ──→ unit
hex_grid (contract)    ──→ rules_engine
hex_grid (contract)    ──→ editor_ui
editor_ui (contract)   ──→ cell
editor_ui (contract)   ──→ unit
ontology (contract)    ──→ rules_engine
ontology (contract)    ──→ editor_ui
validation (contract)  ──→ hex_grid
validation (contract)  ──→ unit
validation (contract)  ──→ editor_ui

camera: independent
hex_grid: depends on validation contract (M4: move overlays)
game_system: independent (provides EntityTypeRegistry)
ontology: depends on game_system contract
cell: depends on hex_grid + game_system + editor_ui contracts
unit: depends on hex_grid + game_system + editor_ui + validation contracts
rules_engine: depends on game_system + ontology + hex_grid contracts
editor_ui: depends on hex_grid + game_system + ontology + validation contracts
```

## Implementation Phases (M4)

M4 proceeds in three phases:

1. **Phase 1 — Unify EntityType**: Migrate game_system, cell, unit, editor_ui to unified types. No
   new behavior, all M3 tests updated and passing.
2. **Phase 2 — Ontology Framework**: New ontology contract + OntologyPlugin. ConceptRegistry,
   RelationRegistry, ConstraintRegistry. Schema validation. Editor UI panels.
3. **Phase 3 — Rules Engine + Visual Rendering**: New validation contract + RulesEnginePlugin.
   ValidMoveSet computation. Unit movement checks. Move overlay rendering.

## Merge Lock

> Only one merge to `main` at a time. See `docs/git-guide.md` → Merge Lock Protocol for full rules.

| Branch                | Version | Claimed By | Status  |
| --------------------- | ------- | ---------- | ------- |
| m4/entity-unification | 0.4.0   | agent      | merging |

Status values: `merging` | `done`

Rules:

- Before merging, check this table. If any row is `merging`, wait.
- Claim your row before starting the Pre-Merge Checklist.
- Release (mark `done`) after the tag is created and verified.
- Do not clear another session's `merging` row without investigation.

## Integration Test Checkpoints

| Date       | Features Tested | Result | Notes                                                                                                                                                                                                                                                                                                                                         |
| ---------- | --------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | all M1          | FAIL   | Constitution audit found 5 cross-feature internal imports. Promoted to contracts.                                                                                                                                                                                                                                                             |
| 2026-02-08 | all M1          | PASS   | Re-audit: 0 violations, 44 tests pass, clippy clean. Module privacy enforced.                                                                                                                                                                                                                                                                 |
| 2026-02-09 | all M2          | PASS   | Full 9-point audit: 48 tests pass, clippy clean, no unwrap/unsafe in prod, all pub types Debug, no boundary violations, contracts spec-code parity fixed (terrain.md marked retired, editor_ui refs updated terrain→cell).                                                                                                                    |
| 2026-02-09 | all M2 (final)  | PASS   | M2 Checkpoint audit: 53 tests pass (added 4 integration tests + 1 architecture test), clippy clean, all 9 constitution checks pass. M2 complete.                                                                                                                                                                                              |
| 2026-02-09 | all M3          | PASS   | 71 tests pass (9 unit tests, 5 game_system unit tests, 5 editor_ui tests, 4 integration tests added for M3), clippy clean, no unwrap/unsafe in prod, boundary tests pass.                                                                                                                                                                     |
| 2026-02-10 | all M3 (final)  | PASS   | M3 Checkpoint audit: 71 tests, clippy clean, all 9 constitution checks pass. M3 complete.                                                                                                                                                                                                                                                     |
| 2026-02-10 | all M3 (polish) | PASS   | Post-M3 polish audit: 71 tests, clippy clean, all 9 constitution checks pass. Ring border overlays for hover/selection, click/Escape deselect, camera pan rework, view shortcuts, resize compensation, TileBaseMaterial + PaintPreview contracts added. Specs, logs, and contract docs updated.                                               |
| 2026-02-11 | all M4          | PASS   | M4 constitution audit: 90 tests pass, clippy clean, no unwrap/unsafe in prod, all pub types Debug, no boundary violations, contracts spec-code parity verified, brand palette test passes. Phase 1 (EntityType unification), Phase 2 (ontology framework, 7 tests), Phase 3 (rules engine 8 tests, move overlays 4 tests, unit ValidMoveSet). |

## Known Blockers

- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/bevy-guide.md` and
  `docs/bevy-egui-guide.md`.
- hexx 0.22 API is documented in `docs/bevy-guide.md`.
