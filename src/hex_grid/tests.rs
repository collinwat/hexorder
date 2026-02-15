//! Unit tests for the `hex_grid` plugin.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::contracts::hex_grid::{
    HexGridConfig, HexPosition, HexSelectedEvent, HexTile, MoveOverlay, MoveOverlayState,
    SelectedHex,
};
use crate::contracts::persistence::AppScreen;
use crate::contracts::validation::{ValidMoveSet, ValidationResult};

use super::components::{HexMaterials, HoveredHex};
use super::systems;

/// Helper: create a minimal App with resources needed for `hex_grid` testing.
fn test_app() -> App {
    let mut app = App::new();
    // MinimalPlugins provides the basic scheduler without rendering.
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<bevy::input::mouse::AccumulatedMouseMotion>();
    app
}

/// Helper: create a test app with the full grid startup systems chained.
fn test_app_with_grid() -> App {
    let mut app = test_app();
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );
    app
}

#[test]
fn hex_position_roundtrip() {
    let pos = HexPosition::new(3, -2);
    let hex = pos.to_hex();
    let back = HexPosition::from_hex(hex);
    assert_eq!(pos, back);
}

#[test]
fn hex_position_to_hex_coordinates() {
    let pos = HexPosition::new(5, -3);
    let hex = pos.to_hex();
    assert_eq!(hex.x(), 5);
    assert_eq!(hex.y(), -3);
}

#[test]
fn grid_config_inserted_after_startup() {
    let mut app = test_app();
    app.add_systems(Startup, systems::setup_grid_config);
    app.update();

    let config = app.world().get_resource::<HexGridConfig>();
    assert!(
        config.is_some(),
        "HexGridConfig resource should exist after Startup"
    );

    let config = config.expect("already checked");
    assert_eq!(config.map_radius, 10);
}

#[test]
fn selected_hex_defaults_to_none() {
    let selected = SelectedHex::default();
    assert!(selected.position.is_none());
}

#[test]
fn hovered_hex_defaults_to_none() {
    let hovered = HoveredHex::default();
    assert!(hovered.position.is_none());
}

#[test]
fn tile_count_formula() {
    // radius 0 => 1 tile (just center)
    assert_eq!(systems::tile_count_for_radius(0), 1);
    // radius 1 => 7 tiles (center + 6 neighbors)
    assert_eq!(systems::tile_count_for_radius(1), 7);
    // radius 2 => 19 tiles
    assert_eq!(systems::tile_count_for_radius(2), 19);
    // radius 10 => 331 tiles
    assert_eq!(systems::tile_count_for_radius(10), 331);
}

#[test]
fn grid_spawns_correct_number_of_tiles() {
    let mut app = test_app_with_grid();
    app.update();

    let (map_radius, expected) = {
        let config = app
            .world()
            .get_resource::<HexGridConfig>()
            .expect("config should exist");
        (
            config.map_radius,
            systems::tile_count_for_radius(config.map_radius),
        )
    };

    let mut query = app.world_mut().query_filtered::<Entity, With<HexTile>>();
    let actual = query.iter(app.world()).count();

    assert_eq!(
        actual, expected,
        "Grid with radius {map_radius} should have {expected} tiles, got {actual}"
    );
}

#[test]
fn all_tiles_have_hex_position() {
    let mut app = test_app_with_grid();
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &HexPosition), With<HexTile>>();
    let results: Vec<_> = query.iter(app.world()).collect();

    assert!(
        !results.is_empty(),
        "There should be at least one HexTile entity"
    );

    // All HexTile entities should have HexPosition (guaranteed by the query filter).
    // If we got here, the query matched, so the assertion is implicit.
    // Additionally verify count matches expected.
    let config = app
        .world()
        .get_resource::<HexGridConfig>()
        .expect("config should exist");
    let expected = systems::tile_count_for_radius(config.map_radius);
    assert_eq!(
        results.len(),
        expected,
        "All {expected} tiles should have HexPosition"
    );
}

#[test]
fn all_tiles_at_y_zero() {
    let mut app = test_app_with_grid();
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&Transform, With<HexTile>>();
    for transform in query.iter(app.world()) {
        assert!(
            (transform.translation.y).abs() < f32::EPSILON,
            "All hex tiles should be at Y=0, found Y={}",
            transform.translation.y
        );
    }
}

#[test]
fn hex_materials_resource_exists_after_startup() {
    let mut app = test_app();
    app.add_systems(Startup, systems::setup_materials);
    app.update();

    assert!(
        app.world().get_resource::<HexMaterials>().is_some(),
        "HexMaterials resource should be inserted during Startup"
    );
}

