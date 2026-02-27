//! Unit tests for the `editor_ui` plugin.

use bevy::prelude::*;

use hexorder_contracts::editor_ui::{EditorTool, Selection, ToastEvent, ToastKind};
use hexorder_contracts::game_system::SelectedUnit;
use hexorder_contracts::hex_grid::HexTile;
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

use super::components::{EditorState, GridOverlayVisible, OntologyTab, ToastState};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Creates an `App` with the minimum resources needed by the `editor_ui`
/// observers (`handle_editor_ui_command` and `handle_toast_event`).
fn observer_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.insert_resource(EditorTool::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(EditorState::default());
    app.init_resource::<Selection>();
    app.init_resource::<GridOverlayVisible>();
    app.init_resource::<ToastState>();
    app.init_resource::<super::components::DockLayoutState>();
    app.add_observer(super::handle_editor_ui_command);
    app.add_observer(super::handle_toast_event);
    app.update();
    app
}

// ---------------------------------------------------------------------------
// Existing tests
// ---------------------------------------------------------------------------

#[test]
fn editor_tool_defaults_to_select() {
    let tool = EditorTool::default();
    assert_eq!(tool, EditorTool::Select);
}

#[test]
fn editor_tool_variants_are_distinct() {
    assert_ne!(EditorTool::Select, EditorTool::Paint);
    assert_ne!(EditorTool::Select, EditorTool::Place);
    assert_ne!(EditorTool::Paint, EditorTool::Place);
}

#[test]
fn editor_tool_resource_inserts_correctly() {
    let mut app = App::new();
    app.insert_resource(EditorTool::default());
    app.update();

    let tool = app.world().resource::<EditorTool>();
    assert_eq!(*tool, EditorTool::Select);
}

#[test]
fn editor_state_defaults() {
    let state = EditorState::default();
    assert!(state.new_type_name.is_empty());
    assert_eq!(state.new_type_color, [0.5, 0.5, 0.5]);
    assert_eq!(state.new_type_role_index, 0);
    assert!(state.new_prop_name.is_empty());
    assert_eq!(state.new_prop_type_index, 0);
    assert!(state.new_enum_options.is_empty());
    assert!(state.new_enum_name.is_empty());
    assert!(state.new_enum_option_text.is_empty());
    assert!(state.new_struct_name.is_empty());
    assert!(state.new_struct_field_name.is_empty());
    assert_eq!(state.new_struct_field_type_index, 0);
    assert_eq!(state.active_tab, OntologyTab::Types);
    assert!(state.new_concept_name.is_empty());
    assert!(state.new_relation_name.is_empty());
    assert!(state.new_constraint_name.is_empty());
    assert!(state.editing_concept_id.is_none());
    assert!(state.binding_entity_type_id.is_none());
}

#[test]
fn editor_state_resource_inserts_correctly() {
    let mut app = App::new();
    app.insert_resource(EditorState::default());
    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(state.new_type_name.is_empty());
    assert_eq!(state.new_prop_type_index, 0);
}

#[test]
fn ontology_tab_default_is_types() {
    assert_eq!(OntologyTab::default(), OntologyTab::Types);
}

#[test]
fn ontology_tab_variants_are_distinct() {
    assert_ne!(OntologyTab::Types, OntologyTab::Enums);
    assert_ne!(OntologyTab::Types, OntologyTab::Structs);
    assert_ne!(OntologyTab::Types, OntologyTab::Concepts);
    assert_ne!(OntologyTab::Types, OntologyTab::Relations);
    assert_ne!(OntologyTab::Types, OntologyTab::Constraints);
    assert_ne!(OntologyTab::Types, OntologyTab::Validation);
    assert_ne!(OntologyTab::Enums, OntologyTab::Structs);
    assert_ne!(OntologyTab::Concepts, OntologyTab::Relations);
}

// ---------------------------------------------------------------------------
// Scope 1: Toast notification system
// ---------------------------------------------------------------------------

#[test]
fn toast_state_defaults_to_none() {
    let state = ToastState::default();
    assert!(state.active.is_none());
}

#[test]
fn toast_kind_variants_are_distinct() {
    assert_ne!(ToastKind::Success, ToastKind::Error);
    assert_ne!(ToastKind::Success, ToastKind::Info);
    assert_ne!(ToastKind::Error, ToastKind::Info);
}

#[test]
fn toast_event_observer_populates_toast_state() {
    let mut app = observer_app();

    app.world_mut().trigger(ToastEvent {
        message: "Project saved".to_string(),
        kind: ToastKind::Success,
    });
    app.update();

    let state = app.world().resource::<ToastState>();
    let toast = state.active.as_ref().expect("toast should be active");
    assert_eq!(toast.message, "Project saved");
    assert_eq!(toast.kind, ToastKind::Success);
    assert!(toast.remaining > 0.0);
}

#[test]
fn toast_event_replaces_previous_toast() {
    let mut app = observer_app();

    app.world_mut().trigger(ToastEvent {
        message: "First".to_string(),
        kind: ToastKind::Info,
    });
    app.update();

    app.world_mut().trigger(ToastEvent {
        message: "Second".to_string(),
        kind: ToastKind::Error,
    });
    app.update();

    let state = app.world().resource::<ToastState>();
    let toast = state.active.as_ref().expect("toast should be active");
    assert_eq!(toast.message, "Second");
    assert_eq!(toast.kind, ToastKind::Error);
}

// ---------------------------------------------------------------------------
// Scope 2: User-configurable font size
// ---------------------------------------------------------------------------

