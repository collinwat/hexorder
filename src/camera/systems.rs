//! Camera systems for spawning, input handling, and smooth interpolation.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{CameraState, TopDownCamera};
use crate::contracts::editor_ui::ViewportMargins;
use crate::contracts::hex_grid::{HexGridConfig, SelectedHex};

/// Fixed camera height above the ground plane.
const CAMERA_Y: f32 = 100.0;

/// Startup system: spawns the orthographic top-down camera and scene lighting.
pub fn spawn_camera(mut commands: Commands) {
    let default_state = CameraState::default();
    commands.spawn((
        TopDownCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: default_state.target_scale,
            near: 0.1,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, CAMERA_Y, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Directional light pointing downward — illuminates the hex grid and unit
    // tokens uniformly. Angled slightly off-vertical to give subtle depth cues
    // on 3D geometry (unit cylinders).
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::X),
    ));
}

/// Startup system: adjusts camera state bounds based on `HexGridConfig` if available.
/// Sets bounds and an initial fit scale estimate. The actual centering is deferred
/// to `apply_pending_reset` which waits for `ViewportMargins` to be populated.
pub fn configure_bounds_from_grid(
    grid_config: Option<Res<HexGridConfig>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
    mut cameras: Query<&mut Camera, With<TopDownCamera>>,
) {
    // Always request a deferred reset so centering happens once margins are known,
    // even if grid config isn't available yet (apply_pending_reset handles that).
    // Deactivate the camera while the reset is pending to prevent rendering
    // at the wrong position (visible as a flash).
    camera_state.pending_reset = true;
    if let Ok(mut cam) = cameras.single_mut() {
        cam.is_active = false;
    }

    let Some(config) = grid_config else {
        return;
    };

    // Set pan bounds.
    let hex_scale = config.layout.scale.x.max(config.layout.scale.y);
    let grid_extent = config.map_radius as f32 * hex_scale * 2.0;
    camera_state.pan_bounds = grid_extent + hex_scale * 4.0;

    // Compute fit scale using actual window dimensions if available,
    // otherwise estimate from a reasonable default viewport size.
    if let Ok(window) = windows.single() {
        camera_state.target_scale = fit_scale(&config, window, &camera_state);
    } else {
        let estimated_scale = grid_extent / 800.0;
        camera_state.target_scale =
            estimated_scale.clamp(camera_state.min_scale, camera_state.max_scale);
    }

    // Start at the target scale immediately (no animation on startup).
    camera_state.current_scale = camera_state.target_scale;

    // Defer centering until the first frame where ViewportMargins are populated
    // (egui panels must render once before margins are known).
    camera_state.pending_reset = true;
}

/// Applies a deferred reset-view (fit + center) once `ViewportMargins` are
/// populated. Runs every frame in `Update`; does nothing when there is no
/// pending reset or when the editor side panel hasn't rendered yet.
/// Snaps both the target and actual camera transform to avoid a visible flash.
pub fn apply_pending_reset(
    grid_config: Option<Res<HexGridConfig>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
    margins: Res<ViewportMargins>,
    mut camera_q: Query<(&mut Transform, &mut Projection, &mut Camera), With<TopDownCamera>>,
) {
    if !camera_state.pending_reset {
        return;
    }
    // Wait until the editor side panel has rendered at least once.
    // margins.left is always > 0 when the side panel is visible.
    if margins.left == 0.0 {
        return;
    }

    // Recompute fit scale with actual window.
    if let (Ok(window), Some(config)) = (windows.single(), &grid_config) {
        camera_state.target_scale = fit_scale(config, window, &camera_state);
        camera_state.current_scale = camera_state.target_scale;
    }

    // Center accounting for UI margins.
    let scale = camera_state.target_scale;
    camera_state.target_position = ui_center_offset(scale, &margins);

    // Snap the camera transform directly and reactivate rendering.
    if let Ok((mut transform, mut projection, mut cam)) = camera_q.single_mut() {
        transform.translation.x = camera_state.target_position.x;
        transform.translation.z = camera_state.target_position.y;
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = camera_state.current_scale;
        }
        cam.is_active = true;
    }

    camera_state.pending_reset = false;
}