#[test]
fn click_sets_selected_hex() {
    // Test that handle_click updates SelectedHex when there is a hovered hex.
    // handle_click fires on button release (not press) to distinguish clicks from drags.
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex {
        position: Some(HexPosition::new(2, 3)),
    });

    app.add_systems(Update, systems::handle_click);

    // Press the left mouse button.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Clear just-states, then release to trigger the click.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selected = app.world().resource::<SelectedHex>();
    assert_eq!(
        selected.position,
        Some(HexPosition::new(2, 3)),
        "Clicking should set SelectedHex to the hovered position"
    );
}

#[test]
fn click_fires_selected_event() {
    // In Bevy 0.18, events use the observer pattern (commands.trigger / On<Event>).
    // We register an observer that captures the fired event for assertion.
    let received = Arc::new(Mutex::new(Vec::<HexPosition>::new()));
    let received_clone = Arc::clone(&received);

    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex {
        position: Some(HexPosition::new(-1, 4)),
    });

    // Register an observer that captures HexSelectedEvent positions.
    app.add_observer(move |trigger: On<HexSelectedEvent>| {
        let pos = trigger.event().position;
        received_clone
            .lock()
            .expect("mutex should not be poisoned")
            .push(pos);
    });

    app.add_systems(Update, systems::handle_click);

    // Press the left mouse button.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Clear just-states, then release to trigger the click.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let events = received.lock().expect("mutex should not be poisoned");
    assert_eq!(events.len(), 1, "Exactly one HexSelectedEvent should fire");
    assert_eq!(events[0], HexPosition::new(-1, 4));
}

#[test]
fn no_click_no_event() {
    let received = Arc::new(Mutex::new(Vec::<HexPosition>::new()));
    let received_clone = Arc::clone(&received);

    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex {
        position: Some(HexPosition::new(0, 0)),
    });

    // Register an observer that captures HexSelectedEvent positions.
    app.add_observer(move |trigger: On<HexSelectedEvent>| {
        let pos = trigger.event().position;
        received_clone
            .lock()
            .expect("mutex should not be poisoned")
            .push(pos);
    });

    app.add_systems(Update, systems::handle_click);

    // Do NOT press any mouse button.
    app.update();

    let events = received.lock().expect("mutex should not be poisoned");
    assert_eq!(events.len(), 0, "No event should fire without a click");
}

// ---------------------------------------------------------------------------
// Move overlay tests (0.4.0)
// ---------------------------------------------------------------------------

/// Helper: create a test app with grid startup and overlay materials.
fn test_app_with_overlays() -> App {
    let mut app = test_app();
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
            systems::setup_indicators,
        )
            .chain(),
    );
    app.init_resource::<ValidMoveSet>();
    app.add_systems(Update, systems::sync_move_overlays);
    app
}

#[test]
fn move_overlays_spawned_on_unit_select() {
    let mut app = test_app_with_overlays();
    app.update(); // Startup

    // Simulate a unit being selected with some valid positions.
    let unit_entity = app.world_mut().spawn_empty().id();
    let mut valid_positions = std::collections::HashSet::new();
    valid_positions.insert(HexPosition::new(1, 0));
    valid_positions.insert(HexPosition::new(0, 1));

    app.world_mut().insert_resource(ValidMoveSet {
        valid_positions,
        blocked_explanations: std::collections::HashMap::new(),
        for_entity: Some(unit_entity),
    });
    app.update();

    let mut query = app.world_mut().query::<&MoveOverlay>();
    let overlays: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(overlays.len(), 2, "Should have 2 valid move overlays");
    assert!(
        overlays.iter().all(|o| o.state == MoveOverlayState::Valid),
        "All overlays should be Valid state"
    );
}

#[test]
fn move_overlays_despawned_on_deselect() {
    let mut app = test_app_with_overlays();
    app.update(); // Startup

    // First, spawn some overlays.
    let unit_entity = app.world_mut().spawn_empty().id();
    let mut valid_positions = std::collections::HashSet::new();
    valid_positions.insert(HexPosition::new(1, 0));

    app.world_mut().insert_resource(ValidMoveSet {
        valid_positions,
        blocked_explanations: std::collections::HashMap::new(),
        for_entity: Some(unit_entity),
    });
    app.update();

    // Verify overlays exist.
    let mut query = app.world_mut().query::<&MoveOverlay>();
    assert_eq!(query.iter(app.world()).count(), 1);

    // Now deselect — clear ValidMoveSet.
    app.world_mut().insert_resource(ValidMoveSet::default());
    app.update();

    let mut query = app.world_mut().query::<&MoveOverlay>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "All overlays should be despawned after deselect"
    );
}

