# Heightmap Generation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Build the heightmap generation core — noise-based elevation with biome table terrain
assignment — as the foundation for procedural hex map generation.

**Architecture:** Pure functions (`generate_heightmap`, `apply_biome_table`) for testability,
wrapped by a thin Bevy system that writes results to existing tile `EntityData`. The `noise` crate
provides Perlin noise with fractal Brownian motion. Hex positions are converted to world-space XZ
for noise sampling.

**Tech Stack:** Rust, Bevy 0.18, `noise` crate (noise-rs), `hexx` crate (already in project)

**Design doc:** `docs/plans/2026-02-21-map-gen-design.md`

---

### Task 1: Add noise dependency

**Files:**

- Modify: `Cargo.toml`

**Step 1: Add `noise` crate to `[dependencies]`**

In `Cargo.toml`, add after the `hexx` line:

```toml
noise = "0.9"
```

**Step 2: Verify it compiles**

Run: `cargo check`

Expected: Clean compilation. The `noise` crate is pure Rust, no system deps.

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(map_gen): add noise crate dependency"
```

---

### Task 2: Create map_gen module skeleton with MapGenPlugin

**Files:**

- Create: `src/map_gen/mod.rs`
- Create: `src/map_gen/components.rs`
- Create: `src/map_gen/heightmap.rs`
- Create: `src/map_gen/biome.rs`
- Create: `src/map_gen/systems.rs`
- Create: `src/map_gen/tests.rs`
- Modify: `src/main.rs:1-16` (add `mod map_gen`)
- Modify: `src/main.rs:39-50` (add plugin registration)

**Step 1: Create `src/map_gen/components.rs`**

```rust
//! Plugin-local resources for map generation.

use bevy::prelude::*;

/// Parameters controlling heightmap noise generation.
#[derive(Resource, Debug, Clone)]
pub struct MapGenParams {
    /// Random seed for noise generation. Same seed = same output.
    pub seed: u32,
    /// Number of noise octaves layered together. More octaves = more detail.
    pub octaves: usize,
    /// Base frequency of the noise. Lower = larger terrain features.
    pub frequency: f64,
    /// Controls the overall height range of the noise output.
    pub amplitude: f64,
    /// Frequency multiplier per octave. Typical: 2.0.
    pub lacunarity: f64,
    /// Amplitude multiplier per octave. Typical: 0.5.
    pub persistence: f64,
}

