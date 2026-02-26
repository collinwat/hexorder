//! Unit tests for the `hex_grid` plugin.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use hexorder_contracts::editor_ui::Selection;
use hexorder_contracts::hex_grid::{
    HexGridConfig, HexPosition, HexSelectedEvent, HexTile, MoveOverlay, MoveOverlayState,
    SelectedHex,
};
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::validation::{ValidMoveSet, ValidationResult};

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
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<Selection>();
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
            constraint_id: hexorder_contracts::game_system::TypeId::new(),
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

#[test]
fn field_of_view_no_blockers() {
    let origin = HexPosition::new(0, 0);
    let visible = algorithms::field_of_view(origin, 2, |_| false);
    let expected = algorithms::hex_range(origin, 2);
    assert_eq!(
        visible.len(),
        expected.len(),
        "With no blockers, visible set should equal full range"
    );
    for pos in &expected {
        assert!(visible.contains(pos), "Visible set should contain {pos:?}");
    }
}

#[test]
fn field_of_view_with_blocker() {
    let origin = HexPosition::new(0, 0);
    // Block hex (1,0) — hexes behind it in that direction should be hidden.
    let blocker = HexPosition::new(1, 0);
    let visible = algorithms::field_of_view(origin, 3, |pos| pos == blocker);
    // The blocker itself should be visible (you can see the wall).
    assert!(
        visible.contains(&blocker),
        "Blocker itself should be visible"
    );
    // But (2,0) directly behind the blocker should be hidden.
    let behind = HexPosition::new(2, 0);
    assert!(
        !visible.contains(&behind),
        "Hex directly behind blocker should be hidden"
    );
}

#[test]
fn field_of_view_range_zero() {
    let origin = HexPosition::new(5, -3);
    let visible = algorithms::field_of_view(origin, 0, |_| false);
    assert_eq!(visible.len(), 1, "Range 0 should return only origin");
    assert!(visible.contains(&origin));
}

#[test]
fn find_path_straight_line() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    let path = algorithms::find_path(from, to, |_, _| Some(1));
    assert!(path.is_some(), "Unobstructed path should exist");
    let path = path.expect("already checked");
    assert_eq!(*path.first().expect("path is non-empty"), from);
    assert_eq!(*path.last().expect("path is non-empty"), to);
}

#[test]
fn find_path_around_obstacle() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(2, 0);
    let wall = HexPosition::new(1, 0);
    let path = algorithms::find_path(
        from,
        to,
        |_, next| {
            if next == wall { None } else { Some(1) }
        },
    );
    assert!(path.is_some(), "Path around obstacle should exist");
    let path = path.expect("already checked");
    assert!(!path.contains(&wall), "Path should not go through wall");
    assert_eq!(*path.first().expect("path is non-empty"), from);
    assert_eq!(*path.last().expect("path is non-empty"), to);
}

#[test]
fn find_path_no_route() {
    let from = HexPosition::new(0, 0);
    let to = HexPosition::new(3, 0);
    // Block all neighbors of origin — no way out.
    let blocked: std::collections::HashSet<HexPosition> =
        algorithms::neighbors(from).into_iter().collect();
    let path = algorithms::find_path(from, to, |_, next| {
        if blocked.contains(&next) {
            None
        } else {
            Some(1)
        }
    });
    assert!(path.is_none(), "Walled-off path should return None");
}

// ---------------------------------------------------------------------------
// LOS system tests (0.7.0)
// ---------------------------------------------------------------------------

use hexorder_contracts::game_system::SelectedUnit;

#[test]
fn los_ray_not_drawn_without_unit() {
    // Verify draw_los_ray does not panic when SelectedUnit has no entity.
    let mut app = test_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );
    app.insert_resource(SelectedUnit::default());
    app.add_systems(Update, systems::draw_los_ray);
    app.update(); // Startup
    app.update(); // Update — should not panic
}

