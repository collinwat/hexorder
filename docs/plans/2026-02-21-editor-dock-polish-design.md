# Editor Dock Polish — Design

Pitch #147, Cycle 7 (0.12.0). Small Batch.

## Scope 1: EditorDockViewer Refactor

### Current state

`EditorDockViewer<'a>` has 21 flat fields feeding 8 tab render paths via
`egui_dock::TabViewer::ui()`. The `TabViewer` trait takes `&mut self`, so all data must be
pre-extracted into the viewer struct before `DockArea::show()` is called.

### Chosen approach: Tab-specific sub-structs

Group fields by tab ownership. Cross-cutting fields (`editor_state`, `actions`) stay at the top
level. Tab-specific data moves into sub-structs.

```
EditorDockViewer<'a>
├── editor_state: &mut EditorState       (Palette, Design, Rules, Settings)
├── actions: &mut Vec<EditorAction>      (Design, Rules)
├── next_state: &mut NextState           (Palette)
├── viewport_rect: &mut ViewportRect     (Viewport)
├── multi: &Selection                    (Selection)
├── schema_validation: &SchemaValidation (Rules, Validation)
├── palette: PaletteData<'a>
│   ├── editor_tool: &mut EditorTool
│   ├── active_board: &mut ActiveBoardType
│   ├── active_token: &mut ActiveTokenType
│   ├── project_workspace: &Workspace
│   └── project_game_system: &GameSystem
├── design: DesignData<'a>
│   ├── registry: &mut EntityTypeRegistry
│   ├── enum_registry: &mut EnumRegistry
│   ├── struct_registry: &mut StructRegistry
│   ├── concept_registry: &mut ConceptRegistry
│   └── relation_registry: &mut RelationRegistry
└── rules: RulesData<'a>
    ├── constraint_registry: &mut ConstraintRegistry
    ├── turn_structure: &mut TurnStructure
    ├── combat_results_table: &mut CombatResultsTable
    └── combat_modifiers: &mut CombatModifierRegistry
```

### Shared field handling

- `registry` is used by both Palette and Design. Placed in `DesignData` (primary user with 6
  sub-tabs). Palette tab accesses it via `self.design.registry`.
- `concept_registry` is used by both Design and Rules. Placed in `DesignData` (more sub-tabs use
  it). Rules tab accesses it via `self.design.concept_registry`.

### Constraints

- `TabViewer::ui()` takes `&mut self` — sub-structs are `&mut self.palette`, `&mut self.rules`, etc.
  No simultaneous mutable borrows of sibling sub-structs in the same expression.
- Each `match` arm accesses only the fields it needs. Rust's borrow checker allows accessing
  different fields of a struct simultaneously, and accessing sub-structs independently within match
  arms is fine.

### Test impact

Zero — pure structural refactor. All 40 editor_ui tests must pass unchanged.

## Scope 2: Inspector Tab Migration (#144)

Move dead `render_inspector` and `render_unit_inspector` functions into `DockTab::Inspector`. Add
query results to viewer (selected tile data, selected unit data). No new sub-struct needed — just
add inspector-specific fields at the top level or create an `InspectorData` sub-struct if field
count warrants it.

## Scope 3: Dynamic Undo/Redo Labels (#130)

Wire `UndoStack::undo_description()` / `redo_description()` into the Edit menu rendering (inside
`editor_dock_system`'s menu bar section). Add `undo_stack: &UndoStack` to the system params. No
viewer changes — the menu bar renders before the DockArea.

## Scope 4: Font Size Persistence (#128)

Add `font_size_base: f32` field to `GameSystemFile` (or `Workspace`) with `#[serde(default)]`. Sync
on save, restore on load. Bump FORMAT_VERSION 4 to 5 if using `GameSystemFile`.

## Scope 5: CentralPanel Investigation (#145)

Time-boxed to 2 hours. Investigate, document findings regardless of outcome.
