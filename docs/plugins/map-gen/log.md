# Plugin Log: map_gen

## Status: building (Scopes 1-2, 5-6 complete)

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

### 2026-02-22 — UI panel as standalone egui window (not dock tab)

**Context**: Need to add UI controls for generation parameters. The editor uses egui_dock for panel
layout. **Decision**: Render the map generation panel as a standalone `egui::Window` owned by the
map_gen plugin, not as a dock tab in editor_ui. **Rationale**: Adding a dock tab would require
modifying editor_ui internals (DockTab enum, EditorDockViewer), creating a cross-plugin boundary
violation. A standalone window respects plugin boundaries, is self-contained, and can be freely
moved/resized by the user. **Alternatives rejected**: Dock tab (boundary violation), contract
extension for dock registration (over-engineering for one panel).

### 2026-02-22 — Scopes 3-4 closed, hex-edge reframed as tool primitive

**Context**: Scopes 3 (rivers) and 4 (roads) were originally hammered citing missing hex-edge
contract. Issue #150 was reframed: #151 (rivers) and #152 (roads) closed as won't-fix because they
embed game mechanics into tool-level infrastructure. Constitution on main now establishes a Tool /
Game Boundary with three layers: primitives (game-neutral spatial infrastructure), scaffolding
(optional genre-specific starter content), and game mechanics (user-defined rules, never hardcoded).
**Decision**: Add Scope 6 — hex-edge contract as a tool primitive. Provides spatial modeling (edge
identity, adjacency, annotation slots) with annotations resolving against `EntityTypeRegistry`. No
hardcoded edge feature types. **Rationale**: Hex edges are a legitimate spatial primitive any hex
game might need. What edges mean (crossing costs, movement bonuses) is a game mechanic the designer
defines.

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

### 2026-02-22 — Scope 5 complete (UI panel)

```
running 13 tests
test map_gen::tests::panel_visible_defaults_to_true ... ok
(+ 12 previous tests)

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured
```

Full suite: 318 tests pass. Zero clippy warnings. No boundary violations. No unwrap in production.

### 2026-02-22 — Scope 6 complete (hex-edge contract)

```
running 48 tests (hex_grid)
test contracts::hex_grid::tests::hex_edge_canonical_form_lower_origin ... ok
test contracts::hex_grid::tests::hex_edge_canonical_form_swaps_when_needed ... ok
test contracts::hex_grid::tests::hex_edge_between_non_adjacent_returns_none ... ok
test contracts::hex_grid::tests::hex_edge_same_edge_from_both_sides_equal ... ok
test contracts::hex_grid::tests::hex_edge_direction_wraps ... ok
test contracts::hex_grid::tests::hex_edge_neighbor_pair_returns_both_hexes ... ok
test contracts::hex_grid::tests::hex_edge_all_six_directions_produce_unique_edges ... ok
test contracts::hex_grid::tests::hex_edge_new_produces_canonical_form ... ok
test contracts::hex_grid::tests::edge_registry_insert_and_lookup ... ok
test contracts::hex_grid::tests::edge_registry_remove ... ok
test contracts::hex_grid::tests::edge_registry_canonical_lookup ... ok
test contracts::hex_grid::tests::edge_registry_iter ... ok
test contracts::hex_grid::tests::edge_registry_edges_for_hex ... ok
test contracts::hex_grid::tests::edge_feature_type_name_resolves_against_entity_registry ... ok
(+ 34 existing hex_grid tests)

test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured
```

Full suite: 332 tests pass. Zero clippy warnings. No boundary violations. No unwrap in production.

### 2026-02-22 — BiomeTable defaults neutralized (#156)

Default BiomeTable terrain names changed from game-specific (Water, Plains, Forest, Hills,
Mountains) to neutral elevation-band labels (Low, Mid-Low, Mid, Mid-High, High). All 13 map_gen
tests updated and passing.

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- ~~River placement (#151)~~ — closed as won't-fix (game-specific mechanic)
- ~~Road networks (#152)~~ — closed as won't-fix (game-specific mechanic)
- Background threading for large maps (#153) — deferred per pitch rabbit hole guidance
- Display + Error impl for BiomeTableError (#154) — nice to have for future UI integration
- Multi-page or infinite map generation (#155)

## Status Updates

| Date       | Status    | Notes                                               |
| ---------- | --------- | --------------------------------------------------- |
| 2026-02-21 | speccing  | Initial spec created                                |
| 2026-02-22 | building  | Scope 1 complete — heightmap + biome table (+651)   |
| 2026-02-22 | building  | Scope 5 complete — UI panel with parameter controls |
| 2026-02-22 | finishing | Scopes 3-4 hammered — hex-edge contract needed      |
| 2026-02-22 | building  | Scopes 3-4 closed (won't-fix); Scope 6 added (#150) |
| 2026-02-22 | building  | Scope 6 complete — hex-edge contract (14 tests)     |
| 2026-02-22 | building  | BiomeTable defaults neutralized (#156)              |
