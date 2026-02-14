# Plugin: terrain

## Summary

Provides a fixed palette of terrain types that the user can paint onto hex tiles. Terrain is
visually represented by color-coding the hex tile meshes. This is the first "design" action the user
can take — shaping the world.

## Plugin

- Module: `src/terrain/`
- Plugin struct: `TerrainPlugin`
- Schedule: `Startup` (palette resource), `Update` (paint input, visual sync)

## Dependencies

- **Contracts consumed**: `hex_grid` (HexPosition, HexSelectedEvent, HexGridConfig)
- **Contracts produced**: `terrain` (TerrainType, TerrainPalette)
- **Crate dependencies**: none beyond `bevy`

## Requirements

1. [REQ-TYPES] Define a fixed enum of terrain types: Plains, Forest, Water, Mountain, Road. Each
   type has a display name and a color.
2. [REQ-PALETTE] Insert a `TerrainPalette` resource at startup containing the available terrain
   types and their colors.
3. [REQ-DEFAULT] All hex tiles default to `TerrainType::Plains` when the grid is first created.
4. [REQ-PAINT] When in paint mode, clicking a hex tile sets its terrain type to the currently
   selected terrain from the palette. The system listens for `HexSelectedEvent` to know which tile
   was clicked.
5. [REQ-VISUAL] Hex tile mesh color/material updates to reflect its current terrain type. Changes
   are visible immediately after painting.
6. [REQ-COMPONENT] Terrain type is stored as a `Terrain` component on each hex tile entity, making
   it queryable by other systems.

## Success Criteria

- [x] [SC-1] All 5 terrain types exist and have distinct colors
- [x] [SC-2] TerrainPalette resource is available after Startup
- [x] [SC-3] All tiles start as Plains
- [x] [SC-4] Painting a tile changes its terrain type and visual color
- [x] [SC-5] Terrain component is queryable on hex tile entities
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes

## Decomposition

Solo feature — no parallel decomposition needed.

## Constraints

- Terrain painting only works in paint mode (not select mode) — mode is owned by editor_ui
- Terrain types are hardcoded for M1; they become data-driven in M2
- Color changes should update the material, not respawn the mesh entity

## Open Questions

- Should painting support click-and-drag to paint multiple tiles in one stroke? (Suggest: yes, but
  can defer to a fast-follow if complex)
- Should there be a "preview" of the terrain color on hover before committing the paint?
