# Plugin: map_gen

## Summary

Procedural hex map generation using heightmap-based terrain. Generates plausible starting maps with
configurable terrain, rivers, and roads that designers can then refine by hand. Accelerates the
define-play-observe-revise design loop by enabling rapid map prototyping.

## Plugin

- Module: `src/map_gen/`
- Plugin struct: `MapGenPlugin`
- Schedule: `Update` (generation system), `EguiPrimaryContextPass` (UI panel)

## Appetite

- **Size**: Small Batch (1-2 weeks)
- **Pitch**: #102

## Dependencies

- **Contracts consumed**: `hex_grid` (HexPosition, HexGridConfig, HexTile, TileBaseMaterial),
  `game_system` (EntityTypeRegistry, EntityType, EntityData, CellTypeAssignment), `editor_ui`
  (EditorTool)
- **Contracts produced**: none (generates standard hex grid data using existing contracts)
- **Crate dependencies**: `noise` (noise-rs — Perlin/simplex noise generation)

## Scope

1. [SCOPE-1] Heightmap generation — layered Perlin/simplex noise with configurable octaves,
   frequency, amplitude, and seed. Maps noise values to elevation levels via configurable
   thresholds.
2. [SCOPE-2] Biome distribution — configurable biome table mapping elevation ranges to terrain
   (cell) types. Applies terrain assignments to hex tiles based on heightmap elevation.
3. [SCOPE-3] River placement — downhill flow algorithm from random high-elevation hexes, tracing
   steepest descent to water or map edge. Rivers are hex-edge features.
4. [SCOPE-4] Road networks — shortest-path through favorable terrain connecting designated points.
   Roads are hex-edge features using terrain-weighted costs.
5. [SCOPE-5] Seed-based reproducibility — all generation uses a configurable seed. Same seed +
   parameters = same map. UI controls for seed and generation parameters.

## Success Criteria

- [x] [SC-1] `heightmap_generates_consistent_elevations` — same seed produces identical elevation
      values across runs
- [x] [SC-2] `biome_table_assigns_correct_terrain` — elevation ranges correctly map to cell types
- [ ] [SC-3] `rivers_flow_downhill` — river paths always follow decreasing elevation
- [ ] [SC-4] `roads_connect_endpoints` — road pathfinding produces valid connected paths
- [x] [SC-5] `seed_reproducibility` — full generation with same parameters produces identical output
      (core determinism proven; UI controls for seed and all noise parameters implemented)
- [x] [SC-6] Generated maps are fully editable after creation (no link back to generator — writes
      EntityData directly, sync_cell_visuals picks up changes, no generator reference stored)
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other plugins' internals

## UAT Checklist

- [ ] [UAT-1] Launch app, open generation panel, set seed and parameters, click Generate — map
      appears with distinct terrain regions
- [ ] [UAT-2] Generate two maps with same seed — verify visually identical
- [ ] [UAT-3] Generate a map, then manually paint a tile — verify the tile is fully editable
- [ ] [UAT-4] Generate a map with rivers visible flowing from high to low terrain
- [ ] [UAT-5] Generate a map with roads connecting across the terrain

## Decomposition

Solo — no parallel decomposition needed. Scopes are sequential (heightmap first, then biome, then
rivers/roads).

## Constraints

- Hex math must use `hexx` crate (constitution requirement)
- Generated maps must use existing cell type system — no new terrain primitives
- All generation must be deterministic given a seed
- Generation runs on a background thread for large maps (>1000 hexes)
- No `unwrap()` in production code

## Open Questions

- ~~Should generation replace the current map or require a blank map first?~~ Resolved: overwrites
  in-place (design doc decision)
- ~~How does the biome table interact with user-defined cell types?~~ Resolved: maps to existing
  registered cell types by name via `EntityTypeRegistry` lookup

## Deferred Items

- River placement — requires hex-edge contract (#150, #151)
- Road networks — requires hex-edge contract (#150, #152)
- Temperature and moisture axes for biome selection (#102 No Go)
- City/town auto-placement (#102 No Go)
- Historical map import / real-world elevation data (#102 No Go)
- Multi-page or infinite map generation (#155)
- Background threading for large maps (#153)
- Display + Error impl for BiomeTableError (#154)
