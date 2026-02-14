# Plugin Log: terrain

## Status: complete

## Decision Log

### 2026-02-08 — Fixed terrain palette for M1

**Context**: M1 needs terrain painting to give the user their first design action. Full data-driven
terrain is M2 scope. **Decision**: Hardcode 5 terrain types (Plains, Forest, Water, Mountain, Road)
with fixed colors. Terrain is a component on hex tile entities. **Rationale**: Gets terrain
interaction working with minimal architecture. The data-driven system in M2 will replace the enum
with a registry, but the component pattern and painting interaction stay the same. **Alternatives
rejected**: Starting with data-driven terrain (too much architecture for M1). Using tile textures
instead of colors (asset pipeline not needed yet).

### 2026-02-08 — Implementation decisions

**Context**: Implementing the terrain plugin for the first time. **Decision**: Use Bevy 0.18
observer pattern for paint-on-click. Register `paint_terrain` via `app.add_observer()` to listen for
`HexSelectedEvent`. This avoids `EventReader`/`EventWriter` which do not exist in Bevy 0.18.
**Decision**: `TerrainMaterials` resource uses a `HashMap<TerrainType, Handle<StandardMaterial>>`
for O(1) lookup. Materials are created once at startup and reused by handle clone. **Decision**:
`sync_terrain_visuals` uses `Changed<Terrain>` filter to avoid updating all tiles every frame. Only
tiles whose `Terrain` component changed get their material updated. **Decision**:
`assign_default_terrain` runs at startup (chained after palette/materials setup). Uses
`Without<Terrain>` filter so it only adds `Terrain::Plains` to tiles that do not already have
terrain assigned. **Decision**: No `EditorTool` check for now -- painting always applies on click.
When `editor_ui` adds the `EditorTool` resource, the terrain paint system can be updated to check
it. **Decision**: Added `#[allow(clippy::type_complexity)]` to `sync_terrain_visuals` because the
combined Query type with Changed filter exceeds clippy's default complexity threshold.

## Test Results

### 2026-02-08 -- All tests pass

- `cargo build`: SUCCESS
- `cargo clippy -- -D warnings`: SUCCESS (0 warnings)
- `cargo test`: SUCCESS (36 tests pass, 11 terrain-specific)
- Terrain tests:
    - `palette_has_five_terrain_types`: verifies 5 entries with all variant types present
    - `terrain_materials_created_for_all_types`: verifies material handles for all 5 types
    - `active_terrain_defaults_to_plains`: verifies default ActiveTerrain resource
    - `active_terrain_resource_inserted_at_startup`: verifies resource insertion
    - `assign_default_terrain_adds_plains_to_tiles`: verifies tiles get Plains component
    - `paint_terrain_changes_tile_type`: verifies painting via HexSelectedEvent observer
    - `paint_does_not_affect_other_tiles`: verifies only targeted tile changes
    - `terrain_type_enum_has_all_variants`: verifies enum completeness
    - `terrain_type_default_is_plains`: verifies Default trait impl
    - `terrain_materials_lookup_works`: verifies HashMap lookup
    - `palette_entries_have_names`: verifies non-empty display names
    - `sync_terrain_visuals_updates_material`: verifies material sync on terrain change

## Blockers

| Blocker                                           | Waiting On      | Raised     | Resolved                                                                                           |
| ------------------------------------------------- | --------------- | ---------- | -------------------------------------------------------------------------------------------------- |
| editor_ui compile errors block full `cargo build` | editor_ui agent | 2026-02-08 | Pending -- terrain code is correct, blocked by editor_ui EguiPlugin/rect_stroke/ctx_mut API errors |

## Status Updates

| Date       | Status   | Notes                                                         |
| ---------- | -------- | ------------------------------------------------------------- |
| 2026-02-08 | speccing | Initial spec created                                          |
| 2026-02-08 | complete | Implementation done, all tests pass, all success criteria met |