/// The `handle_hex_grid_command` observer must not panic when `SelectedHex`
/// does not exist (e.g., Escape pressed on the Launcher screen before the
/// hex grid is initialized). The observer wraps `SelectedHex` in `Option`.
#[test]
fn deselect_command_without_selected_hex_resource_does_not_panic() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Do NOT insert SelectedHex — simulates Launcher state.
    app.add_observer(systems::handle_hex_grid_command);
    app.update();

    // Fire the deselect command that the observer handles.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.deselect"),
    });

    app.update(); // Must not panic
}

// ---------------------------------------------------------------------------
// Additional coverage tests for systems.rs
// ---------------------------------------------------------------------------

use super::components::{
    HoverIndicator, IndicatorMaterials, MultiSelectIndicator, OverlayMaterials, SelectIndicator,
};
use hexorder_contracts::editor_ui::{EditorTool, PaintPreview};
use hexorder_contracts::game_system::UnitInstance;
use hexorder_contracts::hex_grid::TileBaseMaterial;

/// Helper: build an app with full indicator setup for testing `update_indicators`
/// and `sync_multi_select_indicators`.
fn test_app_with_indicators() -> App {
    let mut app = test_app();
    app.init_resource::<EditorTool>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex::default());
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
    app
}

#[test]
fn setup_grid_config_inserts_selected_and_hovered() {
    let mut app = test_app();
    app.add_systems(Startup, systems::setup_grid_config);
    app.update();

    assert!(
        app.world().get_resource::<SelectedHex>().is_some(),
        "SelectedHex should be inserted by setup_grid_config"
    );
    assert!(
        app.world().get_resource::<HoveredHex>().is_some(),
        "HoveredHex should be inserted by setup_grid_config"
    );
}

#[test]
fn setup_indicators_creates_indicator_entities() {
    let mut app = test_app_with_indicators();
    app.update();

    // Hover indicator entity should exist.
    let mut hover_q = app
        .world_mut()
        .query_filtered::<Entity, With<HoverIndicator>>();
    assert_eq!(
        hover_q.iter(app.world()).count(),
        1,
        "Exactly one HoverIndicator should exist"
    );

    // Select indicator entity should exist.
    let mut select_q = app
        .world_mut()
        .query_filtered::<Entity, With<SelectIndicator>>();
    assert_eq!(
        select_q.iter(app.world()).count(),
        1,
        "Exactly one SelectIndicator should exist"
    );
}

#[test]
fn setup_indicators_inserts_indicator_materials_resource() {
    let mut app = test_app_with_indicators();
    app.update();

    assert!(
        app.world().get_resource::<IndicatorMaterials>().is_some(),
        "IndicatorMaterials resource should be inserted"
    );
}

#[test]
fn setup_indicators_inserts_overlay_materials_resource() {
    let mut app = test_app_with_indicators();
    app.update();

    assert!(
        app.world().get_resource::<OverlayMaterials>().is_some(),
        "OverlayMaterials resource should be inserted"
    );
}

#[test]
fn update_indicators_shows_hover_ring_when_hovered() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    // Set a hovered position.
    app.world_mut().insert_resource(HoveredHex {
        position: Some(HexPosition::new(1, 0)),
    });
    app.update();

    // The hover indicator should be visible.
    let mut hover_q = app
        .world_mut()
        .query_filtered::<&Visibility, With<HoverIndicator>>();
    for vis in hover_q.iter(app.world()) {
        assert_eq!(
            *vis,
            Visibility::Visible,
            "Hover indicator should be visible when a hex is hovered"
        );
    }
}

#[test]
fn update_indicators_hides_hover_when_nothing_hovered() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    // Hovered is None (default).
    app.update();

    let mut hover_q = app
        .world_mut()
        .query_filtered::<&Visibility, With<HoverIndicator>>();
    for vis in hover_q.iter(app.world()) {
        assert_eq!(
            *vis,
            Visibility::Hidden,
            "Hover indicator should be hidden when nothing is hovered"
        );
    }
}

