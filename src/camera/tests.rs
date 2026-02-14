//! Unit tests for the camera plugin.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

use super::components::{CameraState, TopDownCamera};
use super::systems;

/// Helper: build a minimal app with the camera systems for testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<CameraState>();
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
    use crate::contracts::hex_grid::HexGridConfig;

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
