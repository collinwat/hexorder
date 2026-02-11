//! Unit tests for the cell feature plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::game_system::{ActiveCellType, CellData, CellType, CellTypeRegistry, TypeId};
use crate::contracts::hex_grid::{HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial};

use super::components::CellMaterials;
use super::systems;

/// Helper: create a minimal App with resources needed for cell testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app
}

/// Helper: create a test registry with 3 cell types.
fn test_registry() -> CellTypeRegistry {
    CellTypeRegistry {
        types: vec![
            CellType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                color: Color::srgb(0.6, 0.8, 0.4),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Forest".to_string(),
                color: Color::srgb(0.2, 0.5, 0.2),
                properties: Vec::new(),
            },
            CellType {
                id: TypeId::new(),
                name: "Water".to_string(),
                color: Color::srgb(0.2, 0.4, 0.8),
                properties: Vec::new(),
            },
        ],
        enum_definitions: Vec::new(),
    }
}

/// Helper: insert a test registry and run setup_cell_materials.
fn setup_cell_resources(app: &mut App) {
    let registry = test_registry();
    app.insert_resource(registry);
    app.add_systems(Startup, systems::setup_cell_materials);
}

/// Helper: spawn a single hex tile entity with the given position.
fn spawn_test_tile(app: &mut App, q: i32, r: i32) -> Entity {
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        });
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Mesh::from(Cuboid::new(1.0, 0.1, 1.0)));

    app.world_mut()
        .spawn((
            HexTile,
            HexPosition::new(q, r),
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            TileBaseMaterial(material),
            Transform::default(),
        ))
        .id()
}

#[test]
fn cell_materials_created_for_all_types() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let cell_mats = app
        .world()
        .get_resource::<CellMaterials>()
        .expect("CellMaterials should exist after Startup");

    let registry = app.world().resource::<CellTypeRegistry>();
    assert_eq!(
        cell_mats.materials.len(),
        registry.types.len(),
        "Should have a material for each cell type"
    );

    for vt in &registry.types {
        assert!(
            cell_mats.get(vt.id).is_some(),
            "Material should exist for cell type '{}'",
            vt.name
        );
    }
}

#[test]
fn assign_default_cell_data_adds_to_tiles() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    // Spawn tiles without CellData.
    spawn_test_tile(&mut app, 0, 0);
    spawn_test_tile(&mut app, 1, 0);
    spawn_test_tile(&mut app, 0, 1);

    // Run assign_default_cell_data.
    app.add_systems(Update, systems::assign_default_cell_data);
    app.update();

    let registry = app.world().resource::<CellTypeRegistry>();
    let first_id = registry.first().unwrap().id;

    let mut query = app.world_mut().query_filtered::<&CellData, With<HexTile>>();
    let cell_data: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(cell_data.len(), 3, "All 3 tiles should have CellData");
    for cd in cell_data {
        assert_eq!(
            cd.cell_type_id, first_id,
            "Default cell type should be the first in registry"
        );
    }
}

#[test]
fn paint_cell_changes_tile_type() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let registry = app.world().resource::<CellTypeRegistry>();
    let first_id = registry.types[0].id;
    let second_id = registry.types[1].id;

    // Spawn a tile with default cell data.
    let tile_entity = spawn_test_tile(&mut app, 2, 3);
    app.world_mut().entity_mut(tile_entity).insert(CellData {
        cell_type_id: first_id,
        properties: HashMap::new(),
    });

    // Set active cell type to the second type and tool to Paint mode.
    app.world_mut().insert_resource(ActiveCellType {
        cell_type_id: Some(second_id),
    });
    app.world_mut().insert_resource(EditorTool::Paint);

    app.add_observer(systems::paint_cell);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(2, 3),
    });
    app.update();

    let cell_data = app
        .world()
        .entity(tile_entity)
        .get::<CellData>()
        .expect("Tile should have CellData");

    assert_eq!(
        cell_data.cell_type_id, second_id,
        "Cell type should have been painted to the second type"
    );
}

