# Feature: cell

## Summary
Replaces the M1 `terrain` plugin. Handles painting user-defined cell types onto hex tiles and syncing their visual appearance. The cell type definitions come from the `game_system` plugin's registry — this plugin consumes them and provides the painting interaction and material management.

## Plugin
- Module: `src/cell/`
- Plugin struct: `CellPlugin`
- Schedule: `Startup` (materials, default cell data), `Update` (visual sync), Observer (paint on click)

## Dependencies
- **Contracts consumed**: `hex_grid` (HexPosition, HexSelectedEvent, HexTile), `game_system` (CellType, CellTypeRegistry, CellData, ActiveCellType, CellTypeId), `editor_ui` (EditorTool)
- **Contracts produced**: none
- **Crate dependencies**: none beyond `bevy`

## Requirements
1. [REQ-MATERIALS] Create and manage material handles for each cell type's color. When cell types are added or their colors change, materials must be updated.
2. [REQ-DEFAULT-DATA] At startup, assign a default `CellData` component to all hex tiles that don't have one. The default references the first cell type in the registry.
3. [REQ-PAINT] When `EditorTool::Paint` is active and a hex tile is clicked (`HexSelectedEvent`), set the tile's `CellData` to reference the `ActiveCellType` with default property values for that type.
4. [REQ-VISUAL-SYNC] When a tile's `CellData` changes, update its material to match the referenced cell type's color. Use change detection to avoid per-frame updates.
5. [REQ-DYNAMIC-MATERIALS] The material set must react to changes in the cell type registry (new types added, colors changed, types removed). This can run periodically or on change detection.

## Success Criteria
- [x] [SC-1] Materials exist for all registered cell types
- [x] [SC-2] All hex tiles have CellData after startup
- [x] [SC-3] Painting in Paint mode sets the tile's cell type
- [x] [SC-4] Painting does NOT occur in Select mode
- [x] [SC-5] Tile color updates when its CellData changes
- [x] [SC-6] Adding a new cell type at runtime creates a corresponding material
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other features' internals

## Decomposition
Solo feature — no parallel decomposition needed.

## Constraints
- This plugin replaces `terrain`. The `terrain` module and `terrain` contract are retired in M2.
- Must handle the case where a cell type is deleted while tiles reference it (reassign to a fallback type or mark as untyped)
- Property values on CellData are per-tile instance data — painting assigns defaults from the cell type, but the user can edit individual tile properties via the inspector

## Open Questions
- Should painting copy the cell type's default property values, or should tiles inherit from the type definition and only store overrides? (Suggest: copy defaults for M2 simplicity; inheritance is an optimization for later)
