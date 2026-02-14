# Feature Log: camera

## Status: complete

## Decision Log

### 2026-02-08 — Top-down orthographic camera with 2D constraint

**Context**: 0.1.0 restricts camera to top-down view to keep the initial design cycle in 2D
thinking. The code remains 3D-ready. **Decision**: Orthographic projection, locked perpendicular to
ground plane, pan + zoom only. **Rationale**: Eliminates 3D navigation complexity. Lets the user
focus on hex grid layout and terrain without perspective distortion. Rotation can be unlocked in a
future release when 3D features (elevation, unit models) arrive. **Alternatives rejected**:
Perspective camera with angle lock (still introduces foreshortening, makes hex selection math
harder). Free orbit camera (too much complexity for 0.1.0).

### 2026-02-08 — Bevy 0.18 API adaptations

**Context**: Bevy 0.18 has significant API changes from earlier versions. **Decisions**:

- `EventReader<MouseWheel>` replaced by `Res<AccumulatedMouseScroll>` (resource with `.delta` Vec2
  and `.unit` MouseScrollUnit).
- `EventReader<CursorMoved>` replaced by `Res<AccumulatedMouseMotion>` (resource with `.delta`
  Vec2).
- `OrthographicProjection` is no longer a standalone `Component`. Must be wrapped in
  `Projection::Orthographic(...)` enum and queried as `Query<&Projection>` or
  `Query<&mut Projection>`.
- System ordering with `.after(system_fn)` on bare function items does not work. Use `SystemSet`
  with `.in_set()` and `configure_sets()` instead, or use `.chain()`.
- Camera spawned with tuple:
  `(TopDownCamera, Camera3d::default(), Projection::Orthographic(...), Transform)`.

### 2026-02-08 — Camera state design

**Context**: Need smooth interpolation and consistent feel at all zoom levels. **Decisions**:

- `CameraState` resource holds target position (Vec2 on XZ plane), target/current scale, bounds,
  speeds.
- Pan speed scales with `current_scale` so movement feels consistent at any zoom level.
- Mouse drag uses `AccumulatedMouseMotion.delta` scaled by `current_scale * 0.005` factor.
- Zoom is multiplicative: `target_scale *= (1.0 - scroll_amount * zoom_speed)`.
- Smoothing uses exponential interpolation: `t = (smoothing * dt).clamp(0, 1)`.
- Rotation is forcibly reset every frame in `smooth_camera` to prevent drift.
- Default bounds: 50 world units. Adjusted at startup if `HexGridConfig` resource exists.

### 2026-02-08 — HexLayout API

**Context**: `hexx` 0.22 renamed `hex_size` to `scale` in `HexLayout`. **Decision**: Use
`config.layout.scale` instead of `config.layout.hex_size`.

### 2026-02-10 — Swapped A/D key pan directions

**Context**: Camera orientation via `looking_at(Vec3::ZERO, Vec3::Z)` mirrors the X axis (camera
right = -X world). **Decision**: A key pans +X world (screen left) and D key pans -X world (screen
right). This matches the visual expectation: pressing A moves the view left, pressing D moves the
view right.

### 2026-02-10 — Changed mouse pan from middle-click to right-click

**Context**: Middle-click is unreliable on macOS trackpads (requires three-finger click or external
mouse). **Decision**: Use right-click drag for panning instead of middle-click. Right-click is
accessible on all macOS trackpads via two-finger click or Ctrl+click.

### 2026-02-10 — Added left-click drag panning with 5px threshold

**Context**: Right-click alone is not always the most natural pan gesture. Left-click drag is
intuitive but conflicts with click-to-select on hex tiles. **Decision**: Left-click drag pans the
camera, but only after the mouse moves more than 5 pixels from the press point. Uses a `Local<f32>`
accumulator tracking cumulative `AccumulatedMouseMotion` distance while the left button is held. The
accumulator resets on `just_pressed(MouseButton::Left)`. Below the threshold, the click passes
through to selection systems.

### 2026-02-10 — Changed from just_pressed/just_released to pressed() for drag detection

**Context**: When egui run conditions (`not(egui_wants_any_pointer_input)`) block the camera
mouse_pan system, `just_released` events can be missed, causing the camera to remain stuck in a
dragging state. **Decision**: Use `pressed(MouseButton::Right)` and `pressed(MouseButton::Left)`
instead of tracking press/release pairs. The `is_dragging` flag is recomputed each frame from the
current button state, so it self-corrects even if a release event is missed.

### 2026-02-10 — Set WinitSettings to UpdateMode::Continuous

**Context**: Default Bevy windowing uses reactive update mode that can introduce a noticeable delay
when the window first gains focus (waiting for an input event to trigger the first frame).
**Decision**: Set `WinitSettings` to `UpdateMode::Continuous` for both `focused_mode` and
`unfocused_mode` to eliminate the window focus delay. This ensures smooth camera responsiveness at
all times.

