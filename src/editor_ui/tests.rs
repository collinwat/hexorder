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
    assert_eq!(tabs.len(), 9);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
    assert!(tabs.contains(&DockTab::MapGenerator));
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
    // Verify the default layout created 9 tabs.
    let mut count = 0;
    for node in state.dock_state.main_surface().iter() {
        if let Some(tabs) = node.tabs() {
            count += tabs.len();
        }
    }
    assert_eq!(count, 9);
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
fn map_editing_layout_contains_all_9_tabs() {
    let state = super::components::create_default_dock_layout();
    let tabs = collect_tabs(&state);
    assert_eq!(tabs.len(), 9);
    assert!(tabs.contains(&DockTab::Viewport));
    assert!(tabs.contains(&DockTab::Palette));
    assert!(tabs.contains(&DockTab::Design));
    assert!(tabs.contains(&DockTab::Rules));
    assert!(tabs.contains(&DockTab::Inspector));
    assert!(tabs.contains(&DockTab::Settings));
    assert!(tabs.contains(&DockTab::Selection));
    assert!(tabs.contains(&DockTab::Validation));
    assert!(tabs.contains(&DockTab::MapGenerator));
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
    assert_eq!(tabs.len(), 9);
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

use crate::contracts::game_system::{EntityRole, EntityTypeRegistry, EnumDefinition, EnumRegistry};
use crate::contracts::mechanic_reference::{ScaffoldAction, ScaffoldRecipe};
use crate::contracts::mechanics::{
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
    use crate::contracts::game_system::PropertyType;

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
    use crate::contracts::game_system::PropertyType;

    let empty = EnumRegistry::default();
    let result = super::systems::parse_scaffold_prop_type("IntRange(0,20)", &empty);
    assert_eq!(result, PropertyType::IntRange { min: 0, max: 20 });
}

#[test]
fn parse_scaffold_prop_type_float_range() {
    use crate::contracts::game_system::PropertyType;

    let empty = EnumRegistry::default();
    let result = super::systems::parse_scaffold_prop_type("FloatRange(0.0,1.0)", &empty);
    assert_eq!(result, PropertyType::FloatRange { min: 0.0, max: 1.0 });
}

#[test]
fn parse_scaffold_prop_type_enum_lookup() {
    use crate::contracts::game_system::{PropertyType, TypeId};

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
        .resource::<crate::contracts::mechanic_reference::MechanicCatalog>();
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