#[test]
fn editor_state_font_size_defaults_to_15() {
    let state = EditorState::default();
    assert!((state.font_size_base - 15.0).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// Scope 3: Multi-selection system
// ---------------------------------------------------------------------------

#[test]
fn selection_defaults_to_empty() {
    let sel = Selection::default();
    assert!(sel.entities.is_empty());
}

#[test]
fn select_all_command_selects_all_hex_tiles() {
    let mut app = observer_app();

    // Spawn 3 HexTile entities.
    let e1 = app.world_mut().spawn(HexTile).id();
    let e2 = app.world_mut().spawn(HexTile).id();
    let e3 = app.world_mut().spawn(HexTile).id();
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.select_all"),
    });
    app.update();

    let sel = app.world().resource::<Selection>();
    assert_eq!(sel.entities.len(), 3);
    assert!(sel.entities.contains(&e1));
    assert!(sel.entities.contains(&e2));
    assert!(sel.entities.contains(&e3));
}

#[test]
fn delete_command_clears_multi_selection() {
    let mut app = observer_app();

    let e1 = app.world_mut().spawn(HexTile).id();
    let e2 = app.world_mut().spawn(HexTile).id();
    app.update();

    // Pre-populate the selection.
    app.world_mut()
        .resource_mut::<Selection>()
        .entities
        .insert(e1);
    app.world_mut()
        .resource_mut::<Selection>()
        .entities
        .insert(e2);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    let sel = app.world().resource::<Selection>();
    assert!(sel.entities.is_empty());
}

#[test]
fn delete_command_falls_back_to_selected_unit() {
    let mut app = observer_app();

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    let selected = app.world().resource::<SelectedUnit>();
    assert!(selected.entity.is_none());
}

// ---------------------------------------------------------------------------
// Scope: Unit deletion undo (#127)
// ---------------------------------------------------------------------------

/// Helper: create an observer app that also has `UndoStack` + undo/redo pipeline.
fn observer_app_with_undo() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.insert_resource(EditorTool::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(EditorState::default());
    app.init_resource::<Selection>();
    app.init_resource::<GridOverlayVisible>();
    app.init_resource::<ToastState>();
    app.init_resource::<super::components::DockLayoutState>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<hexorder_contracts::undo_redo::UndoStack>();
    app.init_resource::<hexorder_contracts::shortcuts::ShortcutRegistry>();
    app.add_plugins(crate::undo_redo::UndoRedoPlugin);
    app.add_observer(super::handle_editor_ui_command);
    app.add_observer(super::handle_toast_event);
    app.update();
    app
}

/// Helper: spawn a unit entity with all components needed for undo/redo.
fn spawn_test_unit(
    app: &mut App,
    q: i32,
    r: i32,
) -> (Entity, hexorder_contracts::game_system::TypeId) {
    let type_id = hexorder_contracts::game_system::TypeId::new();
    let entity = app
        .world_mut()
        .spawn((
            hexorder_contracts::game_system::UnitInstance,
            hexorder_contracts::hex_grid::HexPosition::new(q, r),
            hexorder_contracts::game_system::EntityData {
                entity_type_id: type_id,
                properties: std::collections::HashMap::new(),
            },
            Mesh3d(Handle::default()),
            MeshMaterial3d::<StandardMaterial>(Handle::default()),
            Transform::from_xyz(1.0, 0.5, 2.0),
        ))
        .id();
    (entity, type_id)
}

/// Deleting a selected unit records a `DeleteUnitCommand` on the undo stack.
#[test]
fn delete_unit_records_undo_command() {
    let mut app = observer_app_with_undo();

    let (entity, _type_id) = spawn_test_unit(&mut app, 0, 0);
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    let stack = app
        .world()
        .resource::<hexorder_contracts::undo_redo::UndoStack>();
    assert!(
        stack.can_undo(),
        "Undo stack should have a command after deleting"
    );
    assert!(
        stack
            .undo_description()
            .expect("should have description")
            .contains("Delete"),
        "Undo description should mention Delete"
    );
}

/// Undoing a unit deletion restores the unit with its original data.
#[test]
fn delete_then_undo_restores_unit() {
    let mut app = observer_app_with_undo();

    let (entity, type_id) = spawn_test_unit(&mut app, 1, -1);
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    // Confirm unit is gone.
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<hexorder_contracts::game_system::UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "Unit should be gone after delete"
    );

    // Undo the deletion.
    app.world_mut()
        .resource_mut::<hexorder_contracts::undo_redo::UndoStack>()
        .request_undo();
    app.update();

    // Unit should be restored with original data.
    let mut query = app.world_mut().query_filtered::<(
        &hexorder_contracts::hex_grid::HexPosition,
        &hexorder_contracts::game_system::EntityData,
    ), With<hexorder_contracts::game_system::UnitInstance>>();
    let units: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(units.len(), 1, "Unit should be restored after undo");
    assert_eq!(
        *units[0].0,
        hexorder_contracts::hex_grid::HexPosition::new(1, -1)
    );
    assert_eq!(units[0].1.entity_type_id, type_id);
}

/// Redo after undoing a deletion removes the unit again.
#[test]
fn delete_undo_redo_removes_again() {
    let mut app = observer_app_with_undo();

    let (entity, _type_id) = spawn_test_unit(&mut app, 3, 0);
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    // Undo (restore).
    app.world_mut()
        .resource_mut::<hexorder_contracts::undo_redo::UndoStack>()
        .request_undo();
    app.update();

    // Redo (delete again).
    app.world_mut()
        .resource_mut::<hexorder_contracts::undo_redo::UndoStack>()
        .request_redo();
    app.update();

    // Unit should be gone again.
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<hexorder_contracts::game_system::UnitInstance>>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "Unit should be deleted again after redo"
    );
}

// ---------------------------------------------------------------------------
// Scope 4: Grid overlay toggle
// ---------------------------------------------------------------------------

#[test]
fn grid_overlay_defaults_to_hidden() {
    let overlay = GridOverlayVisible::default();
    assert!(!overlay.0);
}

#[test]
fn toggle_grid_overlay_command_flips_visibility() {
    let mut app = observer_app();

    assert!(!app.world().resource::<GridOverlayVisible>().0);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_grid_overlay"),
    });
    app.update();

    assert!(app.world().resource::<GridOverlayVisible>().0);

    // Toggle again — should hide.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_grid_overlay"),
    });
    app.update();

    assert!(!app.world().resource::<GridOverlayVisible>().0);
}