/// Update system: handles keyboard panning via registry-bound keys.
///
/// Reads bound keys from `ShortcutRegistry` instead of hardcoded `KeyCode`s.
/// Pan direction commands are continuous (held every frame).
pub fn keyboard_pan(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    registry: Res<crate::contracts::shortcuts::ShortcutRegistry>,
    mut camera_state: ResMut<CameraState>,
) {
    // Ignore WASD when a command modifier is held (e.g. Cmd+S for save).
    if keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]) {
        return;
    }

    let mut direction = Vec2::ZERO;

    // Read bound keys from registry for each pan direction.
    // The camera looks down -Y with up=+Z, so:
    //   screen up = +Z world, screen down = -Z world
    //   screen left = -X world, screen right = +X world
    if registry
        .bindings_for("camera.pan_up")
        .iter()
        .any(|k| keys.pressed(*k))
    {
        direction.y += 1.0; // +Z
    }
    if registry
        .bindings_for("camera.pan_down")
        .iter()
        .any(|k| keys.pressed(*k))
    {
        direction.y -= 1.0; // -Z
    }
    if registry
        .bindings_for("camera.pan_left")
        .iter()
        .any(|k| keys.pressed(*k))
    {
        direction.x += 1.0; // screen left = +X world (camera mirrors X)
    }
    if registry
        .bindings_for("camera.pan_right")
        .iter()
        .any(|k| keys.pressed(*k))
    {
        direction.x -= 1.0; // screen right = -X world (camera mirrors X)
    }

    if direction != Vec2::ZERO {
        let normalized = direction.normalize();
        // Scale pan speed with zoom level so panning feels consistent.
        let speed = camera_state.pan_speed * camera_state.current_scale * time.delta_secs();
        camera_state.target_position += normalized * speed;
    }
}

/// Update system: handles right-click drag panning.
///
/// Uses `AccumulatedMouseMotion` for the mouse delta and `ButtonInput<MouseButton>`
/// to detect right-click. The drag delta is converted from screen-space to
/// world-space using the current orthographic projection scale.
/// Pixel distance the mouse must move before a left-click becomes a drag.
const DRAG_THRESHOLD: f32 = 5.0;

pub fn mouse_pan(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_state: ResMut<CameraState>,
    mut left_drag_acc: Local<f32>,
) {
    // Reset accumulator on fresh left press.
    if mouse_buttons.just_pressed(MouseButton::Left) {
        *left_drag_acc = 0.0;
    }

    let delta = mouse_motion.delta;

    // Accumulate mouse movement while left button is held.
    if mouse_buttons.pressed(MouseButton::Left) {
        *left_drag_acc += delta.length();
    }

    let right_held = mouse_buttons.pressed(MouseButton::Right);
    let left_dragging = mouse_buttons.pressed(MouseButton::Left) && *left_drag_acc > DRAG_THRESHOLD;

    camera_state.is_dragging = right_held || left_dragging;

    if !camera_state.is_dragging || delta == Vec2::ZERO {
        return;
    }

    // Convert screen-space delta to world-space delta ("grab and drag").
    // X is flipped because the camera orientation mirrors the X axis
    // (camera right = -X world). Y maps directly (drag up = -delta.y = -Z).
    let world_delta = Vec2::new(delta.x, delta.y) * camera_state.current_scale;
    camera_state.target_position += world_delta;
}

/// Update system: handles scroll wheel zoom.
///
/// Uses `AccumulatedMouseScroll` for the scroll delta. Multiplicative zoom
/// feels more natural than additive. The target scale is clamped to min/max.
pub fn scroll_zoom(scroll: Res<AccumulatedMouseScroll>, mut camera_state: ResMut<CameraState>) {
    let scroll_amount = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y * 0.01,
    };

    if scroll_amount == 0.0 {
        return;
    }

    // Zoom in = smaller scale (closer), zoom out = larger scale (further).
    let zoom_factor = 1.0 - scroll_amount * camera_state.zoom_speed;
    camera_state.target_scale *= zoom_factor;
    camera_state.target_scale = camera_state
        .target_scale
        .clamp(camera_state.min_scale, camera_state.max_scale);
}

/// Computes the orthographic scale needed to fit the entire hex grid in the viewport.
fn fit_scale(grid_config: &HexGridConfig, window: &Window, camera_state: &CameraState) -> f32 {
    let layout = &grid_config.layout;
    let r = grid_config.map_radius as i32;
    let hex_size = layout.scale.x.max(layout.scale.y);

    // Compute actual world-space extent by checking the 6 boundary hexes.
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;
    let extremes = [
        hexx::Hex::new(r, 0),
        hexx::Hex::new(-r, 0),
        hexx::Hex::new(0, r),
        hexx::Hex::new(0, -r),
        hexx::Hex::new(r, -r),
        hexx::Hex::new(-r, r),
    ];
    for hex in &extremes {
        let pos = layout.hex_to_world_pos(*hex);
        max_x = max_x.max(pos.x.abs());
        max_y = max_y.max(pos.y.abs());
    }

    // Add one hex size for the hex body extending beyond its center point.
    let extent_x = (max_x + hex_size) * 2.0;
    let extent_y = (max_y + hex_size) * 2.0;

    // Fit both dimensions with 5% padding.
    let scale_x = extent_x / window.width();
    let scale_y = extent_y / window.height();
    let scale = scale_x.max(scale_y) * 1.05;
    scale.clamp(camera_state.min_scale, camera_state.max_scale)
}

