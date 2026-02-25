# Hexorder Architecture

## Workspace Structure

Hexorder is a Cargo workspace with two members:

- **`crates/hexorder-contracts/`** — library crate containing shared contract types (components,
  resources, events). All plugins depend on this crate. Mirrors `docs/contracts/`.
- **`hexorder/`** (root) — binary crate containing the application, all plugins, and `main.rs`.
  Depends on `hexorder-contracts`.

This split enables parallel compilation and Cargo-enforced contract boundaries.

## Plugin Load Order

Declared in `main.rs`. Update this when adding a new plugin.

1. DefaultPlugins (built-in)
2. HexGridPlugin
3. ShortcutsPlugin (NEW 0.9.0 — before all plugins that register shortcuts)
4. CameraPlugin
5. GameSystemPlugin (must be before CellPlugin, UnitPlugin, OntologyPlugin, and EditorUiPlugin)
6. OntologyPlugin (must be after GameSystemPlugin, before RulesEnginePlugin)
7. CellPlugin
8. UnitPlugin
9. RulesEnginePlugin (must be after OntologyPlugin, UnitPlugin)
10. ScriptingPlugin (NEW 0.5.0 — after RulesEnginePlugin, before EditorUiPlugin)
11. PersistencePlugin (NEW 0.6.0 — after GameSystemPlugin, before EditorUiPlugin)
12. UndoRedoPlugin (NEW 0.10.0 — after ShortcutsPlugin, before EditorUiPlugin)
13. ExportPlugin (NEW 0.12.0 — after PersistencePlugin, before EditorUiPlugin)
14. SettingsPlugin (NEW 0.13.0 — after ExportPlugin, before EditorUiPlugin)
15. EditorUiPlugin (must be last — reads all resources, renders launcher + editor)

## Cross-Cutting Concerns

- **3D rendering**: Application uses Camera3d with orthographic projection, locked top-down
- **Hex coordinate system**: All plugins using hex positions must use `HexPosition` from
  `hexorder_contracts::hex_grid`
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
- **Keyboard shortcuts (0.9.0)**: All keyboard shortcuts are registered in the centralized
  `ShortcutRegistry` (owned by shortcuts plugin). Discrete commands fire `CommandExecutedEvent` via
  observers. Continuous commands (WASD pan) register for discoverability but read bindings directly.
  Cmd+K opens the command palette for fuzzy search. TOML config overrides supported.
- **Module privacy enforcement**: Plugin sub-modules are `mod` (private). Contract boundary
  violations are compile errors + enforced by `architecture_tests::plugin_modules_are_private`.
- **Ontology**: Concepts, relations, and constraints are designer-defined abstractions. No hardcoded
  game terms — the tool understands only structural relationships, not domain semantics.
- **Constraint evaluation**: The rules_engine evaluates constraints and produces ValidMoveSet. The
  unit plugin checks ValidMoveSet before allowing moves. If no constraints exist, all moves are
  valid (backward compatible with 0.3.0).
- **Move overlays**: Separate lightweight entities above hex tiles, managed by hex_grid. Do not
  modify tile materials or interfere with cell visual sync.

## Plugin Dependency Graph (0.13.0)

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
shortcuts (contract)   ──→ camera
shortcuts (contract)   ──→ hex_grid
shortcuts (contract)   ──→ persistence
shortcuts (contract)   ──→ undo_redo
shortcuts (contract)   ──→ editor_ui
undo_redo (contract)   ──→ editor_ui
game_system (contract) ──→ undo_redo
game_system (contract) ──→ export
hex_grid (contract)    ──→ export
shortcuts (contract)   ──→ export
persistence (contract) ──→ settings
settings (contract)    ──→ editor_ui

shortcuts: independent (provides ShortcutRegistry, CommandExecutedEvent, CommandPaletteState)
undo_redo: depends on shortcuts + game_system contracts (provides UndoStack, UndoableCommand)
camera: depends on shortcuts contract (0.9.0: registry lookups for pan/view keys)
hex_grid: depends on validation + shortcuts contracts
game_system: independent (provides EntityTypeRegistry)
ontology: depends on game_system contract
cell: depends on hex_grid + game_system + editor_ui contracts
unit: depends on hex_grid + game_system + editor_ui + validation contracts
rules_engine: depends on game_system + ontology + hex_grid contracts
persistence: depends on game_system + ontology + hex_grid + validation + persistence + shortcuts contracts
export: depends on game_system + hex_grid + shortcuts contracts (NEW 0.12.0)
settings: depends on persistence contract (NEW 0.13.0)
editor_ui: depends on hex_grid + game_system + ontology + validation + persistence + shortcuts + settings contracts
```