// ---------------------------------------------------------------------------
// Scope 5: About panel
// ---------------------------------------------------------------------------

#[test]
fn editor_state_about_panel_defaults_hidden() {
    let state = EditorState::default();
    assert!(!state.about_panel_visible);
}

#[test]
fn about_command_toggles_about_panel() {
    let mut app = observer_app();

    assert!(!app.world().resource::<EditorState>().about_panel_visible);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("help.about"),
    });
    app.update();

    assert!(app.world().resource::<EditorState>().about_panel_visible);

    // Toggle again — should hide.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("help.about"),
    });
    app.update();

    assert!(!app.world().resource::<EditorState>().about_panel_visible);
}

// ---------------------------------------------------------------------------
// Observer: view toggle commands
// ---------------------------------------------------------------------------

#[test]
fn toggle_inspector_command_flips_visibility() {
    let mut app = observer_app();

    assert!(app.world().resource::<EditorState>().inspector_visible);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_inspector"),
    });
    app.update();

    assert!(!app.world().resource::<EditorState>().inspector_visible);
}

#[test]
fn toggle_toolbar_command_flips_visibility() {
    let mut app = observer_app();

    assert!(app.world().resource::<EditorState>().toolbar_visible);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_toolbar"),
    });
    app.update();

    assert!(!app.world().resource::<EditorState>().toolbar_visible);
}

// ---------------------------------------------------------------------------
// Scope 1 (0.11.0): Dock layout — egui_dock evaluation
// ---------------------------------------------------------------------------

use super::components::{DockLayoutState, DockTab};

#[test]
fn dock_tab_variants_are_distinct() {
    assert_ne!(DockTab::Viewport, DockTab::Palette);
    assert_ne!(DockTab::Viewport, DockTab::Design);
    assert_ne!(DockTab::Viewport, DockTab::Rules);
    assert_ne!(DockTab::Viewport, DockTab::Inspector);
    assert_ne!(DockTab::Viewport, DockTab::Settings);
    assert_ne!(DockTab::Viewport, DockTab::Selection);
    assert_ne!(DockTab::Viewport, DockTab::Validation);
    assert_ne!(DockTab::Viewport, DockTab::MapGenerator);
    assert_ne!(DockTab::Palette, DockTab::Design);
    assert_ne!(DockTab::Design, DockTab::Rules);
    assert_ne!(DockTab::Inspector, DockTab::Settings);
    assert_ne!(DockTab::Settings, DockTab::Selection);
    assert_ne!(DockTab::MapGenerator, DockTab::MechanicReference);
}

#[test]
fn dock_layout_creates_default_layout() {
    let state = super::components::create_default_dock_layout();
    // Collect all tabs across the dock state's main surface.
    let mut tabs = Vec::new();
    for node in state.main_surface().iter() {
        if let Some(node_tabs) = node.tabs() {
            for tab in node_tabs {
                tabs.push(*tab);
            }
        }
    }
    assert_eq!(tabs.len(), 10);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
    assert!(tabs.contains(&DockTab::MapGenerator));
    assert!(tabs.contains(&DockTab::Shortcuts));
}

#[test]
fn viewport_tab_is_not_closeable() {
    assert!(!DockTab::Viewport.is_closeable());
    assert!(DockTab::Palette.is_closeable());
    assert!(DockTab::Design.is_closeable());
    assert!(DockTab::Rules.is_closeable());
    assert!(DockTab::Inspector.is_closeable());
    assert!(DockTab::Settings.is_closeable());
    assert!(DockTab::Selection.is_closeable());
    assert!(DockTab::Validation.is_closeable());
    assert!(DockTab::MapGenerator.is_closeable());
}

#[test]
fn dock_layout_state_resource_inserts_correctly() {
    let mut app = App::new();
    app.init_resource::<DockLayoutState>();
    app.update();

    let state = app.world().resource::<DockLayoutState>();
    // Verify the default layout created 10 tabs (9 content + Shortcuts).
    let mut count = 0;
    for node in state.dock_state.main_surface().iter() {
        if let Some(tabs) = node.tabs() {
            count += tabs.len();
        }
    }
    assert_eq!(count, 10);
}

// ---------------------------------------------------------------------------
// Scope 5 (0.11.0): Workspace presets
// ---------------------------------------------------------------------------

use super::components::WorkspacePreset;

#[test]
fn workspace_preset_defaults_to_map_editing() {
    assert_eq!(WorkspacePreset::default(), WorkspacePreset::MapEditing);
}

#[test]
fn workspace_preset_variants_are_distinct() {
    assert_ne!(WorkspacePreset::MapEditing, WorkspacePreset::UnitDesign);
    assert_ne!(WorkspacePreset::MapEditing, WorkspacePreset::RuleAuthoring);
    assert_ne!(WorkspacePreset::MapEditing, WorkspacePreset::Playtesting);
    assert_ne!(WorkspacePreset::UnitDesign, WorkspacePreset::RuleAuthoring);
    assert_ne!(WorkspacePreset::UnitDesign, WorkspacePreset::Playtesting);
    assert_ne!(WorkspacePreset::RuleAuthoring, WorkspacePreset::Playtesting);
}

#[test]
fn dock_layout_state_defaults_to_map_editing_preset() {
    let state = DockLayoutState::default();
    assert_eq!(state.active_preset, WorkspacePreset::MapEditing);
}

/// Helper: collect all tabs from a dock state.
fn collect_tabs(state: &egui_dock::DockState<DockTab>) -> Vec<DockTab> {
    let mut tabs = Vec::new();
    for node in state.main_surface().iter() {
        if let Some(node_tabs) = node.tabs() {
            for tab in node_tabs {
                tabs.push(*tab);
            }
        }
    }
    tabs
}