/// Computes the camera offset needed to visually center content in the
/// viewport area not covered by editor UI panels.
///
/// Uses actual panel dimensions from `ViewportMargins` (written by editor_ui
/// each frame) instead of hardcoded constants.
///
/// The left panel shifts the visible center right; a right panel shifts it
/// left. The net horizontal offset is `(left - right) / 2`. The camera
/// mirrors X (camera right = -X world), so the X offset is positive when
/// the left panel is wider.
///
/// The menu bar is at the top, so the visible center is shifted down by
/// half the menu bar height.
fn ui_center_offset(scale: f32, margins: &ViewportMargins) -> Vec2 {
    let x = ((margins.left - margins.right) / 2.0) * scale;
    let z = (margins.top / 2.0) * scale;
    Vec2::new(x, z)
}

/// Observer: handles discrete camera commands dispatched via the shortcut registry.
pub fn handle_camera_command(
    trigger: On<crate::contracts::shortcuts::CommandExecutedEvent>,
    grid_config: Option<Res<HexGridConfig>>,
    selected_hex: Res<SelectedHex>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
    margins: Res<ViewportMargins>,
) {
    let zoom_step = 0.2; // 20% per press
    match trigger.event().command_id.0 {
        "camera.zoom_in" => {
            camera_state.target_scale *= 1.0 - zoom_step;
            camera_state.target_scale = camera_state
                .target_scale
                .clamp(camera_state.min_scale, camera_state.max_scale);
        }
        "camera.zoom_out" => {
            camera_state.target_scale *= 1.0 + zoom_step;
            camera_state.target_scale = camera_state
                .target_scale
                .clamp(camera_state.min_scale, camera_state.max_scale);
        }
        "camera.center" => {
            let scale = camera_state.target_scale;
            camera_state.target_position = ui_center_offset(scale, &margins);
        }
        "camera.fit" => {
            if let (Ok(window), Some(config)) = (windows.single(), &grid_config) {
                camera_state.target_scale = fit_scale(config, window, &camera_state);
            }
        }
        "camera.reset_view" => {
            if let (Ok(window), Some(config)) = (windows.single(), &grid_config) {
                camera_state.target_scale = fit_scale(config, window, &camera_state);
            }
            let scale = camera_state.target_scale;
            camera_state.target_position = ui_center_offset(scale, &margins);
        }
        "view.zoom_to_selection" => {
            if let (Some(pos), Some(config)) = (selected_hex.position, &grid_config) {
                let world = config.layout.hex_to_world_pos(pos.to_hex());
                let offset = ui_center_offset(camera_state.target_scale, &margins);
                camera_state.target_position = Vec2::new(world.x + offset.x, world.y + offset.y);
            }
        }
        _ => {}
    }
}

/// Update system: adjusts scale when the window is resized so the same
/// vertical world extent stays visible. Without this, resizing reveals
/// more or less of the grid because the orthographic projection uses
/// `ScalingMode::WindowSize`.
pub fn compensate_resize(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
    mut prev_height: Local<f32>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let height = window.height();

    if *prev_height <= 0.0 {
        // First frame — just record the initial height.
        *prev_height = height;
        return;
    }

    if (height - *prev_height).abs() < 0.5 {
        return;
    }

    let ratio = *prev_height / height;
    camera_state.target_scale *= ratio;
    camera_state.current_scale *= ratio;
    camera_state.target_scale = camera_state
        .target_scale
        .clamp(camera_state.min_scale, camera_state.max_scale);
    camera_state.current_scale = camera_state
        .current_scale
        .clamp(camera_state.min_scale, camera_state.max_scale);

    *prev_height = height;
}

/// Update system: clamps target position within bounds and smoothly
/// interpolates the actual camera transform and projection toward targets.
///
/// Also enforces the rotation lock so the camera always looks straight down -Y.
pub fn smooth_camera(
    time: Res<Time>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<(&mut Transform, &mut Projection), With<TopDownCamera>>,
) {
    // Clamp target position within bounds.
    let bounds = camera_state.pan_bounds;
    camera_state.target_position.x = camera_state.target_position.x.clamp(-bounds, bounds);
    camera_state.target_position.y = camera_state.target_position.y.clamp(-bounds, bounds);

    let Ok((mut transform, mut projection)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let t = (camera_state.smoothing * dt).clamp(0.0, 1.0);

    // Smoothly interpolate position (XZ plane).
    let current_x = transform.translation.x;
    let current_z = transform.translation.z;
    let target_x = camera_state.target_position.x;
    let target_z = camera_state.target_position.y; // Vec2.y maps to world Z

    transform.translation.x = current_x + (target_x - current_x) * t;
    transform.translation.z = current_z + (target_z - current_z) * t;
    // Y is fixed.
    transform.translation.y = CAMERA_Y;

    // Enforce the top-down orientation: looking down -Y, up = +Z.
    // This prevents any rotation drift (REQ-LOCK).
    transform.rotation = Transform::from_xyz(0.0, CAMERA_Y, 0.0)
        .looking_at(Vec3::ZERO, Vec3::Z)
        .rotation;

    // Smoothly interpolate projection scale.
    let current_scale = camera_state.current_scale;
    let target_scale = camera_state.target_scale;
    let new_scale = current_scale + (target_scale - current_scale) * t;
    camera_state.current_scale = new_scale;

    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = new_scale;
    }
}