impl Default for MapGenParams {
    fn default() -> Self {
        Self {
            seed: 42,
            octaves: 6,
            frequency: 0.03,
            amplitude: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

/// A single entry in the biome table mapping an elevation range to a terrain name.
#[derive(Debug, Clone)]
pub struct BiomeEntry {
    /// Minimum elevation (inclusive).
    pub min_elevation: f64,
    /// Maximum elevation (exclusive, except for the last entry which is inclusive).
    pub max_elevation: f64,
    /// Name of the terrain type, matched against `EntityTypeRegistry` by name.
    pub terrain_name: String,
}

/// Maps elevation ranges to terrain type names. Entries must be sorted by
/// `min_elevation` with no gaps covering the full [0.0, 1.0] range.
#[derive(Resource, Debug, Clone)]
pub struct BiomeTable {
    pub entries: Vec<BiomeEntry>,
}

impl Default for BiomeTable {
    fn default() -> Self {
        Self {
            entries: vec![
                BiomeEntry {
                    min_elevation: 0.0,
                    max_elevation: 0.2,
                    terrain_name: "Water".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.2,
                    max_elevation: 0.4,
                    terrain_name: "Plains".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.4,
                    max_elevation: 0.6,
                    terrain_name: "Forest".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.6,
                    max_elevation: 0.8,
                    terrain_name: "Hills".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.8,
                    max_elevation: 1.0,
                    terrain_name: "Mountains".to_string(),
                },
            ],
        }
    }
}

/// Marker resource that triggers map generation when inserted.
/// Consumed (removed) after generation completes.
#[derive(Resource, Debug)]
pub struct GenerateMap;
```

**Step 2: Create `src/map_gen/heightmap.rs`**

```rust
//! Pure heightmap generation using layered Perlin noise.

use std::collections::HashMap;

use noise::{NoiseFn, Perlin};

use crate::contracts::hex_grid::HexPosition;

/// Generate a heightmap for the given hex positions.
///
/// Returns elevation values normalized to [0.0, 1.0] for each position.
/// Uses layered Perlin noise (fractal Brownian motion) sampled at
/// world-space coordinates derived from the hex layout.
pub fn generate_heightmap(
    seed: u32,
    octaves: usize,
    frequency: f64,
    lacunarity: f64,
    persistence: f64,
    positions: &[HexPosition],
    layout: &hexx::HexLayout,
) -> HashMap<HexPosition, f64> {
    let perlin = Perlin::new(seed);
    let mut result = HashMap::with_capacity(positions.len());

    for &pos in positions {
        let world = layout.hex_to_world_pos(pos.to_hex());
        let value = fbm_sample(
            &perlin,
            world.x as f64 * frequency,
            world.y as f64 * frequency,
            octaves,
            lacunarity,
            persistence,
        );
        // Normalize from roughly [-1, 1] to [0, 1]
        let normalized = (value + 1.0) * 0.5;
        result.insert(pos, normalized.clamp(0.0, 1.0));
    }

    result
}

/// Fractal Brownian motion: layer multiple octaves of Perlin noise.
fn fbm_sample(
    noise: &Perlin,
    x: f64,
    y: f64,
    octaves: usize,
    lacunarity: f64,
    persistence: f64,
) -> f64 {
    let mut total = 0.0;
    let mut freq = 1.0;
    let mut amp = 1.0;
    let mut max_amp = 0.0;

    for _ in 0..octaves {
        total += noise.get([x * freq, y * freq]) * amp;
        max_amp += amp;
        freq *= lacunarity;
        amp *= persistence;
    }

    // Normalize by max possible amplitude so output stays in [-1, 1]
    if max_amp > 0.0 {
        total / max_amp
    } else {
        0.0
    }
}
```

**Step 3: Create `src/map_gen/biome.rs`**

```rust
//! Biome table logic — maps elevation values to terrain type names.

use std::collections::HashMap;

use crate::contracts::hex_grid::HexPosition;

use super::components::{BiomeEntry, BiomeTable};

/// Look up the terrain name for a given elevation value.
///
/// Returns `None` if no biome entry covers the given elevation.
pub fn lookup_biome(table: &BiomeTable, elevation: f64) -> Option<&str> {
    for (i, entry) in table.entries.iter().enumerate() {
        let is_last = i == table.entries.len() - 1;
        if is_last {
            // Last entry: inclusive on both ends
            if elevation >= entry.min_elevation && elevation <= entry.max_elevation {
                return Some(&entry.terrain_name);
            }
        } else {
            // Non-last entries: inclusive min, exclusive max
            if elevation >= entry.min_elevation && elevation < entry.max_elevation {
                return Some(&entry.terrain_name);
            }
        }
    }
    None
}

/// Apply a biome table to a heightmap, returning terrain names per hex position.
///
/// Positions whose elevation doesn't match any biome entry are omitted from
/// the result.
pub fn apply_biome_table(
    heightmap: &HashMap<HexPosition, f64>,
    table: &BiomeTable,
) -> HashMap<HexPosition, String> {
    let mut result = HashMap::with_capacity(heightmap.len());

    for (&pos, &elevation) in heightmap {
        if let Some(name) = lookup_biome(table, elevation) {
            result.insert(pos, name.to_string());
        }
    }

    result
}

/// Validate that a biome table covers the full [0.0, 1.0] range with no gaps.
pub fn validate_biome_table(table: &BiomeTable) -> Result<(), BiomeTableError> {
    if table.entries.is_empty() {
        return Err(BiomeTableError::Empty);
    }

    let sorted: Vec<&BiomeEntry> = {
        let mut entries: Vec<_> = table.entries.iter().collect();
        entries.sort_by(|a, b| {
            a.min_elevation
                .partial_cmp(&b.min_elevation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries
    };

    if sorted[0].min_elevation > 0.0 {
        return Err(BiomeTableError::GapAtStart(sorted[0].min_elevation));
    }

    let last = sorted.last().expect("non-empty checked above");
    if last.max_elevation < 1.0 {
        return Err(BiomeTableError::GapAtEnd(last.max_elevation));
    }

    for window in sorted.windows(2) {
        let current = window[0];
        let next = window[1];
        if (current.max_elevation - next.min_elevation).abs() > f64::EPSILON {
            return Err(BiomeTableError::Gap {
                after: current.terrain_name.clone(),
                before: next.terrain_name.clone(),
            });
        }
    }

    Ok(())
}

/// Errors from biome table validation.
#[derive(Debug)]
pub enum BiomeTableError {
    Empty,
    GapAtStart(f64),
    GapAtEnd(f64),
    Gap { after: String, before: String },
}
```

**Step 4: Create `src/map_gen/systems.rs`**

```rust
//! Bevy systems for map generation.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityData, EntityRole, EntityTypeRegistry, PropertyValue,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};

use super::biome::apply_biome_table;
use super::components::{BiomeTable, GenerateMap, MapGenParams};
use super::heightmap::generate_heightmap;

/// System that runs when `GenerateMap` resource is present.
/// Generates a heightmap, applies the biome table, and writes
/// `EntityData` to all tile entities. Removes `GenerateMap` when done.
pub fn run_generation(
    mut commands: Commands,
    params: Res<MapGenParams>,
    biome_table: Res<BiomeTable>,
    grid_config: Res<HexGridConfig>,
    registry: Res<EntityTypeRegistry>,
    generate: Option<Res<GenerateMap>>,
    mut tiles: Query<(&HexPosition, &mut EntityData), With<HexTile>>,
) {
    // Only run when GenerateMap marker resource is present.
    if generate.is_none() {
        return;
    }

    // Collect all tile positions.
    let positions: Vec<HexPosition> = tiles.iter().map(|(pos, _)| *pos).collect();

    if positions.is_empty() {
        commands.remove_resource::<GenerateMap>();
        return;
    }

    // Generate heightmap.
    let heightmap = generate_heightmap(
        params.seed,
        params.octaves,
        params.frequency,
        params.lacunarity,
        params.persistence,
        &positions,
        &grid_config.layout,
    );

    // Apply biome table to get terrain names.
    let terrain_names = apply_biome_table(&heightmap, &biome_table);

    // Build name-to-TypeId lookup for BoardPosition entity types.
    let name_to_type: HashMap<&str, _> = registry
        .types_by_role(EntityRole::BoardPosition)
        .into_iter()
        .map(|et| (et.name.as_str(), et))
        .collect();

    // Write EntityData to tiles.
    for (pos, mut entity_data) in &mut tiles {
        if let Some(terrain_name) = terrain_names.get(pos) {
            if let Some(entity_type) = name_to_type.get(terrain_name.as_str()) {
                let new_properties: HashMap<_, _> = entity_type
                    .properties
                    .iter()
                    .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
                    .collect();

                entity_data.entity_type_id = entity_type.id;
                entity_data.properties = new_properties;
            } else {
                warn!(
                    "Biome table references terrain '{}' not found in registry",
                    terrain_name
                );
            }
        }
    }

    // Remove the marker to prevent re-running.
    commands.remove_resource::<GenerateMap>();
}
```

**Step 5: Create `src/map_gen/tests.rs`** (empty placeholder)

```rust
//! Unit tests for map generation.

#[cfg(test)]
mod tests {
    // Tests will be added in Task 3.
}
```

**Step 6: Create `src/map_gen/mod.rs`**

```rust
//! Procedural hex map generation plugin.
//!
//! Generates heightmap-based terrain using layered Perlin noise and
//! a configurable biome table that maps elevation ranges to cell types.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

pub mod biome;
pub mod components;
pub mod heightmap;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that provides procedural map generation.
#[derive(Debug)]
pub struct MapGenPlugin;

impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::MapGenParams>()
            .init_resource::<components::BiomeTable>()
            .add_systems(
                Update,
                systems::run_generation.run_if(in_state(AppScreen::Editor)),
            );
    }
}
```

**Step 7: Register plugin in `src/main.rs`**

Add `mod map_gen;` after `mod hex_grid;` (line ~9) in the module declarations.

Add `.add_plugins(map_gen::MapGenPlugin)` after the `undo_redo` plugin line and before `editor_ui`
(since map_gen reads hex_grid and game_system resources but doesn't interact with editor_ui).

**Step 8: Verify it compiles**

Run: `cargo build --features dev`

Expected: Clean compilation.

**Step 9: Commit**

```bash
git add src/map_gen/ src/main.rs
git commit -m "feat(map_gen): add plugin skeleton with heightmap and biome modules"
```

---

### Task 3: Write heightmap tests (TDD red phase)

**Files:**

- Modify: `src/map_gen/tests.rs`

**Step 1: Write failing tests**

Replace the placeholder in `src/map_gen/tests.rs`:

```rust
//! Unit tests for map generation.

use std::collections::HashMap;

use crate::contracts::hex_grid::HexPosition;
use super::biome::{apply_biome_table, lookup_biome, validate_biome_table};
use super::components::{BiomeTable, MapGenParams};
use super::heightmap::generate_heightmap;

fn default_layout() -> hexx::HexLayout {
    hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        origin: glam::Vec2::ZERO,
        hex_size: glam::Vec2::splat(1.0),
        invert_x: false,
        invert_y: false,
    }
}

fn sample_positions() -> Vec<HexPosition> {
    // A small set of hex positions for testing: center + ring of 6 neighbors
    let mut positions = vec![HexPosition::new(0, 0)];
    let hex = hexx::Hex::ZERO;
    for neighbor in hex.all_neighbors() {
        positions.push(HexPosition::from_hex(neighbor));
    }
    positions
}

#[test]
fn heightmap_deterministic() {
    let params = MapGenParams::default();
    let layout = default_layout();
    let positions = sample_positions();

    let map1 = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );
    let map2 = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );

    assert_eq!(map1.len(), map2.len());
    for (pos, val1) in &map1 {
        let val2 = map2.get(pos).expect("same positions");
        assert!(
            (val1 - val2).abs() < f64::EPSILON,
            "Position ({}, {}): {val1} != {val2}",
            pos.q, pos.r,
        );
    }
}

#[test]
fn heightmap_normalized() {
    let params = MapGenParams::default();
    let layout = default_layout();
    let positions = sample_positions();

    let heightmap = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );

    for (pos, &val) in &heightmap {
        assert!(
            (0.0..=1.0).contains(&val),
            "Position ({}, {}): elevation {val} out of [0.0, 1.0]",
            pos.q, pos.r,
        );
    }
}

#[test]
fn heightmap_different_seeds_differ() {
    let layout = default_layout();
    let positions = sample_positions();
    let params = MapGenParams::default();

    let map1 = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );
    let map2 = generate_heightmap(
        params.seed + 1, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );

    // At least some positions should differ between different seeds
    let mut any_differ = false;
    for (pos, val1) in &map1 {
        if let Some(val2) = map2.get(pos) {
            if (val1 - val2).abs() > 0.001 {
                any_differ = true;
                break;
            }
        }
    }
    assert!(any_differ, "Different seeds should produce different heightmaps");
}

#[test]
fn heightmap_spatial_coherence() {
    let layout = default_layout();
    let positions = sample_positions();
    let params = MapGenParams {
        frequency: 0.01,  // low frequency = smoother terrain
        ..MapGenParams::default()
    };

    let heightmap = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );

    let center_val = heightmap[&HexPosition::new(0, 0)];
    // With low frequency, neighbors should be close to the center value
    let hex = hexx::Hex::ZERO;
    for neighbor in hex.all_neighbors() {
        let pos = HexPosition::from_hex(neighbor);
        let neighbor_val = heightmap[&pos];
        let diff = (center_val - neighbor_val).abs();
        assert!(
            diff < 0.5,
            "At low frequency, adjacent hexes should be similar. Center={center_val}, neighbor({},{})={neighbor_val}, diff={diff}",
            pos.q, pos.r,
        );
    }
}