#[test]
fn map_editing_layout_contains_all_tabs() {
    let state = super::components::create_default_dock_layout();
    let tabs = collect_tabs(&state);
    assert_eq!(tabs.len(), 10);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
    assert!(tabs.contains(&DockTab::MapGenerator));
    assert!(tabs.contains(&DockTab::Shortcuts));
}

#[test]
fn unit_design_layout_contains_viewport_and_design_tabs() {
    let state = super::components::create_unit_design_layout();
    let tabs = collect_tabs(&state);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Palette));
}

#[test]
fn rule_authoring_layout_has_design_rules_and_validation() {
    let state = super::components::create_rule_authoring_layout();
    let tabs = collect_tabs(&state);
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Validation));
}

#[test]
fn playtesting_layout_has_viewport_and_validation() {
    let state = super::components::create_playtesting_layout();
    let tabs = collect_tabs(&state);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Validation));
    // Minimal layout — only 2 tabs.
    assert_eq!(tabs.len(), 2);
}

#[test]
fn apply_preset_switches_layout_and_tracks_preset() {
    let mut layout = DockLayoutState::default();
    assert_eq!(layout.active_preset, WorkspacePreset::MapEditing);

    layout.apply_preset(WorkspacePreset::Playtesting);
    assert_eq!(layout.active_preset, WorkspacePreset::Playtesting);
    let tabs = collect_tabs(&layout.dock_state);
    assert_eq!(tabs.len(), 2);

    layout.apply_preset(WorkspacePreset::UnitDesign);
    assert_eq!(layout.active_preset, WorkspacePreset::UnitDesign);
    let tabs = collect_tabs(&layout.dock_state);
    assert!(tabs.contains(&DockTab::Design));

    layout.apply_preset(WorkspacePreset::MapEditing);
    assert_eq!(layout.active_preset, WorkspacePreset::MapEditing);
    let tabs = collect_tabs(&layout.dock_state);
    assert_eq!(tabs.len(), 10);
}

#[test]
fn workspace_command_switches_preset() {
    let mut app = observer_app();

    // Default is MapEditing.
    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::MapEditing,
    );

    // Switch to Playtesting via command.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("workspace.playtesting"),
    });
    app.update();

    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::Playtesting,
    );

    // Switch to Rule Authoring.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("workspace.rule_authoring"),
    });
    app.update();

    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::RuleAuthoring,
    );
}

#[test]
fn workspace_preset_display_names() {
    assert_eq!(WorkspacePreset::MapEditing.to_string(), "Map Editing");
    assert_eq!(WorkspacePreset::UnitDesign.to_string(), "Unit Design");
    assert_eq!(WorkspacePreset::RuleAuthoring.to_string(), "Rule Authoring");
    assert_eq!(WorkspacePreset::Playtesting.to_string(), "Playtesting");
}

// ---------------------------------------------------------------------------
// Scope 6 (0.11.0): Layout persistence
// ---------------------------------------------------------------------------

#[test]
fn workspace_preset_id_round_trip() {
    for preset in [
        WorkspacePreset::MapEditing,
        WorkspacePreset::UnitDesign,
        WorkspacePreset::RuleAuthoring,
        WorkspacePreset::Playtesting,
    ] {
        let id = preset.as_id();
        assert_eq!(WorkspacePreset::from_id(id), preset);
    }
}

#[test]
fn workspace_preset_from_id_unknown_defaults_to_map_editing() {
    assert_eq!(
        WorkspacePreset::from_id("unknown"),
        WorkspacePreset::MapEditing,
    );
    assert_eq!(WorkspacePreset::from_id(""), WorkspacePreset::MapEditing,);
}

#[test]
fn workspace_preset_as_id_values_are_stable() {
    assert_eq!(WorkspacePreset::MapEditing.as_id(), "map_editing");
    assert_eq!(WorkspacePreset::UnitDesign.as_id(), "unit_design");
    assert_eq!(WorkspacePreset::RuleAuthoring.as_id(), "rule_authoring");
    assert_eq!(WorkspacePreset::Playtesting.as_id(), "playtesting");
}

#[test]
fn workspace_default_has_empty_preset() {
    let ws = hexorder_contracts::persistence::Workspace::default();
    assert!(ws.workspace_preset.is_empty());
}

#[test]
fn dock_layout_file_ron_round_trip() {
    use super::components::DockLayoutFile;

    let layout = DockLayoutState::default();
    let file = DockLayoutFile {
        preset: layout.active_preset,
        dock_state: layout.dock_state.clone(),
    };

    let config = ron::ser::PrettyConfig::default();
    let ron_str = ron::ser::to_string_pretty(&file, config).expect("serialize");
    let loaded: DockLayoutFile = ron::from_str(&ron_str).expect("deserialize");

    assert_eq!(loaded.preset, WorkspacePreset::MapEditing);
    let original_tabs = collect_tabs(&layout.dock_state);
    let loaded_tabs = collect_tabs(&loaded.dock_state);
    assert_eq!(original_tabs.len(), loaded_tabs.len());
    for tab in &original_tabs {
        assert!(loaded_tabs.contains(tab), "missing tab: {tab}");
    }
}

#[test]
fn dock_layout_file_preserves_preset_variants() {
    use super::components::DockLayoutFile;

    for preset in [
        WorkspacePreset::MapEditing,
        WorkspacePreset::UnitDesign,
        WorkspacePreset::RuleAuthoring,
        WorkspacePreset::Playtesting,
    ] {
        let mut layout = DockLayoutState::default();
        layout.apply_preset(preset);
        let file = DockLayoutFile {
            preset: layout.active_preset,
            dock_state: layout.dock_state.clone(),
        };

        let ron_str = ron::to_string(&file).expect("serialize");
        let loaded: DockLayoutFile = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(loaded.preset, preset);
    }
}

// ---------------------------------------------------------------------------
// Scope 5: Template application (mechanic_reference)
// ---------------------------------------------------------------------------

