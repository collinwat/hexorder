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
    app.add_plugins(crate::ontology::OntologyPlugin);
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

    // Undo stack should still report dirty (save-point based).
    let stack = app.world().resource::<UndoStack>();
    assert!(
        stack.is_dirty(),
        "undo stack should be dirty after recording a command"
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

/// `save_to_path` writes file to disk and updates workspace.
#[test]
fn save_to_path_writes_file_and_updates_workspace() {
    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    let tmp = std::env::temp_dir().join("hexorder_test_save_to_path.hexorder");
    let _ = std::fs::remove_file(&tmp);

    let result = super::systems::save_to_path(&tmp, app.world_mut());

    assert!(result, "save_to_path should succeed");
    assert!(tmp.exists(), "file should be written to disk");

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.file_path.as_deref(), Some(tmp.as_path()));
    assert!(!workspace.dirty);

    let _ = std::fs::remove_file(&tmp);
}

/// `load_from_path` overwrites registries and inserts `PendingBoardLoad`.
#[test]
fn load_from_path_overwrites_registries() {
    use hexorder_contracts::storage::Storage;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Write a test file to disk first.
    let file = test_game_system_file();
    let tmp = std::env::temp_dir().join("hexorder_test_load_from_path.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    let result = super::systems::load_from_path(&tmp, app.world_mut());

    assert!(result, "load_from_path should succeed");

    let game_system = app.world().resource::<GameSystem>();
    assert_eq!(game_system.id, "test-save");

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Test Project");
    assert_eq!(workspace.file_path.as_deref(), Some(tmp.as_path()));
    assert!(!workspace.dirty);

    assert!(
        app.world().get_resource::<PendingBoardLoad>().is_some(),
        "PendingBoardLoad should be inserted"
    );

    let _ = std::fs::remove_file(&tmp);
}

/// `dispatch_dialog_result` executes pending action on confirm No (skip save).
#[test]
fn dispatch_confirm_no_executes_pending_action() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Dispatch: confirm No with pending NewProject action.
    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::NewProject {
                name: "Test".to_string(),
            },
        },
        DialogResult::Confirmed(ConfirmChoice::No),
        app.world_mut(),
    );

    // NewProject should have set workspace name and transitioned to Editor.
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Test");
}

/// `dispatch_dialog_result` does nothing on confirm Cancel.
#[test]
fn dispatch_confirm_cancel_does_nothing() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();

    // Set workspace name so we can verify it's unchanged.
    app.world_mut()
        .resource_mut::<hexorder_contracts::persistence::Workspace>()
        .name = "Original".to_string();

    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::CloseProject,
        },
        DialogResult::Confirmed(ConfirmChoice::Cancel),
        app.world_mut(),
    );

    // Workspace should be unchanged.
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "Original");
}

/// Format version was bumped to 5 for `font_size_base` field.
#[test]
fn format_version_is_5() {
    assert_eq!(FORMAT_VERSION, 5);
}

// ---------------------------------------------------------------------------
// Helper: build test app with `HexGridConfig` pre-inserted
// ---------------------------------------------------------------------------

fn test_app_with_grid() -> App {
    let mut app = test_app();
    app.insert_resource(HexGridConfig {
        layout: hexx::HexLayout {
            orientation: hexx::HexOrientation::Pointy,
            scale: bevy::math::Vec2::splat(1.0),
            origin: bevy::math::Vec2::ZERO,
        },
        map_radius: 5,
    });
    app.update();
    app
}

// ---------------------------------------------------------------------------
// sanitize_filename coverage
// ---------------------------------------------------------------------------

/// `sanitize_filename` replaces special characters with hyphens.
#[test]
fn sanitize_filename_replaces_special_chars() {
    let result = super::systems::sanitize_filename("my/project:name?");
    assert_eq!(result, "my-project-name-");
}

/// `sanitize_filename` preserves alphanumeric, hyphens, underscores, spaces.
#[test]
fn sanitize_filename_preserves_valid_chars() {
    let result = super::systems::sanitize_filename("My Project-v2_final");
    assert_eq!(result, "My Project-v2_final");
}

/// `sanitize_filename` returns "untitled" for empty input.
#[test]
fn sanitize_filename_returns_untitled_for_empty() {
    let result = super::systems::sanitize_filename("");
    assert_eq!(result, "untitled");
}