#[test]
fn blocked_positions_get_red_overlay() {
    let mut app = test_app_with_overlays();
    app.update(); // Startup

    let unit_entity = app.world_mut().spawn_empty().id();
    let blocked_pos = HexPosition::new(2, 0);
    let mut blocked = std::collections::HashMap::new();
    blocked.insert(
        blocked_pos,
        vec![ValidationResult {
            constraint_id: crate::contracts::game_system::TypeId::new(),
            constraint_name: "Test".to_string(),
            satisfied: false,
            explanation: "Blocked".to_string(),
        }],
    );

    app.world_mut().insert_resource(ValidMoveSet {
        valid_positions: std::collections::HashSet::new(),
        blocked_explanations: blocked,
        for_entity: Some(unit_entity),
    });
    app.update();

    let mut query = app.world_mut().query::<&MoveOverlay>();
    let overlays: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(overlays.len(), 1, "Should have 1 blocked overlay");
    assert_eq!(overlays[0].state, MoveOverlayState::Blocked);
    assert_eq!(overlays[0].position, blocked_pos);
}

#[test]
fn no_overlays_when_valid_move_set_empty() {
    let mut app = test_app_with_overlays();
    app.update(); // Startup

    // ValidMoveSet is default (empty) — no overlays should exist.
    // Run a second update so the system has a chance to process.
    app.update();

    let mut query = app.world_mut().query::<&MoveOverlay>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "No overlays when ValidMoveSet is empty"
    );
}

// ---------------------------------------------------------------------------
// Algorithm tests (0.7.0)
// ---------------------------------------------------------------------------

use super::algorithms;

#[test]
fn neighbors_returns_six() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::neighbors(center);
    assert_eq!(result.len(), 6, "Should have exactly 6 neighbors");
}

#[test]
fn neighbors_are_adjacent() {
    let center = HexPosition::new(3, -2);
    let result = algorithms::neighbors(center);
    let center_hex = center.to_hex();
    for neighbor in &result {
        let dist = center_hex.unsigned_distance_to(neighbor.to_hex());
        assert_eq!(dist, 1, "Each neighbor should be distance 1 from center");
    }
}

#[test]
fn ring_at_radius() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::ring(center, 2);
    assert_eq!(result.len(), 12, "Ring at radius 2 should have 12 hexes");
    let center_hex = center.to_hex();
    for pos in &result {
        let dist = center_hex.unsigned_distance_to(pos.to_hex());
        assert_eq!(dist, 2, "All ring hexes should be at exact distance 2");
    }
}

#[test]
fn ring_at_radius_zero() {
    let center = HexPosition::new(1, 1);
    let result = algorithms::ring(center, 0);
    assert_eq!(result.len(), 1, "Ring at radius 0 is just the center");
    assert_eq!(result[0], center);
}

#[test]
fn hex_range_count() {
    let center = HexPosition::new(0, 0);
    let result = algorithms::hex_range(center, 3);
    // 3*3*(3+1)+1 = 37
    let expected = 3 * 3 * (3 + 1) + 1;
    assert_eq!(
        result.len(),
        expected as usize,
        "Range at radius 3 should have {expected} hexes"
    );
    let center_hex = center.to_hex();
    for pos in &result {
        let dist = center_hex.unsigned_distance_to(pos.to_hex());
        assert!(dist <= 3, "All range hexes should be within distance 3");
    }
}

#[test]
fn line_of_sight_clear_path() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let result = algorithms::line_of_sight(from, to, |_| false);
    assert!(result.clear, "Path with no blockers should be clear");
    assert!(result.blocked_by.is_none());
    assert_eq!(result.origin, from);
    assert_eq!(result.target, to);
    assert!(
        result.path.len() >= 2,
        "Path should include at least origin and target"
    );
    assert_eq!(*result.path.first().expect("path is non-empty"), from);
    assert_eq!(*result.path.last().expect("path is non-empty"), to);
}

#[test]
fn line_of_sight_blocked() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let blocker = HexPosition::new(2, 0);
    let result = algorithms::line_of_sight(from, to, |pos| pos == blocker);
    assert!(!result.clear, "Path should be blocked");
    assert_eq!(result.blocked_by, Some(blocker));
}

#[test]
fn line_of_sight_same_hex() {
    let pos = HexPosition::new(2, -1);
    let result = algorithms::line_of_sight(pos, pos, |_| false);
    assert!(result.clear);
    assert_eq!(result.path.len(), 1);
    assert_eq!(result.path[0], pos);
}

#[test]
fn line_of_sight_adjacent() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(1, 0);
    let result = algorithms::line_of_sight(from, to, |_| false);
    assert!(result.clear);
    assert_eq!(result.path.len(), 2);
    assert_eq!(result.path[0], from);
    assert_eq!(result.path[1], to);
}
