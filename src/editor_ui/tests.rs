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

// ===========================================================================
// Coverage: editor_ui/systems.rs — pure logic helpers
// ===========================================================================

// ---------------------------------------------------------------------------
// format_property_type
// ---------------------------------------------------------------------------

#[test]
fn format_property_type_all_variants() {
    use hexorder_contracts::game_system::{PropertyType, TypeId};

    let cases: Vec<(PropertyType, &str)> = vec![
        (PropertyType::Bool, "Bool"),
        (PropertyType::Int, "Int"),
        (PropertyType::Float, "Float"),
        (PropertyType::String, "String"),
        (PropertyType::Color, "Color"),
        (PropertyType::Enum(TypeId::new()), "Enum"),
        (PropertyType::EntityRef(None), "EntityRef"),
        (PropertyType::List(Box::new(PropertyType::Int)), "List"),
        (
            PropertyType::Map(TypeId::new(), Box::new(PropertyType::Int)),
            "Map",
        ),
        (PropertyType::Struct(TypeId::new()), "Struct"),
        (PropertyType::IntRange { min: 0, max: 10 }, "IntRange"),
        (
            PropertyType::FloatRange { min: 0.0, max: 1.0 },
            "FloatRange",
        ),
    ];

    for (pt, expected) in &cases {
        assert_eq!(
            super::systems::format_property_type(pt),
            *expected,
            "format_property_type({pt:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// index_to_property_type
// ---------------------------------------------------------------------------

#[test]
fn index_to_property_type_all_indices() {
    use hexorder_contracts::game_system::PropertyType;

    // Index 0 and any out-of-range → Bool (wildcard).
    assert!(matches!(
        super::systems::index_to_property_type(0),
        PropertyType::Bool
    ));
    assert!(matches!(
        super::systems::index_to_property_type(1),
        PropertyType::Int
    ));
    assert!(matches!(
        super::systems::index_to_property_type(2),
        PropertyType::Float
    ));
    assert!(matches!(
        super::systems::index_to_property_type(3),
        PropertyType::String
    ));
    assert!(matches!(
        super::systems::index_to_property_type(4),
        PropertyType::Color
    ));
    assert!(matches!(
        super::systems::index_to_property_type(5),
        PropertyType::Enum(_)
    ));
    assert!(matches!(
        super::systems::index_to_property_type(6),
        PropertyType::EntityRef(_)
    ));
    assert!(matches!(
        super::systems::index_to_property_type(7),
        PropertyType::List(_)
    ));
    assert!(matches!(
        super::systems::index_to_property_type(8),
        PropertyType::Map(_, _)
    ));
    assert!(matches!(
        super::systems::index_to_property_type(9),
        PropertyType::Struct(_)
    ));
    assert!(matches!(
        super::systems::index_to_property_type(10),
        PropertyType::IntRange { .. }
    ));
    assert!(matches!(
        super::systems::index_to_property_type(11),
        PropertyType::FloatRange { .. }
    ));
    // Out of range → Bool.
    assert!(matches!(
        super::systems::index_to_property_type(99),
        PropertyType::Bool
    ));
}

// ---------------------------------------------------------------------------
// format_compare_op / index_to_compare_op
// ---------------------------------------------------------------------------

#[test]
fn format_compare_op_all_variants() {
    use hexorder_contracts::ontology::CompareOp;

    assert_eq!(super::systems::format_compare_op(CompareOp::Eq), "==");
    assert_eq!(super::systems::format_compare_op(CompareOp::Ne), "!=");
    assert_eq!(super::systems::format_compare_op(CompareOp::Lt), "<");
    assert_eq!(super::systems::format_compare_op(CompareOp::Le), "<=");
    assert_eq!(super::systems::format_compare_op(CompareOp::Gt), ">");
    assert_eq!(super::systems::format_compare_op(CompareOp::Ge), ">=");
}

#[test]
fn index_to_compare_op_all_indices() {
    use hexorder_contracts::ontology::CompareOp;

    assert_eq!(super::systems::index_to_compare_op(0), CompareOp::Eq);
    assert_eq!(super::systems::index_to_compare_op(1), CompareOp::Ne);
    assert_eq!(super::systems::index_to_compare_op(2), CompareOp::Lt);
    assert_eq!(super::systems::index_to_compare_op(3), CompareOp::Le);
    assert_eq!(super::systems::index_to_compare_op(4), CompareOp::Gt);
    assert_eq!(super::systems::index_to_compare_op(5), CompareOp::Ge);
    // Out of range → Eq.
    assert_eq!(super::systems::index_to_compare_op(99), CompareOp::Eq);
}

// ---------------------------------------------------------------------------
// index_to_modify_operation
// ---------------------------------------------------------------------------

#[test]
fn index_to_modify_operation_all_indices() {
    use hexorder_contracts::ontology::ModifyOperation;

    assert_eq!(
        super::systems::index_to_modify_operation(0),
        ModifyOperation::Add
    );
    assert_eq!(
        super::systems::index_to_modify_operation(1),
        ModifyOperation::Subtract
    );
    assert_eq!(
        super::systems::index_to_modify_operation(2),
        ModifyOperation::Multiply
    );
    assert_eq!(
        super::systems::index_to_modify_operation(3),
        ModifyOperation::Min
    );
    assert_eq!(
        super::systems::index_to_modify_operation(4),
        ModifyOperation::Max
    );
    // Out of range → Add.
    assert_eq!(
        super::systems::index_to_modify_operation(99),
        ModifyOperation::Add
    );
}

// ---------------------------------------------------------------------------
// format_relation_effect
// ---------------------------------------------------------------------------

#[test]
fn format_relation_effect_modify_property() {
    use hexorder_contracts::ontology::{ModifyOperation, RelationEffect};

    let effect = RelationEffect::ModifyProperty {
        target_property: "hp".to_string(),
        source_property: "damage".to_string(),
        operation: ModifyOperation::Subtract,
    };
    assert_eq!(
        super::systems::format_relation_effect(&effect),
        "hp - damage"
    );
}

#[test]
fn format_relation_effect_block_and_allow() {
    use hexorder_contracts::ontology::RelationEffect;

    let block = RelationEffect::Block { condition: None };
    assert_eq!(super::systems::format_relation_effect(&block), "Block");

    let allow = RelationEffect::Allow { condition: None };
    assert_eq!(super::systems::format_relation_effect(&allow), "Allow");
}

// ---------------------------------------------------------------------------
// format_constraint_expr
// ---------------------------------------------------------------------------

#[test]
fn format_constraint_expr_property_compare() {
    use hexorder_contracts::game_system::{PropertyValue, TypeId};
    use hexorder_contracts::ontology::{CompareOp, ConstraintExpr};

    let expr = ConstraintExpr::PropertyCompare {
        role_id: TypeId::new(),
        property_name: "hp".to_string(),
        operator: CompareOp::Gt,
        value: PropertyValue::Int(0),
    };
    assert_eq!(super::systems::format_constraint_expr(&expr), "hp > Int(0)");
}

#[test]
fn format_constraint_expr_cross_compare() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::{CompareOp, ConstraintExpr};

    let expr = ConstraintExpr::CrossCompare {
        left_role_id: TypeId::new(),
        left_property: "attack".to_string(),
        right_role_id: TypeId::new(),
        right_property: "defense".to_string(),
        operator: CompareOp::Ge,
    };
    assert_eq!(
        super::systems::format_constraint_expr(&expr),
        "attack >= defense"
    );
}

#[test]
fn format_constraint_expr_is_type_and_not_type() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::ConstraintExpr;

    let is_type = ConstraintExpr::IsType {
        role_id: TypeId::new(),
        entity_type_id: TypeId::new(),
    };
    assert_eq!(super::systems::format_constraint_expr(&is_type), "is type");

    let is_not = ConstraintExpr::IsNotType {
        role_id: TypeId::new(),
        entity_type_id: TypeId::new(),
    };
    assert_eq!(
        super::systems::format_constraint_expr(&is_not),
        "is not type"
    );
}

#[test]
fn format_constraint_expr_path_budget() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::ConstraintExpr;

    let expr = ConstraintExpr::PathBudget {
        concept_id: TypeId::new(),
        cost_property: "movement_cost".to_string(),
        cost_role_id: TypeId::new(),
        budget_property: "speed".to_string(),
        budget_role_id: TypeId::new(),
    };
    assert_eq!(
        super::systems::format_constraint_expr(&expr),
        "sum(path.movement_cost) <= speed"
    );
}