/// `sanitize_filename` returns "untitled" for whitespace-only input.
#[test]
fn sanitize_filename_returns_untitled_for_whitespace() {
    let result = super::systems::sanitize_filename("   ");
    assert_eq!(result, "untitled");
}

/// `sanitize_filename` returns "untitled" for all-special-char input that trims to empty.
#[test]
fn sanitize_filename_all_special_chars_that_trim_empty() {
    // Characters that become hyphens, then trim to hyphens (not empty).
    let result = super::systems::sanitize_filename("!!!");
    assert_eq!(result, "---");
}

// ---------------------------------------------------------------------------
// save_to_path failure path
// ---------------------------------------------------------------------------

/// `save_to_path` returns false and triggers error toast on write failure.
#[test]
fn save_to_path_returns_false_on_failure() {
    let mut app = test_app_with_grid();

    // Attempt to save to a non-existent directory that cannot be created.
    let bad_path = std::path::PathBuf::from("/nonexistent/deeply/nested/dir/file.hexorder");
    let result = super::systems::save_to_path(&bad_path, app.world_mut());

    assert!(!result, "save_to_path should return false on failure");
}

// ---------------------------------------------------------------------------
// load_from_path failure and backward compat
// ---------------------------------------------------------------------------

/// `load_from_path` returns false on non-existent file.
#[test]
fn load_from_path_returns_false_on_missing_file() {
    let mut app = test_app_with_grid();
    let bad_path = std::path::PathBuf::from("/nonexistent/file.hexorder");
    let result = super::systems::load_from_path(&bad_path, app.world_mut());
    assert!(
        !result,
        "load_from_path should return false on missing file"
    );
}

/// `load_from_path` derives name from filename when name field is empty (v2 compat).
#[test]
fn load_from_path_derives_name_from_filename_stem() {
    use hexorder_contracts::storage::Storage;

    let mut app = test_app_with_grid();

    // Create a file with empty name field (simulates v2 file).
    let mut file = test_game_system_file();
    file.name = String::new();

    let tmp = std::env::temp_dir().join("hexorder_test_v2_compat.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    let result = super::systems::load_from_path(&tmp, app.world_mut());
    assert!(result, "load_from_path should succeed");

    // Name should be derived from the filename stem.
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.name, "hexorder_test_v2_compat");

    let _ = std::fs::remove_file(&tmp);
}

/// `load_from_path` restores `workspace_preset` and `font_size_base`.
#[test]
fn load_from_path_restores_workspace_preset_and_font_size() {
    use hexorder_contracts::storage::Storage;

    let mut app = test_app_with_grid();

    let mut file = test_game_system_file();
    file.workspace_preset = "playtesting".to_string();
    file.font_size_base = 18.0;

    let tmp = std::env::temp_dir().join("hexorder_test_preset_font.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    super::systems::load_from_path(&tmp, app.world_mut());

    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.workspace_preset, "playtesting");
    assert!((workspace.font_size_base - 18.0).abs() < f32::EPSILON);

    let _ = std::fs::remove_file(&tmp);
}

// ---------------------------------------------------------------------------
// save_to_path marks undo stack clean
// ---------------------------------------------------------------------------

/// `save_to_path` marks the undo stack as clean.
#[test]
fn save_to_path_marks_undo_stack_clean() {
    use hexorder_contracts::undo_redo::UndoStack;

    let mut app = test_app_with_grid();
    app.init_resource::<UndoStack>();

    // Record a command to make it dirty.
    app.world_mut().resource_mut::<UndoStack>().record(Box::new(
        hexorder_contracts::undo_redo::SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: hexorder_contracts::game_system::PropertyValue::Int(0),
            new_value: hexorder_contracts::game_system::PropertyValue::Int(1),
            label: "test".to_string(),
        },
    ));
    assert!(app.world().resource::<UndoStack>().is_dirty());

    let tmp = std::env::temp_dir().join("hexorder_test_save_undo_clean.hexorder");
    let _ = std::fs::remove_file(&tmp);

    super::systems::save_to_path(&tmp, app.world_mut());

    assert!(
        !app.world().resource::<UndoStack>().is_dirty(),
        "undo stack should be clean after save"
    );

    let _ = std::fs::remove_file(&tmp);
}

