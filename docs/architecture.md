# Hexorder Architecture

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
