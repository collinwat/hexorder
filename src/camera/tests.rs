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

// ---------------------------------------------------------------------------
// Plugin registration tests (camera/mod.rs coverage)
// ---------------------------------------------------------------------------

#[test]
fn camera_plugin_registers_shortcuts() {
    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    // Verify all 9 camera commands are registered.
    let commands = registry.commands();
    let camera_cmds: Vec<_> = commands
        .iter()
        .filter(|c| c.id.0.starts_with("camera."))
        .collect();
    assert_eq!(camera_cmds.len(), 9, "expected 9 camera shortcut commands");

    // Verify specific command IDs.
    let ids: Vec<&str> = camera_cmds.iter().map(|c| c.id.0).collect();
    assert!(ids.contains(&"camera.pan_up"));
    assert!(ids.contains(&"camera.pan_down"));
    assert!(ids.contains(&"camera.pan_left"));
    assert!(ids.contains(&"camera.pan_right"));
    assert!(ids.contains(&"camera.zoom_in"));
    assert!(ids.contains(&"camera.zoom_out"));
    assert!(ids.contains(&"camera.center"));
    assert!(ids.contains(&"camera.fit"));
    assert!(ids.contains(&"camera.reset_view"));
}

#[test]
fn camera_plugin_continuous_commands_are_marked() {
    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    // Pan commands should be continuous.
    for cmd in registry.commands() {
        if cmd.id.0.starts_with("camera.pan_") {
            assert!(cmd.continuous, "{} should be continuous", cmd.id.0);
        }
    }

    // View commands should be discrete.
    for id in &[
        "camera.zoom_in",
        "camera.zoom_out",
        "camera.center",
        "camera.fit",
        "camera.reset_view",
    ] {
        let cmd = registry
            .commands()
            .iter()
            .find(|c| c.id.0 == *id)
            .expect(id);
        assert!(!cmd.continuous, "{id} should be discrete");
    }
}

#[test]
fn camera_plugin_pan_bindings_correct() {
    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    // Pan up should be W and ArrowUp.
    let pan_up_keys = registry.bindings_for("camera.pan_up");
    assert_eq!(pan_up_keys.len(), 2);
    assert!(pan_up_keys.contains(&KeyCode::KeyW));
    assert!(pan_up_keys.contains(&KeyCode::ArrowUp));

    // Pan down should be S and ArrowDown.
    let pan_down_keys = registry.bindings_for("camera.pan_down");
    assert_eq!(pan_down_keys.len(), 2);
    assert!(pan_down_keys.contains(&KeyCode::KeyS));
    assert!(pan_down_keys.contains(&KeyCode::ArrowDown));
}

// ---------------------------------------------------------------------------
// Camera command observer tests
// ---------------------------------------------------------------------------

#[test]
fn camera_zoom_in_command_decreases_scale() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.zoom_in"),
    });
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        new_scale < initial_scale,
        "zoom_in should decrease scale (zoom in = smaller ortho scale)"
    );
}

#[test]
fn camera_zoom_out_command_increases_scale() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.zoom_out"),
    });
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        new_scale > initial_scale,
        "zoom_out should increase scale (zoom out = larger ortho scale)"
    );
}

#[test]
fn camera_center_command_resets_position() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Move camera away from center.
    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::new(10.0, 10.0);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.center"),
    });
    app.update();

    let pos = app.world().resource::<CameraState>().target_position;
    assert!(
        pos.x.abs() < f32::EPSILON && pos.y.abs() < f32::EPSILON,
        "center should reset position to origin, got {pos:?}"
    );
}

#[test]
fn camera_reset_view_command_resets_position() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Move camera away.
    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::new(5.0, 5.0);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.reset_view"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    // Without a Window + HexGridConfig, fit_scale is skipped,
    // but position should still be reset to the UI center offset (≈ 0).
    assert!(
        state.target_position.length() < 1.0,
        "reset should approximately center camera, got {:?}",
        state.target_position
    );
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

// ---------------------------------------------------------------------------
// Mouse pan tests
// ---------------------------------------------------------------------------

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};

fn mouse_pan_app() -> App {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<AccumulatedMouseMotion>();
    app.add_systems(Update, systems::mouse_pan);
    app.update(); // Startup tick
    app
}

