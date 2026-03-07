//! Unit tests for the unit plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use hexorder_contracts::editor_ui::EditorTool;
use hexorder_contracts::game_system::{
    ActiveTokenType, EntityData, EntityRole, EntityType, EntityTypeRegistry, SelectedUnit, TypeId,
    UnitInstance,
};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexSelectedEvent};
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::shortcuts::ShortcutRegistry;
use hexorder_contracts::undo_redo::UndoStack;
use hexorder_contracts::validation::ValidMoveSet;

use super::components::{UnitMaterials, UnitMesh};
use super::systems;

/// Helper: create a minimal App with resources needed for unit testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<UndoStack>();
    app.init_resource::<hexorder_contracts::hex_grid::StackingRule>();
    app
}

/// Helper: create a test entity type registry with 2 Token types.
fn test_registry() -> EntityTypeRegistry {
    EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.2, 0.4, 0.7),
                properties: Vec::new(),
            },
            EntityType {
                id: TypeId::new(),
                name: "Cavalry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.7, 0.3, 0.2),
                properties: Vec::new(),
            },
        ],
    }
}

/// Helper: create a test grid config.
fn test_grid_config() -> HexGridConfig {
    HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    }
}

/// Helper: insert test registry, grid config, and run `setup_unit_visuals`.
fn setup_unit_resources(app: &mut App) {
    let registry = test_registry();
    app.insert_resource(registry);
    app.insert_resource(test_grid_config());
    app.add_systems(Startup, systems::setup_unit_visuals);
}

#[test]
fn unit_materials_created_for_all_types() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let unit_mats = app
        .world()
        .get_resource::<UnitMaterials>()
        .expect("UnitMaterials should exist after Startup");

    let registry = app.world().resource::<EntityTypeRegistry>();
    let token_types = registry.types_by_role(EntityRole::Token);
    assert_eq!(
        unit_mats.materials.len(),
        token_types.len(),
        "Should have a material for each Token entity type"
    );

    for et in &token_types {
        assert!(
            unit_mats.get(et.id).is_some(),
            "Material should exist for entity type '{}'",
            et.name
        );
    }
}

#[test]
fn unit_mesh_resource_exists() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    assert!(
        app.world().get_resource::<UnitMesh>().is_some(),
        "UnitMesh should exist after Startup"
    );
}

#[test]
fn place_unit_creates_entity() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &HexPosition, &EntityData), With<UnitInstance>>();
    let units: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(units.len(), 1, "Should have spawned one unit");
    let (_, pos, data) = units[0];
    assert_eq!(*pos, HexPosition::new(0, 0));
    assert_eq!(data.entity_type_id, first_id);
}

#[test]
fn place_unit_skipped_in_select_mode() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Select);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    let count = query.iter(app.world()).count();

    assert_eq!(count, 0, "No units should be placed in Select mode");
}

#[test]
fn select_unit_sets_selected() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);
    app.world_mut().insert_resource(SelectedUnit::default());
    app.init_resource::<ValidMoveSet>();

    // Manually spawn a unit at (1, 1).
    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(1, 1),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.add_observer(systems::handle_unit_interaction);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(1, 1),
    });
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert_eq!(
        selected.entity,
        Some(unit_entity),
        "SelectedUnit should be set to the clicked unit"
    );
}

#[test]
fn move_unit_updates_position() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);
    app.init_resource::<ValidMoveSet>();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    // Pre-select the unit.
    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    app.add_observer(systems::handle_unit_interaction);

    // Click a different tile to move the unit.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(2, 1),
    });
    app.update();

    let pos = app
        .world()
        .entity(unit_entity)
        .get::<HexPosition>()
        .expect("Unit should have HexPosition");
    assert_eq!(
        *pos,
        HexPosition::new(2, 1),
        "Unit should have moved to (2, 1)"
    );

    let selected = app.world().resource::<SelectedUnit>();
    assert!(
        selected.entity.is_none(),
        "Unit should be deselected after moving"
    );
}

#[test]
fn move_unit_respects_grid_bounds() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);
    app.init_resource::<ValidMoveSet>();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    app.add_observer(systems::handle_unit_interaction);

    // Try to move to a position outside the grid (radius 5).
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(10, 10),
    });
    app.update();

    let pos = app
        .world()
        .entity(unit_entity)
        .get::<HexPosition>()
        .expect("Unit should have HexPosition");
    assert_eq!(
        *pos,
        HexPosition::new(0, 0),
        "Unit should not have moved outside grid bounds"
    );
}

