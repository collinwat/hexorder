# Hexorder Coordination

## Active Cycle

**Cycle 1 — "The Process Matures"** | Type: Process (no code) | Appetite: Small Batch

### Current Bets

| Pitch                                   | Appetite    | Status      |
| --------------------------------------- | ----------- | ----------- |
| Shape Up workflow documentation rewrite | Small Batch | in-progress |

_Bets are set at the betting table during cool-down. See `.specs/roadmap.md` → Cool-Down Protocol._

## Active Features

Features are scopes within a build cycle. Status and ownership are tracked in GitHub Issues and the
GitHub Project:

```bash
gh issue list --state open                    # all open work items
gh issue list --milestone "<milestone>"       # items for a specific release
gh project view 1 --owner collinwat           # project board
```

### Historical Feature Summary (through 0.6.0)

| Feature      | Last Updated | Notes                                                           |
| ------------ | ------------ | --------------------------------------------------------------- |
| hex_grid     | 0.4.0        | Move overlay rendering, 19 tests                                |
| camera       | 0.6.0        | Orthographic top-down, pan + zoom, shortcut guard               |
| game_system  | 0.4.0        | EntityType unification, EntityTypeRegistry                      |
| cell         | 0.4.0        | EntityTypeRegistry/EntityData migration, 10 tests               |
| unit         | 0.4.0        | EntityTypeRegistry/EntityData migration, ValidMoveSet, 10 tests |
| editor_ui    | 0.6.0        | Launcher screen, file menu, 26 UI tests                         |
| ontology     | 0.4.0        | Concepts, relations, constraints, schema validation, 7 tests    |
| rules_engine | 0.4.0        | Constraint evaluation, ValidMoveSet BFS, 8 tests                |
| scripting    | 0.5.0        | Embedded Lua (LuaJIT), read-only registry access, 11 tests      |
| persistence  | 0.6.0        | Save/load .hexorder RON files, keyboard shortcuts, 10 tests     |

## Plugin Load Order

Declared in `main.rs`. Update this when adding a new plugin.

1. DefaultPlugins (built-in)
2. HexGridPlugin
3. CameraPlugin
4. GameSystemPlugin (must be before CellPlugin, UnitPlugin, OntologyPlugin, and EditorUiPlugin)
5. OntologyPlugin (must be after GameSystemPlugin, before RulesEnginePlugin)
6. CellPlugin
7. UnitPlugin
8. RulesEnginePlugin (must be after OntologyPlugin, UnitPlugin)
9. ScriptingPlugin (NEW 0.5.0 — after RulesEnginePlugin, before EditorUiPlugin)
10. PersistencePlugin (NEW 0.6.0 — after GameSystemPlugin, before EditorUiPlugin)
11. EditorUiPlugin (must be last — reads all resources, renders launcher + editor)

## Pending Contract Changes

Contract change proposals are tracked as GitHub Issues with `area:contracts` label:
`gh issue list --label "area:contracts" --state open`

Before changing a contract, create an issue describing the change, list affected features, and wait
for approval before implementing. See the Shared Contracts Protocol in CLAUDE.md.

### Historical Contract Changes (through 0.6.0)

| Contract    | Release | Change                                                                |
| ----------- | ------- | --------------------------------------------------------------------- |
| game_system | 0.4.0   | EntityType unification, EntityTypeRegistry replaces CellType/UnitType |
| ontology    | 0.4.0   | Concepts, relations, constraints, registries                          |
| validation  | 0.4.0   | ValidMoveSet, SchemaValidation                                        |
| hex_grid    | 0.4.0   | MoveOverlay, MoveOverlayState                                         |
| persistence | 0.6.0   | AppScreen, GameSystemFile, save/load types, events                    |
| game_system | 0.6.0   | Serialize/Deserialize + Clone on registries                           |
| ontology    | 0.6.0   | Serialize/Deserialize + Clone on registries                           |
| hex_grid    | 0.6.0   | Serialize/Deserialize on HexPosition                                  |

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
- **Terminology (0.4.0)**: Entity types have a role (BoardPosition or Token). Hex tiles on the board
  have EntityData with a BoardPosition-role type. Game pieces on tiles have EntityData with a
  Token-role type plus UnitInstance marker. "Cell" and "unit" are informal shorthand for
  BoardPosition and Token entities respectively. CellType/UnitType terminology is retired in 0.4.0
  (unified as EntityType).
- **Entity placement**: Token entities are separate from hex tile entities. They share HexPosition
  for grid location. Multiple tokens can occupy the same tile.
- **Enum definitions**: Consolidated into single EntityTypeRegistry (0.4.0 resolves the 0.3.0
  duplication concern).
- **Serialization (0.6.0)**: All persistent types (registries, HexPosition, PropertyValue) derive
  Serialize/Deserialize. Save format is RON via `ron 0.12`. File extension: `.hexorder`.
- **Editor tool mode**: `EditorTool` resource (owned by editor_ui) must be checked by cell and unit
  before painting/placing.
- **Module privacy enforcement**: Feature sub-modules are `mod` (private). Contract boundary
  violations are compile errors + enforced by `architecture_tests::feature_modules_are_private`.
- **Ontology**: Concepts, relations, and constraints are designer-defined abstractions. No hardcoded
  game terms — the tool understands only structural relationships, not domain semantics.
- **Constraint evaluation**: The rules_engine evaluates constraints and produces ValidMoveSet. The
  unit plugin checks ValidMoveSet before allowing moves. If no constraints exist, all moves are
  valid (backward compatible with 0.3.0).