use hexorder_contracts::game_system::{
    EntityRole, EntityTypeRegistry, EnumDefinition, EnumRegistry,
};
use hexorder_contracts::mechanic_reference::{ScaffoldAction, ScaffoldRecipe};
use hexorder_contracts::mechanics::{
    CombatModifierRegistry, CombatResultsTable, CrtColumnType, ModifierSource, PhaseType,
    TurnStructure,
};

#[test]
fn apply_scaffold_creates_entity_types() {
    let mut registry = EntityTypeRegistry::default();
    let mut enum_registry = EnumRegistry::default();
    let mut turn_structure = TurnStructure::default();
    let mut crt = CombatResultsTable::default();
    let mut modifiers = CombatModifierRegistry::default();

    let recipe = ScaffoldRecipe {
        template_id: "test".to_string(),
        description: "Test recipe".to_string(),
        actions: vec![ScaffoldAction::CreateEntityType {
            name: "TestCell".to_string(),
            role: "Cell".to_string(),
            color: [0.5, 0.5, 0.5],
        }],
    };

    super::systems::apply_scaffold_recipe(
        &recipe,
        &mut registry,
        &mut enum_registry,
        &mut turn_structure,
        &mut crt,
        &mut modifiers,
    );

    assert_eq!(registry.types.len(), 1);
    assert_eq!(registry.types[0].name, "TestCell");
    assert_eq!(registry.types[0].role, EntityRole::BoardPosition);
}

#[test]
fn apply_scaffold_creates_enums_and_links_properties() {
    let mut registry = EntityTypeRegistry::default();
    let mut enum_registry = EnumRegistry::default();
    let mut turn_structure = TurnStructure::default();
    let mut crt = CombatResultsTable::default();
    let mut modifiers = CombatModifierRegistry::default();

    let recipe = ScaffoldRecipe {
        template_id: "test".to_string(),
        description: "Test recipe".to_string(),
        actions: vec![
            ScaffoldAction::CreateEntityType {
                name: "Unit".to_string(),
                role: "Token".to_string(),
                color: [0.3, 0.3, 0.3],
            },
            ScaffoldAction::CreateEnum {
                name: "MoveType".to_string(),
                options: vec!["Foot".to_string(), "Mech".to_string()],
            },
            ScaffoldAction::AddProperty {
                entity_name: "Unit".to_string(),
                prop_name: "movement_type".to_string(),
                prop_type: "Enum(MoveType)".to_string(),
            },
        ],
    };

    super::systems::apply_scaffold_recipe(
        &recipe,
        &mut registry,
        &mut enum_registry,
        &mut turn_structure,
        &mut crt,
        &mut modifiers,
    );

    assert_eq!(registry.types.len(), 1);
    assert_eq!(registry.types[0].role, EntityRole::Token);
    assert_eq!(registry.types[0].properties.len(), 1);
    assert_eq!(registry.types[0].properties[0].name, "movement_type");

    assert_eq!(enum_registry.definitions.len(), 1);
    let enum_def = enum_registry
        .definitions
        .values()
        .next()
        .expect("enum exists");
    assert_eq!(enum_def.name, "MoveType");
    assert_eq!(enum_def.options, vec!["Foot", "Mech"]);
}

#[test]
fn apply_scaffold_adds_crt_structure() {
    let mut registry = EntityTypeRegistry::default();
    let mut enum_registry = EnumRegistry::default();
    let mut turn_structure = TurnStructure::default();
    let mut crt = CombatResultsTable::default();
    let mut modifiers = CombatModifierRegistry::default();

    let recipe = ScaffoldRecipe {
        template_id: "test".to_string(),
        description: "Test recipe".to_string(),
        actions: vec![
            ScaffoldAction::AddCrtColumn {
                label: "1:2".to_string(),
                column_type: "OddsRatio".to_string(),
                threshold: 0.5,
            },
            ScaffoldAction::AddCrtColumn {
                label: "1:1".to_string(),
                column_type: "OddsRatio".to_string(),
                threshold: 1.0,
            },
            ScaffoldAction::AddCrtRow {
                label: "1".to_string(),
                die_min: 1,
                die_max: 1,
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 0,
                label: "NE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 1,
                label: "DR".to_string(),
            },
        ],
    };

    super::systems::apply_scaffold_recipe(
        &recipe,
        &mut registry,
        &mut enum_registry,
        &mut turn_structure,
        &mut crt,
        &mut modifiers,
    );

    assert_eq!(crt.columns.len(), 2);
    assert_eq!(crt.columns[0].column_type, CrtColumnType::OddsRatio);
    assert_eq!(crt.rows.len(), 1);
    assert_eq!(crt.outcomes[0][0].label, "NE");
    assert_eq!(crt.outcomes[0][1].label, "DR");
}

#[test]
fn apply_scaffold_adds_phases_and_modifiers() {
    let mut registry = EntityTypeRegistry::default();
    let mut enum_registry = EnumRegistry::default();
    let mut turn_structure = TurnStructure::default();
    let mut crt = CombatResultsTable::default();
    let mut modifiers = CombatModifierRegistry::default();

    let recipe = ScaffoldRecipe {
        template_id: "test".to_string(),
        description: "Test recipe".to_string(),
        actions: vec![
            ScaffoldAction::AddPhase {
                name: "Movement".to_string(),
                phase_type: "Movement".to_string(),
            },
            ScaffoldAction::AddPhase {
                name: "Combat".to_string(),
                phase_type: "Combat".to_string(),
            },
            ScaffoldAction::AddCombatModifier {
                name: "Forest".to_string(),
                source: "DefenderTerrain".to_string(),
                shift: -1,
                priority: 10,
            },
        ],
    };

    super::systems::apply_scaffold_recipe(
        &recipe,
        &mut registry,
        &mut enum_registry,
        &mut turn_structure,
        &mut crt,
        &mut modifiers,
    );

    assert_eq!(turn_structure.phases.len(), 2);
    assert_eq!(turn_structure.phases[0].phase_type, PhaseType::Movement);
    assert_eq!(turn_structure.phases[1].phase_type, PhaseType::Combat);
    assert_eq!(modifiers.modifiers.len(), 1);
    assert_eq!(
        modifiers.modifiers[0].source,
        ModifierSource::DefenderTerrain
    );
    assert_eq!(modifiers.modifiers[0].column_shift, -1);
}