#[test]
fn biome_lookup_assigns_correctly() {
    let table = BiomeTable::default();

    assert_eq!(lookup_biome(&table, 0.0), Some("Water"));
    assert_eq!(lookup_biome(&table, 0.1), Some("Water"));
    assert_eq!(lookup_biome(&table, 0.2), Some("Plains"));
    assert_eq!(lookup_biome(&table, 0.5), Some("Forest"));
    assert_eq!(lookup_biome(&table, 0.7), Some("Hills"));
    assert_eq!(lookup_biome(&table, 0.9), Some("Mountains"));
    assert_eq!(lookup_biome(&table, 1.0), Some("Mountains"));
}

#[test]
fn biome_lookup_boundary_values() {
    let table = BiomeTable::default();

    // Boundary between Water and Plains at 0.2
    assert_eq!(lookup_biome(&table, 0.199_999_999), Some("Water"));
    assert_eq!(lookup_biome(&table, 0.2), Some("Plains"));

    // Boundary between Plains and Forest at 0.4
    assert_eq!(lookup_biome(&table, 0.399_999_999), Some("Plains"));
    assert_eq!(lookup_biome(&table, 0.4), Some("Forest"));
}

#[test]
fn biome_table_apply_maps_all_positions() {
    let table = BiomeTable::default();
    let mut heightmap = HashMap::new();
    heightmap.insert(HexPosition::new(0, 0), 0.1);
    heightmap.insert(HexPosition::new(1, 0), 0.5);
    heightmap.insert(HexPosition::new(0, 1), 0.9);

    let result = apply_biome_table(&heightmap, &table);

    assert_eq!(result.len(), 3);
    assert_eq!(result[&HexPosition::new(0, 0)], "Water");
    assert_eq!(result[&HexPosition::new(1, 0)], "Forest");
    assert_eq!(result[&HexPosition::new(0, 1)], "Mountains");
}

#[test]
fn biome_table_default_validates() {
    let table = BiomeTable::default();
    assert!(validate_biome_table(&table).is_ok());
}

