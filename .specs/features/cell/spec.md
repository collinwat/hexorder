# Feature: cell

## Summary

Handles painting user-defined entity types (BoardPosition role) onto hex tiles and syncing their
visual appearance. The type definitions come from the EntityTypeRegistry — this plugin consumes them
and provides the painting interaction and material management.

M4: Migrates from CellTypeRegistry/CellData to EntityTypeRegistry/EntityData.

## Plugin

- Module: `src/cell/`
- Plugin struct: `CellPlugin`
- Schedule: `Startup` (materials, default entity data), `Update` (visual sync), Observer (paint on
  click)

## Dependencies

- **Contracts consumed**: `hex_grid` (HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial),
  `game_system` (EntityType, EntityRole, EntityTypeRegistry, EntityData, ActiveBoardType, TypeId,
  PropertyValue), `editor_ui` (EditorTool, PaintPreview)
- **Contracts produced**: none
- **Crate dependencies**: none beyond `bevy`

## Requirements

1. [REQ-MATERIALS] Create and manage material handles for each BoardPosition entity type's color.
   When types are added or their colors change, materials must be updated.
2. [REQ-DEFAULT-DATA] At startup, assign a default `EntityData` component to all hex tiles that
   don't have one. The default references the first BoardPosition type in the registry.
3. [REQ-PAINT] When `EditorTool::Paint` is active and a hex tile is clicked (`HexSelectedEvent`),
   set the tile's `EntityData` to reference the `ActiveBoardType` with default property values.
4. [REQ-VISUAL-SYNC] When a tile's `EntityData` changes, update its material to match the referenced
   entity type's color. Use change detection.
5. [REQ-DYNAMIC-MATERIALS] The material set must react to changes in the EntityTypeRegistry
   (filtered by BoardPosition role).

## Success Criteria

- [ ] [SC-1] Materials exist for all registered BoardPosition entity types
- [ ] [SC-2] All hex tiles have EntityData after startup
- [ ] [SC-3] Painting in Paint mode sets the tile's entity type
- [ ] [SC-4] Painting does NOT occur in Select mode
- [ ] [SC-5] Tile color updates when its EntityData changes
- [ ] [SC-6] Adding a new BoardPosition type at runtime creates a corresponding material
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [ ] [SC-TEST] `cargo test` passes
- [ ] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- Must filter EntityTypeRegistry by `EntityRole::BoardPosition` — ignore Token types
- Must handle the case where an entity type is deleted while tiles reference it
- Property values on EntityData are per-tile instance data — painting assigns defaults, the user can
  edit individual tile properties via the inspector

## Open Questions

- None