#[test]
fn mouse_pan_right_click_drag_moves_camera() {
    let mut app = mouse_pan_app();

    // Hold right mouse button and move.
    {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(10.0, 5.0);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(state.is_dragging, "should be in dragging state");
    // Delta is multiplied by current_scale; default scale is 0.03.
    let expected_x = 10.0 * 0.03;
    let expected_y = 5.0 * 0.03;
    assert!(
        (state.target_position.x - expected_x).abs() < 0.001,
        "X position should reflect drag delta, got {}",
        state.target_position.x
    );
    assert!(
        (state.target_position.y - expected_y).abs() < 0.001,
        "Y position should reflect drag delta, got {}",
        state.target_position.y
    );
}

#[test]
fn mouse_pan_no_movement_when_no_buttons() {
    let mut app = mouse_pan_app();

    // Move mouse but don't press any button.
    {
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(20.0, 20.0);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(!state.is_dragging);
    assert_eq!(state.target_position, Vec2::ZERO);
}

#[test]
fn mouse_pan_left_click_below_threshold_does_not_drag() {
    let mut app = mouse_pan_app();

    // Press left button with small movement (below DRAG_THRESHOLD of 5.0).
    {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(1.0, 0.0);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(!state.is_dragging, "should not drag below threshold");
    assert_eq!(state.target_position, Vec2::ZERO);
}

#[test]
fn mouse_pan_left_click_above_threshold_drags() {
    let mut app = mouse_pan_app();

    // Press left button.
    {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        // First frame: accumulate past threshold (need > 5.0 total).
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(6.0, 0.0);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        state.is_dragging,
        "should be dragging after exceeding threshold"
    );
    // Should have moved by the delta.
    assert!(
        state.target_position.x.abs() > 0.0,
        "position should have changed"
    );
}

#[test]
fn mouse_pan_zero_delta_noop_when_dragging() {
    let mut app = mouse_pan_app();

    // Press right button but zero motion.
    {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    // is_dragging is set based on button state, but position unchanged.
    assert_eq!(state.target_position, Vec2::ZERO);
}

#[test]
fn mouse_pan_left_drag_accumulator_resets_on_fresh_press() {
    let mut app = mouse_pan_app();

    // First left press with large movement to exceed threshold.
    {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(10.0, 0.0);
    }
    app.update();

    // Release.
    {
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.release(MouseButton::Left);
    }
    app.update();

    // Reset camera position.
    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::ZERO;

    // Fresh press — accumulator should reset, small movement under threshold.
    {
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(1.0, 0.0);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        !state.is_dragging,
        "fresh press should reset accumulator, so small movement should not drag"
    );
}

// ---------------------------------------------------------------------------
// Scroll zoom tests
// ---------------------------------------------------------------------------

fn scroll_zoom_app() -> App {
    let mut app = test_app();
    app.init_resource::<AccumulatedMouseScroll>();
    app.add_systems(Update, systems::scroll_zoom);
    app.update();
    app
}

#[test]
fn scroll_zoom_in_decreases_scale() {
    let mut app = scroll_zoom_app();
    let initial_scale = app.world().resource::<CameraState>().target_scale;

    // Scroll up = zoom in (positive delta.y).
    {
        let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
        scroll.delta = Vec2::new(0.0, 1.0);
        scroll.unit = MouseScrollUnit::Line;
    }
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        new_scale < initial_scale,
        "scroll up should zoom in (decrease scale), got {new_scale} vs {initial_scale}"
    );
}

#[test]
fn scroll_zoom_out_increases_scale() {
    let mut app = scroll_zoom_app();
    let initial_scale = app.world().resource::<CameraState>().target_scale;

    // Scroll down = zoom out (negative delta.y).
    {
        let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
        scroll.delta = Vec2::new(0.0, -1.0);
        scroll.unit = MouseScrollUnit::Line;
    }
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        new_scale > initial_scale,
        "scroll down should zoom out (increase scale), got {new_scale} vs {initial_scale}"
    );
}

#[test]
fn scroll_zoom_pixel_unit_scaled() {
    let mut app = scroll_zoom_app();
    let initial_scale = app.world().resource::<CameraState>().target_scale;

    // Pixel scroll unit is multiplied by 0.01.
    {
        let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
        scroll.delta = Vec2::new(0.0, 100.0); // = 1.0 after * 0.01
        scroll.unit = MouseScrollUnit::Pixel;
    }
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        new_scale < initial_scale,
        "pixel scroll should zoom in, got {new_scale} vs {initial_scale}"
    );
}

#[test]
fn scroll_zoom_zero_delta_noop() {
    let mut app = scroll_zoom_app();
    let initial_scale = app.world().resource::<CameraState>().target_scale;

    {
        let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
        scroll.delta = Vec2::ZERO;
        scroll.unit = MouseScrollUnit::Line;
    }
    app.update();

    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        (new_scale - initial_scale).abs() < f32::EPSILON,
        "zero scroll should not change scale"
    );
}

