# Feature: editor_ui

## Summary
Provides the egui-based editor interface. In M2, this evolves from the minimal M1 panel to include: a dark editor theme, a cell type palette (replacing terrain palette), a cell type editor for creating/editing types and properties, and an inspector panel for viewing/editing property values on selected tiles.

## Plugin
- Module: `src/editor_ui/`
- Plugin struct: `EditorUiPlugin`
- Schedule: `EguiPrimaryContextPass` (UI rendering via bevy_egui)

## Dependencies
- **Contracts consumed**: `hex_grid` (HexPosition, SelectedHex, HexTile), `game_system` (GameSystem, CellType, CellTypeRegistry, CellData, ActiveCellType, PropertyDefinition, PropertyType, PropertyValue, EnumDefinition), `editor_ui` (EditorTool)
- **Contracts produced**: `editor_ui` (EditorTool)
- **Crate dependencies**: `bevy_egui` (see `docs/bevy-egui-guide.md`)

## M1 Requirements (retained)
1. [REQ-MODE] Maintain an `EditorTool` resource with at least two modes: `Select` and `Paint`. Default to `Select` on startup.
2. [REQ-TOOLBAR] Render an egui left side panel with a tool mode selector.
3. [REQ-NO-PASSTHROUGH] When the mouse is over an egui panel, input does not pass through to the hex grid.

## M2 Requirements (new)
4. [REQ-DARK-THEME] Apply a dark color scheme to all egui panels. Use system/monospace fonts for editor controls. Editor panels should be visually distinct from the 3D game view — clear borders/contrast.
5. [REQ-PALETTE-V2] Replace the terrain palette with a cell type palette. Display all cell types from the CellTypeRegistry as clickable color swatches with names. Highlight the active cell type. Visible in Paint mode.
6. [REQ-CELL-EDITOR] Provide UI for creating, editing, and deleting cell types. Editable fields: name, color. Show the list of property definitions on the type. Allow adding/removing property definitions.
7. [REQ-PROPERTY-EDITOR] When editing a cell type, allow adding property definitions with: name, type (dropdown: Bool, Int, Float, String, Color, Enum), and default value. The default value editor should match the property type (checkbox for Bool, number input for Int/Float, text input for String, color picker for Color, dropdown for Enum).
8. [REQ-INSPECTOR] When a hex tile is selected, show an inspector panel (right side or bottom of left panel) displaying: the tile's cell type name, its coordinates, and all property values. Property values are editable using type-appropriate widgets.
9. [REQ-GAME-SYSTEM-INFO] Display the Game System id (abbreviated) and version somewhere visible in the editor (e.g., title bar area of the left panel or a status bar).

## Success Criteria
### M1 (retained)
- [x] [SC-1] Tool mode switches between Select and Paint
- [x] [SC-5] Clicking on UI panels does not trigger hex tile selection

### M2 (new)
- [x] [SC-6] Editor uses a dark theme with clear contrast against the 3D viewport
- [x] [SC-7] Cell type palette shows all registered types with colors and names
- [x] [SC-8] User can create a new cell type with a name and color
- [x] [SC-9] User can add a property definition to a cell type
- [x] [SC-10] All 6 property types have appropriate editor widgets (Bool=checkbox, Int/Float=number, String=text, Color=picker, Enum=dropdown)
- [x] [SC-11] Inspector panel shows property values for the selected tile
- [x] [SC-12] Property values can be edited per-tile via the inspector
- [x] [SC-13] Game System id and version are displayed in the editor
- [x] [SC-BUILD] `cargo build` succeeds
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other features' internals

## Decomposition
This feature has enough scope for parallel work if needed:

| Subtask | Description | Owner | Status |
|---------|-------------|-------|--------|
| Dark theme | egui style configuration, font setup | | |
| Cell palette | Replace terrain palette with cell type palette | | |
| Cell type editor | Create/edit/delete cell types and their properties | | |
| Inspector panel | View/edit per-tile property values | | |

## Constraints
- The dark theme should use egui's built-in `Visuals::dark()` as a starting point, then customize
- System fonts should be used for editor controls (egui supports loading system fonts)
- The inspector panel needs to handle the case where no tile is selected, where a tile has no properties, and where a tile's cell type has been deleted
- Property editors must validate input (e.g., Int fields reject non-numeric input)

## Open Questions
- Should the cell type editor be a separate panel or a collapsible section in the left panel? (Suggest: collapsible section to start)
- Should there be a confirmation dialog when deleting a cell type that's in use? (Suggest: yes, or at minimum show a count of affected tiles)
- Should Enum property editing include creating/managing EnumDefinitions inline, or should that be a separate management UI? (Suggest: inline for M2 simplicity)