#[test]
fn parse_scaffold_prop_type_basic_types() {
    use hexorder_contracts::game_system::PropertyType;

    let empty = EnumRegistry::default();
    assert_eq!(
        super::systems::parse_scaffold_prop_type("Bool", &empty),
        PropertyType::Bool
    );
    assert_eq!(
        super::systems::parse_scaffold_prop_type("Int", &empty),
        PropertyType::Int
    );
    assert_eq!(
        super::systems::parse_scaffold_prop_type("Float", &empty),
        PropertyType::Float
    );
    assert_eq!(
        super::systems::parse_scaffold_prop_type("String", &empty),
        PropertyType::String
    );
    assert_eq!(
        super::systems::parse_scaffold_prop_type("Color", &empty),
        PropertyType::Color
    );
}

#[test]
fn parse_scaffold_prop_type_int_range() {
    use hexorder_contracts::game_system::PropertyType;

    let empty = EnumRegistry::default();
    let result = super::systems::parse_scaffold_prop_type("IntRange(0,20)", &empty);
    assert_eq!(result, PropertyType::IntRange { min: 0, max: 20 });
}

#[test]
fn parse_scaffold_prop_type_float_range() {
    use hexorder_contracts::game_system::PropertyType;

    let empty = EnumRegistry::default();
    let result = super::systems::parse_scaffold_prop_type("FloatRange(0.0,1.0)", &empty);
    assert_eq!(result, PropertyType::FloatRange { min: 0.0, max: 1.0 });
}

#[test]
fn parse_scaffold_prop_type_enum_lookup() {
    use hexorder_contracts::game_system::{PropertyType, TypeId};

    let mut enum_registry = EnumRegistry::default();
    let enum_id = TypeId::new();
    enum_registry.insert(EnumDefinition {
        id: enum_id,
        name: "TerrainType".to_string(),
        options: vec!["Clear".to_string(), "Forest".to_string()],
    });

    let result = super::systems::parse_scaffold_prop_type("Enum(TerrainType)", &enum_registry);
    assert_eq!(result, PropertyType::Enum(enum_id));
}

#[test]
fn apply_crt_combat_template_populates_registries() {
    // End-to-end test: applying the actual crt_combat template.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(crate::mechanic_reference::MechanicReferencePlugin);
    app.init_resource::<EntityTypeRegistry>();
    app.init_resource::<EnumRegistry>();
    app.init_resource::<TurnStructure>();
    app.init_resource::<CombatResultsTable>();
    app.init_resource::<CombatModifierRegistry>();
    app.update();

    let catalog = app
        .world()
        .resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    let recipe = catalog
        .get_template("crt_combat")
        .expect("crt_combat template exists");

    let mut registry_clone = app.world_mut().resource_mut::<EntityTypeRegistry>().clone();
    let mut enum_registry = app.world_mut().resource_mut::<EnumRegistry>().clone();
    let mut turn_structure = app.world_mut().resource_mut::<TurnStructure>().clone();
    let mut crt = app.world_mut().resource_mut::<CombatResultsTable>().clone();
    let mut modifiers = app
        .world_mut()
        .resource_mut::<CombatModifierRegistry>()
        .clone();

    super::systems::apply_scaffold_recipe(
        &recipe,
        &mut registry_clone,
        &mut enum_registry,
        &mut turn_structure,
        &mut crt,
        &mut modifiers,
    );

    // CRT combat template should create columns, rows, and outcomes.
    assert!(
        !crt.columns.is_empty(),
        "CRT combat template should add columns"
    );
    assert!(!crt.rows.is_empty(), "CRT combat template should add rows");
    assert!(
        !crt.outcomes.is_empty(),
        "CRT combat template should set outcomes"
    );
    // Should also add combat modifiers.
    assert!(
        !modifiers.modifiers.is_empty(),
        "CRT combat template should add modifiers"
    );
}

// ---------------------------------------------------------------------------
// Coverage: DockTab Display (components.rs)
// ---------------------------------------------------------------------------

#[test]
fn dock_tab_display_all_variants() {
    assert_eq!(DockTab::Viewport.to_string(), "Viewport");
    assert_eq!(DockTab::Palette.to_string(), "Palette");
    assert_eq!(DockTab::Design.to_string(), "Design");
    assert_eq!(DockTab::Rules.to_string(), "Rules");
    assert_eq!(DockTab::Inspector.to_string(), "Inspector");
    assert_eq!(DockTab::Settings.to_string(), "Settings");
    assert_eq!(DockTab::Selection.to_string(), "Selection");
    assert_eq!(DockTab::Validation.to_string(), "Validation");
    assert_eq!(DockTab::MechanicReference.to_string(), "Mechanic Reference");
    assert_eq!(DockTab::MapGenerator.to_string(), "Map Generator");
    assert_eq!(DockTab::Shortcuts.to_string(), "Shortcuts");
}

// ---------------------------------------------------------------------------
// Coverage: DockLayoutState Debug (components.rs)
// ---------------------------------------------------------------------------

#[test]
fn dock_layout_state_debug_output() {
    let state = DockLayoutState::default();
    let debug = format!("{state:?}");
    assert!(debug.contains("DockLayoutState"));
    assert!(debug.contains("MapEditing"));
}

// ---------------------------------------------------------------------------
// Coverage: EditorState default — extended field checks (components.rs)
// ---------------------------------------------------------------------------