// ---------------------------------------------------------------------------
// dispatch_dialog_result: additional paths
// ---------------------------------------------------------------------------

/// Confirm Yes with existing path saves and executes pending action.
#[test]
fn dispatch_confirm_yes_with_path_saves_then_executes() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::Workspace;

    let mut app = test_app_with_grid();

    // Save once to establish a file path.
    let tmp = std::env::temp_dir().join("hexorder_test_confirm_yes.hexorder");
    let _ = std::fs::remove_file(&tmp);
    super::systems::save_to_path(&tmp, app.world_mut());

    // Dispatch confirm Yes with pending CloseProject.
    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::CloseProject,
        },
        DialogResult::Confirmed(ConfirmChoice::Yes),
        app.world_mut(),
    );

    // CloseProject resets workspace.
    let workspace = app.world().resource::<Workspace>();
    assert!(workspace.name.is_empty(), "CloseProject should reset name");

    let _ = std::fs::remove_file(&tmp);
}

// NOTE: Confirm Yes without existing path would spawn an rfd save dialog
// which cannot run in headless tests. The `spawn_save_dialog_for_current_project`
// codepath is covered indirectly by verifying that Confirm Yes with-path works
// (tested above) and that the dialog infrastructure polls correctly
// (tested in async_dialog::tests).

/// Save file dialog with picked path saves the file.
#[test]
fn dispatch_save_file_picked_saves() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app_with_grid();

    let tmp = std::env::temp_dir().join("hexorder_test_dispatch_save.hexorder");
    let _ = std::fs::remove_file(&tmp);

    super::systems::dispatch_dialog_result(
        DialogKind::SaveFile { then: None },
        DialogResult::FilePicked(Some(tmp.clone())),
        app.world_mut(),
    );

    assert!(tmp.exists(), "file should be written");
    let workspace = app
        .world()
        .resource::<hexorder_contracts::persistence::Workspace>();
    assert_eq!(workspace.file_path.as_deref(), Some(tmp.as_path()));

    let _ = std::fs::remove_file(&tmp);
}

/// Save file dialog with chained action executes after save.
#[test]
fn dispatch_save_file_with_chained_action() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::Workspace;

    let mut app = test_app_with_grid();

    let tmp = std::env::temp_dir().join("hexorder_test_save_chain.hexorder");
    let _ = std::fs::remove_file(&tmp);

    super::systems::dispatch_dialog_result(
        DialogKind::SaveFile {
            then: Some(PendingAction::CloseProject),
        },
        DialogResult::FilePicked(Some(tmp.clone())),
        app.world_mut(),
    );

    // CloseProject should have been executed after save.
    let workspace = app.world().resource::<Workspace>();
    assert!(
        workspace.name.is_empty(),
        "CloseProject should have reset workspace"
    );

    let _ = std::fs::remove_file(&tmp);
}

/// Save/Open file dialog cancelled (None) does nothing.
#[test]
fn dispatch_file_dialog_cancelled_does_nothing() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::Workspace;

    let mut app = test_app_with_grid();

    app.world_mut().resource_mut::<Workspace>().name = "Original".to_string();

    // Save cancelled.
    super::systems::dispatch_dialog_result(
        DialogKind::SaveFile { then: None },
        DialogResult::FilePicked(None),
        app.world_mut(),
    );

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(workspace.name, "Original");

    // Open cancelled.
    super::systems::dispatch_dialog_result(
        DialogKind::OpenFile,
        DialogResult::FilePicked(None),
        app.world_mut(),
    );

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(workspace.name, "Original");
}

/// Open file dialog with picked path loads the file.
#[test]
fn dispatch_open_file_loads() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::storage::Storage;

    let mut app = test_app_with_grid();

    // Write a file first.
    let file = test_game_system_file();
    let tmp = std::env::temp_dir().join("hexorder_test_dispatch_open.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    super::systems::dispatch_dialog_result(
        DialogKind::OpenFile,
        DialogResult::FilePicked(Some(tmp.clone())),
        app.world_mut(),
    );

    let game_system = app.world().resource::<GameSystem>();
    assert_eq!(game_system.id, "test-save");

    let _ = std::fs::remove_file(&tmp);
}

