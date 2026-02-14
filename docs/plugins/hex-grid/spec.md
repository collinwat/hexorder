# Plugin: hex_grid

## Summary

Renders a hexagonal grid on the 3D ground plane and provides tile selection via mouse click. This is
the foundational spatial feature — all other features build on it.

M4: Adds move overlay rendering — when a unit is selected and constraints exist, hex tiles are
overlaid with green (valid) or red (blocked) indicators based on the ValidMoveSet resource.

## Plugin

- Module: `src/hex_grid/`
- Plugin struct: `HexGridPlugin`
- Schedule: `Startup` (grid spawning, config resource, overlay materials), `Update` (selection
  input, overlay rendering)

## Dependencies

- **Contracts consumed**: `validation` (ValidMoveSet), `game_system` (SelectedUnit), `editor_ui`
  (EditorTool, PaintPreview)
- **Contracts produced**: `hex_grid` (HexPosition, HexGridConfig, HexMoveEvent, HexSelectedEvent,
  HexTile, TileBaseMaterial, SelectedHex, MoveOverlay, MoveOverlayState)
- **Crate dependencies**: `hexx` (already in Cargo.toml)

## Requirements

### M1 (retained)

1. [REQ-GRID] Spawn a hex grid of configurable radius on the XZ ground plane (Y=0). Each hex tile is
   an entity with a HexPosition component and a visible mesh.
2. [REQ-CONFIG] Insert a HexGridConfig resource at startup. Default radius: 10.
3. [REQ-MESH] Each hex tile renders as a flat hexagonal mesh on the ground plane.
4. [REQ-SELECT] When the user clicks on a hex tile, fire a HexSelectedEvent. Click fires on
   just_released with a 5px drag threshold.
5. [REQ-DESELECT] Click-toggle and Escape key deselection.
6. [REQ-HIGHLIGHT] Selected tile indicated by opaque white ring border overlay.
7. [REQ-HOVER] Hover feedback via semi-transparent ring overlay. Paint mode uses active paint color.

### M4 (new — move overlays)

8. [REQ-OVERLAY-MATERIALS] At Startup, create shared materials for move overlays:
    - Valid: semi-transparent green
    - Blocked: semi-transparent red New color literals must be added to the brand palette.
9. [REQ-OVERLAY-SPAWN] When ValidMoveSet.for_entity changes from None to Some, spawn lightweight
   overlay entities above tiles (y=0.015) for each position in valid_positions and
   blocked_explanations. Use MoveOverlay component.
10. [REQ-OVERLAY-DESPAWN] When ValidMoveSet.for_entity changes to None (unit deselected), despawn
    all MoveOverlay entities.
11. [REQ-OVERLAY-UPDATE] When ValidMoveSet changes (same entity, different valid set), update
    overlay entities to match. Reuse the entity pool where possible.
12. [REQ-OVERLAY-VISUALS] Valid positions get the green overlay material. Blocked positions get the
    red overlay material. Use the hollow hexagon ring mesh (same shape as hover/select indicators).

## Success Criteria

### M1 (retained)

- [x] [SC-1] Hex grid renders with correct number of tiles for the configured radius
- [x] [SC-2] HexGridConfig resource is available after Startup
- [x] [SC-3] Clicking a tile fires HexSelectedEvent with correct coordinates
- [x] [SC-4] Selected tile is visually distinct via ring border overlay
- [x] [SC-5] Hover feedback is visible when mouse is over a tile
- [x] [SC-6] Clicking a selected tile deselects it
- [x] [SC-7] Pressing Escape clears the current selection
- [x] [SC-8] Tiles always display their real cell type color regardless of selection/hover state

### M4 (new)

- [ ] [SC-9] `move_overlays_spawned_on_unit_select` test — MoveOverlay entities appear when a unit
      is selected and ValidMoveSet is non-empty
- [ ] [SC-10] `move_overlays_despawned_on_deselect` test — all MoveOverlay entities removed when
      unit deselected
- [ ] [SC-11] `valid_positions_get_green_overlay` test — positions in valid_positions have Valid
      state
- [ ] [SC-12] `blocked_positions_get_red_overlay` test — positions in blocked_explanations have
      Blocked state
- [ ] [SC-13] No overlays when ValidMoveSet is empty (no constraints, free movement)
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [ ] [SC-TEST] `cargo test` passes
- [ ] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- Hex math must use `hexx` crate (constitution requirement)
- Axial (q, r) coordinate system (constitution requirement)
- Move overlay entities are separate from tile entities — do not modify tile materials
- Overlay materials must be in the brand palette (`docs/brand.md`)
- Overlays must not interfere with hover/selection ring overlays (different Y offsets)
- When ValidMoveSet is empty (no constraints), no overlays are shown — preserving M3 visual behavior

## Resolved Questions

- **Hex size**: 1.0 outer radius
- **Deselection**: Click-toggle + Escape key
- **Overlay strategy**: Separate lightweight entities above tiles, not tile material modification