#[test]
fn editor_state_default_mechanics_fields() {
    let state = EditorState::default();
    assert!(state.new_phase_name.is_empty());
    assert_eq!(state.new_phase_type_index, 0);
    assert!(state.new_crt_col_label.is_empty());
    assert_eq!(state.new_crt_col_type_index, 0);
    assert!(state.new_crt_col_threshold.is_empty());
    assert!(state.new_crt_row_label.is_empty());
    assert!(state.new_crt_row_die_min.is_empty());
    assert!(state.new_crt_row_die_max.is_empty());
    assert!(state.new_modifier_name.is_empty());
    assert_eq!(state.new_modifier_source_index, 0);
    assert!(state.new_modifier_custom_source.is_empty());
    assert_eq!(state.new_modifier_shift, 0);
    assert_eq!(state.new_modifier_priority, 0);
    assert!(state.crt_outcome_labels.is_empty());
}

#[test]
fn editor_state_default_launcher_fields() {
    let state = EditorState::default();
    assert!(!state.launcher_name_input_visible);
    assert!(state.launcher_project_name.is_empty());
    assert!(!state.launcher_request_focus);
}

#[test]
fn editor_state_default_constraint_fields() {
    let state = EditorState::default();
    assert!(state.new_constraint_description.is_empty());
    assert_eq!(state.new_constraint_concept_index, 0);
    assert_eq!(state.new_constraint_expr_type_index, 0);
    assert_eq!(state.new_constraint_role_index, 0);
    assert!(state.new_constraint_property.is_empty());
    assert_eq!(state.new_constraint_op_index, 0);
    assert!(state.new_constraint_value_str.is_empty());
}

#[test]
fn editor_state_default_combat_and_settings_fields() {
    let state = EditorState::default();
    assert!((state.combat_attacker_strength - 0.0).abs() < f64::EPSILON);
    assert!((state.combat_defender_strength - 0.0).abs() < f64::EPSILON);
    assert_eq!(state.theme_names, vec!["Brand".to_string()]);
    assert_eq!(state.active_theme_name, "brand");
    assert!(state.shortcut_entries.is_empty());
}

#[test]
fn editor_state_default_property_extended_fields() {
    let state = EditorState::default();
    assert_eq!(state.new_prop_entity_ref_role, 0);
    assert_eq!(state.new_prop_list_inner_type, 0);
    assert!(state.new_prop_map_enum_id.is_none());
    assert_eq!(state.new_prop_map_value_type, 0);
    assert!(state.new_prop_struct_id.is_none());
    assert_eq!(state.new_prop_int_range_min, 0);
    assert_eq!(state.new_prop_int_range_max, 100);
    assert!((state.new_prop_float_range_min - 0.0).abs() < f64::EPSILON);
    assert!((state.new_prop_float_range_max - 1.0).abs() < f64::EPSILON);
}

#[test]
fn editor_state_default_relation_fields() {
    let state = EditorState::default();
    assert_eq!(state.new_relation_concept_index, 0);
    assert_eq!(state.new_relation_subject_index, 0);
    assert_eq!(state.new_relation_object_index, 0);
    assert_eq!(state.new_relation_trigger_index, 0);
    assert_eq!(state.new_relation_effect_index, 0);
    assert!(state.new_relation_target_prop.is_empty());
    assert!(state.new_relation_source_prop.is_empty());
    assert_eq!(state.new_relation_operation_index, 0);
}

#[test]
fn editor_state_default_role_allowed_roles() {
    let state = EditorState::default();
    assert_eq!(state.new_role_allowed_roles, vec![false, false]);
    assert!(state.new_role_name.is_empty());
    assert!(state.binding_concept_role_id.is_none());
}

#[test]
fn editor_state_default_visibility_flags() {
    let state = EditorState::default();
    assert!(state.inspector_visible);
    assert!(state.toolbar_visible);
    assert!(!state.debug_panel_visible);
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — tool switching branches
// ---------------------------------------------------------------------------

#[test]
fn tool_select_command_switches_to_select() {
    let mut app = observer_app();
    app.world_mut().insert_resource(EditorTool::Paint);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("tool.select"),
    });
    app.update();

    assert_eq!(*app.world().resource::<EditorTool>(), EditorTool::Select);
}

#[test]
fn tool_paint_command_switches_to_paint() {
    let mut app = observer_app();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("tool.paint"),
    });
    app.update();

    assert_eq!(*app.world().resource::<EditorTool>(), EditorTool::Paint);
}

#[test]
fn tool_place_command_switches_to_place() {
    let mut app = observer_app();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("tool.place"),
    });
    app.update();

    assert_eq!(*app.world().resource::<EditorTool>(), EditorTool::Place);
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — mode switching branches
// ---------------------------------------------------------------------------

#[test]
fn mode_editor_command_sets_editor_state() {
    let mut app = observer_app();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("mode.editor"),
    });
    app.update();

    // NextState should have been set to Editor. After update, the state should
    // be (or transition to) Editor.
    let state = app.world().resource::<State<AppScreen>>();
    assert_eq!(*state.get(), AppScreen::Editor);
}

#[test]
fn mode_close_command_triggers_close_project_event() {
    use hexorder_contracts::persistence::CloseProjectEvent;

    let mut app = observer_app();

    // Add an observer to detect that CloseProjectEvent was triggered.
    let marker = app.world_mut().spawn_empty().id();
    app.add_observer(
        move |_trigger: On<CloseProjectEvent>, mut commands: Commands| {
            commands.entity(marker).despawn();
        },
    );
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("mode.close"),
    });
    app.update();
    // The queued closure runs on the next update.
    app.update();

    assert!(
        app.world().get_entity(marker).is_err(),
        "CloseProjectEvent should have been triggered, despawning the marker"
    );
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — view.toggle_debug_panel
// ---------------------------------------------------------------------------