#[test]
fn scroll_zoom_clamps_to_min() {
    let mut app = scroll_zoom_app();

    // Zoom in excessively.
    for _ in 0..100 {
        {
            let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
            scroll.delta = Vec2::new(0.0, 5.0);
            scroll.unit = MouseScrollUnit::Line;
        }
        app.update();
    }

    let state = app.world().resource::<CameraState>();
    assert!(
        (state.target_scale - state.min_scale).abs() < f32::EPSILON,
        "should clamp at min_scale"
    );
}

#[test]
fn scroll_zoom_clamps_to_max() {
    let mut app = scroll_zoom_app();

    // Zoom out excessively.
    for _ in 0..100 {
        {
            let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
            scroll.delta = Vec2::new(0.0, -5.0);
            scroll.unit = MouseScrollUnit::Line;
        }
        app.update();
    }

    let state = app.world().resource::<CameraState>();
    assert!(
        (state.target_scale - state.max_scale).abs() < f32::EPSILON,
        "should clamp at max_scale"
    );
}

// ---------------------------------------------------------------------------
// Compensate resize tests
// ---------------------------------------------------------------------------

#[test]
fn compensate_resize_noop_without_window() {
    let mut app = test_app();
    app.add_systems(Update, systems::compensate_resize);
    app.update();

    // Without a window, scale should remain at default.
    let state = app.world().resource::<CameraState>();
    let default_state = CameraState::default();
    assert!(
        (state.target_scale - default_state.target_scale).abs() < f32::EPSILON,
        "no window means no resize compensation"
    );
}

// ---------------------------------------------------------------------------
// Apply pending reset tests
// ---------------------------------------------------------------------------

#[test]
fn apply_pending_reset_noop_when_not_pending() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::apply_pending_reset);
    app.update();

    // pending_reset is false by default.
    let state = app.world().resource::<CameraState>();
    assert!(!state.pending_reset);
}

#[test]
fn apply_pending_reset_waits_for_margins() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::apply_pending_reset);
    app.update();

    // Set pending reset but leave margins at zero.
    app.world_mut().resource_mut::<CameraState>().pending_reset = true;
    app.update();

    // Should still be pending because margins.left == 0.
    assert!(
        app.world().resource::<CameraState>().pending_reset,
        "should remain pending when margins are zero"
    );
}

#[test]
fn apply_pending_reset_fires_when_margins_set() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::apply_pending_reset);
    app.update();

    // Set pending reset and non-zero margins.
    {
        app.world_mut().resource_mut::<CameraState>().pending_reset = true;
        let mut margins = app.world_mut().resource_mut::<ViewportMargins>();
        margins.left = 300.0;
        margins.right = 0.0;
        margins.top = 30.0;
        margins.bottom = 0.0;
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        !state.pending_reset,
        "pending_reset should be cleared after margins are set"
    );
}

#[test]
fn apply_pending_reset_recomputes_scale_with_grid_config() {
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
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::apply_pending_reset);
    app.update();

    // Set pending reset and non-zero margins.
    {
        app.world_mut().resource_mut::<CameraState>().pending_reset = true;
        app.world_mut()
            .resource_mut::<CameraState>()
            .target_position = Vec2::new(50.0, 50.0);
        let mut margins = app.world_mut().resource_mut::<ViewportMargins>();
        margins.left = 300.0;
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(!state.pending_reset, "should clear pending reset");
    // Position should be set to ui_center_offset, not (50, 50).
    assert!(
        state.target_position.length() < 50.0,
        "position should be reset toward center, got {:?}",
        state.target_position
    );
}

// ---------------------------------------------------------------------------
// Additional camera command observer tests
// ---------------------------------------------------------------------------

#[test]
fn camera_fit_command_with_grid_config() {
    use hexorder_contracts::hex_grid::HexGridConfig;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Without a window, fit_scale won't fire, but the branch is covered.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.fit"),
    });
    app.update(); // Should not panic
}

#[test]
fn camera_fit_command_without_grid_config_noop() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.fit"),
    });
    app.update();

    // Without HexGridConfig, fit should be a no-op.
    let new_scale = app.world().resource::<CameraState>().target_scale;
    assert!(
        (new_scale - initial_scale).abs() < f32::EPSILON,
        "fit without grid config should not change scale"
    );
}

