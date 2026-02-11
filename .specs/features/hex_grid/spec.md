# Feature: hex_grid

## Summary
Renders a hexagonal grid on the 3D ground plane and provides tile selection via mouse click. This is the foundational spatial feature — all other M1 features build on it.

## Plugin
- Module: `src/hex_grid/`
- Plugin struct: `HexGridPlugin`
- Schedule: `Startup` (grid spawning, config resource), `Update` (selection input)

## Dependencies
- **Contracts consumed**: none
- **Contracts produced**: `hex_grid` (HexPosition, HexGridConfig, HexMoveEvent, HexSelectedEvent)
- **Crate dependencies**: `hexx` (already in Cargo.toml)

## Requirements
1. [REQ-GRID] Spawn a hex grid of configurable radius on the XZ ground plane (Y=0). Each hex tile is an entity with a `HexPosition` component and a visible mesh.
2. [REQ-CONFIG] Insert a `HexGridConfig` resource at startup containing the hex layout (pointy-top) and map radius. Default radius: 10.
3. [REQ-MESH] Each hex tile renders as a flat hexagonal mesh on the ground plane. Default color: a neutral base color (light gray).
4. [REQ-SELECT] When the user clicks on a hex tile, fire a `HexSelectedEvent` with the tile's position. Clicking an already-selected tile deselects it (sets `SelectedHex` to `None`). The click fires on `just_released` with a 5px drag threshold to distinguish clicks from camera drags.
5. [REQ-DESELECT] Clicking on the currently selected tile toggles selection off (`SelectedHex` set to `None`). Pressing the Escape key clears the current selection (gated behind `egui_wants_any_keyboard_input` to avoid conflicts with UI text fields).
6. [REQ-HIGHLIGHT] The currently selected tile is indicated by an opaque white ring border overlay (hollow hexagon mesh) positioned above the tile. Tiles always display their real cell type color; selection never swaps the tile material.
7. [REQ-HOVER] When the mouse hovers over a hex tile, a semi-transparent white ring border overlay (60% opacity) is shown above the tile. In Paint mode, the hover ring color matches the active paint color instead of white, providing a preview of the paint operation.

## Success Criteria
- [x] [SC-1] Hex grid renders with correct number of tiles for the configured radius
- [x] [SC-2] HexGridConfig resource is available after Startup
- [x] [SC-3] Clicking a tile fires HexSelectedEvent with correct coordinates
- [x] [SC-4] Selected tile is visually distinct via ring border overlay (not material swap)
- [x] [SC-5] Hover feedback via semi-transparent ring overlay is visible when mouse is over a tile
- [x] [SC-6] Clicking a selected tile deselects it
- [x] [SC-7] Pressing Escape clears the current selection
- [x] [SC-8] Tiles always display their real cell type color regardless of selection/hover state
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes (71 tests)

## Decomposition
Solo feature — no parallel decomposition needed.

## Constraints
- Hex math must use `hexx` crate (constitution requirement)
- Axial (q, r) coordinate system (constitution requirement)
- No per-frame allocations for grid rendering; meshes are spawned once and updated only when terrain or selection changes
- Grid must render correctly under orthographic top-down camera

## Resolved Questions
- **Hex size**: 1.0 outer radius (set via `HexLayout::default().with_hex_size(1.0)`). The `scale` field on HexLayout is a Vec2, so non-uniform scaling is possible later.
- **Deselection**: Supported via two mechanisms: (1) click-toggle -- clicking the currently selected tile deselects it, (2) Escape key -- clears any active selection. Clicking empty space still does not deselect.
