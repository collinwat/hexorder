# Plugin Log: hex_grid

## Status: in-progress (0.7.0)

## Decision Log

### 2026-02-08 — Initial spec created from 0.1.0 roadmap

**Context**: 0.1.0 release requires a renderable, interactive hex grid as the foundation.
**Decision**: hex_grid owns grid rendering, tile entity spawning, selection input, and hover
feedback. **Rationale**: Keeps spatial foundation in one plugin. Terrain visuals are owned by the
terrain feature. **Alternatives rejected**: Merging terrain and grid into one plugin (violates
single-responsibility; terrain may change independently).

### 2026-02-08 — Implementation with Bevy 0.18 observer events

**Context**: Bevy 0.18 removed `EventWriter`, `EventReader`, `Events<T>` resource, and
`app.add_event()`. Events now use the observer pattern. **Decision**: Use
`commands.trigger(HexSelectedEvent)` to fire events. Consumers register observers via
`app.add_observer(|trigger: On<HexSelectedEvent>| { ... })`. **Rationale**: This is the only event
mechanism available in Bevy 0.18. Observer-based events are triggered immediately on command flush,
which is suitable for selection events. **Impact on consumers**: Any feature that previously would
have used `EventReader<HexSelectedEvent>` must instead use `app.add_observer()` with
`On<HexSelectedEvent>`.

### 2026-02-08 — hexx 0.22 API changes

**Context**: hexx 0.22 renamed `HexLayout.hex_size` to `HexLayout.scale` (Vec2). The builder method
`with_hex_size(f32)` sets a uniform size. No `with_orientation()` method exists; set `orientation`
field directly on the struct. **Decision**: Construct layout via struct initialization with
`orientation: Pointy`, then chain `.with_hex_size(1.0)`. **Impact**: The
`HexGridConfig.layout.scale` field is what to use for hex size, not `hex_size`.

### 2026-02-08 — Bevy 0.18 system ordering

**Context**: In Bevy 0.18, bare function items no longer implement `IntoSystemSet`, so
`.after(system_fn)` does not compile on bare fn pointers within tuple system sets. **Decision**: Use
`.chain()` on system tuples for ordering. Startup systems are chained:
`setup_grid_config -> setup_materials -> spawn_grid`. Update systems are chained:
`update_hover -> handle_click -> update_tile_visuals`.

### 2026-02-08 — Mesh API changes

**Context**: `bevy::render::mesh::Indices` and `bevy::render::mesh::PrimitiveTopology` are private
in Bevy 0.18. They are accessible via `bevy::mesh::Indices` and `bevy::mesh::PrimitiveTopology`.

### 2026-02-08 — Camera module blocking compilation

**Context**: The camera module was added to main.rs by another agent but has Bevy 0.18 API
incompatibilities (EventReader, OrthographicProjection not a Component, system ordering). This
blocked full project compilation. **Decision**: Temporarily removed camera module from main.rs to
unblock hex_grid development and testing. Camera module needs its own Bevy 0.18 migration. **Note**:
This should be resolved when the camera feature is updated for Bevy 0.18.

### 2026-02-08 — Contracts dead_code warnings

**Context**: Contract types not yet consumed by all features produce dead_code warnings, which fail
`cargo clippy -- -D warnings`. **Decision**: Added `#[allow(dead_code)]` on contract module
declarations in `src/contracts/mod.rs`. Also fixed clippy `derivable_impls` lint on
`TerrainType::Default` in terrain.rs. **Rationale**: Contracts are defined ahead of consumers;
dead_code is expected during incremental development.

### 2026-02-10 — Replaced custom hex mesh with Bevy RegularPolygon

**Context**: Custom hex mesh generation was unnecessary given Bevy's built-in RegularPolygon
primitive. **Decision**: Use `RegularPolygon::new(hex_size * 0.95, 6)` rotated -90 degrees on X to
lie flat on the ground plane. **Rationale**: Simpler, fewer lines of code, and leverages Bevy's
tested mesh generation.

### 2026-02-10 — Added TileBaseMaterial component to contracts

**Context**: With ring overlays replacing material swaps, tiles need to remember their "real"
material so hover/selection systems can restore it after cell type changes. **Decision**: Added
`TileBaseMaterial` component to the hex_grid contract. Tiles store their assigned material handle so
visual systems never lose track of the cell type color. **Impact**: Any system that changes a tile's
cell type must also update `TileBaseMaterial`.

### 2026-02-10 — Added PaintPreview resource to editor_ui contract

**Context**: In Paint mode, the hover indicator should preview the active paint color rather than
showing neutral white. **Decision**: Added `PaintPreview` resource to the editor_ui contract. The
hover system checks this to determine whether to use the paint color or default white for the ring
overlay.

### 2026-02-10 — Replaced material-swapping hover/selection with ring border overlays

**Context**: Material swapping caused tiles to lose their cell type color during hover and
selection, creating confusing visuals especially in Paint mode. **Decision**: Implemented
`build_hex_ring_mesh()` which creates a hollow hexagon (ring border) mesh. Two persistent overlay
entities (`HoverIndicator` and `SelectIndicator`) are spawned at startup and repositioned to the
hovered/selected hex each frame. Tiles always retain their real cell type color. **Rationale**: Ring
overlays provide clear visual feedback without altering tile appearance. This is the standard
approach in hex strategy games. **Details**: `HoverIndicator` uses a semi-transparent white material
(60% opacity). `SelectIndicator` uses an opaque white material. Both are positioned slightly above
the tile (Y offset) to avoid z-fighting.