#[test]
fn format_constraint_expr_all_and_any_and_not() {
    use hexorder_contracts::game_system::{PropertyValue, TypeId};
    use hexorder_contracts::ontology::{CompareOp, ConstraintExpr};

    let inner = ConstraintExpr::PropertyCompare {
        role_id: TypeId::new(),
        property_name: "x".to_string(),
        operator: CompareOp::Eq,
        value: PropertyValue::Int(1),
    };

    let all = ConstraintExpr::All(vec![inner.clone(), inner.clone()]);
    let result = super::systems::format_constraint_expr(&all);
    assert!(result.starts_with('('));
    assert!(result.contains(" AND "));

    let any = ConstraintExpr::Any(vec![inner.clone(), inner.clone()]);
    let result = super::systems::format_constraint_expr(&any);
    assert!(result.starts_with('('));
    assert!(result.contains(" OR "));

    let not = ConstraintExpr::Not(Box::new(inner));
    let result = super::systems::format_constraint_expr(&not);
    assert!(result.starts_with("NOT ("));
}

// ---------------------------------------------------------------------------
// Color conversion functions
// ---------------------------------------------------------------------------

#[test]
fn bevy_color_to_egui_srgba() {
    let color = Color::srgba(1.0, 0.0, 0.0, 1.0);
    let egui_color = super::systems::bevy_color_to_egui(color);
    assert_eq!(egui_color.r(), 255);
    assert_eq!(egui_color.g(), 0);
    assert_eq!(egui_color.b(), 0);
    assert_eq!(egui_color.a(), 255);
}

#[test]
fn bevy_color_to_egui_linear_rgba() {
    let color = Color::linear_rgba(1.0, 0.0, 0.0, 1.0);
    let egui_color = super::systems::bevy_color_to_egui(color);
    // LinearRgba(1.0, 0, 0) converts to sRGBA — red should be high.
    assert!(egui_color.r() > 200);
    assert_eq!(egui_color.g(), 0);
    assert_eq!(egui_color.b(), 0);
}

#[test]
fn bevy_color_to_egui_other_variant_falls_back() {
    // Hsla is an "other" variant — should return TEXT_SECONDARY.
    let color = Color::hsla(0.0, 1.0, 0.5, 1.0);
    let egui_color = super::systems::bevy_color_to_egui(color);
    assert_eq!(egui_color, super::components::BrandTheme::TEXT_SECONDARY);
}

#[test]
fn egui_color_to_bevy_round_trip() {
    let egui_color = bevy_egui::egui::Color32::from_rgb(128, 64, 255);
    let bevy_color = super::systems::egui_color_to_bevy(egui_color);
    match bevy_color {
        Color::Srgba(c) => {
            assert!((c.red - 128.0 / 255.0).abs() < 0.01);
            assert!((c.green - 64.0 / 255.0).abs() < 0.01);
            assert!((c.blue - 1.0).abs() < 0.01);
        }
        _ => panic!("Expected Srgba"),
    }
}

#[test]
fn rgb_to_color32_and_back() {
    let rgb = [0.5, 0.25, 0.75];
    let c32 = super::systems::rgb_to_color32(rgb);
    let back = super::systems::color32_to_rgb(c32);
    // Allow rounding error from u8 conversion.
    assert!((back[0] - rgb[0]).abs() < 0.01);
    assert!((back[1] - rgb[1]).abs() < 0.01);
    assert!((back[2] - rgb[2]).abs() < 0.01);
}

#[test]
fn rgb_helper_creates_color32() {
    let c = super::systems::rgb([255, 128, 0]);
    assert_eq!(c.r(), 255);
    assert_eq!(c.g(), 128);
    assert_eq!(c.b(), 0);
}