#[test]
fn update_indicators_hides_hover_when_same_as_selected() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    let pos = HexPosition::new(2, -1);
    app.world_mut().insert_resource(HoveredHex {
        position: Some(pos),
    });
    app.world_mut().insert_resource(SelectedHex {
        position: Some(pos),
    });
    app.update();

    let mut hover_q = app
        .world_mut()
        .query_filtered::<&Visibility, With<HoverIndicator>>();
    for vis in hover_q.iter(app.world()) {
        assert_eq!(
            *vis,
            Visibility::Hidden,
            "Hover indicator should be hidden when hovered == selected"
        );
    }
}

#[test]
fn update_indicators_shows_select_ring_when_selected() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    app.world_mut().insert_resource(SelectedHex {
        position: Some(HexPosition::new(3, 0)),
    });
    app.update();

    let mut select_q = app
        .world_mut()
        .query_filtered::<&Visibility, With<SelectIndicator>>();
    for vis in select_q.iter(app.world()) {
        assert_eq!(
            *vis,
            Visibility::Visible,
            "Select indicator should be visible when a hex is selected"
        );
    }
}

#[test]
fn update_indicators_hides_select_ring_when_nothing_selected() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    // SelectedHex is default (None).
    app.update();

    let mut select_q = app
        .world_mut()
        .query_filtered::<&Visibility, With<SelectIndicator>>();
    for vis in select_q.iter(app.world()) {
        assert_eq!(
            *vis,
            Visibility::Hidden,
            "Select indicator should be hidden when nothing is selected"
        );
    }
}

#[test]
fn update_indicators_paint_mode_uses_paint_preview_material() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    // Switch to Paint mode.
    app.world_mut().insert_resource(EditorTool::Paint);
    app.world_mut().insert_resource(HoveredHex {
        position: Some(HexPosition::new(1, 1)),
    });

    // Add a PaintPreview with a custom material.
    let custom_material = {
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            unlit: true,
            ..default()
        })
    };
    app.world_mut().insert_resource(PaintPreview {
        material: Some(custom_material.clone()),
    });

    app.update();

    // The hover indicator should be visible and using the paint preview material.
    let mut hover_q = app
        .world_mut()
        .query_filtered::<(&Visibility, &MeshMaterial3d<StandardMaterial>), With<HoverIndicator>>();
    for (vis, mat) in hover_q.iter(app.world()) {
        assert_eq!(*vis, Visibility::Visible);
        assert_eq!(
            mat.0, custom_material,
            "Hover indicator should use PaintPreview material in Paint mode"
        );
    }
}

#[test]
fn update_indicators_select_mode_uses_default_hover_material() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    let default_hover_mat = app.world().resource::<IndicatorMaterials>().hover.clone();

    // EditorTool defaults to Select mode.
    app.world_mut().insert_resource(HoveredHex {
        position: Some(HexPosition::new(0, 1)),
    });
    app.update();

    let mut hover_q = app
        .world_mut()
        .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<HoverIndicator>>();
    for mat in hover_q.iter(app.world()) {
        assert_eq!(
            mat.0, default_hover_mat,
            "Hover indicator should use default material in Select mode"
        );
    }
}

// ---------------------------------------------------------------------------
// handle_click: additional branch coverage
// ---------------------------------------------------------------------------

#[test]
fn click_on_already_selected_hex_deselects() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    let pos = HexPosition::new(2, 3);
    app.insert_resource(SelectedHex {
        position: Some(pos),
    });
    app.insert_resource(HoveredHex {
        position: Some(pos),
    });
    app.add_systems(Update, systems::handle_click);

    // Press
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release (click)
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selected = app.world().resource::<SelectedHex>();
    assert_eq!(
        selected.position, None,
        "Clicking on already-selected hex should deselect it"
    );
}

#[test]
fn click_with_no_hovered_hex_does_nothing() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex { position: None });
    app.add_systems(Update, systems::handle_click);

    // Press
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selected = app.world().resource::<SelectedHex>();
    assert_eq!(
        selected.position, None,
        "Clicking with no hovered hex should not select anything"
    );
}

