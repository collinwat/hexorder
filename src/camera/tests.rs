//! Unit tests for the camera plugin.

use bevy::prelude::*;

use hexorder_contracts::editor_ui::ViewportMargins;
use hexorder_contracts::persistence::AppScreen;

use super::components::{CameraState, TopDownCamera};
use super::systems;

/// Helper: build a minimal app with the camera systems for testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<CameraState>();
    app.init_resource::<ViewportMargins>();
    app
}

#[test]
fn camera_state_defaults_are_reasonable() {
    let state = CameraState::default();
    assert!(state.min_scale > 0.0, "min_scale must be positive");
    assert!(
        state.max_scale > state.min_scale,
        "max_scale must exceed min_scale"
    );
    assert!(
        state.target_scale >= state.min_scale && state.target_scale <= state.max_scale,
        "default target_scale must be within min/max range"
    );
    assert!(state.pan_speed > 0.0, "pan_speed must be positive");
    assert!(state.zoom_speed > 0.0, "zoom_speed must be positive");
    assert!(state.pan_bounds > 0.0, "pan_bounds must be positive");
    assert!(state.smoothing > 0.0, "smoothing must be positive");
    assert!(!state.is_dragging, "should not start in dragging state");
}

#[test]
fn camera_state_target_position_starts_at_origin() {
    let state = CameraState::default();
    assert_eq!(state.target_position, Vec2::ZERO);
}

#[test]
fn zoom_clamping_works() {
    let mut state = CameraState::default();

    // Try to zoom in past minimum.
    state.target_scale = 0.0001;
    state.target_scale = state.target_scale.clamp(state.min_scale, state.max_scale);
    assert_eq!(state.target_scale, state.min_scale);

    // Try to zoom out past maximum.
    state.target_scale = 999.0;
    state.target_scale = state.target_scale.clamp(state.min_scale, state.max_scale);
    assert_eq!(state.target_scale, state.max_scale);
}

#[test]
fn pan_bounds_clamping_works() {
    let mut state = CameraState::default();
    let bounds = state.pan_bounds;

    // Move beyond bounds.
    state.target_position = Vec2::new(bounds + 100.0, bounds + 100.0);
    state.target_position.x = state.target_position.x.clamp(-bounds, bounds);
    state.target_position.y = state.target_position.y.clamp(-bounds, bounds);

    assert_eq!(state.target_position.x, bounds);
    assert_eq!(state.target_position.y, bounds);

    // Negative direction.
    state.target_position = Vec2::new(-bounds - 50.0, -bounds - 50.0);
    state.target_position.x = state.target_position.x.clamp(-bounds, bounds);
    state.target_position.y = state.target_position.y.clamp(-bounds, bounds);

    assert_eq!(state.target_position.x, -bounds);
    assert_eq!(state.target_position.y, -bounds);
}

#[test]
fn spawn_camera_creates_entity() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.update();

    let mut query = app
        .world_mut()
        .query::<(&TopDownCamera, &Transform, &Projection)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "exactly one camera should be spawned");

    let (_marker, transform, projection) = results[0];

    // Camera should be above the ground plane.
    assert!(
        transform.translation.y > 0.0,
        "camera Y should be above ground plane"
    );

    // Camera should be at the origin on XZ.
    assert_eq!(transform.translation.x, 0.0, "camera should start at X=0");
    assert_eq!(transform.translation.z, 0.0, "camera should start at Z=0");

    // Projection scale should match default.
    if let Projection::Orthographic(ortho) = projection {
        let default_state = CameraState::default();
        assert!(
            (ortho.scale - default_state.target_scale).abs() < f32::EPSILON,
            "initial projection scale should match default target_scale"
        );
    } else {
        panic!("expected orthographic projection");
    }
}

#[test]
fn camera_looks_down_negative_y() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.update();

    let mut query = app.world_mut().query::<(&TopDownCamera, &Transform)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    let (_marker, transform) = results[0];

    // The camera's forward direction should be -Y (looking straight down).
    let forward = transform.forward();
    assert!(
        forward.y < -0.99,
        "camera forward should be approximately -Y, got {forward:?}"
    );

    // The camera's up direction should be +Z.
    let up = transform.up();
    assert!(
        up.z > 0.99,
        "camera up should be approximately +Z, got {up:?}"
    );
}

#[test]
fn configure_bounds_uses_defaults_without_grid() {
    let mut app = test_app();
    app.add_systems(
        Startup,
        (systems::spawn_camera, systems::configure_bounds_from_grid).chain(),
    );
    app.update();

    let state = app.world().resource::<CameraState>();
    let default_state = CameraState::default();
    // Without HexGridConfig, bounds should remain at default.
    assert_eq!(state.pan_bounds, default_state.pan_bounds);
}

#[test]
fn configure_bounds_adjusts_with_grid_config() {
    use hexorder_contracts::hex_grid::HexGridConfig;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 10,
    });
    app.add_systems(
        Startup,
        (systems::spawn_camera, systems::configure_bounds_from_grid).chain(),
    );
    app.update();

    let state = app.world().resource::<CameraState>();
    let default_state = CameraState::default();
    // With grid config, bounds should differ from default.
    assert_ne!(
        state.pan_bounds, default_state.pan_bounds,
        "bounds should be adjusted when HexGridConfig is present"
    );
}

