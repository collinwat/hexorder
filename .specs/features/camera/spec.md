# Feature: camera

## Summary
Provides a top-down orthographic camera locked perpendicular to the ground plane. Supports pan (right-click drag, left-click drag, WASD/arrow keys), zoom (scroll wheel, keyboard shortcuts), and view navigation shortcuts. No rotation. Enables 2D-style interaction with the 3D hex world.

## Plugin
- Module: `src/camera/`
- Plugin struct: `CameraPlugin`
- Schedule: `Startup` (camera spawn, initial fit+center), `Update` (input handling, resize compensation)

## Dependencies
- **Contracts consumed**: `hex_grid` (reads `HexGridConfig` for bounds and fit calculations)
- **Contracts produced**: none (camera is self-contained; other features use Bevy's built-in camera queries if needed)
- **Crate dependencies**: `bevy`, `bevy_egui` (for input gating run conditions), `hexx` (for boundary hex world positions)

## Requirements
1. [REQ-ORTHO] Spawn an orthographic camera looking straight down the Y axis at the XZ ground plane. The camera's forward direction is -Y, up direction is +Z (so "north" on screen is +Z in world space).
2. [REQ-PAN] The user can pan the camera across the ground plane using right-click drag, left-click drag (with 5px threshold to avoid interfering with click-to-select), or WASD/arrow keys. Middle-click is NOT used for panning (macOS trackpad compatibility).
3. [REQ-ZOOM] The user can zoom in and out using the scroll wheel. Zoom adjusts the orthographic projection scale (not camera Y position).
4. [REQ-BOUNDS] Camera panning is clamped so the view cannot scroll entirely off the hex grid. Some margin is acceptable.
5. [REQ-LOCK] Camera rotation is disabled. The view is always top-down, perpendicular to the ground plane. No pitch, yaw, or roll.
6. [REQ-SMOOTH] Pan and zoom transitions are smooth (interpolated), not instant jumps.
7. [REQ-SHORTCUTS] View shortcuts: C=center grid in viewport (keep zoom), F=zoom-to-fit (keep center), 0=zoom-to-fit and center, -=zoom out (20% step), ==zoom in (20% step).
8. [REQ-STARTUP] Application starts zoomed-to-fit and centered on the hex grid, accounting for the editor panel width offset. No animation on startup (target and current scale match immediately).
9. [REQ-RESIZE] Window resize maintains the same visual view. Scale compensates proportionally for window height changes so the same vertical world extent remains visible.
10. [REQ-PANEL-OFFSET] Grid centering (C, 0, and startup) accounts for the 260px editor panel on the left side. The visible center is offset by half the panel width, converted to world units via the current scale.

## Success Criteria
- [x] [SC-1] Camera renders the hex grid from directly above in orthographic projection
- [x] [SC-2] Right-click drag pans the view across the grid
- [x] [SC-3] Left-click drag pans the view (with 5px threshold so clicks still work for selection)
- [x] [SC-4] Arrow keys / WASD pan the view
- [x] [SC-5] Scroll wheel zooms in and out smoothly
- [x] [SC-6] Camera cannot be rotated by any input
- [x] [SC-7] Camera panning stays within reasonable bounds of the grid
- [x] [SC-8] View shortcuts (C, F, 0, -, =) work correctly
- [x] [SC-9] App starts zoomed-to-fit and centered with panel offset
- [x] [SC-10] Window resize preserves the same visual view
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes

## Decomposition
Solo feature — no parallel decomposition needed.

## Constraints
- Must be orthographic, not perspective
- Camera transform Y position is fixed (only projection scale and XZ translation change)
- Left-click drag for pan uses a 5px threshold (`Local<f32>` accumulator with `AccumulatedMouseMotion`) so that normal clicks still register for hex tile selection
- Camera X axis is mirrored (camera right = -X world) due to `looking_at(Vec3::ZERO, Vec3::Z)` orientation. A/D keys and panel offset calculations account for this inversion.
- Must not conflict with hex tile click/selection input (right-click for pan drag, left-click for selection with drag threshold)
- Uses `pressed()` (not `just_pressed`/`just_released`) for drag detection to avoid sticking when egui run conditions block the release event

## Open Questions (Resolved)
- Default zoom level: scale=0.03 (using `ScalingMode::WindowSize(1.0)`, so scale S means 1 pixel covers S world units). Adjusted dynamically at startup via `fit_scale` if `HexGridConfig` is present.
- Zoom min/max limits: yes, min_scale=0.005, max_scale=0.15
- Pan speed scales with zoom level: yes, speed = pan_speed * current_scale * delta_time
- Pan input: right-click drag + left-click drag with 5px threshold (not middle-click, for macOS trackpad compatibility)
- Panel offset: 260px editor panel width, offset = (PANEL_WIDTH / 2) * scale, positive X due to camera X mirror