#[test]
fn shift_click_adds_entity_to_multi_selection() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());

    let pos = HexPosition::new(1, 0);
    app.insert_resource(HoveredHex {
        position: Some(pos),
    });

    // Spawn a tile entity at the hovered position.
    let tile_entity = app.world_mut().spawn((HexTile, pos)).id();

    app.add_systems(Update, systems::handle_click);

    // Press with shift held.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selection = app.world().resource::<Selection>();
    assert!(
        selection.entities.contains(&tile_entity),
        "Shift+click should add tile to multi-selection"
    );
}

#[test]
fn shift_click_toggles_entity_out_of_multi_selection() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());

    let pos = HexPosition::new(1, 0);
    app.insert_resource(HoveredHex {
        position: Some(pos),
    });

    // Spawn a tile entity at the hovered position.
    let tile_entity = app.world_mut().spawn((HexTile, pos)).id();

    // Pre-populate the selection with this entity.
    let mut selection = Selection::default();
    selection.entities.insert(tile_entity);
    app.insert_resource(selection);

    app.add_systems(Update, systems::handle_click);

    // Press with shift held.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selection = app.world().resource::<Selection>();
    assert!(
        !selection.entities.contains(&tile_entity),
        "Shift+click on already-selected entity should remove it from multi-selection"
    );
}

#[test]
fn normal_click_clears_multi_selection() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());

    let pos = HexPosition::new(1, 0);
    app.insert_resource(HoveredHex {
        position: Some(pos),
    });

    // Pre-populate multi-selection with some entity.
    let some_entity = app.world_mut().spawn_empty().id();
    let mut selection = Selection::default();
    selection.entities.insert(some_entity);
    app.insert_resource(selection);

    app.add_systems(Update, systems::handle_click);

    // Press
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release (normal click, no shift)
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selection = app.world().resource::<Selection>();
    assert!(
        selection.entities.is_empty(),
        "Normal click should clear multi-selection"
    );
}

// ---------------------------------------------------------------------------
// handle_hex_grid_command: deselect with resource present
// ---------------------------------------------------------------------------

#[test]
fn deselect_command_clears_selected_hex() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SelectedHex {
        position: Some(HexPosition::new(5, 3)),
    });
    app.add_observer(systems::handle_hex_grid_command);
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.deselect"),
    });
    app.update();

    let selected = app.world().resource::<SelectedHex>();
    assert_eq!(
        selected.position, None,
        "edit.deselect command should clear SelectedHex"
    );
}

#[test]
fn non_deselect_command_does_not_change_selected_hex() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let pos = HexPosition::new(2, 1);
    app.insert_resource(SelectedHex {
        position: Some(pos),
    });
    app.add_observer(systems::handle_hex_grid_command);
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("file.save"),
    });
    app.update();

    let selected = app.world().resource::<SelectedHex>();
    assert_eq!(
        selected.position,
        Some(pos),
        "Unrelated command should not affect SelectedHex"
    );
}

// ---------------------------------------------------------------------------
// sync_multi_select_indicators
// ---------------------------------------------------------------------------

#[test]
fn sync_multi_select_spawns_indicators_for_selected_tiles() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::sync_multi_select_indicators);
    app.update(); // Startup

    // Find a tile entity to select.
    let tile_entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<HexTile>>();
        q.iter(app.world()).next().expect("should have tiles")
    };

    let mut sel = Selection::default();
    sel.entities.insert(tile_entity);
    app.insert_resource(sel);
    app.update();

    let mut indicator_q = app.world_mut().query::<&MultiSelectIndicator>();
    let indicators: Vec<_> = indicator_q.iter(app.world()).collect();
    assert_eq!(
        indicators.len(),
        1,
        "Should spawn one multi-select indicator for one selected tile"
    );
    assert_eq!(indicators[0].tile_entity, tile_entity);
}

#[test]
fn sync_multi_select_despawns_indicators_when_deselected() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::sync_multi_select_indicators);
    app.update(); // Startup

    // Select a tile.
    let tile_entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<HexTile>>();
        q.iter(app.world()).next().expect("should have tiles")
    };
    let mut sel = Selection::default();
    sel.entities.insert(tile_entity);
    app.insert_resource(sel);
    app.update();

    // Verify indicator exists.
    let mut indicator_q = app
        .world_mut()
        .query_filtered::<Entity, With<MultiSelectIndicator>>();
    assert_eq!(indicator_q.iter(app.world()).count(), 1);

    // Clear selection.
    app.insert_resource(Selection::default());
    app.update();

    let mut indicator_q = app
        .world_mut()
        .query_filtered::<Entity, With<MultiSelectIndicator>>();
    assert_eq!(
        indicator_q.iter(app.world()).count(),
        0,
        "Indicators should be despawned when selection is cleared"
    );
}