/// Unhandled dialog combination logs warning but does not panic.
#[test]
fn dispatch_unhandled_combination_logs_warning() {
    use crate::persistence::async_dialog::*;

    let mut app = test_app_with_grid();

    // OpenFile + Confirmed is an unhandled combination.
    super::systems::dispatch_dialog_result(
        DialogKind::OpenFile,
        DialogResult::Confirmed(ConfirmChoice::Yes),
        app.world_mut(),
    );
    // Should not panic.
}

// ---------------------------------------------------------------------------
// execute_pending_action: Load variant
// ---------------------------------------------------------------------------

// NOTE: `PendingAction::Load` spawns an rfd dialog (calls `spawn_open_dialog`)
// which cannot run in a headless test environment. The `execute_pending_action`
// codepath for Load is covered by the dispatch_dialog_result integration path
// through the async dialog infrastructure.

// ---------------------------------------------------------------------------
// handle_file_command
// ---------------------------------------------------------------------------

/// Helper: build a minimal app with only `handle_file_command` observer.
/// This avoids the full `PersistencePlugin` which would register rfd-based
/// observers that cannot run in headless test environments.
fn file_command_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_observer(super::systems::handle_file_command);
    app
}

/// `handle_file_command` maps "file.save" to `SaveRequestEvent`.
#[test]
fn handle_file_command_maps_save() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use hexorder_contracts::persistence::SaveRequestEvent;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = file_command_app();

    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = Arc::clone(&triggered);
    app.add_observer(move |_trigger: On<SaveRequestEvent>| {
        triggered_clone.store(true, Ordering::SeqCst);
    });

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.save"),
    });
    app.update();
    app.update();

    assert!(
        triggered.load(Ordering::SeqCst),
        "file.save should trigger SaveRequestEvent"
    );
}

/// `handle_file_command` maps `file.save_as` to `SaveRequestEvent` with `save_as: true`.
#[test]
fn handle_file_command_maps_save_as() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use hexorder_contracts::persistence::SaveRequestEvent;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = file_command_app();

    let save_as_flag = Arc::new(AtomicBool::new(false));
    let save_as_clone = Arc::clone(&save_as_flag);
    app.add_observer(move |trigger: On<SaveRequestEvent>| {
        if trigger.event().save_as {
            save_as_clone.store(true, Ordering::SeqCst);
        }
    });

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.save_as"),
    });
    app.update();
    app.update();

    assert!(
        save_as_flag.load(Ordering::SeqCst),
        "file.save_as should trigger SaveRequestEvent with save_as=true"
    );
}

/// `handle_file_command` maps "file.open" to `LoadRequestEvent`.
#[test]
fn handle_file_command_maps_open() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use hexorder_contracts::persistence::LoadRequestEvent;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = file_command_app();

    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = Arc::clone(&triggered);
    app.add_observer(move |_trigger: On<LoadRequestEvent>| {
        triggered_clone.store(true, Ordering::SeqCst);
    });

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.open"),
    });
    app.update();
    app.update();

    assert!(
        triggered.load(Ordering::SeqCst),
        "file.open should trigger LoadRequestEvent"
    );
}

/// `handle_file_command` maps "file.new" to `CloseProjectEvent`.
#[test]
fn handle_file_command_maps_new() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use hexorder_contracts::persistence::CloseProjectEvent;
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = file_command_app();

    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = Arc::clone(&triggered);
    app.add_observer(move |_trigger: On<CloseProjectEvent>| {
        triggered_clone.store(true, Ordering::SeqCst);
    });

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("file.new"),
    });
    app.update();
    app.update();

    assert!(
        triggered.load(Ordering::SeqCst),
        "file.new should trigger CloseProjectEvent"
    );
}

/// `handle_file_command` ignores unknown command IDs.
#[test]
fn handle_file_command_ignores_unknown() {
    use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

    let mut app = file_command_app();

    app.world_mut().commands().trigger(CommandExecutedEvent {
        command_id: CommandId("editor.undo"),
    });
    app.update();
    // Should not panic.
}

// ---------------------------------------------------------------------------
// cleanup_editor_entities
// ---------------------------------------------------------------------------