// ---------------------------------------------------------------------------
// build_constraint_expression
// ---------------------------------------------------------------------------

#[test]
fn build_constraint_expression_property_compare() {
    use hexorder_contracts::ontology::{CompareOp, ConstraintExpr};

    let state = EditorState {
        new_constraint_expr_type_index: 0,
        new_constraint_property: "hp".to_string(),
        new_constraint_op_index: 4, // Gt
        new_constraint_value_str: "10".to_string(),
        ..EditorState::default()
    };

    let expr = super::systems::build_constraint_expression(&state, &[]);
    match expr {
        ConstraintExpr::PropertyCompare {
            property_name,
            operator,
            ..
        } => {
            assert_eq!(property_name, "hp");
            assert_eq!(operator, CompareOp::Gt);
        }
        _ => panic!("Expected PropertyCompare, got {expr:?}"),
    }
}

#[test]
fn build_constraint_expression_path_budget() {
    use hexorder_contracts::ontology::ConstraintExpr;

    let state = EditorState {
        new_constraint_expr_type_index: 3,
        new_constraint_property: "cost".to_string(),
        new_constraint_value_str: "budget".to_string(),
        ..EditorState::default()
    };

    let expr = super::systems::build_constraint_expression(&state, &[]);
    match expr {
        ConstraintExpr::PathBudget {
            cost_property,
            budget_property,
            ..
        } => {
            assert_eq!(cost_property, "cost");
            assert_eq!(budget_property, "budget");
        }
        _ => panic!("Expected PathBudget, got {expr:?}"),
    }
}

#[test]
fn build_constraint_expression_unknown_falls_back_to_all() {
    use hexorder_contracts::ontology::ConstraintExpr;

    let state = EditorState {
        new_constraint_expr_type_index: 99,
        ..EditorState::default()
    };

    let expr = super::systems::build_constraint_expression(&state, &[]);
    assert!(
        matches!(expr, ConstraintExpr::All(v) if v.is_empty()),
        "Unknown type should fall back to All([])"
    );
}

// ---------------------------------------------------------------------------
// dock_layout_config_path
// ---------------------------------------------------------------------------

#[test]
fn dock_layout_config_path_returns_ron_file() {
    let path = super::systems::dock_layout_config_path();
    assert!(
        path.to_string_lossy().ends_with("dock_layout.ron"),
        "Config path should end with dock_layout.ron, got: {path:?}"
    );
}

// ===========================================================================
// Coverage: editor_ui/systems.rs — settings sync systems
// ===========================================================================

#[test]
fn sync_workspace_preset_updates_workspace() {
    use super::components::WorkspacePreset;
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut layout = DockLayoutState::default();
    layout.apply_preset(WorkspacePreset::UnitDesign);
    app.insert_resource(layout);
    app.insert_resource(Workspace::default());
    app.add_systems(Update, super::systems::sync_workspace_preset);
    app.update();

    let ws = app.world().resource::<Workspace>();
    assert_eq!(ws.workspace_preset, "unit_design");
}

#[test]
fn restore_workspace_preset_applies_from_settings() {
    use super::components::WorkspacePreset;
    use hexorder_contracts::settings::SettingsRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut settings = SettingsRegistry::default();
    settings.editor.workspace_preset = "playtesting".to_string();
    app.insert_resource(settings);
    app.init_resource::<DockLayoutState>();

    app.add_systems(Update, super::systems::restore_workspace_preset);
    app.update();

    let layout = app.world().resource::<DockLayoutState>();
    assert_eq!(layout.active_preset, WorkspacePreset::Playtesting);
}

#[test]
fn restore_workspace_preset_noop_when_empty() {
    use super::components::WorkspacePreset;
    use hexorder_contracts::settings::SettingsRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SettingsRegistry::default()); // empty workspace_preset
    app.init_resource::<DockLayoutState>();

    app.add_systems(Update, super::systems::restore_workspace_preset);
    app.update();

    let layout = app.world().resource::<DockLayoutState>();
    assert_eq!(layout.active_preset, WorkspacePreset::MapEditing);
}

#[test]
fn sync_font_size_updates_workspace() {
    use hexorder_contracts::persistence::Workspace;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(EditorState {
        font_size_base: 20.0,
        ..EditorState::default()
    });
    app.insert_resource(Workspace::default());

    app.add_systems(Update, super::systems::sync_font_size);
    app.update();

    let ws = app.world().resource::<Workspace>();
    assert!((ws.font_size_base - 20.0).abs() < f32::EPSILON);
}

#[test]
fn restore_font_size_reads_from_settings() {
    use hexorder_contracts::settings::SettingsRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut settings = SettingsRegistry::default();
    settings.editor.font_size = 18.0;
    app.insert_resource(settings);
    app.insert_resource(EditorState::default());

    app.add_systems(Update, super::systems::restore_font_size);
    app.update();

    let state = app.world().resource::<EditorState>();
    assert!((state.font_size_base - 18.0).abs() < f32::EPSILON);
}

#[test]
fn sync_theme_updates_settings() {
    use hexorder_contracts::settings::SettingsRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(EditorState {
        active_theme_name: "dark".to_string(),
        ..EditorState::default()
    });
    app.insert_resource(SettingsRegistry::default());

    app.add_systems(Update, super::systems::sync_theme);
    app.update();

    let settings = app.world().resource::<SettingsRegistry>();
    assert_eq!(settings.active_theme, "dark");
}