// ---------------------------------------------------------------------------
// cleanup_internal_entities
// ---------------------------------------------------------------------------

#[test]
fn cleanup_internal_entities_removes_indicators() {
    let mut app = test_app_with_indicators();
    app.update(); // Startup

    // Verify indicators exist.
    let mut hover_q = app
        .world_mut()
        .query_filtered::<Entity, With<HoverIndicator>>();
    assert!(hover_q.iter(app.world()).count() > 0);

    let mut select_q = app
        .world_mut()
        .query_filtered::<Entity, With<SelectIndicator>>();
    assert!(select_q.iter(app.world()).count() > 0);

    // Run cleanup.
    app.add_systems(Update, systems::cleanup_internal_entities);
    app.update();

    let mut hover_q = app
        .world_mut()
        .query_filtered::<Entity, With<HoverIndicator>>();
    assert_eq!(
        hover_q.iter(app.world()).count(),
        0,
        "HoverIndicator should be despawned by cleanup"
    );

    let mut select_q = app
        .world_mut()
        .query_filtered::<Entity, With<SelectIndicator>>();
    assert_eq!(
        select_q.iter(app.world()).count(),
        0,
        "SelectIndicator should be despawned by cleanup"
    );
}

// ---------------------------------------------------------------------------
// draw_los_ray with a unit that has a position
// ---------------------------------------------------------------------------

#[test]
fn los_ray_drawn_when_unit_selected_and_hovered() {
    let mut app = test_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );

    // Spawn a unit with a position.
    let unit_pos = HexPosition::new(0, 0);
    let unit_entity = app.world_mut().spawn((UnitInstance, unit_pos)).id();
    app.insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });
    app.insert_resource(HoveredHex {
        position: Some(HexPosition::new(2, 0)),
    });

    app.add_systems(Update, systems::draw_los_ray);
    app.update(); // Startup
    app.update(); // Update -- draws the LOS ray (should not panic)
}

#[test]
fn los_ray_not_drawn_when_hover_equals_unit_position() {
    let mut app = test_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );

    let unit_pos = HexPosition::new(0, 0);
    let unit_entity = app.world_mut().spawn((UnitInstance, unit_pos)).id();
    app.insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });
    // Hover the same hex as the unit -- no ray should be drawn.
    app.insert_resource(HoveredHex {
        position: Some(unit_pos),
    });

    app.add_systems(Update, systems::draw_los_ray);
    app.update(); // Startup
    app.update(); // Update -- should not panic and should early return
}

#[test]
fn los_ray_not_drawn_when_no_hover() {
    let mut app = test_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );

    let unit_pos = HexPosition::new(0, 0);
    let unit_entity = app.world_mut().spawn((UnitInstance, unit_pos)).id();
    app.insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });
    app.insert_resource(HoveredHex { position: None });

    app.add_systems(Update, systems::draw_los_ray);
    app.update(); // Startup
    app.update(); // No hover means no ray
}

#[test]
fn los_ray_not_drawn_when_unit_entity_has_no_position() {
    let mut app = test_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);
    app.add_systems(
        Startup,
        (
            systems::setup_grid_config,
            systems::setup_materials,
            systems::spawn_grid,
        )
            .chain(),
    );

    // Spawn a unit entity WITHOUT HexPosition.
    let unit_entity = app.world_mut().spawn(UnitInstance).id();
    app.insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });
    app.insert_resource(HoveredHex {
        position: Some(HexPosition::new(2, 0)),
    });

    app.add_systems(Update, systems::draw_los_ray);
    app.update();
    app.update(); // Should early return without panic
}