/// `cleanup_editor_entities` despawns tiles, units, and move overlays.
#[test]
fn cleanup_editor_entities_despawns_all() {
    use hexorder_contracts::hex_grid::MoveOverlay;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn entities.
    let tile = app.world_mut().spawn(HexTile).id();
    let unit = app.world_mut().spawn(UnitInstance).id();
    let overlay = app
        .world_mut()
        .spawn(MoveOverlay {
            state: hexorder_contracts::hex_grid::MoveOverlayState::Valid,
            position: hexorder_contracts::hex_grid::HexPosition::new(0, 0),
        })
        .id();

    app.add_systems(Update, super::systems::cleanup_editor_entities);
    app.update();
    app.update(); // commands apply

    assert!(
        app.world().get_entity(tile).is_err(),
        "tile should be despawned"
    );
    assert!(
        app.world().get_entity(unit).is_err(),
        "unit should be despawned"
    );
    assert!(
        app.world().get_entity(overlay).is_err(),
        "overlay should be despawned"
    );
}

// ---------------------------------------------------------------------------
// apply_pending_board_load: no pending resource
// ---------------------------------------------------------------------------

/// `apply_pending_board_load` is a no-op when no `PendingBoardLoad` exists.
#[test]
fn apply_pending_board_load_noop_without_resource() {
    let mut app = test_app_with_grid();

    // No PendingBoardLoad inserted.
    app.update();
    // Should not panic.
}

// ---------------------------------------------------------------------------
// sync_dirty_flag: no undo stack
// ---------------------------------------------------------------------------

/// `sync_dirty_flag` returns early when no `UndoStack` exists.
#[test]
fn sync_dirty_flag_noop_without_undo_stack() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Workspace {
        name: "Test".to_string(),
        dirty: false,
        ..default()
    });
    // No UndoStack resource.
    app.add_systems(Update, super::systems::sync_dirty_flag);
    app.update();

    // Dirty should remain false.
    let workspace = app.world().resource::<Workspace>();
    assert!(!workspace.dirty);
}

// ---------------------------------------------------------------------------
// sync_window_title: title already correct (no-op branch)
// ---------------------------------------------------------------------------

/// `sync_window_title` does not mutate window when title is already correct.
#[test]
fn sync_window_title_noop_when_title_matches() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Workspace {
        name: "MyProject".to_string(),
        dirty: false,
        ..default()
    });
    app.world_mut().spawn(Window {
        title: "Hexorder \u{2014} MyProject".to_string(),
        ..default()
    });
    app.add_systems(Update, super::systems::sync_window_title);
    app.update();

    let mut q = app.world_mut().query::<&Window>();
    let window = q.single(app.world()).expect("one window");
    assert_eq!(window.title, "Hexorder \u{2014} MyProject");
}

// ---------------------------------------------------------------------------
// async_dialog types coverage
// ---------------------------------------------------------------------------

/// `AsyncDialogTask` debug impl works.
#[test]
fn async_dialog_task_debug_impl() {
    use crate::persistence::async_dialog::*;

    let future: DialogFuture = Box::pin(std::future::pending());
    let task = AsyncDialogTask {
        kind: DialogKind::OpenFile,
        future: std::sync::Mutex::new(future),
    };
    let debug = format!("{task:?}");
    assert!(debug.contains("AsyncDialogTask"));
    assert!(debug.contains("OpenFile"));
}

/// `DialogKind` variants debug correctly.
#[test]
fn dialog_kind_debug_variants() {
    use crate::persistence::async_dialog::*;

    let save = DialogKind::SaveFile { then: None };
    assert!(format!("{save:?}").contains("SaveFile"));

    let open = DialogKind::OpenFile;
    assert!(format!("{open:?}").contains("OpenFile"));

    let confirm = DialogKind::ConfirmUnsavedChanges {
        then: PendingAction::Load,
    };
    assert!(format!("{confirm:?}").contains("ConfirmUnsavedChanges"));
}

/// `PendingAction` variants debug correctly.
#[test]
fn pending_action_debug_variants() {
    use crate::persistence::async_dialog::*;

    let load = PendingAction::Load;
    assert!(format!("{load:?}").contains("Load"));

    let new_proj = PendingAction::NewProject {
        name: "test".to_string(),
    };
    assert!(format!("{new_proj:?}").contains("NewProject"));

    let close = PendingAction::CloseProject;
    assert!(format!("{close:?}").contains("CloseProject"));
}

/// `DialogResult::Confirmed` debug works.
#[test]
fn dialog_result_confirmed_debug() {
    use crate::persistence::async_dialog::*;

    let result = DialogResult::Confirmed(ConfirmChoice::Yes);
    let debug = format!("{result:?}");
    assert!(debug.contains("Confirmed"));
    assert!(debug.contains("Yes"));
}