#[test]
fn camera_zoom_to_selection_with_selected_hex() {
    use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, SelectedHex};
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 10,
    });
    app.insert_resource(SelectedHex {
        position: Some(HexPosition::new(3, -2)),
    });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Move camera far away.
    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::new(99.0, 99.0);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.zoom_to_selection"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    // Camera should have moved toward the selected hex position.
    assert!(
        state.target_position.x < 99.0 || state.target_position.y < 99.0,
        "should move camera toward selected hex, got {:?}",
        state.target_position
    );
}

#[test]
fn camera_zoom_to_selection_without_selection_noop() {
    use hexorder_contracts::hex_grid::{HexGridConfig, SelectedHex};
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 10,
    });
    app.insert_resource(SelectedHex { position: None });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    app.world_mut()
        .resource_mut::<CameraState>()
        .target_position = Vec2::new(5.0, 5.0);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.zoom_to_selection"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    // With no selection, position should remain unchanged.
    assert!(
        (state.target_position.x - 5.0).abs() < f32::EPSILON,
        "should not move when no hex is selected"
    );
}

#[test]
fn camera_unknown_command_noop() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;
    let initial_pos = app.world().resource::<CameraState>().target_position;

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("unknown.command"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        (state.target_scale - initial_scale).abs() < f32::EPSILON,
        "unknown command should not change scale"
    );
    assert_eq!(state.target_position, initial_pos);
}

// ---------------------------------------------------------------------------
// Tests with a PrimaryWindow (covers fit_scale and window-dependent paths)
// ---------------------------------------------------------------------------

use bevy::window::PrimaryWindow;

/// Helper: build a test app with a spawned `PrimaryWindow` entity.
fn test_app_with_window() -> App {
    let mut app = test_app();
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(1280, 720),
            ..default()
        },
        PrimaryWindow,
    ));
    app
}

#[test]
fn configure_bounds_with_window_computes_fit_scale() {
    use hexorder_contracts::hex_grid::HexGridConfig;

    let mut app = test_app_with_window();
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
    // With a window, fit_scale is called (not the estimated_scale fallback).
    assert!(
        state.target_scale > 0.0,
        "fit_scale should produce a positive scale"
    );
    assert_eq!(
        state.current_scale, state.target_scale,
        "current_scale should snap to target on startup"
    );
}

#[test]
fn apply_pending_reset_with_window_recomputes_fit_scale() {
    use hexorder_contracts::hex_grid::HexGridConfig;

    let mut app = test_app_with_window();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 10,
    });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::apply_pending_reset);
    app.update();

    {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.pending_reset = true;
        state.target_scale = 0.01; // Force different from fit scale.
        let mut margins = app.world_mut().resource_mut::<ViewportMargins>();
        margins.left = 300.0;
        margins.right = 0.0;
        margins.top = 30.0;
        margins.bottom = 0.0;
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(!state.pending_reset, "should clear pending");
    assert_eq!(
        state.current_scale, state.target_scale,
        "current should snap to target"
    );
}

#[test]
fn camera_fit_command_with_window() {
    use hexorder_contracts::hex_grid::HexGridConfig;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app_with_window();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Manually set an initial scale different from what fit_scale will return.
    app.world_mut().resource_mut::<CameraState>().target_scale = 0.01;

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.fit"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    // fit_scale should have changed the target_scale.
    assert!(
        (state.target_scale - 0.01).abs() > f32::EPSILON,
        "fit command with window should change scale, got {}",
        state.target_scale
    );
}

#[test]
fn camera_reset_view_with_window_fits_and_centers() {
    use hexorder_contracts::hex_grid::HexGridConfig;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = test_app_with_window();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.add_systems(Startup, systems::spawn_camera);
    app.add_observer(systems::handle_camera_command);
    app.update();

    // Move camera away and change scale.
    {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.target_position = Vec2::new(20.0, 20.0);
        state.target_scale = 0.01;
    }

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("camera.reset_view"),
    });
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        (state.target_scale - 0.01).abs() > f32::EPSILON,
        "reset_view should fit scale"
    );
    assert_eq!(
        state.current_scale, state.target_scale,
        "current should snap to target on reset"
    );
    assert!(!state.pending_reset, "pending_reset should be cleared");
}

#[test]
fn compensate_resize_records_initial_height() {
    let mut app = test_app_with_window();
    app.add_systems(Update, systems::compensate_resize);
    app.update();

    // First frame — just records height. Scale should remain unchanged.
    let state = app.world().resource::<CameraState>();
    let default_state = CameraState::default();
    assert!(
        (state.target_scale - default_state.target_scale).abs() < f32::EPSILON,
        "first frame should only record height, not change scale"
    );
}