#[test]
fn biome_table_empty_fails_validation() {
    let table = BiomeTable {
        entries: Vec::new(),
    };
    assert!(validate_biome_table(&table).is_err());
}

#[test]
fn biome_table_gap_fails_validation() {
    let table = BiomeTable {
        entries: vec![
            super::components::BiomeEntry {
                min_elevation: 0.0,
                max_elevation: 0.3,
                terrain_name: "Water".to_string(),
            },
            // Gap from 0.3 to 0.5
            super::components::BiomeEntry {
                min_elevation: 0.5,
                max_elevation: 1.0,
                terrain_name: "Land".to_string(),
            },
        ],
    };
    assert!(validate_biome_table(&table).is_err());
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib map_gen`

Expected: All tests pass. (The code is already written in Task 2, so this is green from the start.
The tests validate the pure functions work correctly.)

**Step 3: Commit**

```bash
git add src/map_gen/tests.rs
git commit -m "test(map_gen): add heightmap and biome table unit tests"
```

---

### Task 4: Run full test suite and fix any issues

**Files:**

- Possibly modify: any file with issues

**Step 1: Run clippy**

Run: `cargo clippy --all-targets`

Expected: Zero warnings. Fix any that appear.

**Step 2: Run full test suite**

Run: `cargo test`

Expected: All tests pass (existing + new map_gen tests).

**Step 3: Run boundary check**

Run: `mise check:boundary`

Expected: No cross-plugin import violations. map_gen only imports from `crate::contracts::`.

**Step 4: Run unwrap check**

Run: `mise check:unwrap`

Expected: No `unwrap()` in production code (map_gen has none).

**Step 5: Commit if any fixes were needed**

```bash
git add -A
git commit -m "fix(map_gen): resolve lint and test issues"
```

---

### Task 5: Verify end-to-end generation works

**Step 1: Write a manual integration smoke test**

Add to `src/map_gen/tests.rs`:

```rust
#[test]
fn full_generation_pipeline() {
    // End-to-end test: params → heightmap → biome → terrain names
    let params = MapGenParams::default();
    let layout = default_layout();

    // Generate a larger grid: radius 3 = 37 hexes
    let positions: Vec<HexPosition> = hexx::shapes::hexagon(hexx::Hex::ZERO, 3)
        .map(HexPosition::from_hex)
        .collect();

    let heightmap = generate_heightmap(
        params.seed, params.octaves, params.frequency,
        params.lacunarity, params.persistence, &positions, &layout,
    );

    assert_eq!(heightmap.len(), positions.len());

    let table = BiomeTable::default();
    let terrain = apply_biome_table(&heightmap, &table);

    // Every position should get a terrain assignment
    assert_eq!(terrain.len(), positions.len());

    // All terrain names should be from the default biome table
    let valid_names: std::collections::HashSet<&str> =
        ["Water", "Plains", "Forest", "Hills", "Mountains"]
            .iter()
            .copied()
            .collect();
    for name in terrain.values() {
        assert!(
            valid_names.contains(name.as_str()),
            "Unexpected terrain name: {name}"
        );
    }
}
```

**Step 2: Run it**

Run: `cargo test --lib map_gen::tests::full_generation_pipeline`

Expected: PASS.

**Step 3: Commit**

```bash
git add src/map_gen/tests.rs
git commit -m "test(map_gen): add full generation pipeline integration test"
```

---

### Task 6: Update spec and log, post progress

**Files:**

- Modify: `docs/plugins/map-gen/spec.md` (mark SC-1, SC-2, SC-5 partially)
- Modify: `docs/plugins/map-gen/log.md` (record test results)

**Step 1: Update spec success criteria**

Mark as complete:

- `[x] [SC-1] heightmap_generates_consistent_elevations`
- `[x] [SC-2] biome_table_assigns_correct_terrain`
- `[x] [SC-5] seed_reproducibility` (partially — core determinism proven, UI deferred)

**Step 2: Update log with test results**

Add a test results entry with the output of `cargo test --lib map_gen`.

**Step 3: Commit**

```bash
git add docs/plugins/map-gen/spec.md docs/plugins/map-gen/log.md
git commit -m "docs(map_gen): update spec criteria and log with scope 1 test results"
```

**Step 4: Post scope completion comment**

```bash
gh issue comment 102 --body "Scope 1 complete (commit <SHA>, +N/-M across K files): Heightmap generation with layered Perlin noise and biome table terrain assignment. Pure function architecture validated — 11 unit tests pass. Determinism, normalization, spatial coherence, and boundary handling all verified. No abstraction needed — the functions are self-contained and the design naturally supports future scopes (rivers, roads) by composing additional pure functions on top of the same heightmap."
```
