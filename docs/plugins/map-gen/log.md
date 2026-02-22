# Plugin Log: map_gen

## Status: building (Scope 1 complete)

## Decision Log

### 2026-02-21 — Plugin naming and structure

**Context**: Pitch #102 targets hex map generation as a new capability. Need to decide whether this
extends `hex_grid` or becomes a new plugin. **Decision**: New plugin `map_gen` — procedural
generation is a separate concern from grid rendering/selection. **Rationale**: hex_grid owns the
spatial foundation (grid spawning, selection, hover). Map generation is a design tool feature that
operates on the grid. Separation keeps hex_grid focused and allows map_gen to evolve independently.
**Alternatives rejected**: Extending hex_grid (would bloat a foundational plugin with optional
design-tool features).

### 2026-02-21 — Noise library selection

**Context**: Pitch suggests noise-rs as the noise library. Need to confirm before adding dependency.
**Decision**: Use `noise` crate (noise-rs) — well-maintained, supports Perlin/simplex, no unsafe.
**Rationale**: Pitch explicitly recommends it. Supports multiple noise types needed for terrain
generation. Pure Rust with no system dependencies. **Alternatives rejected**: simdnoise (less
maintained), bracket-noise (more game-focused, less flexible).

### 2026-02-22 — Pure function architecture

**Context**: Scope 1 requires heightmap generation and biome table assignment. Need to decide where
the computation boundary lies. **Decision**: Pure functions (`generate_heightmap`,
`apply_biome_table`) for all computation; thin Bevy system for ECS integration. **Rationale**:
Testable without Bevy App, composable for future scopes (rivers/roads can layer on top of the same
heightmap). **Alternatives rejected**: All-in-one system (harder to test, harder to extend).

### 2026-02-22 — Amplitude parameter integration

**Context**: Code review flagged `amplitude` field as unused with `#[allow(dead_code)]`.
**Decision**: Integrated as the initial amplitude in FBM — controls terrain roughness by setting the
starting amplitude that decays per octave via persistence. **Rationale**: Removes dead code
suppression while making the parameter functional. Users can control terrain character through this
knob.

### 2026-02-22 — Biome table validation in generation system

**Context**: Code review noted `lookup_biome` assumes sorted entries but `run_generation` didn't
validate. **Decision**: Added `validate_biome_table` call at start of `run_generation` — logs a
warning and skips generation if table is invalid. **Rationale**: Defensive for user-defined biome
tables in future scopes. Low cost, prevents silent incorrect generation.

## Test Results

### 2026-02-22 — Scope 1 complete

```
running 12 tests
test map_gen::tests::map_gen_params_default_has_expected_seed ... ok
test map_gen::tests::empty_biome_table_fails_validation ... ok
test map_gen::tests::lookup_biome_covers_full_range ... ok
test map_gen::tests::default_biome_table_is_valid ... ok
test map_gen::tests::biome_table_gap_detected ... ok
test map_gen::tests::heightmap_deterministic_with_same_seed ... ok
test map_gen::tests::heightmap_values_in_unit_range ... ok
test map_gen::tests::heightmap_different_seeds_differ ... ok
test map_gen::tests::heightmap_spatial_coherence ... ok
test map_gen::tests::biome_lookup_boundary_values ... ok
test map_gen::tests::biome_table_apply_maps_all_positions ... ok
test map_gen::tests::full_generation_pipeline ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

Full suite: 316 tests pass. Zero clippy warnings. No boundary violations. No unwrap in production.

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- Hexside features (rivers/roads) require contract extension — not modeled yet
- Background threading for large maps (>1000 hexes) — deferred per pitch rabbit hole guidance
- Display + Error impl for BiomeTableError — nice to have for future UI integration

## Status Updates

| Date       | Status   | Notes                                             |
| ---------- | -------- | ------------------------------------------------- |
| 2026-02-21 | speccing | Initial spec created                              |
| 2026-02-22 | building | Scope 1 complete — heightmap + biome table (+651) |
