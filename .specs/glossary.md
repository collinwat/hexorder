# Hexorder Glossary

Canonical terminology for the Hexorder project. All agents, specs, and code must use these terms
consistently.

## Hex Grid Geometry

| Term                   | Definition                                                                                                                                                                      | Dimension  | Code Reference                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------- |
| **Cell**               | The hexagonal area on the board. The fundamental spatial unit. Each cell occupies one hex position and has a type with properties.                                              | 2D region  | `HexTile` (marker), `CellData` (component), `CellType` (definition)                |
| **Edge**               | The shared boundary between two adjacent cells. Six edges per cell.                                                                                                             | 1D segment | — (not yet modeled)                                                                |
| **Vertex** (geometric) | The point where three cells meet. Six vertices per cell. Used only in mesh construction.                                                                                        | 0D point   | `hex_grid/systems.rs` mesh builder                                                 |
| **Hex position**       | An axial coordinate pair (q, r) identifying a cell on the grid. Cube coordinate (q, r, s) is derived (s = -q - r).                                                              | —          | `HexPosition { q, r }`                                                             |
| **Neighbor**           | One of the six cells sharing an edge with a given cell.                                                                                                                         | —          | `hexx::Hex` adjacency methods                                                      |
| **Ring**               | The set of all cells at a fixed distance from a center cell.                                                                                                                    | —          | `hexx::shapes::hexagon`                                                            |
| **Radius**             | The distance in hex steps from the grid center to its outermost ring.                                                                                                           | —          | `HexGridConfig.map_radius`                                                         |
| **Unit**               | A game entity placed on the hex grid. Occupies a cell position. Defined by a unit type from the Game System. Multiple units may occupy the same cell (no stacking rules in M3). | ECS entity | `UnitInstance` (marker), `UnitData` (component), `HexPosition` (shared with cells) |

## Game System Domain

| Term                    | Definition                                                                                                                                                          | Code Reference                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| **Game System**         | The root design artifact. A named, versioned container for all user-defined rules, types, and definitions.                                                          | `GameSystem` resource                  |
| **Cell type**           | A definition within the Game System describing what a board position can be (e.g., Plains, Forest, Water). Has a name, color, and property schema.                  | `CellType`, `CellTypeRegistry`         |
| **Property definition** | A named, typed schema entry on a cell type (or future entity type). Defines the data shape, not the value.                                                          | `PropertyDefinition`                   |
| **Property value**      | A concrete instance of a property definition, stored per-cell.                                                                                                      | `PropertyValue`, `CellData.properties` |
| **Property type**       | The data type of a property: Bool, Int, Float, String, Color, or Enum.                                                                                              | `PropertyType` enum                    |
| **Enum definition**     | A named set of string options for Enum-type properties (e.g., "Movement Mode" → Foot, Wheeled, Tracked).                                                            | `EnumDefinition`                       |
| **TypeId**              | A UUID v4 identifier used for cell types, property definitions, and enum definitions. Stable across serialization.                                                  | `TypeId(Uuid)`                         |
| **Active cell type**    | The cell type currently selected for painting in the editor palette.                                                                                                | `ActiveCellType` resource              |
| **Unit type**           | A definition within the Game System describing what a game entity on the board can be (e.g., Infantry, Cavalry, Artillery). Has a name, color, and property schema. | `UnitType`, `UnitTypeRegistry`         |
| **Unit type ID**        | A UUID v4 identifier for a unit type. Same `TypeId` wrapper as cell types.                                                                                          | `UnitTypeId` (alias for `TypeId`)      |
| **Unit data**           | Per-instance data attached to a unit entity: which unit type it is and its property values.                                                                         | `UnitData` component                   |
| **Unit instance**       | Marker component identifying an entity as a unit on the hex grid.                                                                                                   | `UnitInstance` component               |
| **Active unit type**    | The unit type currently selected for placement in the editor palette.                                                                                               | `ActiveUnitType` resource              |
| **Selected unit**       | The unit entity currently selected by the user for inspection or movement.                                                                                          | `SelectedUnit` resource                |
| **Unit placed event**   | Observer event fired when a unit is placed on the grid. Contains the entity, position, and unit type.                                                               | `UnitPlacedEvent` event                |

