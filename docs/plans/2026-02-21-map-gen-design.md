# Map Generation Plugin — Design Document

**Date**: 2026-02-21 **Pitch**: #102 — Procedural hex map generation **Branch**: `0.12.0-map-gen`

## Overview

A new `map_gen` plugin that generates procedural hex maps using heightmap-based terrain. The
generator is a pure computation layer that writes results into existing tile entities via the
`EntityData` contract. No new contract types are needed for the core generation.

## Architecture

```
MapGenParams (Resource)         ← user-configurable seed/noise settings
BiomeTable (Resource)           ← elevation ranges → terrain type names
  ↓
generate_heightmap() (pure fn)  ← params + hex positions → HashMap<HexPosition, f64>
apply_biome_table() (pure fn)   ← heightmap + biome table + registry → HashMap<HexPosition, TypeId>
  ↓
apply_generation() (system)     ← writes EntityData to tile entities
  ↓
sync_cell_visuals (existing)    ← picks up Changed<EntityData> automatically
```

Pure functions for testability. Thin Bevy system layer for ECS integration.

## Data Model

### MapGenParams (Resource)

```rust
#[derive(Resource, Debug, Clone)]
pub struct MapGenParams {
    pub seed: u32,
    pub octaves: u8,         // default: 6
    pub frequency: f64,      // default: 0.03
    pub amplitude: f64,      // default: 1.0
    pub lacunarity: f64,     // default: 2.0 (frequency multiplier per octave)
    pub persistence: f64,    // default: 0.5 (amplitude multiplier per octave)
}
```

### BiomeEntry and BiomeTable (Resource)

```rust
#[derive(Debug, Clone)]
pub struct BiomeEntry {
    pub min_elevation: f64,  // inclusive
    pub max_elevation: f64,  // exclusive (last entry inclusive)
    pub terrain_name: String, // matched against EntityTypeRegistry by name
}

#[derive(Resource, Debug, Clone)]
pub struct BiomeTable {
    pub entries: Vec<BiomeEntry>, // sorted by min_elevation, no gaps
}
```

Default biome table:

| Elevation Range | Terrain Name |
| --------------- | ------------ |
| 0.00 – 0.20     | Water        |
| 0.20 – 0.40     | Plains       |
| 0.40 – 0.60     | Forest       |
| 0.60 – 0.80     | Hills        |
| 0.80 – 1.00     | Mountains    |

### No New Contracts

The generator writes to existing `EntityData` components on tile entities. The heightmap is
ephemeral — computed, applied, discarded. No elevation component is stored on tiles (rivers/roads in
later scopes will regenerate or cache the heightmap if needed).

## Hex-to-Noise Coordinate Mapping

Convert `HexPosition` to world-space XZ coordinates via `hexx::HexLayout::hex_to_world_pos()`, then
sample noise at `(x * frequency, z * frequency)`. This ensures spatial coherence — neighboring hexes
sample nearby noise values and get similar elevations.

## Generation Flow

1. User triggers generation (future: UI button; for now: system triggered by resource change)
2. Collect all `HexPosition` values from tile entities
3. `generate_heightmap(params, positions, layout)` → `HashMap<HexPosition, f64>`
4. `apply_biome_table(heightmap, biome_table)` → `HashMap<HexPosition, String>` (terrain names)
5. Resolve terrain names to `TypeId` via `EntityTypeRegistry` name lookup
6. Write `EntityData { entity_type_id, properties: defaults }` to each tile entity
7. `sync_cell_visuals` picks up changes automatically via `Changed<EntityData>`

## Generation Behavior

- **Overwrites in-place**: All tile EntityData is replaced. No confirmation dialog.
- **Unmatched terrain names**: If a biome table entry names a terrain type not in the registry,
  those tiles are skipped (left as-is). A warning is logged.
- **Deterministic**: Same seed + params + grid = same output across runs.
- **Synchronous**: Generation runs on the main thread. Background threading deferred per pitch
  rabbit hole guidance.

## Testing Strategy

Pure function tests (no Bevy App needed):

- `heightmap_deterministic` — same seed + params → identical output
- `heightmap_normalized` — all values in [0.0, 1.0]
- `heightmap_spatial_coherence` — adjacent hexes have similar elevations
- `biome_table_assigns_correctly` — elevation values map to correct terrain names
- `biome_table_covers_full_range` — [0.0, 1.0] fully covered

ECS integration test:

- `generation_updates_tile_entity_data` — after generation, tile EntityData matches expected terrain

## Scopes (from pitch)

1. **Heightmap generation** (this design) — noise + biome table + entity assignment
2. **Biome distribution** — UI for biome table editing, more terrain types
3. **River placement** — hexside features, downhill flow algorithm
4. **Road networks** — hexside features, pathfinding
5. **Seed-based reproducibility** — UI controls, parameter persistence

## Dependencies

- **Crate**: `noise` (noise-rs) for Perlin/simplex noise
- **Contracts consumed**: `hex_grid` (HexPosition, HexGridConfig, HexTile), `game_system`
  (EntityTypeRegistry, EntityData, EntityRole)
- **Contracts produced**: none

## File Layout

```
src/map_gen/
  mod.rs          # Plugin definition, MapGenPlugin
  components.rs   # MapGenParams, BiomeTable, BiomeEntry (plugin-local resources)
  systems.rs      # apply_generation system
  heightmap.rs    # generate_heightmap pure function
  biome.rs        # BiomeTable logic, apply_biome_table pure function
  tests.rs        # Unit tests
```
