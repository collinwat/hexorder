# Changelog

All notable changes to Hexorder are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] — 2026-02-09
### Added
- Unit type definitions with custom properties (game_system)
- Unit placement on hex grid with click-to-place (unit)
- Unit movement via click-to-move (unit)
- Unit deletion (unit)
- Unit visual sync with type color and selection highlight (unit)
- UnitTypeRegistry with 3 starter unit types: Infantry, Cavalry, Artillery (game_system)
- Place tool mode for unit placement (editor_ui)
- Unit palette panel with type selection (editor_ui)
- Unit type editor for creating and editing unit types (editor_ui)
- Unit inspector panel showing selected unit details (editor_ui)
- ActiveUnitType and SelectedUnit resources (game_system)
- UnitPlacedEvent observer event (game_system)

### Changed
- EditorTool gains Place variant for unit placement mode (editor_ui)

## [0.2.0] — 2026-02-09
### Added
- Game System container with id and version (game_system)
- Property system with 6 data types: Bool, Int, Float, String, Color, Enum (game_system)
- User-defined cell types with custom properties (game_system)
- CellTypeRegistry resource replacing TerrainPalette (game_system)
- Cell painting with dynamic cell types (cell)
- Cell type editor for creating, editing, and deleting cell types (editor_ui)
- Inspector panel for viewing and editing cell property values (editor_ui)
- Editor dark theme with brand palette (editor_ui)
- Brand palette enforcement via architecture test (editor_ui)
- Hidden-window pattern to prevent white flash on launch (camera)

### Removed
- Hardcoded TerrainType enum, replaced by dynamic CellTypeId (game_system)
- Terrain plugin, replaced by Cell plugin (cell)
- TerrainPalette, TerrainEntry, ActiveTerrain resources (game_system)

## [0.1.0] — 2026-02-08
### Added
- Hex grid rendering on XZ ground plane with configurable radius (hex_grid)
- Axial coordinate system using hexx crate (hex_grid)
- Hex tile selection via raycast with visual highlight (hex_grid)
- HexSelectedEvent observer event (hex_grid)
- Orthographic top-down camera locked to Y-axis (camera)
- Camera pan via middle-click drag (camera)
- Camera zoom via scroll wheel (camera)
- Terrain painting with 5 hardcoded terrain types (terrain)
- Terrain visual sync with color-coded hex tiles (terrain)
- Minimal editor UI with tool selector and terrain palette (editor_ui)
- bevy_egui integration for editor panels (editor_ui)
- Contract boundary enforcement via architecture tests (project)
- Module privacy enforcement via private sub-modules (project)