#[test]
fn sync_unit_visuals_updates_material() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let dummy_material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            MeshMaterial3d(dummy_material),
        ))
        .id();

    app.add_systems(Update, systems::sync_unit_visuals);
    app.update();

    let expected_handle = {
        let unit_mats = app.world().resource::<UnitMaterials>();
        unit_mats
            .get(first_id)
            .expect("Material should exist for first entity type")
            .clone()
    };

    let unit_material = app
        .world()
        .entity(unit_entity)
        .get::<MeshMaterial3d<StandardMaterial>>()
        .expect("Unit should have MeshMaterial3d");

    assert_eq!(
        unit_material.0, expected_handle,
        "Unit material should match the entity type material"
    );
}

#[test]
fn sync_unit_materials_adds_new_type() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let initial_count = app.world().resource::<UnitMaterials>().materials.len();
    assert_eq!(initial_count, 2);

    // Add a new Token entity type to the registry.
    let new_id = TypeId::new();
    app.world_mut()
        .resource_mut::<EntityTypeRegistry>()
        .types
        .push(EntityType {
            id: new_id,
            name: "Artillery".to_string(),
            role: EntityRole::Token,
            color: Color::srgb(0.6, 0.6, 0.2),
            properties: Vec::new(),
        });

    app.add_systems(Update, systems::sync_unit_materials);
    app.update();

    let unit_mats = app.world().resource::<UnitMaterials>();
    assert_eq!(
        unit_mats.materials.len(),
        3,
        "Should have 3 materials after adding a new type"
    );
    assert!(
        unit_mats.get(new_id).is_some(),
        "New type should have a material"
    );
}

#[test]
fn place_unit_records_undo_command() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let stack = app.world().resource::<UndoStack>();
    assert!(
        stack.can_undo(),
        "Undo stack should have a command after placing"
    );
    assert!(
        stack
            .undo_description()
            .expect("should have description")
            .contains("Place"),
        "Undo description should mention Place"
    );
}

#[test]
fn place_then_undo_removes_unit() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);
    app.init_resource::<ShortcutRegistry>();
    app.add_plugins(hexorder_undo_redo::UndoRedoPlugin);

    // Place a unit.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(1, 1),
    });
    app.update();

    // Confirm unit exists.
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        1,
        "Unit should exist after placement"
    );

    // Request undo.
    app.world_mut().resource_mut::<UndoStack>().request_undo();
    app.update();

    // Unit should be gone.
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "Unit should be despawned after undo"
    );
}

#[test]
fn place_undo_redo_restores_unit() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);
    app.init_resource::<ShortcutRegistry>();
    app.add_plugins(hexorder_undo_redo::UndoRedoPlugin);

    // Place a unit.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(2, 0),
    });
    app.update();

    // Undo (remove).
    app.world_mut().resource_mut::<UndoStack>().request_undo();
    app.update();

    // Redo (restore).
    app.world_mut().resource_mut::<UndoStack>().request_redo();
    app.update();

    // Unit should be back.
    let mut query = app
        .world_mut()
        .query_filtered::<(&HexPosition, &EntityData), With<UnitInstance>>();
    let units: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(units.len(), 1, "Unit should be restored after redo");
    assert_eq!(*units[0].0, HexPosition::new(2, 0));
    assert_eq!(units[0].1.entity_type_id, first_id);
}

/// Placement does nothing when `ActiveTokenType.entity_type_id` is `None`.
#[test]
fn place_unit_noop_when_no_active_type() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: None,
    });

    app.add_observer(systems::handle_unit_placement);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "No unit should be placed when active type is None"
    );
}

/// Placement does nothing when the active type ID is not in the registry.
#[test]
fn place_unit_noop_when_type_not_in_registry() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let unknown_id = TypeId::new(); // Not registered

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(unknown_id),
    });

    app.add_observer(systems::handle_unit_placement);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "No unit should be placed when type is not in registry"
    );
}

/// Placement does nothing when the position is outside grid bounds.
#[test]
fn place_unit_noop_when_out_of_bounds() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    app.world_mut().insert_resource(EditorTool::Place);
    app.world_mut().insert_resource(ActiveTokenType {
        entity_type_id: Some(first_id),
    });

    app.add_observer(systems::handle_unit_placement);

    // Grid radius is 5, so (10, 10) is out of bounds.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(10, 10),
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "No unit should be placed outside grid bounds"
    );
}

/// `delete_selected_unit` clears selection when the selected entity no longer exists.
#[test]
fn delete_selected_unit_clears_when_entity_gone() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    // Spawn a unit and select it.
    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    // Despawn the entity.
    app.world_mut().despawn(unit_entity);

    // Run the system.
    app.add_systems(Update, systems::delete_selected_unit);
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert!(
        selected.entity.is_none(),
        "Selection should be cleared when entity is despawned"
    );
}

/// `delete_selected_unit` does nothing when selected entity still exists.
#[test]
fn delete_selected_unit_noop_when_entity_exists() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    app.add_systems(Update, systems::delete_selected_unit);
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert_eq!(
        selected.entity,
        Some(unit_entity),
        "Selection should remain when entity exists"
    );
}