### 2026-02-10 — Ring mesh vertex alignment

**Context**: The ring overlay vertices must align with Bevy's `RegularPolygon` vertex placement for
the ring to sit correctly on top of tiles. **Decision**: Ring mesh vertices start at angle pi/2 (90
degrees) to match Bevy's RegularPolygon which places the first vertex at the top.

### 2026-02-10 — Click-to-deselect

**Context**: Users needed a way to deselect a tile without selecting a different one. **Decision**:
Clicking an already-selected cell sets `SelectedHex` to `None`. The `handle_click` system compares
the clicked position against the current selection and toggles accordingly.

### 2026-02-10 — Escape-to-deselect

**Context**: Users needed a keyboard shortcut to clear selection entirely. **Decision**: Added
`deselect_on_escape` system that clears `SelectedHex` when Escape is pressed. System is gated behind
`egui_wants_any_keyboard_input` to prevent conflicts when the user is typing in an egui text field.

### 2026-02-10 — Click detection uses just_released with drag threshold

**Context**: Left-click was firing on `just_pressed`, which meant camera drags (which also start
with a left-click press) were incorrectly registering as tile selections. **Decision**:
`handle_click` now fires on `just_released` with a 5px drag threshold. If the mouse moves more than
5 pixels between press and release, the click is treated as a drag and ignored for selection
purposes. **Rationale**: Standard UX pattern for distinguishing click from drag in applications with
both selection and camera manipulation.

### 2026-02-15 — Hex algorithms and LOS (0.7.0)

**Context**: Pitch #78 calls for hexx integration, LOS, and fog of war. After scope hammering, fog
of war was deferred (needs sides/factions) and elevation-based LOS was deferred (future cycle). LOS
terrain blocking uses a placeholder predicate until property system (#81) ships.

**Decision**: Add `algorithms.rs` private module with pure functions wrapping hexx behind
`HexPosition`. LOS visual uses Bevy gizmos (zero allocation, auto-cleared). No entity spawning for
the ray — gizmos are the right tool for transient hover feedback.

**Rationale**: Pure functions are testable without a Bevy app. Closure-based blocking predicates
decouple the algorithm from the data source. Gizmos avoid the complexity of managing overlay
entities for a visual that changes every frame.

**Deferred**: Fog of war (needs factions), elevation LOS (needs per-tile height), LOS terrain
blocking (needs property system #81).

## Test Results

### 2026-02-08 — All 13 tests passing

```
running 13 tests
test hex_grid::tests::hex_position_roundtrip ... ok
test hex_grid::tests::hex_position_to_hex_coordinates ... ok
test hex_grid::tests::hovered_hex_defaults_to_none ... ok
test hex_grid::tests::hex_materials_resource_exists_after_startup ... ok
test hex_grid::tests::grid_config_inserted_after_startup ... ok
test hex_grid::tests::click_fires_selected_event ... ok
test hex_grid::tests::selected_hex_defaults_to_none ... ok
test hex_grid::tests::tile_count_formula ... ok
test hex_grid::tests::no_click_no_event ... ok
test hex_grid::tests::click_sets_selected_hex ... ok
test hex_grid::tests::grid_spawns_correct_number_of_tiles ... ok
test hex_grid::tests::all_tiles_have_hex_position ... ok
test hex_grid::tests::all_tiles_at_y_zero ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- `cargo build` -- passes (0 warnings)
- `cargo clippy -- -D warnings` -- passes (0 warnings)
- `cargo test` -- 13/13 tests pass

### 2026-02-10 — All 71 tests passing (post-0.3.0 polish)

- `cargo build` -- passes
- `cargo clippy -- -D warnings` -- passes (0 warnings)
- `cargo test` -- 71/71 tests pass

### 2026-02-11 — All 90 tests passing (0.4.0 complete)

- `cargo build` -- passes
- `cargo clippy -- -D warnings` -- passes (0 warnings)
- `cargo test` -- 90/90 tests pass (19 hex_grid tests including 4 new overlay tests)

### 2026-02-15 — All 156 tests passing (0.7.0 algorithms + LOS)

- `cargo build` -- passes
- `cargo clippy --all-targets` -- passes (0 warnings)
- `cargo test` -- 156/156 tests pass (33 hex_grid tests including 11 new algorithm/LOS tests)

## Blockers

| Blocker                                           | Waiting On           | Raised     | Resolved                         |
| ------------------------------------------------- | -------------------- | ---------- | -------------------------------- |
| Camera module has Bevy 0.18 API incompatibilities | camera feature owner | 2026-02-08 | Temporarily removed from main.rs |

## Status Updates

| Date       | Status      | Notes                                                                    |
| ---------- | ----------- | ------------------------------------------------------------------------ |
| 2026-02-08 | speccing    | Initial spec created                                                     |
| 2026-02-08 | complete    | Implementation done, all tests passing, clippy clean                     |
| 2026-02-10 | complete    | Post-0.3.0 polish: ring overlays, deselect, mesh fixes                   |
| 2026-02-11 | complete    | 0.4.0: move overlays (sync_move_overlays), OverlayMaterials, 4 new tests |
| 2026-02-15 | in-progress | 0.7.0: hexx algorithms, LOS algorithm + gizmo visual                     |