#[test]
fn toggle_debug_panel_command_flips_visibility() {
    let mut app = observer_app();

    assert!(!app.world().resource::<EditorState>().debug_panel_visible);

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_debug_panel"),
    });
    app.update();

    assert!(app.world().resource::<EditorState>().debug_panel_visible);
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — view.toggle_fullscreen
// ---------------------------------------------------------------------------

#[test]
fn toggle_fullscreen_command_switches_window_mode() {
    use bevy::window::{MonitorSelection, WindowMode};

    let mut app = observer_app();
    app.world_mut().spawn(Window {
        mode: WindowMode::Windowed,
        ..Default::default()
    });
    app.update();

    // Toggle to fullscreen.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_fullscreen"),
    });
    app.update();

    let window = app
        .world_mut()
        .query::<&Window>()
        .single(app.world())
        .expect("single window");
    assert_eq!(
        window.mode,
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    );

    // Toggle back to windowed.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("view.toggle_fullscreen"),
    });
    app.update();

    let window = app
        .world_mut()
        .query::<&Window>()
        .single(app.world())
        .expect("single window");
    assert_eq!(window.mode, WindowMode::Windowed);
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — edit.deselect (escape)
// ---------------------------------------------------------------------------

#[test]
fn deselect_command_exits_fullscreen() {
    use bevy::window::{MonitorSelection, WindowMode};

    let mut app = observer_app();
    app.world_mut().spawn(Window {
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        ..Default::default()
    });
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.deselect"),
    });
    app.update();

    let window = app
        .world_mut()
        .query::<&Window>()
        .single(app.world())
        .expect("single window");
    assert_eq!(window.mode, WindowMode::Windowed);
}

#[test]
fn deselect_command_noop_when_already_windowed() {
    use bevy::window::WindowMode;

    let mut app = observer_app();
    app.world_mut().spawn(Window {
        mode: WindowMode::Windowed,
        ..Default::default()
    });
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.deselect"),
    });
    app.update();

    let window = app
        .world_mut()
        .query::<&Window>()
        .single(app.world())
        .expect("single window");
    assert_eq!(window.mode, WindowMode::Windowed);
}

// ---------------------------------------------------------------------------
// Coverage: handle_editor_ui_command — unknown command (wildcard arm)
// ---------------------------------------------------------------------------

#[test]
fn unknown_command_is_noop() {
    let mut app = observer_app();
    let tool_before = *app.world().resource::<EditorTool>();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("nonexistent.command"),
    });
    app.update();

    // Nothing should have changed.
    assert_eq!(*app.world().resource::<EditorTool>(), tool_before);
}

// ---------------------------------------------------------------------------
// Coverage: register_shortcuts
// ---------------------------------------------------------------------------

#[test]
fn register_shortcuts_populates_all_expected_commands() {
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    let expected = [
        "tool.select",
        "tool.paint",
        "tool.place",
        "mode.editor",
        "mode.close",
        "workspace.map_editing",
        "workspace.unit_design",
        "workspace.rule_authoring",
        "workspace.playtesting",
        "edit.select_all",
        "edit.delete",
        "view.toggle_inspector",
        "view.toggle_toolbar",
        "view.toggle_grid_overlay",
        "view.zoom_to_selection",
        "view.toggle_fullscreen",
        "help.about",
    ];

    for id in &expected {
        let found = registry.commands().iter().any(|entry| entry.id.0 == *id);
        assert!(found, "command '{id}' should be registered");
    }
}

#[test]
fn register_shortcuts_tool_bindings_use_digit_keys() {
    use bevy::input::keyboard::KeyCode;
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);

    let select_bindings = registry.bindings_for("tool.select");
    assert!(
        select_bindings.contains(&KeyCode::Digit1),
        "tool.select should bind to Digit1"
    );

    let paint_bindings = registry.bindings_for("tool.paint");
    assert!(
        paint_bindings.contains(&KeyCode::Digit2),
        "tool.paint should bind to Digit2"
    );

    let place_bindings = registry.bindings_for("tool.place");
    assert!(
        place_bindings.contains(&KeyCode::Digit3),
        "tool.place should bind to Digit3"
    );
}

// ---------------------------------------------------------------------------
// Coverage: delete_unit fallback (entity without unit components)
// ---------------------------------------------------------------------------

#[test]
fn workspace_map_editing_command_applies_preset() {
    use super::components::WorkspacePreset;

    let mut app = observer_app();

    // First switch away from MapEditing.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("workspace.playtesting"),
    });
    app.update();
    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::Playtesting,
    );

    // Now switch to MapEditing.
    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("workspace.map_editing"),
    });
    app.update();

    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::MapEditing,
    );
}

#[test]
fn workspace_unit_design_command_applies_preset() {
    use super::components::WorkspacePreset;

    let mut app = observer_app();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("workspace.unit_design"),
    });
    app.update();

    assert_eq!(
        app.world().resource::<DockLayoutState>().active_preset,
        WorkspacePreset::UnitDesign,
    );
}

// ---------------------------------------------------------------------------
// NOTE: EditorUiPlugin::build() is a structural ceiling (lines 40-152).
// EguiPlugin requires Assets<Shader> and the rendering pipeline, which
// MinimalPlugins does not provide. The build() method is 113 lines of
// plugin wiring (resource inserts + system registrations) that cannot
// be tested without a full rendering context.

// ---------------------------------------------------------------------------
// Coverage: delete_unit fallback (entity without unit components)
// ---------------------------------------------------------------------------

#[test]
fn delete_unit_plain_despawn_without_unit_components() {
    let mut app = observer_app();

    // Spawn a plain entity (no UnitInstance components).
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);
    app.update();

    app.world_mut().trigger(CommandExecutedEvent {
        command_id: CommandId("edit.delete"),
    });
    app.update();

    // Entity should be despawned even without unit components.
    assert!(
        app.world().get_entity(entity).is_err(),
        "Plain entity should be despawned by delete fallback"
    );
    assert!(app.world().resource::<SelectedUnit>().entity.is_none());
}
