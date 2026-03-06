use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

mod macros;

mod cell;
mod editor_ui;
mod game_system;
mod hex_grid;
mod ontology;
mod persistence;
mod rules_engine;
mod shortcuts;
mod unit;

use hexorder_contracts::persistence::AppScreen;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hexorder".to_string(),
                window_theme: Some(bevy::window::WindowTheme::Dark),
                // Start hidden to prevent OS-default white flash before
                // the GPU renders its first frame with our dark ClearColor.
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppScreen>()
        .add_plugins(shortcuts::ShortcutsPlugin)
        .add_plugins(hex_grid::HexGridPlugin)
        .add_plugins(hexorder_camera::CameraPlugin)
        .add_plugins(game_system::GameSystemPlugin)
        .add_plugins(ontology::OntologyPlugin)
        .add_plugins(cell::CellPlugin)
        .add_plugins(unit::UnitPlugin)
        .add_plugins(rules_engine::RulesEnginePlugin)
        .add_plugins(hexorder_simulation::SimulationPlugin)
        .add_plugins(hexorder_scripting::ScriptingPlugin)
        .add_plugins(persistence::PersistencePlugin)
        .add_plugins(hexorder_undo_redo::UndoRedoPlugin)
        .add_plugins(hexorder_map_gen::MapGenPlugin)
        .add_plugins(hexorder_mechanic_ref::MechanicReferencePlugin)
        .add_plugins(hexorder_export::ExportPlugin)
        .add_plugins(hexorder_settings::SettingsPlugin)
        .add_plugins(editor_ui::EditorUiPlugin)
        .add_systems(Update, reveal_window)
        .run();
}

/// Reveal the hidden window after 3 frames, once the GPU has rendered
/// dark content. Runs once via `Local<bool>` guard.
fn reveal_window(
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut frames: Local<u32>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    *frames += 1;
    if *frames >= 3 {
        if let Ok(mut window) = windows.single_mut() {
            window.visible = true;
        }
        *done = true;
    }
}

#[cfg(test)]
mod reveal_window_tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;

    #[test]
    fn no_primary_window_does_not_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        for _ in 0..5 {
            app.update();
        }

        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn reveals_window_after_three_frames() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        app.world_mut().spawn((
            Window {
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));

        app.update();
        app.update();
        {
            let mut query = app
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>();
            let window = query
                .single(app.world())
                .expect("PrimaryWindow should exist");
            assert!(
                !window.visible,
                "Window should still be hidden after 2 frames"
            );
        }

        app.update();
        {
            let mut query = app
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>();
            let window = query
                .single(app.world())
                .expect("PrimaryWindow should exist");
            assert!(window.visible, "Window should be visible after 3 frames");
        }
    }

    #[test]
    fn reveal_window_is_idempotent_after_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, super::reveal_window);

        app.world_mut().spawn((
            Window {
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));

        for _ in 0..4 {
            app.update();
        }

        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        let window = query.single(app.world()).expect("window should exist");
        assert!(
            window.visible,
            "Window should still be visible after extra frame"
        );
    }
}