### 2026-02-10 — Added view shortcuts (C, F, 0, -, =)

**Context**: Users need quick ways to re-orient the view without manual panning/zooming.
**Decision**: Keyboard shortcuts in `view_shortcuts` system:

- **C** — center the grid in the viewport (keep current zoom), using panel offset
- **F** — zoom to fit the entire grid (keep current center position)
- **0** — zoom to fit and center (combines F + C)
- **=** — zoom in by 20% (`target_scale *= 0.8`)
- **-** — zoom out by 20% (`target_scale *= 1.2`)

### 2026-02-10 — fit_scale computes from actual boundary hex world positions

**Context**: The old approach estimated grid extent from `map_radius * hex_size * 2.0`, which was
inaccurate for non-square hex layouts. **Decision**: `fit_scale` computes the actual world-space
extent by checking the 6 boundary hexes (`(r,0), (-r,0), (0,r), (0,-r), (r,-r), (-r,r)`), finding
the maximum absolute X and Y positions, adding one hex size for the hex body, and fitting both
dimensions with 5% padding. The scale is `max(scale_x, scale_y) * 1.05`, clamped to min/max.

### 2026-02-10 — panel_center_offset accounts for 260px editor panel

**Context**: The editor panel on the left side of the window means the visible viewport center is
not the window center. When centering the grid, it should appear centered in the _visible_ area
(window minus panel), not the full window. **Decision**: `panel_center_offset(scale)` computes the
world-space X offset as `(PANEL_WIDTH / 2.0) * scale`. The offset is positive X because the camera
mirrors X (camera right = -X world), so shifting the camera in +X world moves the view right,
centering content in the visible area to the right of the panel.

### 2026-02-10 — compensate_resize adjusts scale on window height changes

**Context**: With `ScalingMode::WindowSize`, resizing the window changes how much of the world is
visible because the projection maps 1 pixel to `scale` world units. Making the window taller reveals
more of the grid vertically. **Decision**: `compensate_resize` system tracks the previous window
height via `Local<f32>`. When the height changes by more than 0.5px, it scales both `target_scale`
and `current_scale` by the ratio `prev_height / new_height`, clamped to min/max. This preserves the
same vertical world extent across resizes. First frame just records the initial height without
adjusting.

### 2026-02-10 — Startup view uses fit_scale + panel_center_offset

**Context**: The app should launch showing the entire hex grid, properly centered relative to the
editor panel. **Decision**: `configure_bounds_from_grid` (Startup, chained after `spawn_camera`)
computes `fit_scale` from the window dimensions and `HexGridConfig`, sets both `target_scale` and
`current_scale` to the result (no animation), and sets `target_position` to
`(panel_center_offset(scale), 0.0)`.

## Test Results

### 2026-02-08 — All tests passing

```
cargo test: 10/10 camera tests passed (24 total including hex_grid)
cargo clippy -- -D warnings: clean (0 warnings)
cargo build: success
```

Tests:

- camera_state_defaults_are_reasonable: PASS
- camera_state_target_position_starts_at_origin: PASS
- zoom_clamping_works: PASS
- pan_bounds_clamping_works: PASS
- spawn_camera_creates_entity: PASS
- camera_looks_down_negative_y: PASS
- configure_bounds_uses_defaults_without_grid: PASS
- configure_bounds_adjusts_with_grid_config: PASS
- smooth_camera_interpolates_position: PASS
- smooth_camera_enforces_bounds: PASS
- smooth_camera_enforces_rotation_lock: PASS

### 2026-02-10 — Post-0.3.0 polish, all tests passing

```
cargo test: 11/11 camera tests passed (71 total project-wide)
cargo clippy -- -D warnings: clean (0 warnings)
cargo build: success
```

Tests:

- camera_state_defaults_are_reasonable: PASS
- camera_state_target_position_starts_at_origin: PASS
- zoom_clamping_works: PASS
- pan_bounds_clamping_works: PASS
- spawn_camera_creates_entity: PASS
- camera_looks_down_negative_y: PASS
- configure_bounds_uses_defaults_without_grid: PASS
- configure_bounds_adjusts_with_grid_config: PASS
- smooth_camera_interpolates_position: PASS
- smooth_camera_enforces_bounds: PASS
- smooth_camera_enforces_rotation_lock: PASS

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Status Updates

| Date       | Status   | Notes                                                                                                                                                                |
| ---------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-02-08 | speccing | Initial spec created                                                                                                                                                 |
| 2026-02-08 | complete | Implementation done. All tests pass, clippy clean, build succeeds.                                                                                                   |
| 2026-02-10 | complete | Post-0.3.0 polish: pan rework (right-click + left-click-drag with threshold), view shortcuts (C/F/0/-/=), resize compensation, startup fit+center with panel offset. |