// ---------------------------------------------------------------------------
// All tiles have TileBaseMaterial and correct rotation
// ---------------------------------------------------------------------------

#[test]
fn all_tiles_have_tile_base_material() {
    let mut app = test_app_with_grid();
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&TileBaseMaterial, With<HexTile>>();
    let count = query.iter(app.world()).count();

    let config = app
        .world()
        .get_resource::<HexGridConfig>()
        .expect("config should exist");
    let expected = systems::tile_count_for_radius(config.map_radius);

    assert_eq!(count, expected, "All tiles should have TileBaseMaterial");
}

#[test]
fn all_tiles_have_flat_rotation() {
    let mut app = test_app_with_grid();
    app.update();

    let expected_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    let mut query = app
        .world_mut()
        .query_filtered::<&Transform, With<HexTile>>();
    for transform in query.iter(app.world()) {
        let dot = transform.rotation.dot(expected_rotation).abs();
        assert!(
            (dot - 1.0).abs() < 1e-5,
            "Tile rotation should be -90 degrees around X axis"
        );
    }
}

// ---------------------------------------------------------------------------
// Plugin build coverage (mod.rs)
// ---------------------------------------------------------------------------

#[test]
fn hex_grid_plugin_builds_without_panic() {
    use hexorder_contracts::editor_ui::ViewportRect;
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<ShortcutRegistry>();
    app.init_resource::<ValidMoveSet>();
    app.init_resource::<SelectedUnit>();
    app.init_resource::<Selection>();
    app.init_resource::<EditorTool>();
    app.init_resource::<ViewportRect>();
    app.init_resource::<bevy::input::mouse::AccumulatedMouseMotion>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::gizmos::GizmoPlugin);

    app.add_plugins(super::HexGridPlugin);
    app.update(); // Should not panic
}

#[test]
fn register_shortcuts_adds_deselect_command() {
    use hexorder_contracts::shortcuts::{KeyBinding, Modifiers, ShortcutRegistry};

    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    let binding = KeyBinding::new(bevy::input::keyboard::KeyCode::Escape, Modifiers::NONE);
    let found = registry.lookup(&binding);
    assert!(
        found.is_some(),
        "Escape should be bound after register_shortcuts"
    );
    assert_eq!(
        found.expect("already checked").id.0,
        "edit.deselect",
        "Escape should map to edit.deselect"
    );
}

// ---------------------------------------------------------------------------
// Shift+click with ShiftRight key
// ---------------------------------------------------------------------------

#[test]
fn shift_right_click_adds_to_multi_selection() {
    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());

    let pos = HexPosition::new(1, 0);
    app.insert_resource(HoveredHex {
        position: Some(pos),
    });

    let tile_entity = app.world_mut().spawn((HexTile, pos)).id();
    app.add_systems(Update, systems::handle_click);

    // Press with ShiftRight held.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftRight);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    // Release
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let selection = app.world().resource::<Selection>();
    assert!(
        selection.entities.contains(&tile_entity),
        "ShiftRight+click should add tile to multi-selection"
    );
}

// ---------------------------------------------------------------------------
// Paint mode with no PaintPreview material (None)
// ---------------------------------------------------------------------------

#[test]
fn update_indicators_paint_mode_no_preview_uses_default() {
    let mut app = test_app_with_indicators();
    app.add_systems(Update, systems::update_indicators);
    app.update(); // Startup

    let default_hover_mat = app.world().resource::<IndicatorMaterials>().hover.clone();

    // Switch to Paint mode but PaintPreview.material is None.
    app.world_mut().insert_resource(EditorTool::Paint);
    app.world_mut()
        .insert_resource(PaintPreview { material: None });
    app.world_mut().insert_resource(HoveredHex {
        position: Some(HexPosition::new(1, 1)),
    });
    app.update();

    let mut hover_q = app
        .world_mut()
        .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<HoverIndicator>>();
    for mat in hover_q.iter(app.world()) {
        assert_eq!(
            mat.0, default_hover_mat,
            "Paint mode with no PaintPreview material should fall back to default"
        );
    }
}