#[test]
fn restore_theme_populates_editor_state() {
    use hexorder_contracts::settings::{SettingsRegistry, ThemeDefinition, ThemeLibrary};

    fn test_theme(name: &str) -> ThemeDefinition {
        ThemeDefinition {
            name: name.to_string(),
            bg_deep: [0; 3],
            bg_panel: [0; 3],
            bg_surface: [0; 3],
            widget_inactive: [0; 3],
            widget_hovered: [0; 3],
            widget_active: [0; 3],
            accent_primary: [0; 3],
            accent_secondary: [0; 3],
            text_primary: [0; 3],
            text_secondary: [0; 3],
            border: [0; 3],
            danger: [0; 3],
            success: [0; 3],
        }
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(SettingsRegistry {
        active_theme: "custom".to_string(),
        ..SettingsRegistry::default()
    });
    let library = ThemeLibrary {
        themes: vec![test_theme("Brand"), test_theme("Custom")],
    };
    app.insert_resource(library);
    app.insert_resource(EditorState::default());

    app.add_systems(Update, super::systems::restore_theme);
    app.update();

    let state = app.world().resource::<EditorState>();
    assert_eq!(state.theme_names, vec!["Brand", "Custom"]);
    assert_eq!(state.active_theme_name, "custom");
}

#[test]
fn restore_shortcuts_populates_shortcut_entries() {
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut registry = ShortcutRegistry::default();
    super::register_shortcuts(&mut registry);
    app.insert_resource(registry);
    app.insert_resource(EditorState::default());

    app.add_systems(Update, super::systems::restore_shortcuts);
    app.update();

    let state = app.world().resource::<EditorState>();
    assert!(
        !state.shortcut_entries.is_empty(),
        "restore_shortcuts should populate shortcut_entries"
    );
    // Should have entries sorted by category.
    let categories: Vec<&str> = state
        .shortcut_entries
        .iter()
        .map(|e| e.category.as_str())
        .collect();
    assert!(categories.contains(&"Tool"));
    assert!(categories.contains(&"Edit"));
    assert!(categories.contains(&"View"));
}

// ---------------------------------------------------------------------------
// apply_actions tests
// ---------------------------------------------------------------------------

/// Resource used to pass actions into the system wrapper.
#[derive(Resource)]
struct TestActions(Vec<super::components::EditorAction>);

/// System that drains `TestActions` and forwards them to `apply_actions`.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn run_apply_actions(
    mut test_actions: ResMut<TestActions>,
    (mut registry, mut enum_registry, mut struct_registry): (
        ResMut<hexorder_contracts::game_system::EntityTypeRegistry>,
        ResMut<hexorder_contracts::game_system::EnumRegistry>,
        ResMut<hexorder_contracts::game_system::StructRegistry>,
    ),
    mut tile_data_query: Query<
        &mut hexorder_contracts::game_system::EntityData,
        Without<hexorder_contracts::game_system::UnitInstance>,
    >,
    mut active_board: ResMut<hexorder_contracts::game_system::ActiveBoardType>,
    mut active_token: ResMut<hexorder_contracts::game_system::ActiveTokenType>,
    mut selected_unit: ResMut<hexorder_contracts::game_system::SelectedUnit>,
    editor_state: Res<super::components::EditorState>,
    mut commands: Commands,
    mut concept_registry: ResMut<hexorder_contracts::ontology::ConceptRegistry>,
    mut relation_registry: ResMut<hexorder_contracts::ontology::RelationRegistry>,
    mut constraint_registry: ResMut<hexorder_contracts::ontology::ConstraintRegistry>,
    (mut turn_structure, mut combat_results_table, mut combat_modifiers, mechanic_catalog): (
        ResMut<hexorder_contracts::mechanics::TurnStructure>,
        ResMut<hexorder_contracts::mechanics::CombatResultsTable>,
        ResMut<hexorder_contracts::mechanics::CombatModifierRegistry>,
        Res<hexorder_contracts::mechanic_reference::MechanicCatalog>,
    ),
) {
    let actions = std::mem::take(&mut test_actions.0);
    super::systems::apply_actions(
        actions,
        &mut registry,
        &mut enum_registry,
        &mut struct_registry,
        &mut tile_data_query,
        &mut active_board,
        &mut active_token,
        &mut selected_unit,
        &editor_state,
        &mut commands,
        &mut concept_registry,
        &mut relation_registry,
        &mut constraint_registry,
        &mut turn_structure,
        &mut combat_results_table,
        &mut combat_modifiers,
        &mechanic_catalog,
    );
}

/// Creates an `App` with all resources needed by `apply_actions`, runs the
/// given actions through the system wrapper, and returns the app for assertions.
fn actions_app(actions: Vec<super::components::EditorAction>) -> App {
    use hexorder_contracts::game_system::{
        ActiveBoardType, ActiveTokenType, EntityTypeRegistry, EnumRegistry, SelectedUnit,
        StructRegistry,
    };
    use hexorder_contracts::mechanic_reference::MechanicCatalog;
    use hexorder_contracts::mechanics::{
        CombatModifierRegistry, CombatResultsTable, TurnStructure,
    };
    use hexorder_contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<EntityTypeRegistry>();
    app.init_resource::<EnumRegistry>();
    app.init_resource::<StructRegistry>();
    app.init_resource::<ActiveBoardType>();
    app.init_resource::<ActiveTokenType>();
    app.init_resource::<SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<ConceptRegistry>();
    app.init_resource::<RelationRegistry>();
    app.init_resource::<ConstraintRegistry>();
    app.init_resource::<TurnStructure>();
    app.init_resource::<CombatResultsTable>();
    app.init_resource::<CombatModifierRegistry>();
    app.init_resource::<MechanicCatalog>();
    app.insert_resource(TestActions(actions));
    app.add_systems(Update, run_apply_actions);
    app.update();
    app
}

#[test]
fn apply_actions_create_entity_type() {
    use hexorder_contracts::game_system::{EntityRole, EntityTypeRegistry};

    let app = actions_app(vec![super::components::EditorAction::CreateEntityType {
        name: "Infantry".to_string(),
        role: EntityRole::Token,
        color: Color::WHITE,
    }]);

    let registry = app.world().resource::<EntityTypeRegistry>();
    assert_eq!(registry.types.len(), 1);
    assert_eq!(registry.types[0].name, "Infantry");
    assert_eq!(registry.types[0].role, EntityRole::Token);
}