## Editor & Interaction

| Term                 | Definition                                                                                                                             | Code Reference                               |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| **Editor tool**      | The current interaction mode: Select (inspect cells), Paint (assign cell types), or Place (place units).                               | `EditorTool` resource                        |
| **Palette**          | The UI panel listing available types. The cell palette lists cell types for painting; the unit palette lists unit types for placement. | `render_cell_palette`, `render_unit_palette` |
| **Inspector**        | The UI panel showing the selected entity's type and property values. Shows cell or unit details depending on selection.                | `render_inspector`                           |
| **Cell type editor** | The UI panel for creating, renaming, recoloring, and deleting cell type definitions.                                                   | `render_cell_type_editor`                    |
| **Unit type editor** | The UI panel for creating, renaming, recoloring, and deleting unit type definitions.                                                   | `render_unit_type_editor`                    |
| **Place tool**       | The editor tool mode for placing units on the hex grid. Click a tile to place a unit of the active unit type.                          | `EditorTool::Place`                          |

## Bevy Architecture

| Term         | Definition                                                                                                                                                              | Usage                                                                                             |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| **Plugin**   | A Bevy `Plugin` implementing `build()`. Each feature is exactly one plugin.                                                                                             | `HexGridPlugin`, `CameraPlugin`, `GameSystemPlugin`, `CellPlugin`, `UnitPlugin`, `EditorUiPlugin` |
| **Contract** | A Rust module in `src/contracts/` containing shared types. Mirrored by a spec in `.specs/contracts/`. Features depend on contracts, never on other features' internals. | `contracts::hex_grid`, `contracts::game_system`, `contracts::editor_ui`                           |
| **Feature**  | A self-contained plugin under `src/<name>/`. Sub-modules are private (`mod`, not `pub mod`).                                                                            | `hex_grid`, `camera`, `game_system`, `cell`, `unit`, `editor_ui`                                  |

## Retired Terms

These terms were used in earlier milestones and must **not** appear in new code or specs.

| Retired Term                                             | Replaced By                            | Reason                                                                                                                          | When |
| -------------------------------------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ---- |
| **Terrain** / **TerrainType**                            | Cell / CellType                        | "Terrain" was M1's name for board position types. Replaced by the Game System's cell type model in M2.                          | M2   |
| **Vertex** (as board position)                           | Cell                                   | "Vertex" in hex geometry means a corner point (0D), not the hexagonal area (2D). Confusing for the tabletop wargaming audience. | M2   |
| **VertexType** / **VertexData** / **VertexTypeRegistry** | CellType / CellData / CellTypeRegistry | Part of the Vertex → Cell rename.                                                                                               | M2   |
| **TerrainPalette** / **ActiveTerrain**                   | CellTypeRegistry / ActiveCellType      | Part of the Terrain → Cell rename.                                                                                              | M2   |

## Disambiguation

| Confusing pair                          | How to tell them apart                                                                                                                                                   |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Cell (grid) vs. Cell (biology)          | Context: this is a hex grid tool. "Cell" always means a hexagonal board position.                                                                                        |
| Vertex (geometric) vs. Vertex (retired) | Geometric vertex = mesh corner point in `hex_grid/systems.rs`. The retired "Vertex" meaning "board position" no longer exists in the codebase.                           |
| Tile vs. Cell                           | `HexTile` is the ECS marker component on the entity. "Cell" is the game design concept (type + properties). A hex tile entity _is_ a cell once it has `CellData`.        |
| Type (Rust) vs. Type (Game System)      | Rust types are code constructs. Game System "types" (cell types, property types) are user-defined design definitions. Use "cell type" or "property type" to be specific. |
| Property definition vs. Property value  | Definition = schema (name + type + default). Value = concrete data stored on a cell or unit instance.                                                                    |
| Cell vs. Unit                           | A cell is a hex position on the board (the tile itself). A unit is a game entity placed on a cell (a piece on the tile). Cells are painted; units are placed.            |
| Unit type vs. Cell type                 | Both are Game System definitions with properties. Cell types describe board positions. Unit types describe game entities that move on the board.                         |
