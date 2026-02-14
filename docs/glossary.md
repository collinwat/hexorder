# Hexorder Glossary

Canonical terminology for the Hexorder project. All agents, specs, and code must use these terms
consistently.

## Hex Grid Geometry

| Term                   | Definition                                                                                                                                                            | Dimension  | Code Reference                                                          |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------- |
| **Cell**               | The hexagonal area on the board. The fundamental spatial unit. Each cell occupies one hex position and has a type with properties.                                    | 2D region  | `HexTile` (marker), `EntityData` (component), `EntityType` (definition) |
| **Edge**               | The shared boundary between two adjacent cells. Six edges per cell.                                                                                                   | 1D segment | — (not yet modeled)                                                     |
| **Vertex** (geometric) | The point where three cells meet. Six vertices per cell. Used only in mesh construction.                                                                              | 0D point   | `hex_grid/systems.rs` mesh builder                                      |
| **Hex position**       | An axial coordinate pair (q, r) identifying a cell on the grid. Cube coordinate (q, r, s) is derived (s = -q - r).                                                    | —          | `HexPosition { q, r }`                                                  |
| **Neighbor**           | One of the six cells sharing an edge with a given cell.                                                                                                               | —          | `hexx::Hex` adjacency methods                                           |
| **Ring**               | The set of all cells at a fixed distance from a center cell.                                                                                                          | —          | `hexx::shapes::hexagon`                                                 |
| **Radius**             | The distance in hex steps from the grid center to its outermost ring.                                                                                                 | —          | `HexGridConfig.map_radius`                                              |
| **Unit**               | A game entity placed on the hex grid. Occupies a cell position. Defined by an entity type (Token role) from the Game System. Multiple units may occupy the same cell. | ECS entity | `UnitInstance` (marker), `EntityData` (component), `HexPosition`        |

## Game System Domain

| Term                    | Definition                                                                                                                                                  | Code Reference                           |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| **Game System**         | The root design artifact. A named, versioned container for all user-defined rules, types, and definitions.                                                  | `GameSystem` resource                    |
| **Entity type**         | A unified type definition within the Game System. Has a name, color, role, and property schema. Replaces the separate CellType and UnitType from pre-0.4.0. | `EntityType`, `EntityTypeRegistry`       |
| **Entity role**         | How an entity type participates in the world: BoardPosition (hex tile) or Token (game piece).                                                               | `EntityRole` enum                        |
| **Entity data**         | Per-instance data attached to a board tile or unit entity: which entity type it is and its property values.                                                 | `EntityData` component                   |
| **Property definition** | A named, typed schema entry on an entity type. Defines the data shape, not the value.                                                                       | `PropertyDefinition`                     |
| **Property value**      | A concrete instance of a property definition, stored per-entity.                                                                                            | `PropertyValue`, `EntityData.properties` |
| **Property type**       | The data type of a property: Bool, Int, Float, String, Color, or Enum.                                                                                      | `PropertyType` enum                      |
| **Enum definition**     | A named set of string options for Enum-type properties (e.g., "Movement Mode" → Foot, Wheeled, Tracked).                                                    | `EnumDefinition`                         |
| **TypeId**              | A UUID v4 identifier used for entity types, property definitions, and enum definitions. Stable across serialization.                                        | `TypeId(Uuid)`                           |
| **Active board type**   | The entity type (BoardPosition role) currently selected for painting in the editor palette.                                                                 | `ActiveBoardType` resource               |
| **Active token type**   | The entity type (Token role) currently selected for placement in the editor palette.                                                                        | `ActiveTokenType` resource               |
| **Unit instance**       | Marker component identifying an entity as a unit (Token) on the hex grid.                                                                                   | `UnitInstance` component                 |
| **Selected unit**       | The unit entity currently selected by the user for inspection or movement.                                                                                  | `SelectedUnit` resource                  |
| **Unit placed event**   | Observer event fired when a unit is placed on the grid. Contains the entity, position, and entity type.                                                     | `UnitPlacedEvent` event                  |

## Editor & Interaction

| Term                   | Definition                                                                                                                                         | Code Reference                               |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| **Editor tool**        | The current interaction mode: Select (inspect entities), Paint (assign board types), or Place (place tokens).                                      | `EditorTool` resource                        |
| **Palette**            | The UI panel listing available types. The board palette lists BoardPosition types for painting; the token palette lists Token types for placement. | `render_cell_palette`, `render_unit_palette` |
| **Inspector**          | The UI panel showing the selected entity's type and property values. Shows cell or unit details depending on selection.                            | `render_inspector`                           |
| **Entity type editor** | The UI panel for creating, renaming, recoloring, and deleting entity type definitions. Unified editor with role selector.                          | `render_entity_type_editor`                  |
| **Place tool**         | The editor tool mode for placing units on the hex grid. Click a tile to place a unit of the active token type.                                     | `EditorTool::Place`                          |

## Bevy Architecture

