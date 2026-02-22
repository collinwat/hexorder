//! Unit tests for the `editor_ui` plugin.

use bevy::prelude::*;

use crate::contracts::editor_ui::{EditorTool, Selection, ToastEvent, ToastKind};
use crate::contracts::game_system::SelectedUnit;
use crate::contracts::hex_grid::HexTile;
use crate::contracts::persistence::AppScreen;
use crate::contracts::shortcuts::{CommandExecutedEvent, CommandId};

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
    assert_ne!(DockTab::Palette, DockTab::Design);
    assert_ne!(DockTab::Design, DockTab::Rules);
    assert_ne!(DockTab::Inspector, DockTab::Settings);
    assert_ne!(DockTab::Settings, DockTab::Selection);
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
    assert_eq!(tabs.len(), 8);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
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
}

#[test]
fn dock_layout_state_resource_inserts_correctly() {
    let mut app = App::new();
    app.init_resource::<DockLayoutState>();
    app.update();

    let state = app.world().resource::<DockLayoutState>();
    // Verify the default layout created 8 tabs.
    let mut count = 0;
    for node in state.dock_state.main_surface().iter() {
        if let Some(tabs) = node.tabs() {
            count += tabs.len();
        }
    }
    assert_eq!(count, 8);
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
fn map_editing_layout_contains_all_8_tabs() {
    let state = super::components::create_default_dock_layout();
    let tabs = collect_tabs(&state);
    assert_eq!(tabs.len(), 8);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
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
    assert_eq!(tabs.len(), 8);
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
    let ws = crate::contracts::persistence::Workspace::default();
    assert!(ws.workspace_preset.is_empty());
}
