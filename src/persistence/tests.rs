//! Tests for the persistence plugin.

use std::collections::HashMap;

use bevy::prelude::*;

use hexorder_contracts::game_system::{
    EntityData, EntityRole, EntityType, EntityTypeRegistry, EnumRegistry, GameSystem,
    StructRegistry, TypeId, UnitInstance,
};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use hexorder_contracts::mechanics::{CombatModifierRegistry, CombatResultsTable, TurnStructure};
use hexorder_contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use hexorder_contracts::persistence::{
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
    app.init_resource::<hexorder_contracts::shortcuts::ShortcutRegistry>();
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
        workspace_preset: String::new(),
        font_size_base: 15.0,
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

    // Spawn a tile entity with default EntityData (mirrors the state after
    // assign_default_cell_data has run — the system waits for this).
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

/// `apply_pending_board_load` defers when tiles lack `EntityData`.
#[test]
fn apply_pending_board_load_defers_until_tiles_have_entity_data() {
    let mut app = test_app();

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

    // Spawn tile WITHOUT EntityData (mirrors spawn_grid).
    let tile_entity = app
        .world_mut()
        .spawn((HexTile, HexPosition::new(0, 0)))
        .id();

    app.insert_resource(PendingBoardLoad {
        tiles: file.tiles.clone(),
        units: file.units.clone(),
    });

    app.update(); // System defers — tiles lack EntityData.

    assert!(
        app.world().get_resource::<PendingBoardLoad>().is_some(),
        "PendingBoardLoad should remain when tiles lack EntityData"
    );

    // Simulate assign_default_cell_data adding EntityData.
    app.world_mut().entity_mut(tile_entity).insert(EntityData {
        entity_type_id: TypeId::new(),
        properties: HashMap::new(),
    });

    app.update(); // System proceeds — tiles now have EntityData.

    // Verify tile data was overwritten with saved data.
    let tile_data = app.world().get::<EntityData>(tile_entity);
    assert!(tile_data.is_some(), "Tile should have EntityData");
    assert_eq!(tile_data.expect("checked").entity_type_id, tile_type_id);

    // Verify PendingBoardLoad was consumed.
    assert!(
        app.world().get_resource::<PendingBoardLoad>().is_none(),
        "PendingBoardLoad should be removed after application"
    );
}

/// `GameSystemFile` `workspace_preset` defaults to empty for backward compat.
#[test]
fn game_system_file_workspace_preset_defaults_on_deserialize() {
    // Minimal RON without workspace_preset (simulates v3 file).
    let file = test_game_system_file();
    let ron_str =
        ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()).expect("serialize");

    // Remove workspace_preset from serialized RON to simulate old file.
    let without_preset = ron_str.replace("    workspace_preset: \"\",\n", "");

    let loaded: GameSystemFile =
        ron::from_str(&without_preset).expect("deserialize without workspace_preset");
    assert!(loaded.workspace_preset.is_empty());
}

/// Round-trip: `workspace_preset` survives serialization.
#[test]
fn game_system_file_workspace_preset_round_trip() {
    let mut file = test_game_system_file();
    file.workspace_preset = "playtesting".to_string();

    let ron_str =
        ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()).expect("serialize");
    let loaded: GameSystemFile = ron::from_str(&ron_str).expect("deserialize");

    assert_eq!(loaded.workspace_preset, "playtesting");
}

/// `sync_dirty_flag` sets `workspace.dirty` when `UndoStack` has new records.
#[test]
fn sync_dirty_flag_sets_dirty_on_new_records() {
    use hexorder_contracts::undo_redo::UndoStack;

    let mut app = test_app();
    app.init_resource::<UndoStack>();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Record a command.
    app.world_mut().resource_mut::<UndoStack>().record(Box::new(
        hexorder_contracts::undo_redo::SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: hexorder_contracts::game_system::PropertyValue::Int(0),
            new_value: hexorder_contracts::game_system::PropertyValue::Int(1),
            label: "test".to_string(),
        },
    ));

    app.update(); // sync_dirty_flag runs

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert!(workspace.dirty, "workspace should be dirty after record");

    // Flag should be acknowledged.
    let stack = app.world().resource::<UndoStack>();
    assert!(
        !stack.has_new_records(),
        "has_new_records should be cleared after sync"
    );
}

/// `sync_window_title` shows "Hexorder" when workspace has no name.
#[test]
fn sync_window_title_shows_app_name_when_no_project() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Workspace::default());
    app.world_mut().spawn(Window {
        title: "initial".to_string(),
        ..default()
    });
    app.add_systems(Update, super::systems::sync_window_title);
    app.update();

    let mut q = app.world_mut().query::<&Window>();
    let window = q.single(app.world()).expect("one window");
    assert_eq!(window.title, "Hexorder");
}

/// `sync_window_title` shows project name when clean.
#[test]
fn sync_window_title_shows_project_name_when_clean() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Workspace {
        name: "MyProject".to_string(),
        dirty: false,
        ..default()
    });
    app.world_mut().spawn(Window {
        title: "initial".to_string(),
        ..default()
    });
    app.add_systems(Update, super::systems::sync_window_title);
    app.update();

    let mut q = app.world_mut().query::<&Window>();
    let window = q.single(app.world()).expect("one window");
    assert_eq!(window.title, "Hexorder \u{2014} MyProject");
}

/// `sync_window_title` appends asterisk when dirty.
#[test]
fn sync_window_title_shows_asterisk_when_dirty() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Workspace {
        name: "MyProject".to_string(),
        dirty: true,
        ..default()
    });
    app.world_mut().spawn(Window {
        title: "initial".to_string(),
        ..default()
    });
    app.add_systems(Update, super::systems::sync_window_title);
    app.update();

    let mut q = app.world_mut().query::<&Window>();
    let window = q.single(app.world()).expect("one window");
    assert_eq!(window.title, "Hexorder \u{2014} MyProject*");
}

/// Format version was bumped to 5 for `font_size_base` field.
#[test]
fn format_version_is_5() {
    assert_eq!(FORMAT_VERSION, 5);
}