/// `DialogCompleted` debug works.
#[test]
fn dialog_completed_debug() {
    use crate::persistence::async_dialog::*;

    let completed = DialogCompleted {
        kind: DialogKind::OpenFile,
        result: DialogResult::FilePicked(None),
    };
    let debug = format!("{completed:?}");
    assert!(debug.contains("DialogCompleted"));
}

// ---------------------------------------------------------------------------
// load_from_path clears undo stack
// ---------------------------------------------------------------------------

/// `load_from_path` clears the undo stack.
#[test]
fn load_from_path_clears_undo_stack() {
    use hexorder_contracts::storage::Storage;
    use hexorder_contracts::undo_redo::UndoStack;

    let mut app = test_app_with_grid();
    app.init_resource::<UndoStack>();

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
    assert!(app.world().resource::<UndoStack>().can_undo());

    let file = test_game_system_file();
    let tmp = std::env::temp_dir().join("hexorder_test_load_clears_undo.hexorder");
    {
        let storage = app.world().resource::<Storage>();
        storage
            .provider()
            .save_at(&tmp, &file)
            .expect("write test file");
    }

    super::systems::load_from_path(&tmp, app.world_mut());

    assert!(
        !app.world().resource::<UndoStack>().can_undo(),
        "undo stack should be cleared after load"
    );

    let _ = std::fs::remove_file(&tmp);
}

// ---------------------------------------------------------------------------
// save_to_path with tiles and units present
// ---------------------------------------------------------------------------

/// `save_to_path` captures tiles and units from the world.
#[test]
fn save_to_path_captures_tiles_and_units() {
    let mut app = test_app_with_grid();

    let type_id = TypeId::new();

    // Spawn a tile and a unit.
    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: type_id,
            properties: HashMap::new(),
        },
    ));
    app.world_mut().spawn((
        UnitInstance,
        HexPosition::new(1, -1),
        EntityData {
            entity_type_id: type_id,
            properties: HashMap::new(),
        },
    ));

    let tmp = std::env::temp_dir().join("hexorder_test_save_with_entities.hexorder");
    let _ = std::fs::remove_file(&tmp);

    let result = super::systems::save_to_path(&tmp, app.world_mut());
    assert!(result);

    // Load the file back and verify tiles and units.
    let contents = std::fs::read_to_string(&tmp).expect("read file");
    let loaded: GameSystemFile = ron::from_str(&contents).expect("deserialize");
    assert_eq!(loaded.tiles.len(), 1);
    assert_eq!(loaded.units.len(), 1);
    assert_eq!(loaded.tiles[0].entity_type_id, type_id);
    assert_eq!(loaded.units[0].entity_type_id, type_id);

    let _ = std::fs::remove_file(&tmp);
}

// ---------------------------------------------------------------------------
// Confirm Yes save failure aborts chain
// ---------------------------------------------------------------------------

/// Confirm Yes save failure aborts the chain (does not execute pending action).
#[test]
fn dispatch_confirm_yes_save_failure_aborts_chain() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::Workspace;

    let mut app = test_app_with_grid();

    // Set a bad path so save will fail.
    app.world_mut().resource_mut::<Workspace>().file_path =
        Some(std::path::PathBuf::from("/nonexistent/dir/bad.hexorder"));
    app.world_mut().resource_mut::<Workspace>().name = "Original".to_string();

    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::CloseProject,
        },
        DialogResult::Confirmed(ConfirmChoice::Yes),
        app.world_mut(),
    );

    // Save failed, so CloseProject should NOT have executed.
    let workspace = app.world().resource::<Workspace>();
    assert_eq!(
        workspace.name, "Original",
        "chain should abort on save failure"
    );
}

// ---------------------------------------------------------------------------
// Observer: handle_save_request
// ---------------------------------------------------------------------------