#[test]
fn smooth_camera_interpolates_position() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::smooth_camera);
    app.update(); // Startup

    // Set a target position.
    {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.target_position = Vec2::new(10.0, 10.0);
    }

    // Run a few update ticks.
    for _ in 0..10 {
        app.update();
    }

    let mut query = app.world_mut().query::<(&TopDownCamera, &Transform)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    let (_marker, transform) = results[0];

    // After several frames, camera should have moved toward the target.
    // (may not have reached it yet due to interpolation)
    assert!(
        transform.translation.x > 0.0,
        "camera X should have moved toward target"
    );
    assert!(
        transform.translation.z > 0.0,
        "camera Z should have moved toward target"
    );
}

#[test]
fn smooth_camera_enforces_bounds() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::smooth_camera);
    app.update(); // Startup

    let bounds = {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.target_position = Vec2::new(9999.0, 9999.0);
        state.pan_bounds
    };

    // Run updates so smooth_camera clamps.
    for _ in 0..5 {
        app.update();
    }

    let state = app.world().resource::<CameraState>();
    assert!(
        state.target_position.x <= bounds,
        "target X should be clamped to bounds"
    );
    assert!(
        state.target_position.y <= bounds,
        "target Y should be clamped to bounds"
    );
}

#[test]
fn smooth_camera_enforces_rotation_lock() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::smooth_camera);
    app.update(); // Startup

    // Manually corrupt the rotation to simulate any drift.
    {
        let mut query = app
            .world_mut()
            .query_filtered::<&mut Transform, With<TopDownCamera>>();
        for mut transform in query.iter_mut(app.world_mut()) {
            transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.5, 0.3, 0.1);
        }
    }

    app.update(); // smooth_camera should reset rotation

    let mut query = app.world_mut().query::<(&TopDownCamera, &Transform)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    let (_marker, transform) = results[0];

    let forward = transform.forward();
    assert!(
        forward.y < -0.99,
        "rotation should be reset to look down -Y, got forward {forward:?}"
    );
}

// ---------------------------------------------------------------------------
// Keyboard pan direction regression tests
// ---------------------------------------------------------------------------

use bevy::input::keyboard::KeyCode;
use hexorder_contracts::shortcuts::ShortcutRegistry;

/// Build a minimal app with the `keyboard_pan` system and shortcut bindings.
fn pan_test_app() -> App {
    let mut app = test_app();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ShortcutRegistry>();
    super::register_shortcuts(&mut app.world_mut().resource_mut::<ShortcutRegistry>());
    app.add_systems(Update, systems::keyboard_pan);
    // Run one tick so `Time` has a non-zero delta for subsequent frames.
    app.update();
    app
}

/// Press a key, run one update, and return the resulting `target_position`.
fn pan_with_key(app: &mut App, key: KeyCode) -> Vec2 {
    // Reset camera to origin.
    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::ZERO;

    // Press the key.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);

    app.update();

    // Release so it doesn't carry over.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);

    app.world().resource::<CameraState>().target_position
}

#[test]
fn keyboard_pan_up_increases_y() {
    let mut app = pan_test_app();
    let pos = pan_with_key(&mut app, KeyCode::ArrowUp);
    assert!(
        pos.y > 0.0,
        "Pan up should increase target_position.y (move toward +Z), got {pos:?}"
    );
    assert!(
        pos.x.abs() < f32::EPSILON,
        "Pan up should not affect X, got {pos:?}"
    );
}

#[test]
fn keyboard_pan_down_decreases_y() {
    let mut app = pan_test_app();
    let pos = pan_with_key(&mut app, KeyCode::ArrowDown);
    assert!(
        pos.y < 0.0,
        "Pan down should decrease target_position.y (move toward -Z), got {pos:?}"
    );
}

#[test]
fn keyboard_pan_left_increases_x() {
    let mut app = pan_test_app();
    let pos = pan_with_key(&mut app, KeyCode::ArrowLeft);
    assert!(
        pos.x > 0.0,
        "Pan left should increase target_position.x (move toward +X), got {pos:?}"
    );
    assert!(
        pos.y.abs() < f32::EPSILON,
        "Pan left should not affect Y, got {pos:?}"
    );
}

#[test]
fn keyboard_pan_right_decreases_x() {
    let mut app = pan_test_app();
    let pos = pan_with_key(&mut app, KeyCode::ArrowRight);
    assert!(
        pos.x < 0.0,
        "Pan right should decrease target_position.x (move toward -X), got {pos:?}"
    );
}

#[test]
fn keyboard_pan_wasd_matches_arrow_directions() {
    let mut app = pan_test_app();

    let w = pan_with_key(&mut app, KeyCode::KeyW);
    assert!(w.y > 0.0, "W should pan up (increase Y), got {w:?}");

    let s = pan_with_key(&mut app, KeyCode::KeyS);
    assert!(s.y < 0.0, "S should pan down (decrease Y), got {s:?}");

    let a = pan_with_key(&mut app, KeyCode::KeyA);
    assert!(a.x > 0.0, "A should pan left (increase X), got {a:?}");

    let d = pan_with_key(&mut app, KeyCode::KeyD);
    assert!(d.x < 0.0, "D should pan right (decrease X), got {d:?}");
}

/// The `handle_camera_command` observer must not panic when `SelectedHex`
/// does not exist (e.g., zoom command dispatched before entering the Editor
/// state). The observer wraps `SelectedHex` in `Option`.
#[test]
fn camera_command_without_selected_hex_resource_does_not_panic() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CameraState>();
    app.init_resource::<ViewportMargins>();
    // Do NOT insert SelectedHex — simulates Launcher state.
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Fire a camera command (zoom in).
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.zoom_in"),
    });

    app.update(); // Must not panic
}