- **Move overlays**: Separate lightweight entities above hex tiles, managed by hex_grid. Do not
  modify tile materials or interfere with cell visual sync.

## Feature Dependency Graph (0.6.0)

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
hex_grid: depends on validation contract (0.4.0: move overlays)
game_system: independent (provides EntityTypeRegistry)
ontology: depends on game_system contract
cell: depends on hex_grid + game_system + editor_ui contracts
unit: depends on hex_grid + game_system + editor_ui + validation contracts
rules_engine: depends on game_system + ontology + hex_grid contracts
persistence: depends on game_system + ontology + hex_grid + validation + persistence contracts
editor_ui: depends on hex_grid + game_system + ontology + validation + persistence contracts
```

## Implementation Phases (0.4.0)

0.4.0 proceeds in three phases:

1. **Phase 1 — Unify EntityType**: Migrate game_system, cell, unit, editor_ui to unified types. No
   new behavior, all 0.3.0 tests updated and passing.
2. **Phase 2 — Ontology Framework**: New ontology contract + OntologyPlugin. ConceptRegistry,
   RelationRegistry, ConstraintRegistry. Schema validation. Editor UI panels.
3. **Phase 3 — Rules Engine + Visual Rendering**: New validation contract + RulesEnginePlugin.
   ValidMoveSet computation. Unit movement checks. Move overlay rendering.

## Merge Lock

> Only one merge to `main` at a time. See `docs/git-guide.md` → Merge Lock Protocol for full rules.

| Branch                   | Version | Claimed By | Status  |
| ------------------------ | ------- | ---------- | ------- |
| 0.4.0/entity-unification | 0.4.0   | agent      | merging |

Status values: `merging` | `done`

Rules:

- Before merging, check this table. If any row is `merging`, wait.
- Claim your row before starting the Pre-Merge Checklist.
- Release (mark `done`) after the tag is created and verified.
- Do not clear another session's `merging` row without investigation.

## Integration Test Checkpoints

| Date       | Features Tested    | Result | Notes                                                                                                                                                                                                                                                                                                                                                      |
| ---------- | ------------------ | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | all 0.1.0          | FAIL   | Constitution audit found 5 cross-feature internal imports. Promoted to contracts.                                                                                                                                                                                                                                                                          |
| 2026-02-08 | all 0.1.0          | PASS   | Re-audit: 0 violations, 44 tests pass, clippy clean. Module privacy enforced.                                                                                                                                                                                                                                                                              |
| 2026-02-09 | all 0.2.0          | PASS   | Full 9-point audit: 48 tests pass, clippy clean, no unwrap/unsafe in prod, all pub types Debug, no boundary violations, contracts spec-code parity fixed (terrain.md marked retired, editor_ui refs updated terrain→cell).                                                                                                                                 |
| 2026-02-09 | all 0.2.0 (final)  | PASS   | 0.2.0 Checkpoint audit: 53 tests pass (added 4 integration tests + 1 architecture test), clippy clean, all 9 constitution checks pass. 0.2.0 complete.                                                                                                                                                                                                     |
| 2026-02-09 | all 0.3.0          | PASS   | 71 tests pass (9 unit tests, 5 game_system unit tests, 5 editor_ui tests, 4 integration tests added for 0.3.0), clippy clean, no unwrap/unsafe in prod, boundary tests pass.                                                                                                                                                                               |
| 2026-02-10 | all 0.3.0 (final)  | PASS   | 0.3.0 Checkpoint audit: 71 tests, clippy clean, all 9 constitution checks pass. 0.3.0 complete.                                                                                                                                                                                                                                                            |
| 2026-02-10 | all 0.3.0 (polish) | PASS   | Post-0.3.0 polish audit: 71 tests, clippy clean, all 9 constitution checks pass. Ring border overlays for hover/selection, click/Escape deselect, camera pan rework, view shortcuts, resize compensation, TileBaseMaterial + PaintPreview contracts added. Specs, logs, and contract docs updated.                                                         |
| 2026-02-11 | all 0.4.0          | PASS   | 0.4.0 constitution audit: 90 tests pass, clippy clean, no unwrap/unsafe in prod, all pub types Debug, no boundary violations, contracts spec-code parity verified, brand palette test passes. Phase 1 (EntityType unification), Phase 2 (ontology framework, 7 tests), Phase 3 (rules engine 8 tests, move overlays 4 tests, unit ValidMoveSet).           |
| 2026-02-13 | all 0.5.0          | PASS   | 0.5.0 constitution audit: 129 tests pass (92 from 0.4.0 + 26 egui_kittest UI tests + 11 Lua scripting tests), clippy clean, all automated checks pass. Phase 1 (Reflect derives on ~43 types), Phase 2 (editor_ui render function extraction), Phase 3 (egui_kittest UI tests), Phase 4 (mlua scripting plugin). Version 0.5.0.                            |
| 2026-02-13 | all 0.6.0          | PASS   | 0.6.0 constitution audit: 139 tests pass (129 from 0.5.0 + 3 serde round-trip + 4 file I/O + 3 persistence plugin), clippy clean, all automated checks pass. Phase 1 (serde), Phase 2 (RON file I/O), Phase 3 (AppScreen state machine), Phase 4 (save/load systems, rfd dialogs, keyboard shortcuts), Phase 5 (launcher UI, specs, audit). Version 0.6.0. |

## Known Blockers

- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/bevy-guide.md` and
  `docs/bevy-egui-guide.md`.
- hexx 0.22 API is documented in `docs/bevy-guide.md`.