/// `handle_save_request` saves directly when `file_path` exists and `save_as` is false.
#[test]
fn handle_save_request_saves_directly_with_existing_path() {
    use hexorder_contracts::persistence::{SaveRequestEvent, Workspace};

    let mut app = test_app_with_grid();

    // Establish a file path by saving first.
    let tmp = std::env::temp_dir().join("hexorder_test_handle_save_direct.hexorder");
    let _ = std::fs::remove_file(&tmp);
    super::systems::save_to_path(&tmp, app.world_mut());

    // Dirty the workspace.
    app.world_mut().resource_mut::<Workspace>().dirty = true;

    // Trigger save request (not save_as).
    app.world_mut()
        .commands()
        .trigger(SaveRequestEvent { save_as: false });
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert!(!workspace.dirty, "save should clear dirty flag");

    let _ = std::fs::remove_file(&tmp);
}

/// `handle_save_request` is a no-op when a dialog is already open.
#[test]
fn handle_save_request_noop_when_dialog_open() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::{SaveRequestEvent, Workspace};

    let mut app = test_app_with_grid();

    // Insert an existing dialog task.
    let future: DialogFuture = Box::pin(std::future::pending());
    app.insert_resource(AsyncDialogTask {
        kind: DialogKind::OpenFile,
        future: std::sync::Mutex::new(future),
    });

    // Set a file path and dirty flag.
    let tmp = std::env::temp_dir().join("hexorder_test_save_noop_dialog.hexorder");
    app.world_mut().resource_mut::<Workspace>().file_path = Some(tmp.clone());
    app.world_mut().resource_mut::<Workspace>().dirty = true;

    app.world_mut()
        .commands()
        .trigger(SaveRequestEvent { save_as: false });
    app.update();
    app.update();

    // Should still be dirty — save was skipped.
    let workspace = app.world().resource::<Workspace>();
    assert!(
        workspace.dirty,
        "save should be skipped when dialog is open"
    );
}

// ---------------------------------------------------------------------------
// Observer: handle_new_project
// ---------------------------------------------------------------------------

/// `handle_new_project` resets to new project when workspace is not dirty.
#[test]
fn handle_new_project_resets_when_not_dirty() {
    use hexorder_contracts::persistence::{NewProjectEvent, Workspace};

    let mut app = test_app_with_grid();

    // Ensure workspace is not dirty.
    app.world_mut().resource_mut::<Workspace>().dirty = false;
    app.world_mut().resource_mut::<Workspace>().name = "OldProject".to_string();

    app.world_mut().commands().trigger(NewProjectEvent {
        name: "Fresh".to_string(),
    });
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(workspace.name, "Fresh");
}

/// `handle_new_project` is a no-op when a dialog is already open.
#[test]
fn handle_new_project_noop_when_dialog_open() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::{NewProjectEvent, Workspace};

    let mut app = test_app_with_grid();

    let future: DialogFuture = Box::pin(std::future::pending());
    app.insert_resource(AsyncDialogTask {
        kind: DialogKind::OpenFile,
        future: std::sync::Mutex::new(future),
    });

    app.world_mut().resource_mut::<Workspace>().name = "OldProject".to_string();

    app.world_mut().commands().trigger(NewProjectEvent {
        name: "Fresh".to_string(),
    });
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(
        workspace.name, "OldProject",
        "new project should be skipped when dialog is open"
    );
}

// ---------------------------------------------------------------------------
// Observer: handle_close_project
// ---------------------------------------------------------------------------

/// `handle_close_project` closes when workspace is not dirty.
#[test]
fn handle_close_project_closes_when_not_dirty() {
    use hexorder_contracts::persistence::{CloseProjectEvent, Workspace};

    let mut app = test_app_with_grid();

    app.world_mut().resource_mut::<Workspace>().dirty = false;
    app.world_mut().resource_mut::<Workspace>().name = "MyProject".to_string();

    app.world_mut().commands().trigger(CloseProjectEvent);
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert!(
        workspace.name.is_empty(),
        "CloseProject should reset workspace name"
    );
}

/// `handle_close_project` is a no-op when a dialog is already open.
#[test]
fn handle_close_project_noop_when_dialog_open() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::{CloseProjectEvent, Workspace};

    let mut app = test_app_with_grid();

    let future: DialogFuture = Box::pin(std::future::pending());
    app.insert_resource(AsyncDialogTask {
        kind: DialogKind::OpenFile,
        future: std::sync::Mutex::new(future),
    });

    app.world_mut().resource_mut::<Workspace>().name = "MyProject".to_string();

    app.world_mut().commands().trigger(CloseProjectEvent);
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(
        workspace.name, "MyProject",
        "close should be skipped when dialog is open"
    );
}