#[test]
fn paint_does_not_affect_other_tiles() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let registry = app.world().resource::<CellTypeRegistry>();
    let first_id = registry.types[0].id;
    let third_id = registry.types[2].id;

    let tile_a = spawn_test_tile(&mut app, 0, 0);
    let tile_b = spawn_test_tile(&mut app, 1, 1);
    app.world_mut().entity_mut(tile_a).insert(CellData {
        cell_type_id: first_id,
        properties: HashMap::new(),
    });
    app.world_mut().entity_mut(tile_b).insert(CellData {
        cell_type_id: first_id,
        properties: HashMap::new(),
    });

    app.world_mut().insert_resource(ActiveCellType {
        cell_type_id: Some(third_id),
    });
    app.world_mut().insert_resource(EditorTool::Paint);

    app.add_observer(systems::paint_cell);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let cd_a = app.world().entity(tile_a).get::<CellData>().unwrap();
    let cd_b = app.world().entity(tile_b).get::<CellData>().unwrap();

    assert_eq!(cd_a.cell_type_id, third_id);
    assert_eq!(
        cd_b.cell_type_id, first_id,
        "Unpainted tile should remain with first cell type"
    );
}

#[test]
fn paint_skipped_in_select_mode() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let registry = app.world().resource::<CellTypeRegistry>();
    let first_id = registry.types[0].id;
    let second_id = registry.types[1].id;

    let tile_entity = spawn_test_tile(&mut app, 0, 0);
    app.world_mut().entity_mut(tile_entity).insert(CellData {
        cell_type_id: first_id,
        properties: HashMap::new(),
    });

    app.world_mut().insert_resource(ActiveCellType {
        cell_type_id: Some(second_id),
    });
    app.world_mut().insert_resource(EditorTool::Select);

    app.add_observer(systems::paint_cell);

    app.world_mut().commands().trigger(HexSelectedEvent {
        position: HexPosition::new(0, 0),
    });
    app.update();

    let cell_data = app.world().entity(tile_entity).get::<CellData>().unwrap();
    assert_eq!(
        cell_data.cell_type_id, first_id,
        "Cell type should remain unchanged when tool is Select"
    );
}

#[test]
fn sync_cell_visuals_updates_material() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let registry = app.world().resource::<CellTypeRegistry>();
    let first_id = registry.types[0].id;

    let tile_entity = spawn_test_tile(&mut app, 0, 0);
    app.world_mut().entity_mut(tile_entity).insert(CellData {
        cell_type_id: first_id,
        properties: HashMap::new(),
    });

    app.add_systems(Update, systems::sync_cell_visuals);
    app.update();

    let expected_handle = {
        let cell_mats = app.world().resource::<CellMaterials>();
        cell_mats
            .get(first_id)
            .expect("Material should exist for first cell type")
            .clone()
    };

    let tile_material = app
        .world()
        .entity(tile_entity)
        .get::<MeshMaterial3d<StandardMaterial>>()
        .expect("Tile should have MeshMaterial3d");

    assert_eq!(
        tile_material.0, expected_handle,
        "Tile material should match the cell type material"
    );
}

#[test]
fn cell_materials_lookup_works() {
    let mut materials_map = HashMap::new();
    let id_a = TypeId::new();
    let id_b = TypeId::new();
    let dummy_handle = Handle::<StandardMaterial>::default();
    materials_map.insert(id_a, dummy_handle);

    let cell_mats = CellMaterials {
        materials: materials_map,
    };

    assert!(cell_mats.get(id_a).is_some());
    assert!(cell_mats.get(id_b).is_none());
}

#[test]
fn sync_cell_materials_adds_new_type() {
    let mut app = test_app();
    setup_cell_resources(&mut app);
    app.update();

    let initial_count = app.world().resource::<CellMaterials>().materials.len();
    assert_eq!(initial_count, 3);

    // Add a new cell type to the registry.
    let new_id = TypeId::new();
    app.world_mut()
        .resource_mut::<CellTypeRegistry>()
        .types
        .push(CellType {
            id: new_id,
            name: "Desert".to_string(),
            color: Color::srgb(0.9, 0.8, 0.5),
            properties: Vec::new(),
        });

    app.add_systems(Update, systems::sync_cell_materials);
    app.update();

    let cell_mats = app.world().resource::<CellMaterials>();
    assert_eq!(
        cell_mats.materials.len(),
        4,
        "Should have 4 materials after adding a new type"
    );
    assert!(
        cell_mats.get(new_id).is_some(),
        "New type should have a material"
    );
}