#[test]
fn apply_actions_delete_entity_type_board_position_with_fallback() {
    use hexorder_contracts::game_system::{
        ActiveBoardType, EntityRole, EntityType, EntityTypeRegistry, TypeId,
    };

    let id_to_delete = TypeId::new();
    let fallback_id = TypeId::new();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: id_to_delete,
        name: "Plains".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    registry.types.push(EntityType {
        id: fallback_id,
        name: "Forest".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::BLACK,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.insert_resource(ActiveBoardType {
        entity_type_id: Some(id_to_delete),
    });
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::DeleteEntityType { id: id_to_delete },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    assert_eq!(reg.types.len(), 1);
    assert_eq!(reg.types[0].name, "Forest");
    let ab = app.world().resource::<ActiveBoardType>();
    assert_eq!(ab.entity_type_id, Some(fallback_id));
}

#[test]
fn apply_actions_delete_entity_type_token_with_fallback() {
    use hexorder_contracts::game_system::{
        ActiveTokenType, EntityRole, EntityType, EntityTypeRegistry, TypeId,
    };

    let id_to_delete = TypeId::new();
    let fallback_id = TypeId::new();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: id_to_delete,
        name: "Tank".to_string(),
        role: EntityRole::Token,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    registry.types.push(EntityType {
        id: fallback_id,
        name: "Soldier".to_string(),
        role: EntityRole::Token,
        color: Color::BLACK,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.insert_resource(ActiveTokenType {
        entity_type_id: Some(id_to_delete),
    });
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::DeleteEntityType { id: id_to_delete },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    assert_eq!(reg.types.len(), 1);
    assert_eq!(reg.types[0].name, "Soldier");
    let at = app.world().resource::<ActiveTokenType>();
    assert_eq!(at.entity_type_id, Some(fallback_id));
}

#[test]
fn apply_actions_add_property_simple_types() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, PropertyType, TypeId,
    };

    let type_id = TypeId::new();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Plains".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Defense".to_string(),
            prop_type: PropertyType::Int,
            enum_options: String::new(),
        },
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Label".to_string(),
            prop_type: PropertyType::String,
            enum_options: String::new(),
        },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    let et = &reg.types[0];
    assert_eq!(et.properties.len(), 2);
    assert_eq!(et.properties[0].name, "Defense");
    assert!(matches!(et.properties[0].property_type, PropertyType::Int));
    assert_eq!(et.properties[1].name, "Label");
}

#[test]
fn apply_actions_add_property_enum_creates_enum_def() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, EnumRegistry, PropertyType, TypeId,
    };

    let type_id = TypeId::new();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Terrain".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::AddProperty {
            type_id,
            name: "TerrainType".to_string(),
            prop_type: PropertyType::Enum(TypeId::default()),
            enum_options: "Forest, Mountain, Plains".to_string(),
        },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let enum_reg = app.world().resource::<EnumRegistry>();
    assert_eq!(enum_reg.definitions.len(), 1);
    let def = enum_reg.definitions.values().next().expect("one enum");
    assert_eq!(def.name, "TerrainType");
    assert_eq!(def.options, vec!["Forest", "Mountain", "Plains"]);
}

#[test]
fn apply_actions_add_property_int_range_and_float_range() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, PropertyType, TypeId,
    };

    let type_id = TypeId::new();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Unit".to_string(),
        role: EntityRole::Token,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState {
        new_prop_int_range_min: 1,
        new_prop_int_range_max: 10,
        new_prop_float_range_min: 0.0,
        new_prop_float_range_max: 100.0,
        ..super::components::EditorState::default()
    });
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Strength".to_string(),
            prop_type: PropertyType::IntRange { min: 0, max: 0 },
            enum_options: String::new(),
        },
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Morale".to_string(),
            prop_type: PropertyType::FloatRange { min: 0.0, max: 0.0 },
            enum_options: String::new(),
        },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    let et = &reg.types[0];
    assert_eq!(et.properties.len(), 2);
    assert!(matches!(
        et.properties[0].property_type,
        PropertyType::IntRange { min: 1, max: 10 }
    ));
    assert!(matches!(
        et.properties[1].property_type,
        PropertyType::FloatRange { min, max } if min == 0.0 && max == 100.0
    ));
}

#[test]
fn apply_actions_add_property_list_and_map_and_struct() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, PropertyType, TypeId,
    };

    let type_id = TypeId::new();
    let struct_id = TypeId::new();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Unit".to_string(),
        role: EntityRole::Token,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState {
        new_prop_list_inner_type: 2, // Float
        new_prop_map_value_type: 1,  // Int
        new_prop_struct_id: Some(struct_id),
        ..super::components::EditorState::default()
    });
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Modifiers".to_string(),
            prop_type: PropertyType::List(Box::new(PropertyType::Bool)),
            enum_options: String::new(),
        },
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Stats".to_string(),
            prop_type: PropertyType::Map(TypeId::default(), Box::new(PropertyType::Bool)),
            enum_options: String::new(),
        },
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Equipment".to_string(),
            prop_type: PropertyType::Struct(TypeId::default()),
            enum_options: String::new(),
        },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    let et = &reg.types[0];
    assert_eq!(et.properties.len(), 3);
    assert!(matches!(
        et.properties[0].property_type,
        PropertyType::List(ref inner) if matches!(**inner, PropertyType::Float)
    ));
    assert!(matches!(
        et.properties[1].property_type,
        PropertyType::Map(_, ref val) if matches!(**val, PropertyType::Int)
    ));
    assert_eq!(
        et.properties[2].property_type,
        PropertyType::Struct(struct_id)
    );
}

#[test]
fn apply_actions_add_property_entity_ref() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, PropertyType, TypeId,
    };

    let type_id = TypeId::new();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Unit".to_string(),
        role: EntityRole::Token,
        color: Color::WHITE,
        properties: Vec::new(),
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState {
        new_prop_entity_ref_role: 2, // Token
        ..super::components::EditorState::default()
    });
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::AddProperty {
            type_id,
            name: "Target".to_string(),
            prop_type: PropertyType::EntityRef(None),
            enum_options: String::new(),
        },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    let et = &reg.types[0];
    assert_eq!(et.properties.len(), 1);
    assert!(matches!(
        et.properties[0].property_type,
        PropertyType::EntityRef(Some(EntityRole::Token))
    ));
}