/// Clicking the same selected unit deselects it.
#[test]
fn click_selected_unit_deselects() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);
    app.init_resource::<ValidMoveSet>();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(1, 1),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    app.add_observer(systems::handle_unit_interaction);

    // Click the same hex where the selected unit stands.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(1, 1),
    });
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert!(
        selected.entity.is_none(),
        "Clicking selected unit should deselect it"
    );
}

/// Movement is blocked when the destination is not in `ValidMoveSet.valid_positions`.
#[test]
fn move_unit_blocked_by_valid_move_set() {
    use std::collections::HashSet;

    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    // Only allow movement to (1, 0).
    app.world_mut().insert_resource(ValidMoveSet {
        valid_positions: HashSet::from([HexPosition::new(1, 0)]),
        blocked_explanations: HashMap::new(),
        for_entity: Some(unit_entity),
    });

    app.add_observer(systems::handle_unit_interaction);

    // Try to move to (2, 1) which is NOT in valid positions.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(2, 1),
    });
    app.update();

    let pos = app
        .world()
        .entity(unit_entity)
        .get::<HexPosition>()
        .expect("Unit should have HexPosition");
    assert_eq!(
        *pos,
        HexPosition::new(0, 0),
        "Unit should not move to position outside valid move set"
    );
}

/// Movement succeeds when the destination IS in `ValidMoveSet.valid_positions`.
#[test]
fn move_unit_allowed_by_valid_move_set() {
    use std::collections::HashSet;

    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    // Allow movement to (1, 0).
    app.world_mut().insert_resource(ValidMoveSet {
        valid_positions: HashSet::from([HexPosition::new(1, 0)]),
        blocked_explanations: HashMap::new(),
        for_entity: Some(unit_entity),
    });

    app.add_observer(systems::handle_unit_interaction);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(1, 0),
    });
    app.update();

    let pos = app
        .world()
        .entity(unit_entity)
        .get::<HexPosition>()
        .expect("Unit should have HexPosition");
    assert_eq!(
        *pos,
        HexPosition::new(1, 0),
        "Unit should move to valid position"
    );
}

/// Interaction clears selection when the selected entity has been despawned.
#[test]
fn interaction_clears_selection_when_entity_despawned() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    app.world_mut().insert_resource(EditorTool::Select);
    app.init_resource::<ValidMoveSet>();

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types_by_role(EntityRole::Token)[0].id;

    let unit_entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.world_mut().insert_resource(SelectedUnit {
        entity: Some(unit_entity),
    });

    // Despawn the unit entity before the interaction.
    app.world_mut().despawn(unit_entity);

    app.add_observer(systems::handle_unit_interaction);

    // Click an empty tile — should hit the `units.get_mut(selected_entity)` Err path.
    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(1, 0),
    });
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert!(
        selected.entity.is_none(),
        "Selection should be cleared when entity is despawned"
    );
}

/// `sync_unit_materials` removes materials for deleted entity types.
#[test]
fn sync_unit_materials_removes_deleted_type() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.update();

    let initial_count = app.world().resource::<UnitMaterials>().materials.len();
    assert_eq!(initial_count, 2);

    // Remove one Token type from the registry.
    app.world_mut()
        .resource_mut::<EntityTypeRegistry>()
        .types
        .retain(|et| et.name != "Cavalry");

    app.add_systems(Update, systems::sync_unit_materials);
    app.update();

    let unit_mats = app.world().resource::<UnitMaterials>();
    assert_eq!(
        unit_mats.materials.len(),
        1,
        "Should have 1 material after removing a type"
    );
}

/// `assign_unit_visuals` adds mesh and material to bare unit entities.
#[test]
fn assign_unit_visuals_adds_mesh_to_bare_units() {
    let mut app = test_app();
    setup_unit_resources(&mut app);
    app.add_systems(Update, systems::assign_unit_visuals);
    app.update(); // Startup — creates UnitMaterials and UnitMesh.

    let registry = app.world().resource::<EntityTypeRegistry>();
    let first_id = registry.types[0].id;

    // Spawn a unit WITHOUT Mesh3d (as apply_pending_board_load does).
    let entity = app
        .world_mut()
        .spawn((
            UnitInstance,
            HexPosition::new(0, 0),
            EntityData {
                entity_type_id: first_id,
                properties: HashMap::new(),
            },
            Transform::default(),
        ))
        .id();

    app.update(); // assign_unit_visuals runs.
    app.update(); // Deferred commands applied.

    assert!(
        app.world().get::<Mesh3d>(entity).is_some(),
        "Unit should have Mesh3d after assign_unit_visuals"
    );
    assert!(
        app.world()
            .get::<MeshMaterial3d<StandardMaterial>>(entity)
            .is_some(),
        "Unit should have MeshMaterial3d after assign_unit_visuals"
    );
}