/// Cross-plugin integration tests.
///
/// These tests assemble real plugins in a headless Bevy app and verify
/// they cooperate correctly. Plugins requiring rendering are excluded.
#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;

    use hexorder_contracts::editor_ui::EditorTool;
    use hexorder_contracts::game_system::{
        ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityTypeRegistry, GameSystem,
        SelectedUnit, UnitInstance,
    };
    use hexorder_contracts::hex_grid::{
        HexGridConfig, HexPosition, HexSelectedEvent, HexTile, TileBaseMaterial,
    };

    fn headless_app() -> App {
        use hexorder_contracts::persistence::AppScreen;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.insert_state(AppScreen::Editor);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(EditorTool::default());
        app.insert_resource(HexGridConfig {
            layout: hexx::HexLayout {
                orientation: hexx::HexOrientation::Pointy,
                scale: bevy::math::Vec2::splat(1.0),
                origin: bevy::math::Vec2::ZERO,
            },
            map_radius: 5,
        });
        app.add_plugins(crate::game_system::GameSystemPlugin);
        app.init_resource::<hexorder_contracts::undo_redo::UndoStack>();
        app.add_plugins(crate::cell::CellPlugin);
        app.add_plugins(crate::unit::UnitPlugin);
        app
    }

    fn spawn_test_tile(app: &mut App, q: i32, r: i32) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Cuboid::new(1.0, 0.1, 1.0)));

        app.world_mut()
            .spawn((
                HexTile,
                HexPosition::new(q, r),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
            ))
            .id()
    }

    #[test]
    fn game_system_resources_available_immediately() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::game_system::GameSystemPlugin);

        assert!(
            app.world().get_resource::<GameSystem>().is_some(),
            "GameSystem should exist before first update"
        );
        assert!(
            app.world().get_resource::<EntityTypeRegistry>().is_some(),
            "EntityTypeRegistry should exist before first update"
        );
        assert!(
            app.world().get_resource::<ActiveBoardType>().is_some(),
            "ActiveBoardType should exist before first update"
        );
    }

    #[test]
    fn game_system_and_cell_startup_succeeds() {
        let mut app = headless_app();
        app.update();
        app.update();
    }

    #[test]
    fn cell_assigns_default_data_to_new_tiles() {
        let mut app = headless_app();
        app.update();

        spawn_test_tile(&mut app, 0, 0);
        spawn_test_tile(&mut app, 1, 0);
        spawn_test_tile(&mut app, 0, 1);

        app.update();

        let registry = app.world().resource::<EntityTypeRegistry>();
        let first_id = registry
            .first_by_role(EntityRole::BoardPosition)
            .expect("registry should have BoardPosition types")
            .id;

        let mut query = app
            .world_mut()
            .query_filtered::<&EntityData, With<HexTile>>();
        let entity_data: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(
            entity_data.len(),
            3,
            "All tiles should have EntityData after update"
        );
        for ed in &entity_data {
            assert_eq!(
                ed.entity_type_id, first_id,
                "Default entity type should be the first BoardPosition in registry"
            );
        }
    }

    #[test]
    fn game_system_unit_resources_available_immediately() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::game_system::GameSystemPlugin);

        assert!(
            app.world().get_resource::<EntityTypeRegistry>().is_some(),
            "EntityTypeRegistry should exist before first update"
        );
        assert!(
            app.world().get_resource::<ActiveTokenType>().is_some(),
            "ActiveTokenType should exist before first update"
        );
        assert!(
            app.world().get_resource::<SelectedUnit>().is_some(),
            "SelectedUnit should exist before first update"
        );
    }

    #[test]
    fn game_system_and_unit_startup_succeeds() {
        let mut app = headless_app();
        app.update();
        app.update();

        let registry = app.world().resource::<EntityTypeRegistry>();
        assert!(
            !registry.types_by_role(EntityRole::Token).is_empty(),
            "Token entity types should be registered"
        );
    }

    #[test]
    fn unit_placement_creates_entity_on_grid() {
        let mut app = headless_app();
        app.update();

        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Place;

        let active_id = app
            .world()
            .resource::<ActiveTokenType>()
            .entity_type_id
            .expect("ActiveTokenType should have a type selected");

        app.world_mut().trigger(HexSelectedEvent {
            position: HexPosition::new(0, 0),
        });

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<(&EntityData, &HexPosition), With<UnitInstance>>();
        let units: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(units.len(), 1, "Exactly one unit should be placed");
        assert_eq!(units[0].0.entity_type_id, active_id);
        assert_eq!(*units[0].1, HexPosition::new(0, 0));
    }

    #[test]
    fn unit_movement_updates_position() {
        let mut app = headless_app();
        app.update();

        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Place;
        app.world_mut().trigger(HexSelectedEvent {
            position: HexPosition::new(0, 0),
        });
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<UnitInstance>>();
        let unit_entity = query.iter(app.world()).next().expect("Unit should exist");

        *app.world_mut().resource_mut::<EditorTool>() = EditorTool::Select;
        app.world_mut().resource_mut::<SelectedUnit>().entity = Some(unit_entity);

        app.world_mut().trigger(HexSelectedEvent {
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
            "Unit should have moved to (1, 0)"
        );
    }

    #[test]
    fn cell_visual_sync_after_data_assignment() {
        let mut app = headless_app();
        app.update();

        let original_material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(0.99, 0.99, 0.99),
                ..default()
            });
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Mesh::from(Cuboid::new(1.0, 0.1, 1.0)));

        let tile = app
            .world_mut()
            .spawn((
                HexTile,
                HexPosition::new(0, 0),
                Mesh3d(mesh),
                MeshMaterial3d(original_material.clone()),
                TileBaseMaterial(original_material.clone()),
                Transform::default(),
            ))
            .id();

        app.update();

        let tile_material = app
            .world()
            .entity(tile)
            .get::<MeshMaterial3d<StandardMaterial>>()
            .expect("Tile should have material");

        assert_ne!(
            tile_material.0, original_material,
            "Tile material should have been updated by cell visual sync"
        );
    }
}