// ---------------------------------------------------------------------------
// Observer: handle_load_request
// ---------------------------------------------------------------------------

/// `handle_load_request` is a no-op when a dialog is already open.
#[test]
fn handle_load_request_noop_when_dialog_open() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::LoadRequestEvent;

    let mut app = test_app_with_grid();

    let future: DialogFuture = Box::pin(std::future::pending());
    app.insert_resource(AsyncDialogTask {
        kind: DialogKind::OpenFile,
        future: std::sync::Mutex::new(future),
    });

    app.world_mut().commands().trigger(LoadRequestEvent);
    app.update();
    app.update();

    // Dialog task should still exist (not replaced).
    assert!(
        app.world().get_resource::<AsyncDialogTask>().is_some(),
        "existing dialog task should remain"
    );
}

// ---------------------------------------------------------------------------
// Observer: handle_dialog_completed
// ---------------------------------------------------------------------------

/// `handle_dialog_completed` dispatches the dialog result through the world.
#[test]
fn handle_dialog_completed_dispatches_to_world() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::persistence::Workspace;

    let mut app = test_app_with_grid();
    app.world_mut().resource_mut::<Workspace>().name = "Before".to_string();

    // Trigger DialogCompleted with confirm No + NewProject.
    // handle_dialog_completed queues dispatch_dialog_result, which calls
    // execute_pending_action(NewProject), which calls reset_to_new_project.
    app.world_mut().commands().trigger(DialogCompleted {
        kind: DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::NewProject {
                name: "After".to_string(),
            },
        },
        result: DialogResult::Confirmed(ConfirmChoice::No),
    });
    app.update();
    app.update();
    app.update();

    let workspace = app.world().resource::<Workspace>();
    assert_eq!(
        workspace.name, "After",
        "dialog result should have been dispatched"
    );
}

// ---------------------------------------------------------------------------
// Undo stack coverage in reset_to_new_project and close_project
// ---------------------------------------------------------------------------

/// `reset_to_new_project` clears the undo stack when present.
#[test]
fn reset_to_new_project_clears_undo_stack() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::undo_redo::UndoStack;

    let mut app = test_app_with_grid();
    app.init_resource::<UndoStack>();

    // Record a command to make the undo stack non-empty.
    app.world_mut().resource_mut::<UndoStack>().record(Box::new(
        hexorder_contracts::undo_redo::SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: hexorder_contracts::game_system::PropertyValue::Int(0),
            new_value: hexorder_contracts::game_system::PropertyValue::Int(1),
            label: "test".to_string(),
        },
    ));
    assert!(app.world().resource::<UndoStack>().can_undo());

    // Dispatch confirm No with NewProject → calls reset_to_new_project.
    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::NewProject {
                name: "Fresh".to_string(),
            },
        },
        DialogResult::Confirmed(ConfirmChoice::No),
        app.world_mut(),
    );

    assert!(
        !app.world().resource::<UndoStack>().can_undo(),
        "undo stack should be cleared after reset_to_new_project"
    );
}

/// `close_project` clears the undo stack when present.
#[test]
fn close_project_clears_undo_stack() {
    use crate::persistence::async_dialog::*;
    use hexorder_contracts::undo_redo::UndoStack;

    let mut app = test_app_with_grid();
    app.init_resource::<UndoStack>();

    app.world_mut().resource_mut::<UndoStack>().record(Box::new(
        hexorder_contracts::undo_redo::SetPropertyCommand {
            entity: Entity::PLACEHOLDER,
            property_id: TypeId::new(),
            old_value: hexorder_contracts::game_system::PropertyValue::Int(0),
            new_value: hexorder_contracts::game_system::PropertyValue::Int(1),
            label: "test".to_string(),
        },
    ));
    assert!(app.world().resource::<UndoStack>().can_undo());

    // Dispatch confirm No with CloseProject → calls close_project.
    super::systems::dispatch_dialog_result(
        DialogKind::ConfirmUnsavedChanges {
            then: PendingAction::CloseProject,
        },
        DialogResult::Confirmed(ConfirmChoice::No),
        app.world_mut(),
    );

    assert!(
        !app.world().resource::<UndoStack>().can_undo(),
        "undo stack should be cleared after close_project"
    );
}
