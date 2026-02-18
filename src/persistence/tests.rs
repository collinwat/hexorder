//! Tests for the persistence plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityData, EntityRole, EntityType, EntityTypeRegistry, EnumRegistry, GameSystem,
    StructRegistry, TypeId, UnitInstance,
};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use crate::contracts::mechanics::{CombatModifierRegistry, CombatResultsTable, TurnStructure};
use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use crate::contracts::persistence::{
    AppScreen, FORMAT_VERSION, GameSystemFile, PendingBoardLoad, TileSaveData, UnitSaveData,
};

/// Helper: build a headless app with persistence and game system plugins.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_plugins(crate::game_system::GameSystemPlugin);
    // ShortcutRegistry must exist before PersistencePlugin (registers shortcuts in build).
    app.init_resource::<crate::contracts::shortcuts::ShortcutRegistry>();
    app.add_plugins(crate::persistence::PersistencePlugin);
    app
}

/// Helper: create a minimal `GameSystemFile` for testing.
fn test_game_system_file() -> GameSystemFile {
    let type_id = TypeId::new();
    GameSystemFile {
        format_version: FORMAT_VERSION,
        name: "Test Project".to_string(),
        game_system: GameSystem {
            id: "test-save".to_string(),
            version: "0.1.0".to_string(),
        },
        entity_types: EntityTypeRegistry {
            types: vec![EntityType {
                id: type_id,
                name: "TestTerrain".to_string(),
                role: EntityRole::BoardPosition,
                color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
                properties: Vec::new(),
            }],
        },
        enums: EnumRegistry::default(),
        structs: StructRegistry::default(),
        concepts: ConceptRegistry::default(),
        relations: RelationRegistry::default(),
        constraints: ConstraintRegistry::default(),
        turn_structure: TurnStructure::default(),
        combat_results_table: CombatResultsTable::default(),
        combat_modifiers: CombatModifierRegistry::default(),
        map_radius: 5,
        tiles: vec![TileSaveData {
            position: HexPosition::new(0, 0),
            entity_type_id: type_id,
            properties: HashMap::new(),
        }],
        units: vec![UnitSaveData {
            position: HexPosition::new(1, 0),
            entity_type_id: type_id,
            properties: HashMap::new(),
        }],
    }
}

/// `apply_pending_board_load` matches tile data by position and spawns units.
#[test]
fn apply_pending_board_load_maps_tiles_and_spawns_units() {
    let mut app = test_app();

    // Manually insert HexGridConfig (normally from HexGridPlugin).
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });

    app.update(); // Startup

    let file = test_game_system_file();
    let tile_type_id = file.entity_types.types[0].id;

    // Spawn a tile entity to match against.
    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(), // Will be overwritten by load
            properties: HashMap::new(),
        },
    ));

    // Insert PendingBoardLoad.
    app.insert_resource(PendingBoardLoad {
        tiles: file.tiles.clone(),
        units: file.units.clone(),
    });

    app.update(); // apply_pending_board_load runs

    // Verify tile data was applied.
    let mut tile_query = app
        .world_mut()
        .query_filtered::<&EntityData, With<HexTile>>();
    let tile_data: Vec<_> = tile_query.iter(app.world()).collect();
    assert_eq!(tile_data.len(), 1);
    assert_eq!(tile_data[0].entity_type_id, tile_type_id);

    // Verify unit was spawned.
    let mut unit_query = app
        .world_mut()
        .query_filtered::<(&HexPosition, &EntityData), With<UnitInstance>>();
    let units: Vec<_> = unit_query.iter(app.world()).collect();
    assert_eq!(units.len(), 1);
    assert_eq!(*units[0].0, HexPosition::new(1, 0));

    // Verify PendingBoardLoad was removed.
    assert!(
        app.world().get_resource::<PendingBoardLoad>().is_none(),
        "PendingBoardLoad should be removed after application"
    );
}