| Term         | Definition                                                                                                                                                                                     | Usage                                                                                             |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| **Plugin**   | A Bevy Plugin implementing `build()`. Each plugin is a self-contained module under `src/<name>/` with documentation at `docs/plugins/<name>/`. Sub-modules are private (`mod`, not `pub mod`). | `HexGridPlugin`, `CameraPlugin`, `GameSystemPlugin`, `CellPlugin`, `UnitPlugin`, `EditorUiPlugin` |
| **Contract** | A Rust module in `src/contracts/` containing shared types. Mirrored by a spec in `docs/contracts/`. Plugins depend on contracts, never on other plugins' internals.                            | `contracts::hex_grid`, `contracts::game_system`, `contracts::editor_ui`                           |
| **Feature**  | A feature request or raw idea from a user. Captured as a GitHub Issue. Not a commitment — feeds into the Shape Up shaping process. See the Process table.                                      | GitHub Issues with `type:feature` label                                                           |

## Process (Shape Up)

| Term                | Definition                                                                                                   |
| ------------------- | ------------------------------------------------------------------------------------------------------------ |
| **Raw idea**        | A GitHub Issue capturing an observation, bug, feature idea, or research question. Not a commitment.          |
| **Pitch**           | A shaped, risk-reduced proposal with Problem, Appetite, Solution, Rabbit Holes, No Gos. `type:pitch` label.  |
| **Appetite**        | Time budget declared upfront. The inverse of an estimate. Small Batch (1-2 weeks) or Big Batch (full cycle). |
| **Betting table**   | Decision point during cool-down where the developer reviews pitches and commits to the next cycle's scope.   |
| **Build cycle**     | Fixed-time period for building shaped work. Duration is flexible for solo dev.                               |
| **Cool-down**       | Period after shipping: recovery, retrospective, shaping, and betting for the next cycle.                     |
| **Circuit breaker** | Automatic cancellation of cycles that miss their deadline. Re-shape and re-pitch.                            |
| **Scope**           | A meaningful, independently completable slice of a project (few days). Discovered during building.           |
| **Ship gate**       | Quality bar (constitution audit) that must pass before a cycle's work ships.                                 |
| **Release**         | A versioned code deliverable (0.1.0, 0.2.0, etc.). The output of a build cycle.                              |

## Retired Terms

These terms were used in earlier releases and must **not** appear in new code or specs.

| Retired Term                                             | Replaced By                                  | Reason                                                                                                                                | When  |
| -------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| **Terrain** / **TerrainType**                            | Cell / EntityType (BoardPosition)            | "Terrain" was 0.1.0's name for board position types. Replaced by the Game System's cell type model in 0.2.0.                          | 0.2.0 |
| **Vertex** (as board position)                           | Cell                                         | "Vertex" in hex geometry means a corner point (0D), not the hexagonal area (2D). Confusing for the tabletop wargaming audience.       | 0.2.0 |
| **VertexType** / **VertexData** / **VertexTypeRegistry** | EntityType / EntityData / EntityTypeRegistry | Part of the Vertex → Cell rename, then unified in 0.4.0.                                                                              | 0.2.0 |
| **TerrainPalette** / **ActiveTerrain**                   | EntityTypeRegistry / ActiveBoardType         | Part of the Terrain → Cell rename, then unified in 0.4.0.                                                                             | 0.2.0 |
| **CellType** / **CellTypeRegistry** / **CellData**       | EntityType / EntityTypeRegistry / EntityData | Unified with UnitType into EntityType with role-based filtering.                                                                      | 0.4.0 |
| **ActiveCellType**                                       | ActiveBoardType                              | Renamed during EntityType unification.                                                                                                | 0.4.0 |
| **UnitType** / **UnitTypeRegistry** / **UnitData**       | EntityType / EntityTypeRegistry / EntityData | Unified with CellType into EntityType with role-based filtering.                                                                      | 0.4.0 |
| **ActiveUnitType**                                       | ActiveTokenType                              | Renamed during EntityType unification.                                                                                                | 0.4.0 |
| **Milestone** (M1, M2, M3, M4, M5)                       | Release (0.1.0, 0.2.0, etc.)                 | Replaced by semver release versions. Shape Up uses "cycle" for the time-boxed work period.                                            | 0.7.0 |
| **Feature** (as plugin)                                  | Plugin                                       | Feature previously meant a self-contained plugin module. Now Feature means a feature request (raw idea). Use Plugin for code modules. | 0.7.0 |

## Disambiguation

| Confusing pair                                      | How to tell them apart                                                                                                                                                             |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cell (grid) vs. Cell (biology)                      | Context: this is a hex grid tool. "Cell" always means a hexagonal board position.                                                                                                  |
| Vertex (geometric) vs. Vertex (retired)             | Geometric vertex = mesh corner point in `hex_grid/systems.rs`. The retired "Vertex" meaning "board position" no longer exists in the codebase.                                     |
| Tile vs. Cell                                       | `HexTile` is the ECS marker component on the entity. "Cell" is the game design concept (type + properties). A hex tile entity _is_ a cell once it has `EntityData`.                |
| Type (Rust) vs. Type (Game System)                  | Rust types are code constructs. Game System "types" (entity types, property types) are user-defined design definitions. Use "entity type" or "property type" to be specific.       |
| Property definition vs. Property value              | Definition = schema (name + type + default). Value = concrete data stored on an entity instance.                                                                                   |
| Cell vs. Unit                                       | A cell is a hex position on the board (the tile itself). A unit is a game entity placed on a cell (a piece on the tile). Cells are painted; units are placed.                      |
| Entity type (BoardPosition) vs. Entity type (Token) | Both are EntityType definitions with properties. BoardPosition types describe board positions. Token types describe game entities that move on the board. Use role to distinguish. |