#[test]
fn apply_actions_remove_property() {
    use hexorder_contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType,
        PropertyValue, TypeId,
    };

    let type_id = TypeId::new();
    let prop_id = TypeId::new();
    let keep_id = TypeId::new();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Plains".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::WHITE,
        properties: vec![
            PropertyDefinition {
                id: prop_id,
                name: "ToRemove".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(0),
            },
            PropertyDefinition {
                id: keep_id,
                name: "ToKeep".to_string(),
                property_type: PropertyType::Bool,
                default_value: PropertyValue::Bool(false),
            },
        ],
    });
    app.insert_resource(registry);
    app.init_resource::<hexorder_contracts::game_system::EnumRegistry>();
    app.init_resource::<hexorder_contracts::game_system::StructRegistry>();
    app.init_resource::<hexorder_contracts::game_system::ActiveBoardType>();
    app.init_resource::<hexorder_contracts::game_system::ActiveTokenType>();
    app.init_resource::<hexorder_contracts::game_system::SelectedUnit>();
    app.insert_resource(super::components::EditorState::default());
    app.init_resource::<hexorder_contracts::ontology::ConceptRegistry>();
    app.init_resource::<hexorder_contracts::ontology::RelationRegistry>();
    app.init_resource::<hexorder_contracts::ontology::ConstraintRegistry>();
    app.init_resource::<hexorder_contracts::mechanics::TurnStructure>();
    app.init_resource::<hexorder_contracts::mechanics::CombatResultsTable>();
    app.init_resource::<hexorder_contracts::mechanics::CombatModifierRegistry>();
    app.init_resource::<hexorder_contracts::mechanic_reference::MechanicCatalog>();
    app.insert_resource(TestActions(vec![
        super::components::EditorAction::RemoveProperty { type_id, prop_id },
    ]));
    app.add_systems(Update, run_apply_actions);
    app.update();

    let reg = app.world().resource::<EntityTypeRegistry>();
    assert_eq!(reg.types[0].properties.len(), 1);
    assert_eq!(reg.types[0].properties[0].name, "ToKeep");
}

#[test]
fn apply_actions_delete_selected_unit() {
    use hexorder_contracts::game_system::SelectedUnit;

    let mut app = actions_app(vec![]);
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().resource_mut::<SelectedUnit>().entity = Some(entity);
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteSelectedUnit);
    app.update();

    let su = app.world().resource::<SelectedUnit>();
    assert!(su.entity.is_none());
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn apply_actions_create_and_delete_concept_cascades() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::{
        ConceptBinding, ConceptRegistry, Constraint, ConstraintExpr, ConstraintRegistry, Relation,
        RelationEffect, RelationRegistry, RelationTrigger,
    };

    let concept_id = TypeId::new();
    // Pre-populate registries so we can verify cascade deletes.
    let mut app = actions_app(vec![]);
    {
        let mut cr = app.world_mut().resource_mut::<ConceptRegistry>();
        cr.concepts.push(hexorder_contracts::ontology::Concept {
            id: concept_id,
            name: "Battle".to_string(),
            description: String::new(),
            role_labels: Vec::new(),
        });
        cr.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: TypeId::new(),
            concept_id,
            concept_role_id: TypeId::new(),
            property_bindings: Vec::new(),
        });
    }
    {
        let mut rr = app.world_mut().resource_mut::<RelationRegistry>();
        rr.relations.push(Relation {
            id: TypeId::new(),
            name: "Attack".to_string(),
            concept_id,
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Allow { condition: None },
        });
    }
    {
        let mut csr = app.world_mut().resource_mut::<ConstraintRegistry>();
        csr.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Limit".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::All(Vec::new()),
            auto_generated: false,
        });
    }
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteConcept { id: concept_id });
    app.update();

    let cr = app.world().resource::<ConceptRegistry>();
    assert!(cr.concepts.is_empty());
    assert!(cr.bindings.is_empty());
    let rr = app.world().resource::<RelationRegistry>();
    assert!(rr.relations.is_empty());
    let csr = app.world().resource::<ConstraintRegistry>();
    assert!(csr.constraints.is_empty());
}

#[test]
fn apply_actions_concept_roles_add_and_remove() {
    use hexorder_contracts::game_system::{EntityRole, TypeId};
    use hexorder_contracts::ontology::{ConceptBinding, ConceptRegistry};

    let mut app = actions_app(vec![super::components::EditorAction::CreateConcept {
        name: "Movement".to_string(),
        description: "How units move".to_string(),
    }]);

    // Grab the generated concept ID.
    let concept_id = app.world().resource::<ConceptRegistry>().concepts[0].id;

    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::AddConceptRole {
            concept_id,
            name: "Mover".to_string(),
            allowed_roles: vec![EntityRole::Token],
        },
    );
    app.update();

    let cr = app.world().resource::<ConceptRegistry>();
    assert_eq!(cr.concepts[0].role_labels.len(), 1);
    assert_eq!(cr.concepts[0].role_labels[0].name, "Mover");

    let role_id = cr.concepts[0].role_labels[0].id;
    // Add a binding that references this role, then remove the role.
    {
        let mut cr = app.world_mut().resource_mut::<ConceptRegistry>();
        cr.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: TypeId::new(),
            concept_id,
            concept_role_id: role_id,
            property_bindings: Vec::new(),
        });
    }
    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::RemoveConceptRole {
            concept_id,
            role_id,
        },
    );
    app.update();

    let cr = app.world().resource::<ConceptRegistry>();
    assert!(cr.concepts[0].role_labels.is_empty());
    assert!(cr.bindings.is_empty(), "cascade should remove binding");
}