#[test]
fn compensate_resize_adjusts_scale_on_height_change() {
    let mut app = test_app_with_window();
    app.add_systems(Update, systems::compensate_resize);

    // First update records the initial height.
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;

    // Resize the window (change resolution).
    {
        let mut q = app.world_mut().query::<&mut Window>();
        let mut window = q.single_mut(app.world_mut()).expect("one window");
        window.resolution = bevy::window::WindowResolution::new(1280, 360); // Half height
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    // Ratio = 720 / 360 = 2.0, so scale should double.
    let expected_scale = initial_scale * 2.0;
    assert!(
        (state.target_scale - expected_scale).abs() < 0.001,
        "scale should adjust by height ratio, expected ~{expected_scale}, got {}",
        state.target_scale
    );
    assert!(
        (state.current_scale - expected_scale).abs() < 0.001,
        "current_scale should also adjust"
    );
}

#[test]
fn compensate_resize_noop_when_height_unchanged() {
    let mut app = test_app_with_window();
    app.add_systems(Update, systems::compensate_resize);

    // First update records height.
    app.update();

    let initial_scale = app.world().resource::<CameraState>().target_scale;

    // Second update with same window size — no resize.
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        (state.target_scale - initial_scale).abs() < f32::EPSILON,
        "same height should not change scale"
    );
}

#[test]
fn compensate_resize_clamps_to_bounds() {
    let mut app = test_app_with_window();
    app.add_systems(Update, systems::compensate_resize);
    app.update();

    // Shrink window to extreme to push scale past max.
    {
        let mut q = app.world_mut().query::<&mut Window>();
        let mut window = q.single_mut(app.world_mut()).expect("one window");
        window.resolution = bevy::window::WindowResolution::new(1280, 1);
    }
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(
        state.target_scale <= state.max_scale,
        "scale should be clamped to max"
    );
    assert!(
        state.current_scale <= state.max_scale,
        "current_scale should be clamped to max"
    );
}

// ---------------------------------------------------------------------------
// Smooth camera scale interpolation tests
// ---------------------------------------------------------------------------

#[test]
fn smooth_camera_interpolates_scale() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::smooth_camera);
    app.update();

    // Set a different target scale.
    {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.target_scale = 0.10;
        state.current_scale = 0.03; // default
    }

    for _ in 0..10 {
        app.update();
    }

    let state = app.world().resource::<CameraState>();
    // current_scale should have moved toward target_scale.
    assert!(
        state.current_scale > 0.03,
        "current_scale should interpolate toward target"
    );
}

#[test]
fn smooth_camera_y_stays_fixed() {
    let mut app = test_app();
    app.add_systems(Startup, systems::spawn_camera);
    app.add_systems(Update, systems::smooth_camera);
    app.update();

    {
        let mut state = app.world_mut().resource_mut::<CameraState>();
        state.target_position = Vec2::new(5.0, 5.0);
    }

    for _ in 0..5 {
        app.update();
    }

    let mut query = app.world_mut().query::<(&TopDownCamera, &Transform)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    let (_marker, transform) = results[0];
    assert!(
        (transform.translation.y - 100.0).abs() < f32::EPSILON,
        "camera Y should stay at CAMERA_Y (100.0)"
    );
}

// ---------------------------------------------------------------------------
// Plugin registration coverage
// ---------------------------------------------------------------------------

/// `CameraPlugin::build` wires up all systems, resources, and shortcuts.
///
/// Note: We avoid calling `app.update()` because the run conditions
/// reference `EguiWantsInput` (from `bevy_egui`) which requires the egui
/// plugin and rendering pipeline. We verify that `build()` itself
/// completes and initializes the expected resources.
#[test]
fn camera_plugin_registers_systems_and_shortcuts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<ShortcutRegistry>();
    app.add_plugins(super::CameraPlugin);

    // CameraState should be initialized by the plugin.
    assert!(
        app.world().get_resource::<CameraState>().is_some(),
        "CameraPlugin should init CameraState resource"
    );

    // ViewportMargins should be initialized by the plugin.
    assert!(
        app.world().get_resource::<ViewportMargins>().is_some(),
        "CameraPlugin should init ViewportMargins resource"
    );

    // Shortcuts should be registered.
    let registry = app.world().resource::<ShortcutRegistry>();
    assert!(
        !registry.bindings_for("camera.pan_up").is_empty(),
        "should register camera.pan_up"
    );
    assert!(
        !registry.bindings_for("camera.zoom_in").is_empty(),
        "should register camera.zoom_in"
    );
    assert!(
        !registry.bindings_for("camera.fit").is_empty(),
        "should register camera.fit"
    );
}