#[test]
fn apply_actions_bind_and_unbind_entity() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::ConceptRegistry;

    let entity_type_id = TypeId::new();
    let concept_id = TypeId::new();
    let concept_role_id = TypeId::new();

    let app = actions_app(vec![super::components::EditorAction::BindEntityToConcept {
        entity_type_id,
        concept_id,
        concept_role_id,
    }]);

    let cr = app.world().resource::<ConceptRegistry>();
    assert_eq!(cr.bindings.len(), 1);
    let binding_id = cr.bindings[0].id;

    let mut app = app;
    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::UnbindEntityFromConcept {
            concept_id,
            binding_id,
        },
    );
    app.update();

    let cr = app.world().resource::<ConceptRegistry>();
    assert!(cr.bindings.is_empty());
}

#[test]
fn apply_actions_create_and_delete_relation_cascades() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::{
        Constraint, ConstraintExpr, ConstraintRegistry, RelationEffect, RelationRegistry,
        RelationTrigger,
    };

    let app = actions_app(vec![super::components::EditorAction::CreateRelation {
        name: "Controls".to_string(),
        concept_id: TypeId::new(),
        subject_role_id: TypeId::new(),
        object_role_id: TypeId::new(),
        trigger: RelationTrigger::OnEnter,
        effect: RelationEffect::Allow { condition: None },
    }]);

    let rr = app.world().resource::<RelationRegistry>();
    assert_eq!(rr.relations.len(), 1);
    let relation_id = rr.relations[0].id;

    let mut app = app;
    // Add a constraint tied to this relation.
    {
        let mut csr = app.world_mut().resource_mut::<ConstraintRegistry>();
        csr.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Linked".to_string(),
            description: String::new(),
            concept_id: TypeId::new(),
            relation_id: Some(relation_id),
            expression: ConstraintExpr::All(Vec::new()),
            auto_generated: false,
        });
    }
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteRelation { id: relation_id });
    app.update();

    let rr = app.world().resource::<RelationRegistry>();
    assert!(rr.relations.is_empty());
    let csr = app.world().resource::<ConstraintRegistry>();
    assert!(
        csr.constraints.is_empty(),
        "cascade should remove constraint"
    );
}

#[test]
fn apply_actions_create_and_delete_constraint() {
    use hexorder_contracts::game_system::TypeId;
    use hexorder_contracts::ontology::{ConstraintExpr, ConstraintRegistry};

    let app = actions_app(vec![super::components::EditorAction::CreateConstraint {
        name: "Max3".to_string(),
        description: "Limit to 3".to_string(),
        concept_id: TypeId::new(),
        expression: ConstraintExpr::All(Vec::new()),
    }]);

    let csr = app.world().resource::<ConstraintRegistry>();
    assert_eq!(csr.constraints.len(), 1);
    assert_eq!(csr.constraints[0].name, "Max3");
    assert!(!csr.constraints[0].auto_generated);
    let id = csr.constraints[0].id;

    let mut app = app;
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteConstraint { id });
    app.update();

    let csr = app.world().resource::<ConstraintRegistry>();
    assert!(csr.constraints.is_empty());
}

#[test]
fn apply_actions_enum_create_delete_add_remove_option() {
    use hexorder_contracts::game_system::EnumRegistry;

    let app = actions_app(vec![super::components::EditorAction::CreateEnum {
        name: "Weather".to_string(),
        options: vec!["Clear".to_string(), "Rain".to_string()],
    }]);

    let er = app.world().resource::<EnumRegistry>();
    assert_eq!(er.definitions.len(), 1);
    let enum_id = *er.definitions.keys().next().expect("one enum");

    let mut app = app;
    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::AddEnumOption {
            enum_id,
            option: "Snow".to_string(),
        },
    );
    app.update();

    let er = app.world().resource::<EnumRegistry>();
    assert_eq!(er.definitions[&enum_id].options.len(), 3);

    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::RemoveEnumOption {
            enum_id,
            option: "Rain".to_string(),
        },
    );
    app.update();

    let er = app.world().resource::<EnumRegistry>();
    assert_eq!(er.definitions[&enum_id].options, vec!["Clear", "Snow"]);

    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteEnum { id: enum_id });
    app.update();

    let er = app.world().resource::<EnumRegistry>();
    assert!(er.definitions.is_empty());
}

#[test]
fn apply_actions_struct_create_delete_add_remove_field() {
    use hexorder_contracts::game_system::{PropertyType, StructRegistry};

    let app = actions_app(vec![super::components::EditorAction::CreateStruct {
        name: "Coord".to_string(),
    }]);

    let sr = app.world().resource::<StructRegistry>();
    assert_eq!(sr.definitions.len(), 1);
    let struct_id = *sr.definitions.keys().next().expect("one struct");

    let mut app = app;
    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::AddStructField {
            struct_id,
            name: "x".to_string(),
            prop_type: PropertyType::Int,
        },
    );
    app.update();

    let sr = app.world().resource::<StructRegistry>();
    assert_eq!(sr.definitions[&struct_id].fields.len(), 1);
    let field_id = sr.definitions[&struct_id].fields[0].id;

    app.world_mut().resource_mut::<TestActions>().0.push(
        super::components::EditorAction::RemoveStructField {
            struct_id,
            field_id,
        },
    );
    app.update();

    let sr = app.world().resource::<StructRegistry>();
    assert!(sr.definitions[&struct_id].fields.is_empty());

    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::DeleteStruct { id: struct_id });
    app.update();

    let sr = app.world().resource::<StructRegistry>();
    assert!(sr.definitions.is_empty());
}

#[test]
fn apply_actions_set_player_order_and_phases() {
    use hexorder_contracts::mechanics::{PhaseType, PlayerOrder, TurnStructure};

    let app = actions_app(vec![
        super::components::EditorAction::SetPlayerOrder {
            order: PlayerOrder::Simultaneous,
        },
        super::components::EditorAction::AddPhase {
            name: "Movement".to_string(),
            phase_type: PhaseType::Movement,
        },
        super::components::EditorAction::AddPhase {
            name: "Combat".to_string(),
            phase_type: PhaseType::Combat,
        },
    ]);

    let ts = app.world().resource::<TurnStructure>();
    assert_eq!(ts.player_order, PlayerOrder::Simultaneous);
    assert_eq!(ts.phases.len(), 2);
    assert_eq!(ts.phases[0].name, "Movement");
    assert_eq!(ts.phases[1].name, "Combat");

    // Move phase down, then up.
    let phase_id = ts.phases[0].id;
    let mut app = app;
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::MovePhaseDown { id: phase_id });
    app.update();

    let ts = app.world().resource::<TurnStructure>();
    assert_eq!(ts.phases[0].name, "Combat");
    assert_eq!(ts.phases[1].name, "Movement");

    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::MovePhaseUp { id: phase_id });
    app.update();

    let ts = app.world().resource::<TurnStructure>();
    assert_eq!(ts.phases[0].name, "Movement");

    // Remove a phase.
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::RemovePhase { id: phase_id });
    app.update();

    let ts = app.world().resource::<TurnStructure>();
    assert_eq!(ts.phases.len(), 1);
    assert_eq!(ts.phases[0].name, "Combat");
}

#[test]
fn apply_actions_crt_columns_and_rows() {
    use hexorder_contracts::mechanics::{CombatResultsTable, CrtColumnType};

    let app = actions_app(vec![
        super::components::EditorAction::AddCrtColumn {
            label: "1:1".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 1.0,
        },
        super::components::EditorAction::AddCrtColumn {
            label: "2:1".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 2.0,
        },
        super::components::EditorAction::AddCrtRow {
            label: "1".to_string(),
            die_min: 1,
            die_max: 2,
        },
    ]);

    let crt = app.world().resource::<CombatResultsTable>();
    assert_eq!(crt.columns.len(), 2);
    assert_eq!(crt.rows.len(), 1);
    // Row should have default outcomes matching column count.
    assert_eq!(crt.outcomes.len(), 1);
    assert_eq!(crt.outcomes[0].len(), 2);
    assert_eq!(crt.outcomes[0][0].label, "--");

    // Remove a column — should also trim outcomes.
    let mut app = app;
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::RemoveCrtColumn { index: 0 });
    app.update();

    let crt = app.world().resource::<CombatResultsTable>();
    assert_eq!(crt.columns.len(), 1);
    assert_eq!(crt.outcomes[0].len(), 1);

    // Remove the row.
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::RemoveCrtRow { index: 0 });
    app.update();

    let crt = app.world().resource::<CombatResultsTable>();
    assert!(crt.rows.is_empty());
    assert!(crt.outcomes.is_empty());
}

#[test]
fn apply_actions_set_crt_outcome() {
    use hexorder_contracts::mechanics::{CombatResultsTable, CrtColumnType};

    let app = actions_app(vec![
        super::components::EditorAction::AddCrtColumn {
            label: "1:1".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 1.0,
        },
        super::components::EditorAction::AddCrtRow {
            label: "1".to_string(),
            die_min: 1,
            die_max: 1,
        },
        super::components::EditorAction::SetCrtOutcome {
            row: 0,
            col: 0,
            label: "AE".to_string(),
        },
    ]);

    let crt = app.world().resource::<CombatResultsTable>();
    assert_eq!(crt.outcomes[0][0].label, "AE");
}

#[test]
fn apply_actions_combat_modifiers() {
    use hexorder_contracts::mechanics::{CombatModifierRegistry, ModifierSource};

    let app = actions_app(vec![super::components::EditorAction::AddCombatModifier {
        name: "Terrain Bonus".to_string(),
        source: ModifierSource::DefenderTerrain,
        shift: 2,
        priority: 1,
    }]);

    let cm = app.world().resource::<CombatModifierRegistry>();
    assert_eq!(cm.modifiers.len(), 1);
    assert_eq!(cm.modifiers[0].name, "Terrain Bonus");
    assert_eq!(cm.modifiers[0].column_shift, 2);
    let mod_id = cm.modifiers[0].id;

    let mut app = app;
    app.world_mut()
        .resource_mut::<TestActions>()
        .0
        .push(super::components::EditorAction::RemoveCombatModifier { id: mod_id });
    app.update();

    let cm = app.world().resource::<CombatModifierRegistry>();
    assert!(cm.modifiers.is_empty());
}

#[test]
fn apply_actions_generate_map_inserts_resource() {
    let mut app = actions_app(vec![super::components::EditorAction::GenerateMap]);
    // GenerateMap resource is inserted via commands, needs another update to flush.
    app.update();
    assert!(
        app.world()
            .get_resource::<hexorder_contracts::map_gen::GenerateMap>()
            .is_some(),
        "GenerateMap resource should be inserted"
    );
}

// ===========================================================================
// Coverage: EditorUiPlugin::build() — plugin registration wiring
// ===========================================================================

/// Builds the full `EditorUiPlugin` in a headless context to cover the
/// resource insertion and system registration wiring in `build()`.
///
/// `EguiPlugin` gracefully skips render-pipeline setup when `RenderApp` is
/// absent, so MinimalPlugins + AssetPlugin is sufficient.
#[test]
fn editor_ui_plugin_builds_without_panic() {
    use hexorder_contracts::shortcuts::ShortcutRegistry;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    // EguiPlugin's load_internal_asset! needs Assets<Shader>.
    app.init_resource::<Assets<bevy::shader::Shader>>();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppScreen>();
    app.init_resource::<ShortcutRegistry>();

    // EditorUiPlugin::build() registers resources, systems, and observers.
    // Note: we skip app.update() — EguiPlugin runtime systems need full
    // windowing/input infrastructure. build() coverage is achieved by
    // add_plugins alone.
    app.add_plugins(super::EditorUiPlugin);

    // Verify resources inserted by build().
    assert!(
        app.world().get_resource::<EditorTool>().is_some(),
        "EditorTool should be inserted by build()"
    );
    assert!(
        app.world().get_resource::<EditorState>().is_some(),
        "EditorState should be inserted by build()"
    );
    assert!(
        app.world().get_resource::<Selection>().is_some(),
        "Selection should be inserted by build()"
    );
    assert!(
        app.world().get_resource::<ToastState>().is_some(),
        "ToastState should be inserted by build()"
    );
    assert!(
        app.world().get_resource::<GridOverlayVisible>().is_some(),
        "GridOverlayVisible should be inserted by build()"
    );
}
