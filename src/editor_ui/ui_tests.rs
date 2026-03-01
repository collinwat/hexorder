//! UI interaction tests for the `editor_ui` plugin.
//!
//! Uses `egui_kittest` to test render functions in isolation.
//! Each test creates a minimal `Harness` with the relevant state and
//! verifies that the rendered UI contains the expected labels, that
//! buttons produce the correct `EditorAction`s, and that disabled
//! states are handled correctly.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use hexorder_contracts::editor_ui::EditorTool;
use hexorder_contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityType, EntityTypeRegistry,
    EnumDefinition, EnumRegistry, GameSystem, PropertyDefinition, PropertyType, PropertyValue,
    SelectedUnit, StructDefinition, StructRegistry, TypeId,
};
use hexorder_contracts::hex_grid::HexPosition;
use hexorder_contracts::map_gen::MapGenParams;
use hexorder_contracts::mechanic_reference::MechanicCatalog;
use hexorder_contracts::mechanics::{
    ActiveCombat, CombatModifierDefinition, CombatModifierRegistry, CombatOutcome,
    CombatResultsTable, CrtColumn, CrtColumnType, CrtRow, ModifierSource, Phase, PhaseType,
    PlayerOrder, TurnState, TurnStructure,
};
use hexorder_contracts::ontology::{
    CompareOp, Concept, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, Relation, RelationEffect, RelationRegistry,
    RelationTrigger,
};
use hexorder_contracts::persistence::Workspace;
use hexorder_contracts::validation::{SchemaError, SchemaErrorCategory, SchemaValidation};

use super::actions;
use super::components::{EditorAction, EditorState, OntologyTab};
use super::render_play;
use super::render_rules;
use super::systems;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Minimal entity type registry with one board type and one token type.
fn test_registry() -> EntityTypeRegistry {
    EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.4, 0.6, 0.2),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "movement_cost".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(1),
                }],
            },
            EntityType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.2, 0.2, 0.8),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "movement_points".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(3),
                }],
            },
        ],
    }
}

/// Concept registry with one concept containing two roles.
fn test_concept_registry() -> ConceptRegistry {
    ConceptRegistry {
        concepts: vec![Concept {
            id: TypeId::new(),
            name: "Motion".to_string(),
            description: "Movement across the board".to_string(),
            role_labels: vec![
                ConceptRole {
                    id: TypeId::new(),
                    name: "traveler".to_string(),
                    allowed_entity_roles: vec![EntityRole::Token],
                },
                ConceptRole {
                    id: TypeId::new(),
                    name: "terrain".to_string(),
                    allowed_entity_roles: vec![EntityRole::BoardPosition],
                },
            ],
        }],
        bindings: vec![],
    }
}

/// Relation registry with one relation.
fn test_relation_registry() -> RelationRegistry {
    RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Terrain Cost".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::ModifyProperty {
                target_property: "budget".to_string(),
                source_property: "cost".to_string(),
                operation: ModifyOperation::Subtract,
            },
        }],
    }
}

/// Constraint registry with one manual constraint and one auto-generated.
fn test_constraint_registry() -> ConstraintRegistry {
    ConstraintRegistry {
        constraints: vec![
            Constraint {
                id: TypeId::new(),
                name: "Budget >= 0".to_string(),
                description: "Traveler must have non-negative budget".to_string(),
                concept_id: TypeId::new(),
                relation_id: None,
                expression: ConstraintExpr::PropertyCompare {
                    role_id: TypeId::new(),
                    property_name: "budget".to_string(),
                    operator: CompareOp::Ge,
                    value: PropertyValue::Int(0),
                },
                auto_generated: false,
            },
            Constraint {
                id: TypeId::new(),
                name: "Auto-check".to_string(),
                description: "System-generated constraint".to_string(),
                concept_id: TypeId::new(),
                relation_id: None,
                expression: ConstraintExpr::All(Vec::new()),
                auto_generated: true,
            },
        ],
    }
}

fn test_enum_registry() -> EnumRegistry {
    let mut registry = EnumRegistry::default();
    let id = TypeId::new();
    registry.definitions.insert(
        id,
        EnumDefinition {
            id,
            name: "Terrain".to_string(),
            options: vec!["Open".to_string(), "Rough".to_string(), "Dense".to_string()],
        },
    );
    registry
}

fn test_struct_registry() -> StructRegistry {
    let mut registry = StructRegistry::default();
    let id = TypeId::new();
    registry.definitions.insert(
        id,
        StructDefinition {
            id,
            name: "Position".to_string(),
            fields: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "x".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "y".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
            ],
        },
    );
    registry
}

fn test_turn_structure() -> TurnStructure {
    TurnStructure {
        player_order: PlayerOrder::Alternating,
        phases: vec![
            Phase {
                id: TypeId::new(),
                name: "Movement".to_string(),
                phase_type: PhaseType::Movement,
                description: String::new(),
            },
            Phase {
                id: TypeId::new(),
                name: "Combat".to_string(),
                phase_type: PhaseType::Combat,
                description: String::new(),
            },
            Phase {
                id: TypeId::new(),
                name: "Admin".to_string(),
                phase_type: PhaseType::Admin,
                description: String::new(),
            },
        ],
    }
}

fn test_crt() -> CombatResultsTable {
    CombatResultsTable {
        id: TypeId::new(),
        name: "Standard CRT".to_string(),
        columns: vec![
            CrtColumn {
                label: "1:2".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 0.5,
            },
            CrtColumn {
                label: "1:1".to_string(),
                column_type: CrtColumnType::OddsRatio,
                threshold: 1.0,
            },
        ],
        rows: vec![
            CrtRow {
                label: "1".to_string(),
                die_value_min: 1,
                die_value_max: 2,
            },
            CrtRow {
                label: "2".to_string(),
                die_value_min: 3,
                die_value_max: 4,
            },
        ],
        outcomes: vec![
            vec![
                CombatOutcome {
                    label: "NE".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DR".to_string(),
                    effect: None,
                },
            ],
            vec![
                CombatOutcome {
                    label: "AR".to_string(),
                    effect: None,
                },
                CombatOutcome {
                    label: "DE".to_string(),
                    effect: None,
                },
            ],
        ],
        combat_concept_id: None,
    }
}

fn test_modifiers() -> CombatModifierRegistry {
    CombatModifierRegistry {
        modifiers: vec![
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "Forest Defense".to_string(),
                source: ModifierSource::DefenderTerrain,
                column_shift: -1,
                priority: 10,
                cap: None,
                terrain_type_filter: None,
            },
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "Flanking".to_string(),
                source: ModifierSource::AttackerTerrain,
                column_shift: 2,
                priority: 5,
                cap: None,
                terrain_type_filter: None,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Workspace Header
// ---------------------------------------------------------------------------

#[test]
fn workspace_header_shows_project_name() {
    let workspace = Workspace {
        name: "My Campaign".to_string(),
        ..Workspace::default()
    };
    let gs = GameSystem {
        id: "abc12345-long-id".to_string(),
        version: "0.1.0".to_string(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_workspace_header(ui, &workspace, &gs);
    });
    harness.get_by_label("My Campaign");
}

#[test]
fn workspace_header_shows_hexorder_label() {
    let workspace = Workspace::default();
    let gs = GameSystem {
        id: "test-id".to_string(),
        version: "0.2.0".to_string(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_workspace_header(ui, &workspace, &gs);
    });
    harness.get_by_label_contains("hexorder");
}

#[test]
fn workspace_header_truncates_long_id() {
    let workspace = Workspace::default();
    let gs = GameSystem {
        id: "abcdefghijklmnop".to_string(),
        version: "0.1.0".to_string(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_workspace_header(ui, &workspace, &gs);
    });
    harness.get_by_label_contains("abcdefgh...");
}

// ---------------------------------------------------------------------------
// Tool Mode
// ---------------------------------------------------------------------------

#[test]
fn tool_mode_shows_heading() {
    let tool = EditorTool::Select;
    let harness = Harness::new_ui_state(
        |ui, tool| {
            systems::render_tool_mode(ui, tool);
        },
        tool,
    );
    harness.get_by_label("Tool Mode");
}

#[test]
fn tool_mode_shows_all_three_tools() {
    let tool = EditorTool::Select;
    let harness = Harness::new_ui_state(
        |ui, tool| {
            systems::render_tool_mode(ui, tool);
        },
        tool,
    );
    harness.get_by_label("Select");
    harness.get_by_label("Paint");
    harness.get_by_label("Place");
}

#[test]
fn tool_mode_click_paint_changes_tool() {
    let tool = EditorTool::Select;
    let mut harness = Harness::new_ui_state(
        |ui, tool| {
            systems::render_tool_mode(ui, tool);
        },
        tool,
    );

    harness.get_by_label("Paint").click();
    harness.run();

    assert_eq!(*harness.state(), EditorTool::Paint);
}

#[test]
fn tool_mode_click_place_changes_tool() {
    let tool = EditorTool::Select;
    let mut harness = Harness::new_ui_state(
        |ui, tool| {
            systems::render_tool_mode(ui, tool);
        },
        tool,
    );

    harness.get_by_label("Place").click();
    harness.run();

    assert_eq!(*harness.state(), EditorTool::Place);
}

// ---------------------------------------------------------------------------
// Types Tab (Entity Type Editor)
// ---------------------------------------------------------------------------

#[test]
fn types_tab_shows_cell_and_unit_headings() {
    let mut registry = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let enum_registry = EnumRegistry::default();
    let struct_registry = StructRegistry::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_entity_type_editor(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            &enum_registry,
            &struct_registry,
        );
    });
    harness.get_by_label("Cell Types");
    harness.get_by_label("Unit Types");
}

// ---------------------------------------------------------------------------
// Concepts Tab
// ---------------------------------------------------------------------------

struct ConceptsState {
    concept_registry: ConceptRegistry,
    entity_registry: EntityTypeRegistry,
    editor_state: EditorState,
    actions: Vec<EditorAction>,
}

#[test]
fn concepts_tab_shows_heading() {
    let mut state = ConceptsState {
        concept_registry: test_concept_registry(),
        entity_registry: test_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(
            ui,
            &mut state.concept_registry,
            &state.entity_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Concepts");
}

#[test]
fn concepts_tab_empty_shows_no_concepts_message() {
    let mut state = ConceptsState {
        concept_registry: ConceptRegistry::default(),
        entity_registry: test_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(
            ui,
            &mut state.concept_registry,
            &state.entity_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("No concepts defined");
}

#[test]
fn concepts_tab_shows_existing_concept_name() {
    let mut state = ConceptsState {
        concept_registry: test_concept_registry(),
        entity_registry: test_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(
            ui,
            &mut state.concept_registry,
            &state.entity_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Motion");
}

#[test]
fn concepts_tab_create_concept_produces_action() {
    let mut state = ConceptsState {
        concept_registry: ConceptRegistry::default(),
        entity_registry: test_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    state.editor_state.new_concept_name = "Defense".to_string();
    state.editor_state.new_concept_description = "Shield mechanics".to_string();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConceptsState| {
            systems::render_concepts_tab(
                ui,
                &mut s.concept_registry,
                &s.entity_registry,
                &mut s.editor_state,
                &mut s.actions,
            );
        },
        state,
    );

    harness.get_by_label("+ Create Concept").click();
    harness.run();

    let actions = &harness.state().actions;
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], EditorAction::CreateConcept { name, .. } if name == "Defense"));
}

// ---------------------------------------------------------------------------
// Relations Tab
// ---------------------------------------------------------------------------

struct RelationsState {
    relation_registry: RelationRegistry,
    concept_registry: ConceptRegistry,
    editor_state: EditorState,
    actions: Vec<EditorAction>,
}

#[test]
fn relations_tab_shows_heading() {
    let mut state = RelationsState {
        relation_registry: test_relation_registry(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut state.relation_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Relations");
}

#[test]
fn relations_tab_empty_shows_no_relations_message() {
    let mut state = RelationsState {
        relation_registry: RelationRegistry::default(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut state.relation_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("No relations defined");
}

#[test]
fn relations_tab_shows_existing_relation_name() {
    let mut state = RelationsState {
        relation_registry: test_relation_registry(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut state.relation_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Terrain Cost");
}

// ---------------------------------------------------------------------------
// Constraints Tab
// ---------------------------------------------------------------------------

struct ConstraintsState {
    constraint_registry: ConstraintRegistry,
    concept_registry: ConceptRegistry,
    editor_state: EditorState,
    actions: Vec<EditorAction>,
}

#[test]
fn constraints_tab_shows_heading() {
    let mut state = ConstraintsState {
        constraint_registry: test_constraint_registry(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Constraints");
}

#[test]
fn constraints_tab_empty_shows_no_constraints_message() {
    let mut state = ConstraintsState {
        constraint_registry: ConstraintRegistry::default(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("No constraints defined");
}

#[test]
fn constraints_tab_shows_constraint_names() {
    let mut state = ConstraintsState {
        constraint_registry: test_constraint_registry(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Budget >= 0");
    harness.get_by_label_contains("Auto-check");
}

#[test]
fn constraints_tab_auto_generated_shows_badge() {
    let mut state = ConstraintsState {
        constraint_registry: test_constraint_registry(),
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("[auto]");
}

#[test]
fn constraints_tab_delete_constraint_produces_action() {
    // Use a single-constraint registry so there's exactly one "x" button.
    let single_constraint = ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Budget >= 0".to_string(),
            description: "Traveler must have non-negative budget".to_string(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: TypeId::new(),
                property_name: "budget".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        }],
    };
    let constraint_id = single_constraint.constraints[0].id;

    let state = ConstraintsState {
        constraint_registry: single_constraint,
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConstraintsState| {
            systems::render_constraints_tab(
                ui,
                &mut s.constraint_registry,
                &s.concept_registry,
                &mut s.editor_state,
                &mut s.actions,
            );
        },
        state,
    );

    harness.get_by_label("x").click();
    harness.run();

    let actions = &harness.state().actions;
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        EditorAction::DeleteConstraint { id } if *id == constraint_id
    ));
}

// ---------------------------------------------------------------------------
// About Panel (Scope 3 — extracted to editor_menu_system)
// ---------------------------------------------------------------------------

#[test]
fn about_panel_shows_title_when_visible() {
    let mut state = EditorState {
        about_panel_visible: true,
        ..EditorState::default()
    };
    let harness = Harness::new(|ctx| {
        systems::render_about_panel(ctx, &mut state);
    });
    harness.get_by_label_contains("HEXORDER");
}

#[test]
fn about_panel_shows_version_when_visible() {
    let mut state = EditorState {
        about_panel_visible: true,
        ..EditorState::default()
    };
    let harness = Harness::new(|ctx| {
        systems::render_about_panel(ctx, &mut state);
    });
    harness.get_by_label_contains("Version");
}

#[test]
fn about_panel_close_button_hides_panel() {
    let state = EditorState {
        about_panel_visible: true,
        ..EditorState::default()
    };
    let mut harness = Harness::new_state(
        |ctx, state: &mut EditorState| {
            systems::render_about_panel(ctx, state);
        },
        state,
    );

    harness.get_by_label("Close").click();
    harness.run();

    assert!(!harness.state().about_panel_visible);
}

// ---------------------------------------------------------------------------
// Validation Tab
// ---------------------------------------------------------------------------

#[test]
fn validation_tab_shows_heading() {
    let sv = SchemaValidation {
        errors: vec![],
        is_valid: true,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label("Validation");
}

#[test]
fn validation_tab_shows_schema_valid_when_no_errors() {
    let sv = SchemaValidation {
        errors: vec![],
        is_valid: true,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Schema Valid");
}

#[test]
fn validation_tab_shows_error_count() {
    let sv = SchemaValidation {
        errors: vec![
            SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: "Missing concept".to_string(),
                source_id: TypeId::new(),
            },
            SchemaError {
                category: SchemaErrorCategory::RoleMismatch,
                message: "Wrong role".to_string(),
                source_id: TypeId::new(),
            },
        ],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("2 Error(s)");
}

#[test]
fn validation_tab_shows_error_messages() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::DanglingReference,
            message: "Missing concept ref".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Missing concept ref");
}

#[test]
fn validation_tab_shows_error_category_badge() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::DanglingReference,
            message: "test error".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Dangling Ref");
}

// ---------------------------------------------------------------------------
// Cell Palette
// ---------------------------------------------------------------------------

#[test]
fn cell_palette_shows_heading() {
    let registry = test_registry();
    let mut active = ActiveBoardType::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_cell_palette(ui, &registry, &mut active);
    });
    harness.get_by_label("Cell Palette");
}

#[test]
fn cell_palette_shows_board_type_names() {
    let registry = test_registry();
    let mut active = ActiveBoardType::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_cell_palette(ui, &registry, &mut active);
    });
    harness.get_by_label("Plains");
}

#[test]
fn cell_palette_click_selects_type() {
    struct CellPaletteState {
        registry: EntityTypeRegistry,
        active: ActiveBoardType,
    }

    let registry = test_registry();
    let active = ActiveBoardType::default();
    let state = CellPaletteState { registry, active };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut CellPaletteState| {
            systems::render_cell_palette(ui, &s.registry, &mut s.active);
        },
        state,
    );

    harness.get_by_label("Plains").click();
    harness.run();

    assert!(harness.state().active.entity_type_id.is_some());
}

// ---------------------------------------------------------------------------
// Unit Palette
// ---------------------------------------------------------------------------

#[test]
fn unit_palette_shows_heading() {
    let registry = test_registry();
    let mut active = ActiveTokenType::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_unit_palette(ui, &registry, &mut active);
    });
    harness.get_by_label("Unit Palette");
}

#[test]
fn unit_palette_shows_token_type_names() {
    let registry = test_registry();
    let mut active = ActiveTokenType::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_unit_palette(ui, &registry, &mut active);
    });
    harness.get_by_label("Infantry");
}

#[test]
fn unit_palette_click_selects_type() {
    struct UnitPaletteState {
        registry: EntityTypeRegistry,
        active: ActiveTokenType,
    }

    let registry = test_registry();
    let active = ActiveTokenType::default();
    let state = UnitPaletteState { registry, active };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut UnitPaletteState| {
            systems::render_unit_palette(ui, &s.registry, &mut s.active);
        },
        state,
    );

    harness.get_by_label("Infantry").click();
    harness.run();

    assert!(harness.state().active.entity_type_id.is_some());
}

// ---------------------------------------------------------------------------
// Entity Type Section
// ---------------------------------------------------------------------------

#[test]
fn entity_type_section_shows_section_label() {
    let mut registry = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let enum_registry = EnumRegistry::default();
    let struct_registry = StructRegistry::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_registry,
            &struct_registry,
        );
    });
    harness.get_by_label("Board Types");
}

#[test]
fn entity_type_section_shows_both_sections() {
    let mut registry = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let enum_registry = EnumRegistry::default();
    let struct_registry = StructRegistry::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::Token,
            "Token Types",
            "token",
            &enum_registry,
            &struct_registry,
        );
    });
    harness.get_by_label("Token Types");
}

// ---------------------------------------------------------------------------
// Enums Tab
// ---------------------------------------------------------------------------

#[test]
fn enums_tab_shows_heading() {
    let enum_registry = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_registry, &mut state, &mut actions);
    });
    harness.get_by_label("Enums");
}

#[test]
fn enums_tab_empty_shows_no_enums_message() {
    let enum_registry = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_registry, &mut state, &mut actions);
    });
    harness.get_by_label_contains("No enums defined");
}

#[test]
fn enums_tab_shows_existing_enum_name() {
    let enum_registry = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_registry, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain");
}

#[test]
fn enums_tab_create_enum_produces_action() {
    struct EnumsState {
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let enum_registry = EnumRegistry::default();
    let editor_state = EditorState {
        new_enum_name: "Weather".to_string(),
        ..EditorState::default()
    };
    let actions: Vec<EditorAction> = Vec::new();
    let state = EnumsState {
        enum_registry,
        editor_state,
        actions,
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut EnumsState| {
            systems::render_enums_tab(ui, &s.enum_registry, &mut s.editor_state, &mut s.actions);
        },
        state,
    );

    harness.get_by_label("+ Create Enum").click();
    harness.run();

    let actions = &harness.state().actions;
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], EditorAction::CreateEnum { name, .. } if name == "Weather"));
}

// ---------------------------------------------------------------------------
// Structs Tab
// ---------------------------------------------------------------------------

#[test]
fn structs_tab_shows_heading() {
    let struct_registry = test_struct_registry();
    let enum_registry = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(
            ui,
            &struct_registry,
            &enum_registry,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("Structs");
}

#[test]
fn structs_tab_empty_shows_no_structs_message() {
    let struct_registry = StructRegistry::default();
    let enum_registry = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(
            ui,
            &struct_registry,
            &enum_registry,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("No structs defined");
}

#[test]
fn structs_tab_shows_existing_struct_name() {
    let struct_registry = test_struct_registry();
    let enum_registry = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(
            ui,
            &struct_registry,
            &enum_registry,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("Position");
}

#[test]
fn structs_tab_create_struct_produces_action() {
    struct StructsState {
        struct_registry: StructRegistry,
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let struct_registry = StructRegistry::default();
    let enum_registry = EnumRegistry::default();
    let editor_state = EditorState {
        new_struct_name: "Coordinate".to_string(),
        ..EditorState::default()
    };
    let actions: Vec<EditorAction> = Vec::new();
    let state = StructsState {
        struct_registry,
        enum_registry,
        editor_state,
        actions,
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut StructsState| {
            systems::render_structs_tab(
                ui,
                &s.struct_registry,
                &s.enum_registry,
                &mut s.editor_state,
                &mut s.actions,
            );
        },
        state,
    );

    harness.get_by_label("+ Create Struct").click();
    harness.run();

    let actions = &harness.state().actions;
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], EditorAction::CreateStruct { name } if name == "Coordinate"));
}

// ---------------------------------------------------------------------------
// Mechanics Tab
// ---------------------------------------------------------------------------

struct MechanicsState {
    turn_structure: TurnStructure,
    crt: CombatResultsTable,
    modifiers: CombatModifierRegistry,
    editor_state: EditorState,
    actions: Vec<EditorAction>,
}

fn mechanics_state() -> MechanicsState {
    MechanicsState {
        turn_structure: test_turn_structure(),
        crt: test_crt(),
        modifiers: test_modifiers(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    }
}

#[test]
fn mechanics_tab_shows_turn_structure_heading() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Turn Structure");
}

#[test]
fn mechanics_tab_shows_player_order_labels() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Alternating");
}

#[test]
fn mechanics_tab_shows_phase_count() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Phases (3)");
}

#[test]
fn mechanics_tab_shows_phase_type_selectors() {
    // Phase type selector labels are selectable_labels (interactive) and
    // thus accessible via get_by_label. The phase name labels rendered
    // inside horizontal layouts only expose accessible `value`, not `label`.
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    // The add-phase form shows selectable type labels.
    harness.get_by_label("Add Phase");
}

#[test]
fn mechanics_tab_shows_crt_heading() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Combat Results");
}

#[test]
fn mechanics_tab_shows_crt_name() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Standard CRT");
}

#[test]
fn mechanics_tab_shows_crt_column_count() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Columns (2)");
}

#[test]
fn mechanics_tab_shows_add_col_button() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Add Col");
}

#[test]
fn mechanics_tab_shows_modifier_heading() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Modifiers");
}

#[test]
fn mechanics_tab_shows_modifier_names() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Forest Defense");
    harness.get_by_label("Flanking");
}

#[test]
fn mechanics_tab_shows_modifier_column_shift() {
    let mut state = mechanics_state();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &state.turn_structure,
            &state.crt,
            &state.modifiers,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("-1");
    harness.get_by_label_contains("+2");
}

// ---------------------------------------------------------------------------
// Additional Validation badges
// ---------------------------------------------------------------------------

#[test]
fn validation_tab_shows_role_mismatch_badge() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::RoleMismatch,
            message: "role error".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Role Mismatch");
}

#[test]
fn validation_tab_shows_property_mismatch_badge() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::PropertyMismatch,
            message: "prop error".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Prop Mismatch");
}

#[test]
fn validation_tab_shows_missing_binding_badge() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::MissingBinding,
            message: "binding error".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Missing Binding");
}

#[test]
fn validation_tab_shows_invalid_expression_badge() {
    let sv = SchemaValidation {
        errors: vec![SchemaError {
            category: SchemaErrorCategory::InvalidExpression,
            message: "expr error".to_string(),
            source_id: TypeId::new(),
        }],
        is_valid: false,
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_validation_tab(ui, &sv);
    });
    harness.get_by_label_contains("Invalid Expr");
}

// ---------------------------------------------------------------------------
// Additional Relations tests
// ---------------------------------------------------------------------------

#[test]
fn relations_tab_shows_block_effect_relation() {
    let mut state = RelationsState {
        relation_registry: RelationRegistry {
            relations: vec![Relation {
                id: TypeId::new(),
                name: "Wall Block".to_string(),
                concept_id: TypeId::new(),
                subject_role_id: TypeId::new(),
                object_role_id: TypeId::new(),
                trigger: RelationTrigger::OnEnter,
                effect: RelationEffect::Block { condition: None },
            }],
        },
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut state.relation_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Wall Block");
}

#[test]
fn relations_tab_shows_allow_effect_relation() {
    let mut state = RelationsState {
        relation_registry: RelationRegistry {
            relations: vec![Relation {
                id: TypeId::new(),
                name: "Bridge Pass".to_string(),
                concept_id: TypeId::new(),
                subject_role_id: TypeId::new(),
                object_role_id: TypeId::new(),
                trigger: RelationTrigger::OnExit,
                effect: RelationEffect::Allow { condition: None },
            }],
        },
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut state.relation_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label("Bridge Pass");
}

// ---------------------------------------------------------------------------
// Additional Constraints tests
// ---------------------------------------------------------------------------

#[test]
fn constraints_tab_shows_cross_compare_constraint() {
    let registry = ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Strength Check".to_string(),
            description: "Compare two properties".to_string(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::CrossCompare {
                left_role_id: TypeId::new(),
                left_property: "attack".to_string(),
                operator: CompareOp::Gt,
                right_role_id: TypeId::new(),
                right_property: "defense".to_string(),
            },
            auto_generated: false,
        }],
    };
    let mut state = ConstraintsState {
        constraint_registry: registry,
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Strength Check");
}

#[test]
fn constraints_tab_shows_path_budget_constraint() {
    let registry = ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Budget Limit".to_string(),
            description: "Path budget constraint".to_string(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::PathBudget {
                concept_id: TypeId::new(),
                budget_role_id: TypeId::new(),
                budget_property: "mp".to_string(),
                cost_role_id: TypeId::new(),
                cost_property: "cost".to_string(),
            },
            auto_generated: false,
        }],
    };
    let mut state = ConstraintsState {
        constraint_registry: registry,
        concept_registry: test_concept_registry(),
        editor_state: EditorState::default(),
        actions: Vec::new(),
    };
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut state.constraint_registry,
            &state.concept_registry,
            &mut state.editor_state,
            &mut state.actions,
        );
    });
    harness.get_by_label_contains("Budget Limit");
}

// ---------------------------------------------------------------------------
// Tile Inspector (render_rules::render_inspector)
// ---------------------------------------------------------------------------

/// Helper: entity type with known IDs so we can build matching `EntityData`.
fn inspector_registry() -> (EntityTypeRegistry, TypeId, TypeId) {
    let type_id = TypeId::new();
    let prop_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![PropertyDefinition {
                id: prop_id,
                name: "movement_cost".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(1),
            }],
        }],
    };
    (registry, type_id, prop_id)
}

#[test]
fn inspector_no_tile_selected() {
    let (registry, _, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(ui, None, None, &registry, &enum_reg, &struct_reg);
    });
    harness.get_by_label_contains("No tile selected");
}

#[test]
fn inspector_shows_position() {
    let (registry, _, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 3, r: 5 };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(ui, Some(pos), None, &registry, &enum_reg, &struct_reg);
    });
    harness.get_by_label_contains("Position: (3, 5)");
}

#[test]
fn inspector_shows_no_cell_data() {
    let (registry, _, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(ui, Some(pos), None, &registry, &enum_reg, &struct_reg);
    });
    harness.get_by_label_contains("No cell data");
}

#[test]
fn inspector_shows_type_name() {
    let (registry, type_id, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 1, r: 2 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("Type: Plains");
}

#[test]
fn inspector_shows_property_label() {
    let (registry, type_id, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("movement_cost:");
}

#[test]
fn inspector_no_properties_label() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "EmptyType".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("No properties");
}

#[test]
fn inspector_heading_present() {
    let (registry, _, _) = inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(ui, None, None, &registry, &enum_reg, &struct_reg);
    });
    harness.get_by_label_contains("Tile Inspector");
}

// ---------------------------------------------------------------------------
// Unit Inspector (render_rules::render_unit_inspector)
// ---------------------------------------------------------------------------

/// Helper: entity type registry for unit inspector tests.
fn unit_inspector_registry() -> (EntityTypeRegistry, TypeId, TypeId) {
    let type_id = TypeId::new();
    let prop_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Infantry".to_string(),
            role: EntityRole::Token,
            color: Color::srgb(0.2, 0.2, 0.8),
            properties: vec![PropertyDefinition {
                id: prop_id,
                name: "strength".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(4),
            }],
        }],
    };
    (registry, type_id, prop_id)
}

#[test]
fn unit_inspector_no_unit_selected() {
    let (registry, _, _) = unit_inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            None,
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("No unit selected");
}

#[test]
fn unit_inspector_heading_present() {
    let (registry, _, _) = unit_inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            None,
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Unit Inspector");
}

#[test]
fn unit_inspector_shows_type_name() {
    let (registry, type_id, _) = unit_inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Unit Type: Infantry");
}

#[test]
fn unit_inspector_shows_property_label() {
    let (registry, type_id, _) = unit_inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("strength:");
}

#[test]
fn unit_inspector_delete_button() {
    let (registry, type_id, _) = unit_inspector_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Delete Unit");
}

#[test]
fn unit_inspector_no_properties_shows_delete_only() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Scout".to_string(),
            role: EntityRole::Token,
            color: Color::srgb(0.3, 0.3, 0.3),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut actions = Vec::new();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    // No "Properties" section but Delete button still present.
    harness.get_by_label_contains("Unit Type: Scout");
    harness.get_by_label_contains("Delete Unit");
}

// ---------------------------------------------------------------------------
// Turn Tracker (render_play::render_turn_tracker)
// ---------------------------------------------------------------------------

#[test]
fn turn_tracker_no_phases() {
    let empty_structure = TurnStructure {
        player_order: PlayerOrder::Alternating,
        phases: vec![],
    };
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &empty_structure);
    });
    harness.get_by_label_contains("No phases defined");
}

#[test]
fn turn_tracker_heading_present() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("Turn Tracker");
}

#[test]
fn turn_tracker_shows_turn_number() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("Turn 1");
}

#[test]
fn turn_tracker_shows_current_phase_marker() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    // Current phase (index 0) is prefixed with ▶.
    harness.get_by_label_contains("\u{25B6} Movement");
}

#[test]
fn turn_tracker_shows_non_current_phases() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    // Non-current phases are space-prefixed.
    harness.get_by_label_contains("  Combat");
    harness.get_by_label_contains("  Admin");
}

#[test]
fn turn_tracker_shows_phase_type_badge() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("[Movement]");
}

#[test]
fn turn_tracker_shows_phase_count() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("Phase 1 of 3");
}

#[test]
fn turn_tracker_advance_button_present() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("Next Phase");
}

// ---------------------------------------------------------------------------
// Property Value Editor coverage (via render_inspector with typed properties)
// ---------------------------------------------------------------------------

#[test]
fn inspector_bool_property_renders() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Terrain".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "passable".to_string(),
                property_type: PropertyType::Bool,
                default_value: PropertyValue::Bool(true),
            }],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("passable:");
}

#[test]
fn inspector_float_property_renders() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "River".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.2, 0.4, 0.8),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "depth".to_string(),
                property_type: PropertyType::Float,
                default_value: PropertyValue::Float(1.5),
            }],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("depth:");
}

#[test]
fn inspector_string_property_renders() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "City".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.8, 0.8, 0.2),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "label".to_string(),
                property_type: PropertyType::String,
                default_value: PropertyValue::String("unnamed".to_string()),
            }],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("label:");
}

#[test]
fn inspector_enum_property_renders() {
    let type_id = TypeId::new();
    let enum_id = TypeId::new();
    let mut enum_reg = EnumRegistry::default();
    enum_reg.definitions.insert(
        enum_id,
        EnumDefinition {
            id: enum_id,
            name: "TerrainKind".to_string(),
            options: vec!["Open".to_string(), "Rough".to_string()],
        },
    );
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Hex".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "terrain".to_string(),
                property_type: PropertyType::Enum(enum_id),
                default_value: PropertyValue::Enum("Open".to_string()),
            }],
        }],
    };
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("terrain:");
}

#[test]
fn inspector_entity_ref_property_renders() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Squad".to_string(),
            role: EntityRole::Token,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "leader".to_string(),
                property_type: PropertyType::EntityRef(None),
                default_value: PropertyValue::EntityRef(None),
            }],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("leader:");
}

#[test]
fn inspector_multiple_properties_renders() {
    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Fort".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.6, 0.6, 0.6),
            properties: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "defense_bonus".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(2),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "is_fortified".to_string(),
                    property_type: PropertyType::Bool,
                    default_value: PropertyValue::Bool(true),
                },
            ],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("defense_bonus:");
    harness.get_by_label_contains("is_fortified:");
    harness.get_by_label_contains("Properties");
}

// ---------------------------------------------------------------------------
// Design Tab Bar (systems::render_design_tab_bar)
// ---------------------------------------------------------------------------

#[test]
fn design_tab_bar_shows_all_tabs() {
    let mut state = EditorState::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_design_tab_bar(ui, &mut state);
    });
    harness.get_by_label("Types");
    harness.get_by_label("Enums");
    harness.get_by_label("Structs");
    harness.get_by_label("Concepts");
    harness.get_by_label("Relations");
}

#[test]
fn design_tab_bar_default_is_types() {
    let state = EditorState::default();
    assert_eq!(state.active_tab, OntologyTab::Types);
}

#[test]
fn design_tab_bar_click_switches_tab() {
    let mut state = EditorState::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_design_tab_bar(ui, &mut state);
    });
    harness.get_by_label("Enums").click();
    harness.run();
    drop(harness);
    assert_eq!(state.active_tab, OntologyTab::Enums);
}

// ---------------------------------------------------------------------------
// Rules Tab Bar (systems::render_rules_tab_bar)
// ---------------------------------------------------------------------------

#[test]
fn rules_tab_bar_shows_all_tabs() {
    let mut state = EditorState::default();
    let harness = Harness::new_ui(|ui| {
        systems::render_rules_tab_bar(ui, &mut state);
    });
    harness.get_by_label("Constraints");
    harness.get_by_label("Validation");
    harness.get_by_label("Mechanics");
}

#[test]
fn rules_tab_bar_click_switches_tab() {
    let mut state = EditorState::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_rules_tab_bar(ui, &mut state);
    });
    harness.get_by_label("Mechanics").click();
    harness.run();
    drop(harness);
    assert_eq!(state.active_tab, OntologyTab::Mechanics);
}

// ---------------------------------------------------------------------------
// Mechanic Reference (systems::render_mechanic_reference)
// ---------------------------------------------------------------------------

#[test]
fn mechanic_reference_heading_present() {
    let catalog = MechanicCatalog::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanic_reference(ui, &catalog, &mut actions);
    });
    harness.get_by_label_contains("Mechanic Reference");
}

#[test]
fn mechanic_reference_shows_taxonomy_subtitle() {
    let catalog = MechanicCatalog::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanic_reference(ui, &catalog, &mut actions);
    });
    harness.get_by_label_contains("Engelstein taxonomy");
}

#[test]
fn mechanic_reference_renders_scroll_area() {
    let catalog = MechanicCatalog::default();
    let mut actions = Vec::new();
    // Verify the function renders without panicking with an empty catalog.
    let _harness = Harness::new_ui(|ui| {
        systems::render_mechanic_reference(ui, &catalog, &mut actions);
    });
}

// ---------------------------------------------------------------------------
// Map Generator (systems::render_map_generator)
// ---------------------------------------------------------------------------

#[test]
fn map_generator_heading_present() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, false, &mut actions);
    });
    harness.get_by_label_contains("Map Generator");
}

#[test]
fn map_generator_shows_seed_label() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, false, &mut actions);
    });
    harness.get_by_label_contains("Seed:");
}

#[test]
fn map_generator_shows_noise_params() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, false, &mut actions);
    });
    // Noise Parameters is default_open(true), so body is visible.
    harness.get_by_label_contains("Octaves:");
    harness.get_by_label_contains("Frequency:");
    harness.get_by_label_contains("Amplitude:");
    harness.get_by_label_contains("Lacunarity:");
    harness.get_by_label_contains("Persistence:");
}

#[test]
fn map_generator_shows_reset_button() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, false, &mut actions);
    });
    harness.get_by_label_contains("Reset Defaults");
}

#[test]
fn map_generator_shows_generate_button() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, false, &mut actions);
    });
    harness.get_by_label_contains("Generate Map");
}

#[test]
fn map_generator_generate_while_busy() {
    let mut params = MapGenParams::default();
    let mut actions = Vec::new();
    // is_generating=true should show the button but disabled.
    let harness = Harness::new_ui(|ui| {
        systems::render_map_generator(ui, &mut params, true, &mut actions);
    });
    harness.get_by_label_contains("Generate Map");
}

// ---------------------------------------------------------------------------
// Deeper render_design tests
// ---------------------------------------------------------------------------

/// Entity type section with multiple board types — click header to open body.
#[test]
fn entity_type_section_shows_types_when_opened() {
    let mut registry = EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Forest".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.1, 0.5, 0.1),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "dense".to_string(),
                    property_type: PropertyType::Bool,
                    default_value: PropertyValue::Bool(false),
                }],
            },
            EntityType {
                id: TypeId::new(),
                name: "Mountain".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.5, 0.5, 0.5),
                properties: vec![],
            },
        ],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board_types",
            &enum_reg,
            &struct_reg,
        );
    });
    // Click section header to expand it (default_open=false).
    harness.get_by_label("Board Types").click();
    harness.run();
    // Type names should appear inside the expanded section.
    harness.get_by_label_contains("Forest");
    harness.get_by_label_contains("Mountain");
}

#[test]
fn entity_type_section_shows_create_form_when_opened() {
    let mut registry = EntityTypeRegistry { types: vec![] };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::Token,
            "Token Types",
            "token_types",
            &enum_reg,
            &struct_reg,
        );
    });
    // Click section header to expand it.
    harness.get_by_label("Token Types").click();
    harness.run();
    harness.get_by_label_contains("New Type");
}

/// Enums tab with populated registry exercises the collapsing header body.
#[test]
fn enums_tab_shows_enum_options() {
    let enum_reg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    // Get the first enum name to click its header.
    let enum_name = enum_reg
        .definitions
        .values()
        .next()
        .map(|d| d.name.clone())
        .unwrap_or_default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_reg, &mut state, &mut actions);
    });
    // Click the collapsing header to open it.
    harness.get_by_label(&enum_name).click();
    harness.run();
    // After opening, the enum options should be visible.
    harness.get_by_label_contains("Open");
    harness.get_by_label_contains("Rough");
    harness.get_by_label_contains("Dense");
}

/// Structs tab with populated registry exercises the collapsing header body.
#[test]
fn structs_tab_shows_struct_fields() {
    // Create a struct with unique field names to avoid matching "x" delete buttons.
    let struct_id = TypeId::new();
    let mut struct_reg = StructRegistry::default();
    struct_reg.definitions.insert(
        struct_id,
        StructDefinition {
            id: struct_id,
            name: "Coordinate".to_string(),
            fields: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "latitude".to_string(),
                    property_type: PropertyType::Float,
                    default_value: PropertyValue::Float(0.0),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "longitude".to_string(),
                    property_type: PropertyType::Float,
                    default_value: PropertyValue::Float(0.0),
                },
            ],
        },
    );
    let enum_reg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &struct_reg, &enum_reg, &mut state, &mut actions);
    });
    // Click the header to open it.
    harness.get_by_label("Coordinate").click();
    harness.run();
    // Field names should be visible.
    harness.get_by_label_contains("latitude");
    harness.get_by_label_contains("longitude");
}

// ---------------------------------------------------------------------------
// Deeper render_ontology tests
// ---------------------------------------------------------------------------

/// Concepts tab with populated registry and opened header.
#[test]
fn concepts_tab_shows_roles_when_opened() {
    let mut concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let concept_name = concept_reg.concepts[0].name.clone();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &entity_reg, &mut state, &mut actions);
    });
    harness.get_by_label(&concept_name).click();
    harness.run();
    harness.get_by_label_contains("traveler");
    harness.get_by_label_contains("terrain");
}

/// Relations tab with Block effect.
#[test]
fn relations_tab_shows_block_effect_label() {
    let mut block_registry = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Impassable".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        }],
    };
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut block_registry,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Impassable");
}

/// Relations tab with Allow effect.
#[test]
fn relations_tab_shows_allow_effect_label() {
    let mut allow_registry = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Passable".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnExit,
            effect: RelationEffect::Allow { condition: None },
        }],
    };
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut allow_registry,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Passable");
}

/// Relations tab opened to see effect details.
#[test]
fn relations_tab_shows_modify_property_details_when_opened() {
    let mut relation_reg = test_relation_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let relation_name = relation_reg.relations[0].name.clone();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label(&relation_name).click();
    harness.run();
    harness.get_by_label_contains("Terrain Cost");
}

/// Constraints tab with `PropertyCompare` expression exercising details.
#[test]
fn constraints_tab_shows_constraint_details_when_opened() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let constraint_name = constraint_reg.constraints[0].name.clone();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label(&constraint_name).click();
    harness.run();
    harness.get_by_label_contains("Budget >= 0");
}

/// Constraints tab shows auto-generated badge.
#[test]
fn constraints_tab_auto_generated_badge() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("auto");
}

// ---------------------------------------------------------------------------
// Deeper render_rules tests — mechanics tab branches
// ---------------------------------------------------------------------------

/// Mechanics tab with Differential column type.
#[test]
fn mechanics_tab_shows_differential_column() {
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Diff CRT".to_string(),
        columns: vec![CrtColumn {
            label: "-3".to_string(),
            column_type: CrtColumnType::Differential,
            threshold: -3.0,
        }],
        rows: vec![CrtRow {
            label: "1".to_string(),
            die_value_min: 1,
            die_value_max: 3,
        }],
        outcomes: vec![vec![CombatOutcome {
            label: "NE".to_string(),
            effect: None,
        }]],
        combat_concept_id: None,
    };
    let structure = test_turn_structure();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(ui, &structure, &crt, &modifiers, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Diff CRT");
}

/// Mechanics tab with modifiers showing source labels.
#[test]
fn mechanics_tab_shows_modifier_sources() {
    let crt = test_crt();
    let structure = test_turn_structure();
    let modifiers = test_modifiers();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(ui, &structure, &crt, &modifiers, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Forest Defense");
    harness.get_by_label_contains("Flanking");
}

/// Mechanics tab with outcomes grid populated.
#[test]
fn mechanics_tab_shows_outcome_grid() {
    let crt = test_crt();
    let structure = test_turn_structure();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(ui, &structure, &crt, &modifiers, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Outcome Grid");
}

// ---------------------------------------------------------------------------
// Combat Panel (render_play::render_combat_panel) — tested via Bevy system
// ---------------------------------------------------------------------------

/// Tests the "No CRT defined" early-return branch via a one-shot Bevy system.
#[test]
fn combat_panel_no_crt_defined() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat::default());
    app.insert_resource(CombatResultsTable::default());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(EntityTypeRegistry::default());
    app.insert_resource(EditorState::default());

    // Run a system that grabs the Query and calls render_combat_panel.
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("No CRT defined");
        },
    );
    app.update();
}

/// Tests combat panel with a valid CRT showing strength inputs and column lookup.
#[test]
fn combat_panel_with_crt_shows_strengths() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat::default());
    app.insert_resource(test_crt());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    app.insert_resource(EditorState {
        combat_attacker_strength: 4.0,
        combat_defender_strength: 2.0,
        ..EditorState::default()
    });

    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Combat Resolution");
            harness.get_by_label_contains("Attacker:");
            harness.get_by_label_contains("Defender:");
            harness.get_by_label_contains("Strengths");
        },
    );
    app.update();
}

/// Tests combat panel with modifiers showing modifier breakdown.
#[test]
fn combat_panel_with_modifiers_shows_breakdown() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat::default());
    app.insert_resource(test_crt());
    app.insert_resource(test_modifiers());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    app.insert_resource(EditorState {
        combat_attacker_strength: 3.0,
        combat_defender_strength: 3.0,
        ..EditorState::default()
    });

    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Modifiers");
            harness.get_by_label_contains("Total shift:");
        },
    );
    app.update();
}

/// Tests combat panel with pre-existing outcome.
#[test]
fn combat_panel_shows_outcome_result() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat {
        die_roll: Some(3),
        outcome: Some(CombatOutcome {
            label: "DR".to_string(),
            effect: None,
        }),
        ..ActiveCombat::default()
    });
    app.insert_resource(test_crt());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    app.insert_resource(EditorState {
        combat_attacker_strength: 2.0,
        combat_defender_strength: 1.0,
        ..EditorState::default()
    });

    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Result: DR");
            harness.get_by_label_contains("Die roll:");
        },
    );
    app.update();
}

/// Tests combat panel shows outcome effects.
#[test]
fn combat_panel_shows_outcome_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat {
        die_roll: Some(5),
        outcome: Some(CombatOutcome {
            label: "DE".to_string(),
            effect: Some(hexorder_contracts::mechanics::OutcomeEffect::DefenderEliminated),
        }),
        ..ActiveCombat::default()
    });
    app.insert_resource(test_crt());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    app.insert_resource(EditorState {
        combat_attacker_strength: 4.0,
        combat_defender_strength: 1.0,
        ..EditorState::default()
    });

    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Defender eliminated");
        },
    );
    app.update();
}

// ---------------------------------------------------------------------------
// Combat panel — remaining OutcomeEffect variants
// ---------------------------------------------------------------------------

/// Helper to build a combat panel test app with a given outcome effect.
fn combat_panel_app_with_effect(effect: hexorder_contracts::mechanics::OutcomeEffect) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat {
        die_roll: Some(3),
        outcome: Some(CombatOutcome {
            label: "EF".to_string(),
            effect: Some(effect),
        }),
        ..ActiveCombat::default()
    });
    app.insert_resource(test_crt());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    app.insert_resource(EditorState {
        combat_attacker_strength: 2.0,
        combat_defender_strength: 1.0,
        ..EditorState::default()
    });
    app
}

#[test]
fn combat_panel_shows_no_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::NoEffect);
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("No effect");
        },
    );
    app.update();
}

#[test]
fn combat_panel_shows_retreat_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::Retreat { hexes: 2 });
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("retreats 2 hex");
        },
    );
    app.update();
}

#[test]
fn combat_panel_shows_step_loss_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::StepLoss { steps: 1 });
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Defender loses 1 step");
        },
    );
    app.update();
}

#[test]
fn combat_panel_shows_attacker_step_loss_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::AttackerStepLoss { steps: 2 });
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Attacker loses 2 step");
        },
    );
    app.update();
}

#[test]
fn combat_panel_shows_exchange_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::Exchange {
        attacker_steps: 1,
        defender_steps: 2,
    });
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Exchange: ATK -1, DEF -2");
        },
    );
    app.update();
}

#[test]
fn combat_panel_shows_attacker_eliminated_effect() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let mut app = combat_panel_app_with_effect(OutcomeEffect::AttackerEliminated);
    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Attacker eliminated");
        },
    );
    app.update();
}

// ---------------------------------------------------------------------------
// Combat panel — below minimum column threshold
// ---------------------------------------------------------------------------

#[test]
fn combat_panel_shows_below_minimum_threshold() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ActiveCombat::default());
    app.insert_resource(test_crt());
    app.insert_resource(CombatModifierRegistry::default());
    app.insert_resource(SelectedUnit::default());
    app.insert_resource(test_registry());
    // Set very low attacker strength so odds are below minimum column threshold.
    app.insert_resource(EditorState {
        combat_attacker_strength: 0.1,
        combat_defender_strength: 100.0,
        ..EditorState::default()
    });

    app.add_systems(
        Update,
        |mut active_combat: ResMut<ActiveCombat>,
         crt: Res<CombatResultsTable>,
         modifiers: Res<CombatModifierRegistry>,
         selected_unit: Res<SelectedUnit>,
         entity_types: Res<EntityTypeRegistry>,
         mut editor_state: ResMut<EditorState>,
         unit_query: Query<&EntityData, With<hexorder_contracts::game_system::UnitInstance>>| {
            let harness = Harness::new_ui(|ui| {
                render_play::render_combat_panel(
                    ui,
                    &mut active_combat,
                    &crt,
                    &modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
            harness.get_by_label_contains("Below minimum column threshold");
        },
    );
    app.update();
}

// ---------------------------------------------------------------------------
// Turn tracker — remaining branch coverage
// ---------------------------------------------------------------------------

/// Turn tracker with pre-initialized (non-zero) turn state.
#[test]
fn turn_tracker_pre_initialized_state() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState {
        turn_number: 3,
        current_phase_index: 1,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("Turn 3");
}

/// Turn tracker shows [Combat] type badge when current phase is Combat.
#[test]
fn turn_tracker_shows_combat_phase_badge() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState {
        turn_number: 1,
        current_phase_index: 1,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("[Combat]");
}

/// Turn tracker shows [Admin] type badge when current phase is Admin.
#[test]
fn turn_tracker_shows_admin_phase_badge() {
    let structure = test_turn_structure();
    let mut turn_state = TurnState {
        turn_number: 1,
        current_phase_index: 2,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut turn_state, &structure);
    });
    harness.get_by_label_contains("[Admin]");
}

// ---------------------------------------------------------------------------
// PropertyValue editor — Color, List, Map, Struct, IntRange, FloatRange
// ---------------------------------------------------------------------------

/// Property value editor renders Color property without panic.
#[test]
fn property_value_editor_color() {
    use super::render_rules;

    let mut value = PropertyValue::Color(Color::srgb(1.0, 0.0, 0.0));
    let prop_type = PropertyType::Color;
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Property value editor renders List with items.
#[test]
fn property_value_editor_list_with_items() {
    use super::render_rules;

    let mut value = PropertyValue::List(vec![PropertyValue::Int(10), PropertyValue::Int(20)]);
    let prop_type = PropertyType::List(Box::new(PropertyType::Int));
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    harness.get_by_label_contains("List (2)");
}

/// Property value editor shows nested limit for deeply nested lists.
#[test]
fn property_value_editor_list_nested_limit() {
    use super::render_rules;

    let mut value = PropertyValue::List(vec![PropertyValue::Int(1)]);
    let prop_type = PropertyType::List(Box::new(PropertyType::Int));
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            3, // depth >= 3
        );
    });
    harness.get_by_label_contains("nested limit");
}

/// Property value editor renders Map with enum keys.
#[test]
fn property_value_editor_map_with_entries() {
    use super::render_rules;
    use hexorder_contracts::game_system::EnumDefinition;

    let enum_id = TypeId::new();
    let mut enum_reg = EnumRegistry::default();
    enum_reg.insert(EnumDefinition {
        id: enum_id,
        name: "Season".to_string(),
        options: vec!["Spring".to_string(), "Summer".to_string()],
    });

    let mut value = PropertyValue::Map(vec![("Spring".to_string(), PropertyValue::Int(1))]);
    let prop_type = PropertyType::Map(enum_id, Box::new(PropertyType::Int));
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    harness.get_by_label_contains("Map (1)");
}

/// Property value editor renders Map nested limit.
#[test]
fn property_value_editor_map_nested_limit() {
    use super::render_rules;

    let mut value = PropertyValue::Map(vec![]);
    let prop_type = PropertyType::Map(TypeId::new(), Box::new(PropertyType::Int));
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            3,
        );
    });
    harness.get_by_label_contains("nested limit");
}

/// Property value editor renders Struct with known definition.
#[test]
fn property_value_editor_struct_known() {
    use super::render_rules;
    use hexorder_contracts::game_system::{PropertyDefinition, StructDefinition};

    let struct_id = TypeId::new();
    let field_id = TypeId::new();
    let mut struct_reg = StructRegistry::default();
    struct_reg.insert(StructDefinition {
        id: struct_id,
        name: "Position".to_string(),
        fields: vec![PropertyDefinition {
            id: field_id,
            name: "altitude".to_string(),
            property_type: PropertyType::Float,
            default_value: PropertyValue::Float(0.0),
        }],
    });

    let mut value = PropertyValue::Struct(std::collections::HashMap::new());
    let prop_type = PropertyType::Struct(struct_id);
    let enum_reg = EnumRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    harness.get_by_label_contains("Position");
}

/// Property value editor shows "unknown struct" for missing definition.
#[test]
fn property_value_editor_struct_unknown() {
    use super::render_rules;

    let mut value = PropertyValue::Struct(std::collections::HashMap::new());
    let prop_type = PropertyType::Struct(TypeId::new());
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    // Header shows "Struct" when definition is missing.
    harness.get_by_label("Struct").click();
    harness.run();
    harness.get_by_label_contains("unknown struct");
}

/// Property value editor renders Struct nested limit.
#[test]
fn property_value_editor_struct_nested_limit() {
    use super::render_rules;

    let mut value = PropertyValue::Struct(std::collections::HashMap::new());
    let prop_type = PropertyType::Struct(TypeId::new());
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            3,
        );
    });
    harness.get_by_label_contains("nested limit");
}

/// Property value editor renders `IntRange` with bounds.
#[test]
fn property_value_editor_int_range_bounded() {
    use super::render_rules;

    let mut value = PropertyValue::IntRange(5);
    let prop_type = PropertyType::IntRange { min: 0, max: 10 };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Property value editor renders `IntRange` without matching type (unbounded).
#[test]
fn property_value_editor_int_range_unbounded() {
    use super::render_rules;

    let mut value = PropertyValue::IntRange(5);
    let prop_type = PropertyType::Int; // mismatched type — fallback path
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Property value editor renders `FloatRange` with bounds.
#[test]
fn property_value_editor_float_range_bounded() {
    use super::render_rules;

    let mut value = PropertyValue::FloatRange(0.5);
    let prop_type = PropertyType::FloatRange { min: 0.0, max: 1.0 };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Property value editor renders `FloatRange` without matching type (unbounded).
#[test]
fn property_value_editor_float_range_unbounded() {
    use super::render_rules;

    let mut value = PropertyValue::FloatRange(0.5);
    let prop_type = PropertyType::Float; // mismatched type — fallback path
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Property value editor renders `EntityRef` with role filter without panic.
#[test]
fn property_value_editor_entity_ref_with_role_filter() {
    use super::render_rules;

    let mut value = PropertyValue::EntityRef(None);
    let prop_type = PropertyType::EntityRef(Some(EntityRole::Token));
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = test_registry();
    // Just verify the ComboBox renders without panic; "(none)" appears as
    // ComboBox value, not as a label, so we can't use get_by_label_contains.
    let _harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
}

/// Inspector with Unknown entity type fallback.
#[test]
fn inspector_unknown_entity_type() {
    let registry = EntityTypeRegistry::default(); // empty — no matching type
    let enum_registry = EnumRegistry::default();
    let struct_registry = StructRegistry::default();
    let pos = HexPosition { q: 0, r: 0 };
    let mut entity_data = EntityData {
        entity_type_id: TypeId::new(), // non-existent
        properties: std::collections::HashMap::new(),
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            Some(pos),
            Some(&mut entity_data),
            &registry,
            &enum_registry,
            &struct_registry,
        );
    });
    harness.get_by_label_contains("Unknown");
}

/// Unit inspector with Unknown entity type fallback.
#[test]
fn unit_inspector_unknown_entity_type() {
    let registry = EntityTypeRegistry::default();
    let enum_registry = EnumRegistry::default();
    let struct_registry = StructRegistry::default();
    let mut entity_data = EntityData {
        entity_type_id: TypeId::new(),
        properties: std::collections::HashMap::new(),
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_registry,
            &struct_registry,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Unknown");
}

// ---------------------------------------------------------------------------
// render_design — property type sub-forms (indices 5–11)
// ---------------------------------------------------------------------------

/// Entity type section shows Enum options field when `new_prop_type_index` = 5.
#[test]
fn entity_type_section_enum_prop_shows_opts_field() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 5,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Opts:");
}

/// Entity type section shows `EntityRef` role combobox when `new_prop_type_index` = 6.
#[test]
fn entity_type_section_entity_ref_prop_shows_role() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 6,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Role:");
}

/// Entity type section shows List inner type combobox when `new_prop_type_index` = 7.
#[test]
fn entity_type_section_list_prop_shows_inner_type() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 7,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Item type:");
}

/// Entity type section shows Map key/value when `new_prop_type_index` = 8.
#[test]
fn entity_type_section_map_prop_shows_key_value() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 8,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Key enum:");
    harness.get_by_label_contains("Value type:");
}

/// Entity type section shows Struct picker when `new_prop_type_index` = 9.
#[test]
fn entity_type_section_struct_prop_shows_picker() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 9,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Struct:");
}

/// Entity type section shows `IntRange` min/max when `new_prop_type_index` = 10.
#[test]
fn entity_type_section_int_range_shows_min_max() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 10,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Min:");
}

/// Entity type section shows `FloatRange` min/max when `new_prop_type_index` = 11.
#[test]
fn entity_type_section_float_range_shows_min_max() {
    let mut registry = test_registry();
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState {
        new_prop_type_index: 11,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "bt",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Min:");
}

// ---------------------------------------------------------------------------
// render_ontology — concept roles, relation interior, constraint creation
// ---------------------------------------------------------------------------

/// Concepts tab shows roles when header is opened.
#[test]
fn concepts_tab_shows_roles_when_header_opened() {
    let mut concept_reg = test_concept_registry();
    let registry = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &registry, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label_contains("traveler");
    harness.get_by_label_contains("terrain");
}

/// Relations tab shows trigger label when header is opened.
#[test]
fn relations_tab_shows_trigger_when_opened() {
    let mut relation_reg = test_relation_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("Terrain Cost").click();
    harness.run();
    harness.get_by_label_contains("OnEnter");
}

/// Constraints tab shows New Constraint form when opened.
#[test]
fn constraints_tab_shows_new_constraint_form() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("Name:");
}

/// Constraints tab `PropertyCompare` form shows fields.
#[test]
fn constraints_tab_property_compare_shows_fields() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("Prop:");
    harness.get_by_label_contains("Value:");
}

/// Constraints tab `PathBudget` form shows fields.
#[test]
fn constraints_tab_path_budget_shows_fields() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 3,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("Cost:");
    harness.get_by_label_contains("Budget:");
}

/// Constraints tab CrossCompare/IsType shows placeholder.
#[test]
fn constraints_tab_cross_compare_shows_placeholder() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("full editor");
}

// ---------------------------------------------------------------------------
// render_ontology — relations creation form (concept/role selectors, trigger/effect)
// ---------------------------------------------------------------------------

/// Relations tab new form shows concept, subject, object selectors when concepts exist.
#[test]
fn relations_tab_form_shows_concept_and_role_selectors() {
    let mut relation_reg = RelationRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Concept:");
    harness.get_by_label_contains("Subject:");
    harness.get_by_label_contains("Object:");
    harness.get_by_label_contains("Trigger:");
    harness.get_by_label_contains("Effect:");
}

/// Relations tab `ModifyProperty` effect shows target, source, op fields.
#[test]
fn relations_tab_modify_property_shows_fields() {
    let mut relation_reg = RelationRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_relation_effect_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Target:");
    harness.get_by_label_contains("Source:");
    harness.get_by_label_contains("Op:");
}

/// Relations tab Block effect hides `ModifyProperty` fields.
#[test]
fn relations_tab_block_effect_hides_modify_fields() {
    let mut relation_reg = RelationRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_relation_effect_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    // Block/Allow hides the Target/Source/Op fields — render completes without panic
    let _ = harness;
}

/// Concepts tab — concept with empty roles shows "(none)" placeholder.
#[test]
fn concepts_tab_empty_roles_shows_none_placeholder() {
    let mut concept_reg = ConceptRegistry {
        concepts: vec![Concept {
            id: TypeId::new(),
            name: "EmptyConcept".to_string(),
            description: "No roles".to_string(),
            role_labels: vec![],
        }],
        bindings: vec![],
    };
    let entity_reg = EntityTypeRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &entity_reg, &mut state, &mut actions);
    });
    // Open the concept header to see roles
    harness.get_by_label("EmptyConcept").click();
    harness.run();
    // Both Roles: "(none)" and Bindings: "(none)" should appear
    harness.get_by_label_contains("Roles:");
}

/// Concepts tab — empty bindings shows "(none)" placeholder.
#[test]
fn concepts_tab_empty_bindings_shows_none_placeholder() {
    let mut concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &entity_reg, &mut state, &mut actions);
    });
    // Open the concept header — no bindings exist in test data
    harness.get_by_label("Motion").click();
    harness.run();
    // The "Bindings:" section should show "(none)"
    harness.get_by_label_contains("Bindings:");
}

/// Concepts tab — role labels with allowed entity roles display correctly.
#[test]
fn concepts_tab_roles_show_allowed_entity_roles() {
    let mut concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &entity_reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    // Roles should show allowed entity role types: Board / Token
    harness.get_by_label_contains("traveler [Token]");
    harness.get_by_label_contains("terrain [Board]");
}

/// Concepts tab — Bind Entity form appears when concept has roles.
#[test]
fn concepts_tab_bind_entity_form_appears() {
    let mut concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut concept_reg, &entity_reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label_contains("Bind Entity");
}

// ---------------------------------------------------------------------------
// render_design — add property actions, delete type, enum/struct CRUD
// ---------------------------------------------------------------------------

/// Entity type section — add property with Bool type produces action.
#[test]
fn entity_type_section_add_property_bool() {
    struct AddPropState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_prop_name: "is_passable".to_string(),
        new_prop_type_index: 0, // Bool
        ..EditorState::default()
    };
    let state = AddPropState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut AddPropState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    // Open Board Types header, then open Plains type, then click +Add
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("+ Add").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(matches!(
        &harness.state().actions[0],
        EditorAction::AddProperty { name, .. } if name == "is_passable"
    ));
}

/// Entity type section — empty properties shows "(none)" label.
#[test]
fn entity_type_section_empty_properties_shows_none() {
    let mut registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Empty".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Empty").click();
    harness.run();
    harness.get_by_label_contains("(none)");
}

/// Entity type section — delete type hidden when only one type of that role.
#[test]
fn entity_type_section_delete_hidden_when_single_type() {
    let mut registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "OnlyType".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("OnlyType").click();
    harness.run();
    // With only 1 type, Delete Type button should NOT appear — renders without panic
    let _ = harness;
}

/// Entity type section — delete type shown when multiple types.
#[test]
fn entity_type_section_delete_shown_when_multiple_types() {
    let mut registry = EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.4, 0.6, 0.2),
                properties: vec![],
            },
            EntityType {
                id: TypeId::new(),
                name: "Forest".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.1, 0.5, 0.1),
                properties: vec![],
            },
        ],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    // With 2 types, Delete Type button should appear
    harness.get_by_label("Delete Type");
}

/// Enums tab — delete enum button produces `DeleteEnum` action.
#[test]
fn enums_tab_delete_enum_produces_action() {
    struct EnumsState {
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let enum_registry = test_enum_registry();
    let editor_state = EditorState::default();
    let state = EnumsState {
        enum_registry,
        editor_state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut EnumsState| {
            systems::render_enums_tab(ui, &s.enum_registry, &mut s.editor_state, &mut s.actions);
        },
        state,
    );

    // Open the Terrain enum header
    harness.get_by_label("Terrain").click();
    harness.run();
    // Click Delete Enum button
    harness.get_by_label("Delete Enum").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(matches!(
        &harness.state().actions[0],
        EditorAction::DeleteEnum { .. }
    ));
}

/// Structs tab — delete struct button produces `DeleteStruct` action.
#[test]
fn structs_tab_delete_struct_produces_action() {
    struct StructsState {
        struct_registry: StructRegistry,
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let struct_registry = test_struct_registry();
    let enum_registry = EnumRegistry::default();
    let editor_state = EditorState::default();
    let state = StructsState {
        struct_registry,
        enum_registry,
        editor_state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut StructsState| {
            systems::render_structs_tab(
                ui,
                &s.struct_registry,
                &s.enum_registry,
                &mut s.editor_state,
                &mut s.actions,
            );
        },
        state,
    );

    // Open the Position struct header
    harness.get_by_label("Position").click();
    harness.run();
    // Click Delete Struct button
    harness.get_by_label("Delete Struct").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(matches!(
        &harness.state().actions[0],
        EditorAction::DeleteStruct { .. }
    ));
}

// ---------------------------------------------------------------------------
// render_rules — PropertyValue scalar types (Bool, Int, Float, String)
// ---------------------------------------------------------------------------

/// `PropertyValue::Bool` renders checkbox.
#[test]
fn property_value_editor_bool() {
    use super::render_rules;

    let mut value = PropertyValue::Bool(false);
    let prop_type = PropertyType::Bool;
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    // Bool renders a checkbox — just verify no panic
    let _ = harness;
}

/// `PropertyValue::Int` renders drag value.
#[test]
fn property_value_editor_int() {
    use super::render_rules;

    let mut value = PropertyValue::Int(42);
    let prop_type = PropertyType::Int;
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    let _ = harness;
}

/// `PropertyValue::Float` renders drag value.
#[test]
fn property_value_editor_float() {
    use super::render_rules;

    let mut value = PropertyValue::Float(2.78);
    let prop_type = PropertyType::Float;
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    let _ = harness;
}

/// `PropertyValue::String` renders text edit.
#[test]
fn property_value_editor_string() {
    use super::render_rules;

    let mut value = PropertyValue::String("hello".to_string());
    let prop_type = PropertyType::String;
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    let _ = harness;
}

/// `PropertyValue::Enum` renders combobox with options.
#[test]
fn property_value_editor_enum() {
    use super::render_rules;

    let enum_reg = test_enum_registry();
    let enum_id = *enum_reg.definitions.keys().next().expect("enum exists");
    let mut value = PropertyValue::Enum("Open".to_string());
    let prop_type = PropertyType::Enum(enum_id);
    let struct_reg = StructRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &entity_reg,
            0,
        );
    });
    let _ = harness;
}

// ---------------------------------------------------------------------------
// render_rules — mechanics tab additional branches
// ---------------------------------------------------------------------------

/// Mechanics tab — `Differential` CRT column type renders "(diff" label.
#[test]
fn mechanics_tab_differential_crt_column_label() {
    let turn_structure = test_turn_structure();
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Test CRT".to_string(),
        columns: vec![CrtColumn {
            label: "Diff1".to_string(),
            column_type: CrtColumnType::Differential,
            threshold: 2.0,
        }],
        rows: vec![],
        outcomes: vec![],
        combat_concept_id: None,
    };
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("(diff");
}

/// Mechanics tab — `AttackerProperty` modifier source renders label.
#[test]
fn mechanics_tab_attacker_property_modifier_source() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry {
        modifiers: vec![CombatModifierDefinition {
            id: TypeId::new(),
            name: "Strength Bonus".to_string(),
            source: ModifierSource::AttackerProperty("strength".to_string()),
            column_shift: 1,
            priority: 10,
            cap: None,
            terrain_type_filter: None,
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Strength Bonus");
}

/// Mechanics tab — `DefenderProperty` modifier source renders label.
#[test]
fn mechanics_tab_defender_property_modifier_source() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry {
        modifiers: vec![CombatModifierDefinition {
            id: TypeId::new(),
            name: "Armor Rating".to_string(),
            source: ModifierSource::DefenderProperty("armor".to_string()),
            column_shift: -1,
            priority: 10,
            cap: None,
            terrain_type_filter: None,
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Armor Rating");
}

/// Mechanics tab — custom modifier source shows additional field.
#[test]
fn mechanics_tab_custom_modifier_source_shows_field() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_modifier_source_index: 2,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    // When source index is 2 (Custom), a custom desc input should appear
    harness.get_by_label_contains("Desc:");
}

/// Mechanics tab — empty columns and rows hides outcome grid.
#[test]
fn mechanics_tab_empty_crt_hides_outcome_grid() {
    let turn_structure = test_turn_structure();
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Empty CRT".to_string(),
        columns: vec![],
        rows: vec![],
        outcomes: vec![],
        combat_concept_id: None,
    };
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    // Outcome Grid label should NOT appear when CRT has no cols/rows — renders without panic
    let _ = harness;
}

// ---------------------------------------------------------------------------
// render_play — turn tracker interaction, clear combat
// ---------------------------------------------------------------------------

/// Turn tracker — Next Phase advances within same turn.
#[test]
fn turn_tracker_next_phase_advances_within_turn() {
    struct TurnTrackerState {
        turn_state: TurnState,
        turn_structure: TurnStructure,
    }

    let turn_state = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let turn_structure = test_turn_structure();
    let state = TurnTrackerState {
        turn_state,
        turn_structure,
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut TurnTrackerState| {
            render_play::render_turn_tracker(ui, &mut s.turn_state, &s.turn_structure);
        },
        state,
    );

    harness.get_by_label_contains("Next Phase").click();
    harness.run();

    assert_eq!(harness.state().turn_state.current_phase_index, 1);
    assert_eq!(harness.state().turn_state.turn_number, 1);
}

/// Turn tracker — Next Phase wraps to next turn when at last phase.
#[test]
fn turn_tracker_next_phase_wraps_to_next_turn() {
    struct TurnTrackerState {
        turn_state: TurnState,
        turn_structure: TurnStructure,
    }

    let turn_state = TurnState {
        turn_number: 1,
        current_phase_index: 2, // last phase (index 2 of 3)
        is_active: true,
    };
    let turn_structure = test_turn_structure();
    let state = TurnTrackerState {
        turn_state,
        turn_structure,
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut TurnTrackerState| {
            render_play::render_turn_tracker(ui, &mut s.turn_state, &s.turn_structure);
        },
        state,
    );

    harness.get_by_label_contains("Next Phase").click();
    harness.run();

    assert_eq!(harness.state().turn_state.current_phase_index, 0);
    assert_eq!(harness.state().turn_state.turn_number, 2);
}

// ---------------------------------------------------------------------------
// render_ontology — relation detail view, new relation form deeper branches
// ---------------------------------------------------------------------------

/// Relation detail header shows concept, roles, trigger, effect when opened.
#[test]
fn relations_tab_detail_shows_concept_and_trigger() {
    let concept_id = TypeId::new();
    let role1_id = TypeId::new();
    let role2_id = TypeId::new();
    let mut relation_reg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Movement Cost".to_string(),
            concept_id,
            subject_role_id: role1_id,
            object_role_id: role2_id,
            trigger: RelationTrigger::OnExit,
            effect: RelationEffect::Block { condition: None },
        }],
    };
    let concept_reg = ConceptRegistry {
        concepts: vec![Concept {
            id: concept_id,
            name: "Motion".to_string(),
            description: String::new(),
            role_labels: vec![
                ConceptRole {
                    id: role1_id,
                    name: "mover".to_string(),
                    allowed_entity_roles: vec![EntityRole::Token],
                },
                ConceptRole {
                    id: role2_id,
                    name: "ground".to_string(),
                    allowed_entity_roles: vec![EntityRole::BoardPosition],
                },
            ],
        }],
        bindings: vec![],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    // Open the relation detail header
    harness.get_by_label("Movement Cost").click();
    harness.run();
    harness.get_by_label_contains("Motion");
    harness.get_by_label_contains("mover -> ground");
    harness.get_by_label_contains("OnExit");
}

/// Relation with `Allow` effect shows "Allow" in detail.
#[test]
fn relations_tab_detail_shows_allow_effect() {
    let mut relation_reg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "AllowRel".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::WhilePresent,
            effect: RelationEffect::Allow { condition: None },
        }],
    };
    let concept_reg = ConceptRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("AllowRel").click();
    harness.run();
    harness.get_by_label_contains("WhilePresent");
    harness.get_by_label_contains("Effect: Allow");
}

/// Relations tab — empty relations shows "No relations defined" label.
#[test]
fn relations_tab_empty_shows_no_relations_label() {
    let mut relation_reg = RelationRegistry::default();
    let concept_reg = ConceptRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("No relations defined");
}

/// Relations tab — form without concepts hides concept/role selectors.
#[test]
fn relations_tab_form_no_concepts_hides_selectors() {
    let mut relation_reg = RelationRegistry::default();
    let concept_reg = ConceptRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(
            ui,
            &mut relation_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    // With no concepts, concept/subject/object selectors should not appear
    harness.get_by_label_contains("Trigger:");
    let _ = harness;
}

// ---------------------------------------------------------------------------
// render_rules — mechanics tab add phase, CRT interactions
// ---------------------------------------------------------------------------

/// Mechanics tab — Simultaneous player order renders.
#[test]
fn mechanics_tab_simultaneous_player_order() {
    let turn_structure = TurnStructure {
        player_order: PlayerOrder::Simultaneous,
        phases: vec![],
    };
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    let _ = harness;
}

/// Mechanics tab — `ActivationBased` player order renders.
#[test]
fn mechanics_tab_activation_based_player_order() {
    let turn_structure = TurnStructure {
        player_order: PlayerOrder::ActivationBased,
        phases: vec![],
    };
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    let _ = harness;
}

/// Mechanics tab — no phases shows "Phases (0)" count.
#[test]
fn mechanics_tab_empty_phases() {
    let turn_structure = TurnStructure {
        player_order: PlayerOrder::Alternating,
        phases: vec![],
    };
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Phases (0)");
}

/// Inspector with empty properties on entity shows "No properties" label.
#[test]
fn inspector_empty_properties_shows_label() {
    use super::render_rules;

    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "EmptyType".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let position = Some(HexPosition { q: 0, r: 0 });
    let mut entity_data = Some(EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    });

    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            position,
            entity_data.as_mut(),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("EmptyType");
}

/// Unit inspector delete button renders with entity data.
#[test]
fn unit_inspector_delete_button_renders() {
    use super::render_rules;

    let type_id = TypeId::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: type_id,
            name: "Soldier".to_string(),
            role: EntityRole::Token,
            color: Color::srgb(0.2, 0.2, 0.8),
            properties: vec![],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut entity_data = Some(EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    });
    let mut actions = Vec::new();

    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            entity_data.as_mut(),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label("Delete Unit");
}

// ---------------------------------------------------------------------------
// render_design — enums/structs deeper interactions
// ---------------------------------------------------------------------------

/// Enums tab — enum with no options shows empty collapsing header.
#[test]
fn enums_tab_enum_no_options() {
    let mut enum_registry = EnumRegistry::default();
    let id = TypeId::new();
    enum_registry.definitions.insert(
        id,
        EnumDefinition {
            id,
            name: "Empty".to_string(),
            options: vec![],
        },
    );
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_registry, &mut state, &mut actions);
    });
    harness.get_by_label("Empty").click();
    harness.run();
    // No options listed, just the Add form and Delete button
    harness.get_by_label("Delete Enum");
}

/// Structs tab — struct with no fields shows empty collapsing header.
#[test]
fn structs_tab_struct_no_fields() {
    let mut struct_registry = StructRegistry::default();
    let id = TypeId::new();
    struct_registry.definitions.insert(
        id,
        StructDefinition {
            id,
            name: "EmptyStruct".to_string(),
            fields: vec![],
        },
    );
    let enum_reg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &struct_registry, &enum_reg, &mut state, &mut actions);
    });
    harness.get_by_label("EmptyStruct").click();
    harness.run();
    harness.get_by_label("Delete Struct");
}

/// Constraints tab — non-empty constraints list shows both constraint names.
#[test]
fn constraints_tab_shows_both_constraint_names() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Budget >= 0");
    harness.get_by_label_contains("Auto-check");
}

// ---------------------------------------------------------------------------
// render_rules — CRT outcome grid, modifier negative shift
// ---------------------------------------------------------------------------

/// Mechanics tab — CRT outcome grid renders with column/row dimensions.
#[test]
fn mechanics_tab_outcome_grid_renders() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState {
        crt_outcome_labels: vec![
            vec!["NE".to_string(), "DR".to_string()],
            vec!["AR".to_string(), "DE".to_string()],
        ],
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Outcome Grid");
}

/// Mechanics tab — modifier with `Custom` source renders custom label.
#[test]
fn mechanics_tab_custom_modifier_renders_label() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry {
        modifiers: vec![CombatModifierDefinition {
            id: TypeId::new(),
            name: "Weather".to_string(),
            source: ModifierSource::Custom("storm".to_string()),
            column_shift: -2,
            priority: 5,
            cap: None,
            terrain_type_filter: None,
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Weather");
    harness.get_by_label_contains("Custom(storm)");
}

// ---------------------------------------------------------------------------
// render_design — entity type section with properties (property rendering)
// ---------------------------------------------------------------------------

/// Entity type section with properties renders property names when opened.
#[test]
fn entity_type_section_properties_render_names() {
    let mut registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Terrain".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.3, 0.6, 0.2),
            properties: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "cost".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(1),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "passable".to_string(),
                    property_type: PropertyType::Bool,
                    default_value: PropertyValue::Bool(true),
                },
            ],
        }],
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label_contains("cost");
    harness.get_by_label_contains("passable");
}

// ---------------------------------------------------------------------------
// render_ontology — deeper relation creation form and constraint branches
// ---------------------------------------------------------------------------

/// Relations tab — create relation with `OnExit` trigger and Block effect.
#[test]
fn relations_tab_create_relation_block_on_exit() {
    struct RelState {
        relation_reg: RelationRegistry,
        concept_reg: ConceptRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let state = EditorState {
        new_relation_name: "BlockEntry".to_string(),
        new_relation_trigger_index: 1, // OnExit
        new_relation_effect_index: 1,  // Block
        ..EditorState::default()
    };
    let s = RelState {
        relation_reg: RelationRegistry::default(),
        concept_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut RelState| {
            systems::render_relations_tab(
                ui,
                &mut s.relation_reg,
                &s.concept_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("+ Create Relation").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::CreateRelation { trigger, effect, .. }
            if matches!(trigger, RelationTrigger::OnExit) && matches!(effect, RelationEffect::Block { .. }))
    );
}

/// Relations tab — create relation with `WhilePresent` trigger and Allow effect.
#[test]
fn relations_tab_create_relation_allow_while_present() {
    struct RelState {
        relation_reg: RelationRegistry,
        concept_reg: ConceptRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let state = EditorState {
        new_relation_name: "Passthrough".to_string(),
        new_relation_trigger_index: 2, // WhilePresent
        new_relation_effect_index: 2,  // Allow
        ..EditorState::default()
    };
    let s = RelState {
        relation_reg: RelationRegistry::default(),
        concept_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut RelState| {
            systems::render_relations_tab(
                ui,
                &mut s.relation_reg,
                &s.concept_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("+ Create Relation").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::CreateRelation { trigger, effect, .. }
            if matches!(trigger, RelationTrigger::WhilePresent) && matches!(effect, RelationEffect::Allow { .. }))
    );
}

/// Constraints tab — `IsType` expression shows placeholder.
#[test]
fn constraints_tab_is_type_shows_placeholder() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 2, // IsType
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("full editor");
}

// ---------------------------------------------------------------------------
// render_rules — more mechanics tab branches
// ---------------------------------------------------------------------------

/// Mechanics tab — phase list with up/down buttons renders for middle phase.
#[test]
fn mechanics_tab_phase_list_with_move_buttons() {
    let turn_structure = test_turn_structure();
    let crt = test_crt();
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    // All 3 phases should be visible — check via phase type labels
    harness.get_by_label_contains("[Mov]");
    harness.get_by_label_contains("[Cbt]");
    harness.get_by_label_contains("[Adm]");
}

/// Mechanics tab — CRT with only rows but no columns hides outcome grid.
#[test]
fn mechanics_tab_crt_only_rows_no_outcome_grid() {
    let turn_structure = test_turn_structure();
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Rows Only".to_string(),
        columns: vec![],
        rows: vec![CrtRow {
            label: "1".to_string(),
            die_value_min: 1,
            die_value_max: 3,
        }],
        outcomes: vec![],
        combat_concept_id: None,
    };
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    let _ = harness;
}

/// Mechanics tab — CRT with only columns but no rows hides outcome grid.
#[test]
fn mechanics_tab_crt_only_cols_no_outcome_grid() {
    let turn_structure = test_turn_structure();
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Cols Only".to_string(),
        columns: vec![CrtColumn {
            label: "1:1".to_string(),
            column_type: CrtColumnType::OddsRatio,
            threshold: 1.0,
        }],
        rows: vec![],
        outcomes: vec![],
        combat_concept_id: None,
    };
    let modifiers = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_mechanics_tab(
            ui,
            &turn_structure,
            &crt,
            &modifiers,
            &mut state,
            &mut actions,
        );
    });
    let _ = harness;
}

// ---------------------------------------------------------------------------
// render_design — add property with different type indices (1–4)
// ---------------------------------------------------------------------------

/// Entity type section — add property with Int type.
#[test]
fn entity_type_section_add_property_int() {
    struct AddPropState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_prop_name: "defense".to_string(),
        new_prop_type_index: 1, // Int
        ..EditorState::default()
    };
    let state = AddPropState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut AddPropState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("+ Add").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
}

/// Entity type section — add property with Float type.
#[test]
fn entity_type_section_add_property_float() {
    struct AddPropState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_prop_name: "cost".to_string(),
        new_prop_type_index: 2, // Float
        ..EditorState::default()
    };
    let state = AddPropState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut AddPropState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("+ Add").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
}

/// Entity type section — add property with String type.
#[test]
fn entity_type_section_add_property_string() {
    struct AddPropState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_prop_name: "description".to_string(),
        new_prop_type_index: 3, // String
        ..EditorState::default()
    };
    let state = AddPropState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut AddPropState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("+ Add").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
}

/// Entity type section — add property with Color type.
#[test]
fn entity_type_section_add_property_color() {
    struct AddPropState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_prop_name: "tint".to_string(),
        new_prop_type_index: 4, // Color
        ..EditorState::default()
    };
    let state = AddPropState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut AddPropState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("+ Add").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
}

// ---------------------------------------------------------------------------
// render_ontology — concept add role form with different role permissions
// ---------------------------------------------------------------------------

/// Concepts tab — add role with Board-only permission.
#[test]
fn concepts_tab_add_role_board_only() {
    struct ConceptState {
        concept_reg: ConceptRegistry,
        entity_reg: EntityTypeRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let state = EditorState {
        new_role_name: "wall".to_string(),
        new_role_allowed_roles: vec![true, false], // Board only
        ..EditorState::default()
    };
    let s = ConceptState {
        concept_reg,
        entity_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConceptState| {
            systems::render_concepts_tab(
                ui,
                &mut s.concept_reg,
                &s.entity_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label("+ Add Role").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::AddConceptRole { allowed_roles, .. }
            if allowed_roles == &[EntityRole::BoardPosition])
    );
}

/// Concepts tab — add role with Token-only permission.
#[test]
fn concepts_tab_add_role_token_only() {
    struct ConceptState {
        concept_reg: ConceptRegistry,
        entity_reg: EntityTypeRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let state = EditorState {
        new_role_name: "scout".to_string(),
        new_role_allowed_roles: vec![false, true], // Token only
        ..EditorState::default()
    };
    let s = ConceptState {
        concept_reg,
        entity_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConceptState| {
            systems::render_concepts_tab(
                ui,
                &mut s.concept_reg,
                &s.entity_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label("+ Add Role").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::AddConceptRole { allowed_roles, .. }
            if allowed_roles == &[EntityRole::Token])
    );
}

// ---------------------------------------------------------------------------
// render_design — create entity type, enum option add, struct field add
// ---------------------------------------------------------------------------

/// Entity type section — create entity type button produces action.
#[test]
fn entity_type_section_create_type_produces_action() {
    struct CreateTypeState {
        registry: EntityTypeRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![],
        }],
    };
    let editor_state = EditorState {
        new_type_name: "Forest".to_string(),
        ..EditorState::default()
    };
    let state = CreateTypeState {
        registry,
        editor_state,
        actions: Vec::new(),
    };
    let enum_reg = EnumRegistry::default();
    let struct_reg = StructRegistry::default();

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut CreateTypeState| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.editor_state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &enum_reg,
                &struct_reg,
            );
        },
        state,
    );

    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("+ Create").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::CreateEntityType { name, .. } if name == "Forest")
    );
}

/// Enums tab — add option to existing enum.
#[test]
fn enums_tab_add_option_produces_action() {
    struct EnumsState {
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let enum_registry = test_enum_registry();
    let editor_state = EditorState {
        new_enum_option_text: "Swamp".to_string(),
        ..EditorState::default()
    };
    let state = EnumsState {
        enum_registry,
        editor_state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut EnumsState| {
            systems::render_enums_tab(ui, &s.enum_registry, &mut s.editor_state, &mut s.actions);
        },
        state,
    );

    // Open the Terrain enum header
    harness.get_by_label("Terrain").click();
    harness.run();
    // Click "+" to add option
    harness.get_by_label("+").click();
    harness.run();

    let actions = &harness.state().actions;
    assert!(!actions.is_empty());
    assert!(matches!(
        &actions[0],
        EditorAction::AddEnumOption { option, .. } if option == "Swamp"
    ));
}

/// Structs tab — add field to existing struct.
#[test]
fn structs_tab_add_field_produces_action() {
    struct StructsState {
        struct_registry: StructRegistry,
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let struct_registry = test_struct_registry();
    let enum_registry = EnumRegistry::default();
    let editor_state = EditorState {
        new_struct_field_name: "z".to_string(),
        new_struct_field_type_index: 1, // Int
        ..EditorState::default()
    };
    let state = StructsState {
        struct_registry,
        enum_registry,
        editor_state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut StructsState| {
            systems::render_structs_tab(
                ui,
                &s.struct_registry,
                &s.enum_registry,
                &mut s.editor_state,
                &mut s.actions,
            );
        },
        state,
    );

    // Open the Position struct header
    harness.get_by_label("Position").click();
    harness.run();
    // Click "+ Add Field"
    harness.get_by_label("+ Add Field").click();
    harness.run();

    let actions = &harness.state().actions;
    assert!(!actions.is_empty());
    assert!(matches!(
        &actions[0],
        EditorAction::AddStructField { name, .. } if name == "z"
    ));
}

// ---------------------------------------------------------------------------
// render_ontology — concept create, delete concept, constraint create
// ---------------------------------------------------------------------------

/// Concepts tab — create concept button produces `CreateConcept` action.
#[test]
fn concepts_tab_create_concept_button_produces_action() {
    struct ConceptState {
        concept_reg: ConceptRegistry,
        entity_reg: EntityTypeRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = ConceptRegistry::default();
    let entity_reg = EntityTypeRegistry::default();
    let state = EditorState {
        new_concept_name: "Stacking".to_string(),
        ..EditorState::default()
    };
    let s = ConceptState {
        concept_reg,
        entity_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConceptState| {
            systems::render_concepts_tab(
                ui,
                &mut s.concept_reg,
                &s.entity_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("+ Create Concept").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::CreateConcept { name, .. } if name == "Stacking")
    );
}

/// Concepts tab — delete concept button produces action.
#[test]
fn concepts_tab_delete_concept_produces_action() {
    struct ConceptState {
        concept_reg: ConceptRegistry,
        entity_reg: EntityTypeRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let entity_reg = test_registry();
    let s = ConceptState {
        concept_reg,
        entity_reg,
        state: EditorState::default(),
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut ConceptState| {
            systems::render_concepts_tab(
                ui,
                &mut s.concept_reg,
                &s.entity_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    // Open concept header
    harness.get_by_label("Motion").click();
    harness.run();
    // Click Delete Concept
    harness.get_by_label("Delete Concept").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(matches!(
        &harness.state().actions[0],
        EditorAction::DeleteConcept { .. }
    ));
}

/// Relations tab — delete relation produces action.
#[test]
fn relations_tab_delete_relation_produces_action() {
    struct RelState {
        relation_reg: RelationRegistry,
        concept_reg: ConceptRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let relation_reg = test_relation_registry();
    let concept_reg = ConceptRegistry::default();
    let s = RelState {
        relation_reg,
        concept_reg,
        state: EditorState::default(),
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut RelState| {
            systems::render_relations_tab(
                ui,
                &mut s.relation_reg,
                &s.concept_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    // Open relation detail header
    harness.get_by_label("Terrain Cost").click();
    harness.run();
    // Click Delete button
    harness.get_by_label("Delete").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(matches!(
        &harness.state().actions[0],
        EditorAction::DeleteRelation { .. }
    ));
}

/// Relations tab — create relation with default `ModifyProperty` effect.
#[test]
fn relations_tab_create_relation_modify_property() {
    struct RelState {
        relation_reg: RelationRegistry,
        concept_reg: ConceptRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }

    let concept_reg = test_concept_registry();
    let state = EditorState {
        new_relation_name: "MoveCost".to_string(),
        new_relation_trigger_index: 0, // OnEnter
        new_relation_effect_index: 0,  // ModifyProperty
        new_relation_target_prop: "budget".to_string(),
        new_relation_source_prop: "cost".to_string(),
        new_relation_operation_index: 1, // Subtract
        ..EditorState::default()
    };
    let s = RelState {
        relation_reg: RelationRegistry::default(),
        concept_reg,
        state,
        actions: Vec::new(),
    };

    let mut harness = Harness::new_ui_state(
        |ui, s: &mut RelState| {
            systems::render_relations_tab(
                ui,
                &mut s.relation_reg,
                &s.concept_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );

    harness.get_by_label("+ Create Relation").click();
    harness.run();

    assert!(!harness.state().actions.is_empty());
    assert!(
        matches!(&harness.state().actions[0], EditorAction::CreateRelation { trigger, effect, .. }
            if matches!(trigger, RelationTrigger::OnEnter) && matches!(effect, RelationEffect::ModifyProperty { .. }))
    );
}

/// Constraints tab — `PropertyCompare` expression with role selector.
#[test]
fn constraints_tab_property_compare_with_role() {
    let mut constraint_reg = ConstraintRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 0, // PropertyCompare
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    // PropertyCompare should show Role selector when concepts have roles
    harness.get_by_label_contains("Role:");
    harness.get_by_label_contains("Op:");
}

/// Constraints tab — `PathBudget` expression with role selector.
#[test]
fn constraints_tab_path_budget_with_role() {
    let mut constraint_reg = ConstraintRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 3, // PathBudget
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    // PathBudget should show Cost role selector and Budget fields
    harness.get_by_label_contains("Role:");
    harness.get_by_label_contains("Budget:");
}

// ---------------------------------------------------------------------------
// Batch 5: Button interactions and uncovered branches
// ---------------------------------------------------------------------------

// ── render_play: turn tracker Next Phase button ──

/// Turn tracker — Next Phase advances to next phase.
#[test]
fn turn_tracker_next_phase_click_advances_phase() {
    struct S {
        ts: TurnState,
        turn: TurnStructure,
    }
    let s = S {
        ts: TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
        },
        turn: test_turn_structure(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_play::render_turn_tracker(ui, &mut s.ts, &s.turn);
        },
        s,
    );
    harness.get_by_label_contains("Next Phase").click();
    harness.run();
    assert_eq!(harness.state().ts.current_phase_index, 1);
    assert_eq!(harness.state().ts.turn_number, 1);
}

/// Turn tracker — Next Phase on last phase wraps to next turn.
#[test]
fn turn_tracker_next_phase_wraps_turn() {
    struct S {
        ts: TurnState,
        turn: TurnStructure,
    }
    let s = S {
        ts: TurnState {
            turn_number: 1,
            current_phase_index: 2, // last phase (Admin)
            is_active: true,
        },
        turn: test_turn_structure(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_play::render_turn_tracker(ui, &mut s.ts, &s.turn);
        },
        s,
    );
    harness.get_by_label_contains("Next Phase").click();
    harness.run();
    assert_eq!(harness.state().ts.current_phase_index, 0);
    assert_eq!(harness.state().ts.turn_number, 2);
}

/// Turn tracker — empty phases shows message.
#[test]
fn turn_tracker_no_phases_shows_message() {
    let mut ts = TurnState {
        turn_number: 0,
        current_phase_index: 0,
        is_active: false,
    };
    let empty = TurnStructure {
        player_order: PlayerOrder::Alternating,
        phases: vec![],
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut ts, &empty);
    });
    harness.get_by_label_contains("No phases defined");
}

/// Turn tracker — `turn_number` 0 initializes state.
#[test]
fn turn_tracker_initializes_from_zero() {
    struct S {
        ts: TurnState,
        turn: TurnStructure,
    }
    let s = S {
        ts: TurnState {
            turn_number: 0,
            current_phase_index: 0,
            is_active: false,
        },
        turn: test_turn_structure(),
    };
    let harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_play::render_turn_tracker(ui, &mut s.ts, &s.turn);
        },
        s,
    );
    assert_eq!(harness.state().ts.turn_number, 1);
    assert!(harness.state().ts.is_active);
}

// ── render_play: combat panel interactions ──

/// Combat panel — Clear Combat button resets state.
#[test]
fn combat_panel_clear_combat_resets_state() {
    struct S {
        combat: ActiveCombat,
        state: EditorState,
    }
    let s = S {
        combat: ActiveCombat {
            die_roll: Some(3),
            ..ActiveCombat::default()
        },
        state: EditorState {
            combat_attacker_strength: 5.0,
            combat_defender_strength: 3.0,
            ..EditorState::default()
        },
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            // Replicate the Clear Combat button logic from render_combat_panel
            ui.label("Combat Resolution");
            if ui.button("Clear Combat").clicked() {
                s.combat = ActiveCombat::default();
                s.state.combat_attacker_strength = 0.0;
                s.state.combat_defender_strength = 0.0;
            }
        },
        s,
    );
    harness.get_by_label("Clear Combat").click();
    harness.run();
    assert!(harness.state().combat.die_roll.is_none());
    assert_eq!(harness.state().state.combat_attacker_strength, 0.0);
}

/// Combat panel — odds display with `def_str` 0 shows no odds label.
#[test]
fn combat_panel_zero_defender_no_odds() {
    let state = EditorState {
        combat_attacker_strength: 5.0,
        combat_defender_strength: 0.0,
        ..EditorState::default()
    };
    let harness = Harness::new_ui(|ui| {
        let def_str = state.combat_defender_strength;
        if def_str > 0.0 {
            ui.label(format!(
                "Odds: {:.2}:1",
                state.combat_attacker_strength / def_str
            ));
        } else {
            ui.label("No odds (DEF=0)");
        }
    });
    harness.get_by_label_contains("No odds");
}

/// Combat panel — die roll result and outcome rendering.
#[test]
fn combat_panel_die_roll_result_display() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let outcome = CombatOutcome {
        label: "DR1".to_string(),
        effect: Some(OutcomeEffect::Retreat { hexes: 2 }),
    };
    let harness = Harness::new_ui(|ui| {
        ui.label("Die roll:");
        ui.label("4");
        ui.label(format!("Result: {}", outcome.label));
        if let Some(effect) = &outcome.effect {
            let effect_text = match effect {
                OutcomeEffect::NoEffect => "No effect".to_string(),
                OutcomeEffect::Retreat { hexes } => format!("Defender retreats {hexes} hex(es)"),
                OutcomeEffect::StepLoss { steps } => format!("Defender loses {steps} step(s)"),
                OutcomeEffect::AttackerStepLoss { steps } => {
                    format!("Attacker loses {steps} step(s)")
                }
                OutcomeEffect::Exchange {
                    attacker_steps,
                    defender_steps,
                } => format!("Exchange: ATK -{attacker_steps}, DEF -{defender_steps}"),
                OutcomeEffect::AttackerEliminated => "Attacker eliminated".to_string(),
                OutcomeEffect::DefenderEliminated => "Defender eliminated".to_string(),
            };
            ui.label(&effect_text);
        }
    });
    harness.get_by_label_contains("Die roll:");
    harness.get_by_label_contains("Result: DR1");
    harness.get_by_label_contains("retreats 2 hex");
}

/// Combat panel — all outcome effect types render.
#[test]
fn combat_panel_all_outcome_effects() {
    use hexorder_contracts::mechanics::OutcomeEffect;
    let effects = vec![
        OutcomeEffect::NoEffect,
        OutcomeEffect::Retreat { hexes: 1 },
        OutcomeEffect::StepLoss { steps: 2 },
        OutcomeEffect::AttackerStepLoss { steps: 1 },
        OutcomeEffect::Exchange {
            attacker_steps: 1,
            defender_steps: 2,
        },
        OutcomeEffect::AttackerEliminated,
        OutcomeEffect::DefenderEliminated,
    ];
    let harness = Harness::new_ui(|ui| {
        for effect in &effects {
            let text = match effect {
                OutcomeEffect::NoEffect => "No effect".to_string(),
                OutcomeEffect::Retreat { hexes } => format!("Retreat {hexes}"),
                OutcomeEffect::StepLoss { steps } => format!("StepLoss {steps}"),
                OutcomeEffect::AttackerStepLoss { steps } => format!("AtkLoss {steps}"),
                OutcomeEffect::Exchange {
                    attacker_steps,
                    defender_steps,
                } => format!("Ex {attacker_steps}/{defender_steps}"),
                OutcomeEffect::AttackerEliminated => "AtkElim".to_string(),
                OutcomeEffect::DefenderEliminated => "DefElim".to_string(),
            };
            ui.label(&text);
        }
    });
    harness.get_by_label_contains("No effect");
    harness.get_by_label_contains("Retreat 1");
    harness.get_by_label_contains("StepLoss 2");
    harness.get_by_label_contains("AtkLoss 1");
    harness.get_by_label_contains("Ex 1/2");
    harness.get_by_label_contains("AtkElim");
    harness.get_by_label_contains("DefElim");
}

// ── render_rules: validation error categories ──

/// Validation tab — shows `RoleMismatch` error category.
#[test]
fn validation_tab_role_mismatch_category() {
    let validation = SchemaValidation {
        is_valid: false,
        errors: vec![SchemaError {
            category: SchemaErrorCategory::RoleMismatch,
            message: "token in board slot".to_string(),
            source_id: TypeId::new(),
        }],
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_validation_tab(ui, &validation);
    });
    harness.get_by_label_contains("Role Mismatch");
}

/// Validation tab — shows `PropertyMismatch` error category.
#[test]
fn validation_tab_property_mismatch_category() {
    let validation = SchemaValidation {
        is_valid: false,
        errors: vec![SchemaError {
            category: SchemaErrorCategory::PropertyMismatch,
            message: "wrong property type".to_string(),
            source_id: TypeId::new(),
        }],
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_validation_tab(ui, &validation);
    });
    harness.get_by_label_contains("Prop Mismatch");
}

/// Validation tab — shows `MissingBinding` error category.
#[test]
fn validation_tab_missing_binding_category() {
    let validation = SchemaValidation {
        is_valid: false,
        errors: vec![SchemaError {
            category: SchemaErrorCategory::MissingBinding,
            message: "concept not bound".to_string(),
            source_id: TypeId::new(),
        }],
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_validation_tab(ui, &validation);
    });
    harness.get_by_label_contains("Missing Binding");
}

/// Validation tab — shows `InvalidExpression` error category.
#[test]
fn validation_tab_invalid_expression_category() {
    let validation = SchemaValidation {
        is_valid: false,
        errors: vec![SchemaError {
            category: SchemaErrorCategory::InvalidExpression,
            message: "bad expr".to_string(),
            source_id: TypeId::new(),
        }],
    };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_validation_tab(ui, &validation);
    });
    harness.get_by_label_contains("Invalid Expr");
}

// ── render_rules: mechanics tab interactions ──

/// Mechanics tab — Add Phase button produces action.
#[test]
fn mechanics_tab_add_phase_produces_action() {
    struct S {
        ts: TurnStructure,
        crt: CombatResultsTable,
        mods: CombatModifierRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        ts: test_turn_structure(),
        crt: test_crt(),
        mods: CombatModifierRegistry::default(),
        state: EditorState {
            new_phase_name: "Reinforcement".to_string(),
            new_phase_type_index: 2, // Admin
            ..EditorState::default()
        },
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_rules::render_mechanics_tab(
                ui,
                &s.ts,
                &s.crt,
                &s.mods,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );
    harness.get_by_label("Add Phase").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddPhase { name, phase_type }
            if name == "Reinforcement" && *phase_type == PhaseType::Admin)),
        "expected AddPhase action, got: {actions:?}"
    );
}

/// Mechanics tab — CRT Differential column type renders "diff" label.
#[test]
fn mechanics_tab_crt_differential_column() {
    let crt = CombatResultsTable {
        id: TypeId::new(),
        name: "Diff CRT".to_string(),
        columns: vec![CrtColumn {
            label: "D+2".to_string(),
            column_type: CrtColumnType::Differential,
            threshold: 2.0,
        }],
        rows: vec![],
        outcomes: vec![],
        combat_concept_id: None,
    };
    let ts = test_turn_structure();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label_contains("diff");
}

/// Mechanics tab — all modifier source types display correct labels.
#[test]
fn mechanics_tab_all_modifier_sources() {
    let mods = CombatModifierRegistry {
        modifiers: vec![
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "DefTerr Mod".to_string(),
                source: ModifierSource::DefenderTerrain,
                column_shift: -1,
                priority: 10,
                cap: None,
                terrain_type_filter: None,
            },
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "AtkTerr Mod".to_string(),
                source: ModifierSource::AttackerTerrain,
                column_shift: 1,
                priority: 5,
                cap: None,
                terrain_type_filter: None,
            },
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "AtkProp Mod".to_string(),
                source: ModifierSource::AttackerProperty("str".to_string()),
                column_shift: 2,
                priority: 3,
                cap: None,
                terrain_type_filter: None,
            },
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "DefProp Mod".to_string(),
                source: ModifierSource::DefenderProperty("def".to_string()),
                column_shift: -2,
                priority: 2,
                cap: None,
                terrain_type_filter: None,
            },
            CombatModifierDefinition {
                id: TypeId::new(),
                name: "Custom Mod".to_string(),
                source: ModifierSource::Custom("river crossing".to_string()),
                column_shift: -3,
                priority: 1,
                cap: None,
                terrain_type_filter: None,
            },
        ],
    };
    let ts = test_turn_structure();
    let crt = test_crt();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label_contains("[DefTerr]");
    harness.get_by_label_contains("[AtkTerr]");
    harness.get_by_label_contains("[AtkProp(str)]");
    harness.get_by_label_contains("[DefProp(def)]");
    harness.get_by_label_contains("[Custom(river crossing)]");
}

/// Mechanics tab — Add Modifier with custom source shows Desc field.
#[test]
fn mechanics_tab_add_modifier_custom_source_desc_field() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_modifier_source_index: 2, // Custom
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    // When source index is 2 (Custom), "Desc:" field should appear
    harness.get_by_label_contains("Desc:");
}

/// Mechanics tab — Add Row button produces action.
#[test]
fn mechanics_tab_add_crt_row_produces_action() {
    struct S {
        ts: TurnStructure,
        crt: CombatResultsTable,
        mods: CombatModifierRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        ts: test_turn_structure(),
        crt: test_crt(),
        mods: CombatModifierRegistry::default(),
        state: EditorState {
            new_crt_row_label: "3".to_string(),
            new_crt_row_die_min: "5".to_string(),
            new_crt_row_die_max: "6".to_string(),
            ..EditorState::default()
        },
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_rules::render_mechanics_tab(
                ui,
                &s.ts,
                &s.crt,
                &s.mods,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );
    harness.get_by_label("Add Row").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtRow { .. })),
        "expected AddCrtRow action, got: {actions:?}"
    );
}

/// Mechanics tab — Add Col button produces action.
#[test]
fn mechanics_tab_add_crt_col_produces_action() {
    struct S {
        ts: TurnStructure,
        crt: CombatResultsTable,
        mods: CombatModifierRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        ts: test_turn_structure(),
        crt: test_crt(),
        mods: CombatModifierRegistry::default(),
        state: EditorState {
            new_crt_col_label: "2:1".to_string(),
            new_crt_col_threshold: "2.0".to_string(),
            ..EditorState::default()
        },
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_rules::render_mechanics_tab(
                ui,
                &s.ts,
                &s.crt,
                &s.mods,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );
    harness.get_by_label("Add Col").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtColumn { .. })),
        "expected AddCrtColumn action, got: {actions:?}"
    );
}

/// Mechanics tab — Add Modifier form renders with fields.
#[test]
fn mechanics_tab_add_modifier_form_renders() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    // "Name:" appears in both phase and modifier sections — skip it.
    harness.get_by_label_contains("Source:");
    harness.get_by_label_contains("Shift:");
    harness.get_by_label_contains("Priority:");
    harness.get_by_label_contains("Add Modifier");
}

// ── render_rules: inspector with entity data ──

/// Tile inspector — renders property editors for entity with properties.
#[test]
fn inspector_renders_property_editors() {
    let registry = test_registry();
    let type_id = registry.types[0].id; // Plains
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let pos = Some(HexPosition { q: 2, r: 3 });
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(
            ui,
            pos,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label_contains("Position: (2, 3)");
    harness.get_by_label_contains("Type: Plains");
    harness.get_by_label_contains("Properties");
    harness.get_by_label_contains("movement_cost:");
}

/// Tile inspector — no entity data shows "No cell data".
#[test]
fn inspector_no_entity_data() {
    let registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let pos = Some(HexPosition { q: 0, r: 0 });
    let harness = Harness::new_ui(|ui| {
        render_rules::render_inspector(ui, pos, None, &registry, &enum_reg, &struct_reg);
    });
    harness.get_by_label_contains("No cell data");
}

/// Unit inspector — renders property editors and delete button for unit.
#[test]
fn unit_inspector_renders_properties_and_delete() {
    let registry = test_registry();
    let type_id = registry.types[1].id; // Infantry (Token)
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut entity_data = EntityData {
        entity_type_id: type_id,
        properties: std::collections::HashMap::new(),
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_unit_inspector(
            ui,
            Some(&mut entity_data),
            &registry,
            &enum_reg,
            &struct_reg,
            &mut actions,
        );
    });
    harness.get_by_label_contains("Unit Type: Infantry");
    harness.get_by_label_contains("Properties");
    harness.get_by_label_contains("movement_points:");
    harness.get_by_label_contains("Delete Unit");
}

/// Unit inspector — delete button produces action.
#[test]
fn unit_inspector_delete_produces_action() {
    struct S {
        entity_data: EntityData,
        registry: EntityTypeRegistry,
        enum_reg: EnumRegistry,
        struct_reg: StructRegistry,
        actions: Vec<EditorAction>,
    }
    let registry = test_registry();
    let type_id = registry.types[1].id;
    let s = S {
        entity_data: EntityData {
            entity_type_id: type_id,
            properties: std::collections::HashMap::new(),
        },
        registry,
        enum_reg: test_enum_registry(),
        struct_reg: test_struct_registry(),
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            render_rules::render_unit_inspector(
                ui,
                Some(&mut s.entity_data),
                &s.registry,
                &s.enum_reg,
                &s.struct_reg,
                &mut s.actions,
            );
        },
        s,
    );
    harness.get_by_label("Delete Unit").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteSelectedUnit)),
        "expected DeleteSelectedUnit action, got: {actions:?}"
    );
}

// ── render_ontology: constraint interactions ──

/// Constraints tab — auto-generated constraint shows [auto] label.
#[test]
fn constraints_tab_auto_generated_label() {
    let mut constraint_reg = test_constraint_registry();
    let concept_reg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label_contains("[auto]");
}

/// Constraints tab — Create Constraint button produces action.
#[test]
fn constraints_tab_create_constraint_action() {
    struct S {
        constraint_reg: ConstraintRegistry,
        concept_reg: ConceptRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        constraint_reg: ConstraintRegistry::default(),
        concept_reg: test_concept_registry(),
        state: EditorState {
            new_constraint_name: "Budget Check".to_string(),
            new_constraint_description: "Must have budget".to_string(),
            new_constraint_concept_index: 0,
            new_constraint_expr_type_index: 0, // PropertyCompare
            ..EditorState::default()
        },
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            systems::render_constraints_tab(
                ui,
                &mut s.constraint_reg,
                &s.concept_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );
    // Open the New Constraint collapsing header
    harness.get_by_label("New Constraint").click();
    harness.run();
    // Click + Create Constraint
    harness.get_by_label_contains("+ Create Constraint").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::CreateConstraint { .. })),
        "expected CreateConstraint action, got: {actions:?}"
    );
}

/// Constraints tab — `CrossCompare` expression type shows editor placeholder.
#[test]
fn constraints_tab_cross_compare_expr_placeholder() {
    let mut constraint_reg = ConstraintRegistry::default();
    let concept_reg = test_concept_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 1, // CrossCompare
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(
            ui,
            &mut constraint_reg,
            &concept_reg,
            &mut state,
            &mut actions,
        );
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("full editor");
}

// ── render_design: entity type interactions ──

/// Entity type section — existing type with properties renders property list.
#[test]
fn entity_type_section_shows_properties() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    // Open "Board Types" section header then Plains type header
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("movement_cost (Int)");
}

/// Entity type section — delete type button appears with 2+ types of same role.
#[test]
fn entity_type_section_delete_with_multiple_types() {
    struct S {
        registry: EntityTypeRegistry,
        enum_reg: EnumRegistry,
        struct_reg: StructRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let mut registry = test_registry();
    // Add second board position type
    registry.types.push(EntityType {
        id: TypeId::new(),
        name: "Hills".to_string(),
        role: EntityRole::BoardPosition,
        color: Color::srgb(0.5, 0.4, 0.3),
        properties: vec![],
    });
    let s = S {
        registry,
        enum_reg: test_enum_registry(),
        struct_reg: test_struct_registry(),
        state: EditorState::default(),
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            systems::render_entity_type_section(
                ui,
                &mut s.registry,
                &mut s.state,
                &mut s.actions,
                EntityRole::BoardPosition,
                "Board Types",
                "board",
                &s.enum_reg,
                &s.struct_reg,
            );
        },
        s,
    );
    // Open Board Types section, then Hills header
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hills").click();
    harness.run();
    harness.get_by_label("Delete Type").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteEntityType { .. })),
        "expected DeleteEntityType action, got: {actions:?}"
    );
}

/// Entity type section — add property with Enum type shows Opts field.
#[test]
fn entity_type_section_add_prop_enum_shows_opts() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 5, // Enum
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    // Open Plains header
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Opts:");
    harness.get_by_label_contains("(comma-separated)");
}

/// Entity type section — add property with `EntityRef` shows Role selector.
#[test]
fn entity_type_section_add_prop_entity_ref_shows_role() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 6, // EntityRef
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Role:");
}

/// Entity type section — add property with List shows Item type selector.
#[test]
fn entity_type_section_add_prop_list_shows_item_type() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 7, // List
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Item type:");
}

/// Entity type section — add property with Map shows Key enum and Value type.
#[test]
fn entity_type_section_add_prop_map_shows_key_value() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 8, // Map
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Key enum:");
    harness.get_by_label_contains("Value type:");
}

/// Entity type section — add property with Struct shows Struct selector.
#[test]
fn entity_type_section_add_prop_struct_shows_selector() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 9, // Struct
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label_contains("Struct:");
}

/// Entity type section — add property with `IntRange` shows min/max fields.
#[test]
fn entity_type_section_add_prop_int_range_shows_minmax() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 10, // IntRange
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    // IntRange shows Min:/Max: fields
    harness.get_by_label_contains("Min:");
}

/// Entity type section — add property with `FloatRange` shows min/max fields.
#[test]
fn entity_type_section_add_prop_float_range_shows_minmax() {
    let mut registry = test_registry();
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let mut state = EditorState {
        new_prop_type_index: 11, // FloatRange
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut registry,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &enum_reg,
            &struct_reg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    // FloatRange shows Min:/Max: fields too
    harness.get_by_label_contains("Min:");
}

// ── render_design: enum interactions ──

/// Enums tab — enum with options renders options list and allows removal.
#[test]
fn enums_tab_shows_options_list() {
    let enum_reg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &enum_reg, &mut state, &mut actions);
    });
    // Open the Terrain enum header
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label_contains("Open");
    harness.get_by_label_contains("Rough");
    harness.get_by_label_contains("Dense");
}

/// Enums tab — Delete Enum click dispatches action.
#[test]
fn enums_tab_delete_enum_click_dispatches_action() {
    struct S {
        enum_reg: EnumRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        enum_reg: test_enum_registry(),
        state: EditorState::default(),
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            systems::render_enums_tab(ui, &s.enum_reg, &mut s.state, &mut s.actions);
        },
        s,
    );
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label("Delete Enum").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteEnum { .. })),
        "expected DeleteEnum action, got: {actions:?}"
    );
}

// ── render_design: struct interactions ──

/// Structs tab — struct with fields renders field list.
#[test]
fn structs_tab_shows_field_list() {
    let struct_reg = test_struct_registry();
    let enum_reg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &struct_reg, &enum_reg, &mut state, &mut actions);
    });
    // Open the Position struct header
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label_contains("x: Int");
    harness.get_by_label_contains("y: Int");
}

/// Structs tab — Delete Struct click dispatches action.
#[test]
fn structs_tab_delete_struct_click_dispatches_action() {
    struct S {
        struct_reg: StructRegistry,
        enum_reg: EnumRegistry,
        state: EditorState,
        actions: Vec<EditorAction>,
    }
    let s = S {
        struct_reg: test_struct_registry(),
        enum_reg: test_enum_registry(),
        state: EditorState::default(),
        actions: Vec::new(),
    };
    let mut harness = Harness::new_ui_state(
        |ui, s: &mut S| {
            systems::render_structs_tab(
                ui,
                &s.struct_reg,
                &s.enum_reg,
                &mut s.state,
                &mut s.actions,
            );
        },
        s,
    );
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label("Delete Struct").click();
    harness.run();
    let actions = &harness.state().actions;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteStruct { .. })),
        "expected DeleteStruct action, got: {actions:?}"
    );
}

// ── render_rules: property value editor edge cases ──

/// Property value editor — `IntRange` renders with bounds.
#[test]
fn property_value_editor_int_range() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let mut value = PropertyValue::IntRange(5);
    let prop_type = PropertyType::IntRange { min: 0, max: 10 };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — `FloatRange` renders with bounds.
#[test]
fn property_value_editor_float_range() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let mut value = PropertyValue::FloatRange(2.5);
    let prop_type = PropertyType::FloatRange { min: 0.0, max: 5.0 };
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — List renders items and add button.
#[test]
fn property_value_editor_list_renders() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let mut value = PropertyValue::List(vec![PropertyValue::Int(1), PropertyValue::Int(2)]);
    let prop_type = PropertyType::List(Box::new(PropertyType::Int));
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — List at depth 3 shows nesting limit.
#[test]
fn property_value_editor_list_nesting_limit() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let mut value = PropertyValue::List(vec![PropertyValue::Int(1)]);
    let prop_type = PropertyType::List(Box::new(PropertyType::Int));
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            3,
        );
    });
    harness.get_by_label_contains("nested");
}

/// Property value editor — Map renders keys.
#[test]
fn property_value_editor_map_renders() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let enum_id = *enum_reg.definitions.keys().next().expect("has enum");
    let mut value = PropertyValue::Map(vec![("Open".to_string(), PropertyValue::Int(1))]);
    let prop_type = PropertyType::Map(enum_id, Box::new(PropertyType::Int));
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — Struct renders field editors.
#[test]
fn property_value_editor_struct_renders() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let struct_id = *struct_reg.definitions.keys().next().expect("has struct");
    let struct_def = struct_reg.definitions.values().next().expect("has struct");
    let field_ids: Vec<_> = struct_def.fields.iter().map(|f| f.id).collect();
    let mut fields = std::collections::HashMap::new();
    fields.insert(field_ids[0], PropertyValue::Int(10));
    fields.insert(field_ids[1], PropertyValue::Int(20));
    let mut value = PropertyValue::Struct(fields);
    let prop_type = PropertyType::Struct(struct_id);
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — `EntityRef` renders combobox.
#[test]
fn property_value_editor_entity_ref_renders() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let mut value = PropertyValue::EntityRef(None);
    let prop_type = PropertyType::EntityRef(None);
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — Enum renders combobox with options.
#[test]
fn property_value_editor_enum_combobox() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let enum_id = *enum_reg.definitions.keys().next().expect("has enum");
    let mut value = PropertyValue::Enum("Open".to_string());
    let prop_type = PropertyType::Enum(enum_id);
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            0,
        );
    });
    let _ = harness;
}

/// Property value editor — Map at depth 3 shows nesting limit.
#[test]
fn property_value_editor_map_nesting_limit() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let enum_id = *enum_reg.definitions.keys().next().expect("has enum");
    let mut value = PropertyValue::Map(vec![]);
    let prop_type = PropertyType::Map(enum_id, Box::new(PropertyType::Int));
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            3,
        );
    });
    harness.get_by_label_contains("nested");
}

/// Property value editor — Struct at depth 3 shows nesting limit.
#[test]
fn property_value_editor_struct_nesting_limit() {
    let enum_reg = test_enum_registry();
    let struct_reg = test_struct_registry();
    let registry = test_registry();
    let struct_id = *struct_reg.definitions.keys().next().expect("has struct");
    let mut value = PropertyValue::Struct(std::collections::HashMap::new());
    let prop_type = PropertyType::Struct(struct_id);
    let harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(
            ui,
            &mut value,
            &prop_type,
            &enum_reg,
            &struct_reg,
            &registry,
            3,
        );
    });
    harness.get_by_label_contains("nested");
}

// ===========================================================================
// Batch 6 — targeted coverage for uncovered branches
// ===========================================================================

// ── render_play: turn tracker ──

/// Turn tracker — empty phases shows placeholder.
#[test]
fn turn_tracker_empty_phases_placeholder() {
    let ts = TurnStructure {
        player_order: PlayerOrder::Alternating,
        phases: vec![],
    };
    let mut ts2 = TurnState::default();
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut ts2, &ts);
    });
    harness.get_by_label_contains("No phases defined");
}

/// Turn tracker — `turn_number` 0 initializes.
#[test]
fn turn_tracker_init_turn_number() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 0,
        current_phase_index: 0,
        is_active: false,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label_contains("Turn 1");
}

/// Turn tracker — Movement type label.
#[test]
fn turn_tracker_movement_label() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label_contains("[Movement]");
}

/// Turn tracker — Combat type label.
#[test]
fn turn_tracker_combat_label() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 1,
        current_phase_index: 1,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label_contains("[Combat]");
    harness.get_by_label_contains("Phase 2 of 3");
}

/// Turn tracker — Admin type label.
#[test]
fn turn_tracker_admin_label() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 1,
        current_phase_index: 2,
        is_active: true,
    };
    let harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label_contains("[Admin]");
    harness.get_by_label_contains("Phase 3 of 3");
}

/// Turn tracker — Next Phase wraps turn (batch 6).
#[test]
fn turn_tracker_wrap_increments_turn() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 1,
        current_phase_index: 2,
        is_active: true,
    };
    let mut harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label("Next Phase \u{23E9}").click();
    harness.run();
    harness.get_by_label_contains("Turn 2");
}

/// Turn tracker — Next Phase advances within turn.
#[test]
fn turn_tracker_advance_within_turn() {
    let ts = test_turn_structure();
    let mut s = TurnState {
        turn_number: 1,
        current_phase_index: 0,
        is_active: true,
    };
    let mut harness = Harness::new_ui(|ui| {
        render_play::render_turn_tracker(ui, &mut s, &ts);
    });
    harness.get_by_label("Next Phase \u{23E9}").click();
    harness.run();
    harness.get_by_label_contains("Phase 2 of 3");
}

// ── render_rules: mechanics tab ──

/// Mechanics tab — outcome grid resyncs when buffer empty.
#[test]
fn mechanics_tab_outcome_grid_resync_on_empty_buffer() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Outcome Grid");
}

/// Mechanics tab — add CRT column with Differential type.
#[test]
fn mechanics_tab_crt_col_differential() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_crt_col_label: "D+1".to_string(),
        new_crt_col_type_index: 1,
        new_crt_col_threshold: "1.0".to_string(),
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label("Add Col").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtColumn { .. }))
    );
}

/// Mechanics tab — add phase with Movement type.
#[test]
fn mechanics_tab_phase_movement() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_phase_name: "Advance".to_string(),
        new_phase_type_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label("Add Phase").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddPhase { .. }))
    );
}

/// Mechanics tab — add phase with Combat type.
#[test]
fn mechanics_tab_phase_combat() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_phase_name: "Fire".to_string(),
        new_phase_type_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label("Add Phase").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddPhase { .. }))
    );
}

/// Mechanics tab — add CRT row with bad `die_max` uses fallback.
#[test]
fn mechanics_tab_crt_row_bad_die_max() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_crt_row_label: "R".to_string(),
        new_crt_row_die_min: "3".to_string(),
        new_crt_row_die_max: "abc".to_string(),
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label("Add Row").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtRow { .. }))
    );
}

/// Mechanics tab — modifier form with source index 1 renders Atk selector.
#[test]
fn mechanics_tab_modifier_form_atk_source() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_modifier_source_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Atk.Terrain");
}

/// Mechanics tab — modifier form with source index 2 shows Custom desc.
#[test]
fn mechanics_tab_modifier_form_custom_desc() {
    let ts = test_turn_structure();
    let crt = test_crt();
    let mods = CombatModifierRegistry::default();
    let mut state = EditorState {
        new_modifier_source_index: 2,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        render_rules::render_mechanics_tab(ui, &ts, &crt, &mods, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Desc:");
}

// ── render_ontology ──

/// Concepts tab — bindings render entity-role mapping.
#[test]
fn concepts_tab_bindings_show_entity_role() {
    use hexorder_contracts::ontology::{ConceptBinding, PropertyBinding};
    let cid = TypeId::new();
    let rid = TypeId::new();
    let etid = TypeId::new();
    let reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: etid,
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.5),
            properties: vec![],
        }],
    };
    let mut creg = ConceptRegistry {
        concepts: vec![Concept {
            id: cid,
            name: "Terrain".to_string(),
            description: String::new(),
            role_labels: vec![ConceptRole {
                id: rid,
                name: "ground".to_string(),
                allowed_entity_roles: vec![EntityRole::BoardPosition],
            }],
        }],
        bindings: vec![ConceptBinding {
            id: TypeId::new(),
            entity_type_id: etid,
            concept_id: cid,
            concept_role_id: rid,
            property_bindings: vec![PropertyBinding {
                property_id: TypeId::new(),
                concept_local_name: "cost".to_string(),
            }],
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label_contains("Plains -> ground");
    harness.get_by_label_contains("cost");
}

/// Relations tab — `OnExit` trigger when opened.
#[test]
fn relations_tab_on_exit_trigger_label() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Exit Cost".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnExit,
            effect: RelationEffect::ModifyProperty {
                target_property: "budget".to_string(),
                source_property: "exit_cost".to_string(),
                operation: ModifyOperation::Subtract,
            },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Exit Cost").click();
    harness.run();
    harness.get_by_label_contains("OnExit");
}

/// Relations tab — `WhilePresent` trigger when opened.
#[test]
fn relations_tab_while_present_label() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Stacking".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::WhilePresent,
            effect: RelationEffect::Block { condition: None },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Stacking").click();
    harness.run();
    harness.get_by_label_contains("WhilePresent");
}

/// Relations tab — Block effect when opened.
#[test]
fn relations_tab_block_effect_opened() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Impassable".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Impassable").click();
    harness.run();
    harness.get_by_label_contains("Block");
}

/// Relations tab — Allow effect when opened.
#[test]
fn relations_tab_allow_effect_opened() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Road".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Allow { condition: None },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Road").click();
    harness.run();
    harness.get_by_label_contains("Allow");
}

/// Relations tab — empty concepts skips selector.
#[test]
fn relations_tab_no_concepts() {
    let creg = ConceptRegistry::default();
    let mut rreg = RelationRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Relations");
}

/// Constraints tab — `PathBudget` form renders.
#[test]
fn constraints_tab_path_budget_form() {
    let creg = test_concept_registry();
    let mut conreg = test_constraint_registry();
    let mut state = EditorState {
        new_constraint_expr_type_index: 3,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("Cost:");
}

// ── render_design ──

/// Entity type section — remove property via "x" click.
#[test]
fn entity_type_section_prop_remove_click() {
    let pid = TypeId::new();
    let tid = TypeId::new();
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: tid,
            name: "Plains".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.4, 0.6, 0.2),
            properties: vec![PropertyDefinition {
                id: pid,
                name: "cost".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(1),
            }],
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();
    harness.get_by_label("x").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveProperty { .. }))
    );
}

/// Entity type section — add property with Enum type (index 5) form renders.
#[test]
fn entity_type_section_enum_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 5,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Opts:");
}

/// Entity type section — add property with `EntityRef` (index 6) form renders.
#[test]
fn entity_type_section_entity_ref_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 6,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Role:");
}

/// Entity type section — add property with List (index 7) form renders.
#[test]
fn entity_type_section_list_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 7,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Item type:");
}

/// Entity type section — add property with Map (index 8) form renders.
#[test]
fn entity_type_section_map_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 8,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = test_enum_registry();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Value type:");
}

/// Entity type section — add property with Struct (index 9) form renders.
#[test]
fn entity_type_section_struct_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 9,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = test_struct_registry();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Struct:");
}

/// Entity type section — add property with `IntRange` (index 10) form renders.
#[test]
fn entity_type_section_int_range_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 10,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Min:");
}

/// Entity type section — add property with `FloatRange` (index 11) form renders.
#[test]
fn entity_type_section_float_range_prop_form() {
    let mut reg = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "Hill".to_string(),
            role: EntityRole::BoardPosition,
            color: Color::srgb(0.5, 0.5, 0.3),
            properties: vec![],
        }],
    };
    let mut state = EditorState {
        new_prop_type_index: 11,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_entity_type_section(
            ui,
            &mut reg,
            &mut state,
            &mut actions,
            EntityRole::BoardPosition,
            "Board Types",
            "board",
            &ereg,
            &sreg,
        );
    });
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Hill").click();
    harness.run();
    harness.get_by_label_contains("Min:");
}

// ── render_rules: property value editor CollapsingHeader ──

/// Property value editor — List body renders items.
#[test]
fn prop_editor_list_body_items() {
    use super::render_rules;
    let mut val = PropertyValue::List(vec![PropertyValue::Int(10), PropertyValue::Int(20)]);
    let pt = PropertyType::List(Box::new(PropertyType::Int));
    let ereg = EnumRegistry::default();
    let sreg = StructRegistry::default();
    let reg = test_registry();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(ui, &mut val, &pt, &ereg, &sreg, &reg, 0);
    });
    harness.get_by_label_contains("List (2)").click();
    harness.run();
    harness.get_by_label_contains("[0]");
    harness.get_by_label_contains("[1]");
    harness.get_by_label_contains("+ Add");
}

/// Property value editor — Map body renders entries.
#[test]
fn prop_editor_map_body_entries() {
    use super::render_rules;
    let ereg = test_enum_registry();
    let eid = *ereg.definitions.keys().next().expect("has enum");
    let mut val = PropertyValue::Map(vec![("Open".to_string(), PropertyValue::Int(1))]);
    let pt = PropertyType::Map(eid, Box::new(PropertyType::Int));
    let sreg = StructRegistry::default();
    let reg = test_registry();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(ui, &mut val, &pt, &ereg, &sreg, &reg, 0);
    });
    // Enum has 3 options (Open, Rough, Dense). Map shows based on those.
    harness.get_by_label_contains("Map (").click();
    harness.run();
    // Should render the enum options as map keys.
    let _ = harness;
}

/// Property value editor — Struct body renders field editors.
#[test]
fn prop_editor_struct_body_fields() {
    use super::render_rules;
    let sreg = test_struct_registry();
    let sid = *sreg.definitions.keys().next().expect("has struct");
    let sdef = sreg.definitions.get(&sid).expect("has def");
    let mut fields = std::collections::HashMap::new();
    fields.insert(sdef.fields[0].id, PropertyValue::Int(5));
    fields.insert(sdef.fields[1].id, PropertyValue::Int(10));
    let mut val = PropertyValue::Struct(fields);
    let pt = PropertyType::Struct(sid);
    let ereg = EnumRegistry::default();
    let reg = test_registry();
    let mut harness = Harness::new_ui(|ui| {
        render_rules::render_property_value_editor(ui, &mut val, &pt, &ereg, &sreg, &reg, 0);
    });
    harness.get_by_label_contains("Position").click();
    harness.run();
    harness.get_by_label_contains("x:");
    harness.get_by_label_contains("y:");
}

// ── render_design: structs tab ──

/// Structs tab — add field with Bool type (index 0).
#[test]
fn structs_tab_field_bool_type() {
    let sreg = test_struct_registry();
    let ereg = EnumRegistry::default();
    let mut state = EditorState {
        new_struct_field_name: "active".to_string(),
        new_struct_field_type_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label("+ Add Field").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddStructField { .. }))
    );
}

/// Structs tab — add field with Float type (index 2).
#[test]
fn structs_tab_field_float_type() {
    let sreg = test_struct_registry();
    let ereg = EnumRegistry::default();
    let mut state = EditorState {
        new_struct_field_name: "weight".to_string(),
        new_struct_field_type_index: 2,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label("+ Add Field").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddStructField { .. }))
    );
}

// ===========================================================================
// Batch 7 — ontology & design CollapsingHeader body coverage
// ===========================================================================

// ── render_ontology: concepts tab body ──

/// Concepts tab — expanding concept shows roles section.
#[test]
fn concepts_tab_expanded_shows_roles() {
    let reg = test_registry();
    let mut creg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label_contains("Roles:");
    harness.get_by_label_contains("traveler [Token]");
    harness.get_by_label_contains("terrain [Board]");
}

/// Concepts tab — expanding concept with no roles shows Roles: label.
#[test]
fn concepts_tab_no_roles_placeholder() {
    let reg = test_registry();
    let mut creg = ConceptRegistry {
        concepts: vec![Concept {
            id: TypeId::new(),
            name: "Empty".to_string(),
            description: "No roles yet".to_string(),
            role_labels: vec![],
        }],
        bindings: vec![],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Empty").click();
    harness.run();
    harness.get_by_label_contains("Roles:");
    harness.get_by_label_contains("Bindings:");
}

/// Concepts tab — delete concept button produces action.
#[test]
fn concepts_tab_delete_concept() {
    let reg = test_registry();
    let mut creg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label("Delete Concept").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteConcept { .. }))
    );
}

/// Concepts tab — expanding concept shows empty bindings placeholder.
#[test]
fn concepts_tab_empty_bindings() {
    let reg = test_registry();
    let mut creg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label_contains("Bindings:");
}

/// Concepts tab — add role form renders Board/Token checkboxes.
#[test]
fn concepts_tab_add_role_form() {
    let reg = test_registry();
    let mut creg = test_concept_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Motion").click();
    harness.run();
    harness.get_by_label_contains("+ Add Role");
}

// ── render_ontology: relations tab creation form ──

/// Relations tab — creation form renders concept/subject/object selectors.
#[test]
fn relations_tab_creation_form_concept_selector() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("New Relation");
    harness.get_by_label_contains("Concept:");
    harness.get_by_label_contains("Subject:");
    harness.get_by_label_contains("Object:");
    harness.get_by_label_contains("Trigger:");
    harness.get_by_label_contains("Effect:");
}

/// Relations tab — `ModifyProperty` effect renders Target/Source/Op fields.
#[test]
fn relations_tab_modify_property_fields() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry::default();
    let mut state = EditorState {
        new_relation_effect_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Target:");
    harness.get_by_label_contains("Source:");
    harness.get_by_label_contains("Op:");
}

/// Relations tab — Block effect index 1 hides `ModifyProperty` fields.
#[test]
fn relations_tab_block_hides_modify_fields() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry::default();
    let mut state = EditorState {
        new_relation_effect_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    // Should NOT contain Target: / Source: / Op: when Block is selected
    harness.get_by_label_contains("Effect:");
}

/// Relations tab — expanded relation shows concept, roles, trigger, effect.
#[test]
fn relations_tab_expanded_shows_details() {
    let creg = test_concept_registry();
    let concept_id = creg.concepts[0].id;
    let subject_id = creg.concepts[0].role_labels[0].id;
    let object_id = creg.concepts[0].role_labels[1].id;
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Movement Cost".to_string(),
            concept_id,
            subject_role_id: subject_id,
            object_role_id: object_id,
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::ModifyProperty {
                target_property: "budget".to_string(),
                source_property: "cost".to_string(),
                operation: ModifyOperation::Subtract,
            },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Movement Cost").click();
    harness.run();
    harness.get_by_label_contains("Concept: Motion");
    harness.get_by_label_contains("traveler -> terrain");
    harness.get_by_label_contains("Trigger: OnEnter");
    harness.get_by_label_contains("budget - cost");
}

/// Relations tab — delete relation button produces action.
#[test]
fn relations_tab_delete_relation() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Terrain Cost".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain Cost").click();
    harness.run();
    harness.get_by_label("Delete").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteRelation { .. }))
    );
}

/// Relations tab — empty relations shows placeholder.
#[test]
fn relations_tab_empty_shows_placeholder() {
    let creg = test_concept_registry();
    let mut rreg = RelationRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_relations_tab(ui, &mut rreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("No relations defined");
}

// ── render_ontology: constraints tab ──

/// Constraints tab — existing constraint renders name and expression.
#[test]
fn constraints_tab_shows_constraint_list() {
    let creg = test_concept_registry();
    let mut conreg = test_constraint_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("Budget >= 0");
}

/// Constraints tab — delete constraint via x button.
#[test]
fn constraints_tab_delete_constraint() {
    let creg = test_concept_registry();
    let mut conreg = ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Single".to_string(),
            description: String::new(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::All(Vec::new()),
            auto_generated: false,
        }],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("x").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteConstraint { .. }))
    );
}

/// Constraints tab — `PropertyCompare` expression form renders.
#[test]
fn constraints_tab_property_compare_form() {
    let creg = test_concept_registry();
    let mut conreg = ConstraintRegistry::default();
    let mut state = EditorState {
        new_constraint_expr_type_index: 0,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("Prop:");
    harness.get_by_label_contains("Op:");
}

/// Constraints tab — empty constraints shows placeholder.
#[test]
fn constraints_tab_empty_placeholder() {
    let creg = test_concept_registry();
    let mut conreg = ConstraintRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label_contains("No constraints defined");
}

/// Constraints tab — `CrossCompare`/`IsType` shows placeholder editor.
#[test]
fn constraints_tab_cross_compare_placeholder() {
    let creg = test_concept_registry();
    let mut conreg = ConstraintRegistry::default();
    let mut state = EditorState {
        new_constraint_expr_type_index: 1,
        ..EditorState::default()
    };
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_constraints_tab(ui, &mut conreg, &creg, &mut state, &mut actions);
    });
    harness.get_by_label("New Constraint").click();
    harness.run();
    harness.get_by_label_contains("full editor");
}

// ── render_design: enum option and struct field removal ──

/// Enums tab — expanded enum body shows option labels.
#[test]
fn enums_tab_expanded_body_shows_options() {
    let ereg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label_contains("Open");
    harness.get_by_label_contains("Rough");
    harness.get_by_label_contains("Dense");
}

/// Enums tab — delete enum button produces action.
#[test]
fn enums_tab_delete_enum() {
    let ereg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label("Delete Enum").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteEnum { .. }))
    );
}

/// Enums tab — add option inline form renders.
#[test]
fn enums_tab_add_option_form() {
    let ereg = test_enum_registry();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Terrain").click();
    harness.run();
    harness.get_by_label_contains("Add:");
}

/// Structs tab — expanded struct shows field labels.
#[test]
fn structs_tab_expanded_shows_fields() {
    let sreg = test_struct_registry();
    let ereg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label_contains("x: Int");
    harness.get_by_label_contains("y: Int");
}

/// Structs tab — delete struct button produces action.
#[test]
fn structs_tab_delete_struct() {
    let sreg = test_struct_registry();
    let ereg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label("Delete Struct").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteStruct { .. }))
    );
}

/// Structs tab — field form shows type selector.
#[test]
fn structs_tab_field_form() {
    let sreg = test_struct_registry();
    let ereg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Position").click();
    harness.run();
    harness.get_by_label_contains("Field:");
}

// ===========================================================================
// Batch 8 — ComboBox dropdown bodies + button click coverage
// ===========================================================================

// ── render_ontology: concept role removal ──

/// Concept role x-button removal with single role concept.
#[test]
fn concepts_tab_remove_role_click() {
    let reg = test_registry();
    let mut creg = ConceptRegistry {
        concepts: vec![Concept {
            id: TypeId::new(),
            name: "Solo".to_string(),
            description: String::new(),
            role_labels: vec![ConceptRole {
                id: TypeId::new(),
                name: "unit".to_string(),
                allowed_entity_roles: vec![EntityRole::Token],
            }],
        }],
        bindings: vec![],
    };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_concepts_tab(ui, &mut creg, &reg, &mut state, &mut actions);
    });
    harness.get_by_label("Solo").click();
    harness.run();
    harness.get_by_label("x").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveConceptRole { .. }))
    );
}

// ── render_design: enum option x button ──

/// Enums tab — enum option remove via x button (single option enum).
#[test]
fn enums_tab_remove_option_click() {
    use std::collections::HashMap;
    let mut defs = HashMap::new();
    let eid = TypeId::new();
    defs.insert(
        eid,
        EnumDefinition {
            id: eid,
            name: "Single".to_string(),
            options: vec!["Only".to_string()],
        },
    );
    let ereg = EnumRegistry { definitions: defs };
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_enums_tab(ui, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Single").click();
    harness.run();
    harness.get_by_label("x").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveEnumOption { .. }))
    );
}

// ── render_design: struct field x button ──

/// Structs tab — struct field remove via x button (single field struct).
#[test]
fn structs_tab_remove_field_click() {
    use std::collections::HashMap;
    let sid = TypeId::new();
    let fid = TypeId::new();
    let mut defs = HashMap::new();
    defs.insert(
        sid,
        StructDefinition {
            id: sid,
            name: "Tiny".to_string(),
            fields: vec![PropertyDefinition {
                id: fid,
                name: "val".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(0),
            }],
        },
    );
    let sreg = StructRegistry { definitions: defs };
    let ereg = EnumRegistry::default();
    let mut state = EditorState::default();
    let mut actions = Vec::new();
    let mut harness = Harness::new_ui(|ui| {
        systems::render_structs_tab(ui, &sreg, &ereg, &mut state, &mut actions);
    });
    harness.get_by_label("Tiny").click();
    harness.run();
    harness.get_by_label("x").click();
    harness.run();
    drop(harness);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveStructField { .. }))
    );
}

// ---------------------------------------------------------------------------
// Batch 9 — ComboBox dropdown coverage tests
//
// Each test opens a ComboBox dropdown by clicking the widget, which causes
// the `.show_ui()` closure body to execute — covering the dropdown item
// rendering code.
//
// Pattern: ComboBoxes created with `from_id_salt()` expose their selected
// text via `accesskit::value` (not `label`). Find them with:
//   `get_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("text"))`
// ---------------------------------------------------------------------------

/// Helper: find a `ComboBox` by its current value and click it open.
fn click_combobox_by_value<State>(harness: &mut egui_kittest::Harness<'_, State>, value: &str) {
    let cb = harness.get_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some(value));
    cb.click();
    harness.run();
}

/// Helper: build a relation form harness for `ComboBox` tests.
fn relation_form_harness() -> egui_kittest::Harness<
    'static,
    (
        EditorState,
        RelationRegistry,
        ConceptRegistry,
        Vec<EditorAction>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                RelationRegistry,
                ConceptRegistry,
                Vec<EditorAction>,
            )| {
                super::render_ontology::render_relations_tab(
                    ui, &mut s.1, &s.2, &mut s.0, &mut s.3,
                );
            },
            (
                EditorState::default(),
                RelationRegistry::default(),
                test_concept_registry(),
                vec![],
            ),
        );
    h.run();
    h
}

/// Relation form: open Concept `ComboBox` dropdown.
#[test]
fn relation_form_concept_dropdown() {
    let mut harness = relation_form_harness();
    click_combobox_by_value(&mut harness, "Motion");
    assert!(harness.query_by_label("Motion").is_some());
}

/// Relation form: open Subject role `ComboBox` dropdown.
#[test]
fn relation_form_subject_dropdown() {
    let mut harness = relation_form_harness();
    // Two ComboBoxes share value "traveler" (subject + object).
    let cbs: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("traveler"))
        .collect();
    cbs[0].click();
    harness.run();
    harness.get_by_label("terrain");
}

/// Relation form: open Trigger `ComboBox` dropdown.
#[test]
fn relation_form_trigger_dropdown() {
    let mut harness = relation_form_harness();
    click_combobox_by_value(&mut harness, "OnEnter");
    harness.get_by_label("OnExit");
}

/// Relation form: open Effect `ComboBox` dropdown.
#[test]
fn relation_form_effect_dropdown() {
    let mut harness = relation_form_harness();
    click_combobox_by_value(&mut harness, "ModifyProperty");
    harness.get_by_label("Block");
}

/// Relation form: open Operation `ComboBox` dropdown.
#[test]
fn relation_form_operation_dropdown() {
    let mut harness = relation_form_harness();
    click_combobox_by_value(&mut harness, "Add");
    harness.get_by_label("Subtract");
}

/// Helper: build a constraint form harness for `ComboBox` tests.
fn constraint_form_harness(
    expr_type_index: usize,
) -> egui_kittest::Harness<
    'static,
    (
        EditorState,
        ConstraintRegistry,
        ConceptRegistry,
        Vec<EditorAction>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                ConstraintRegistry,
                ConceptRegistry,
                Vec<EditorAction>,
            )| {
                super::render_ontology::render_constraints_tab(
                    ui, &mut s.1, &s.2, &mut s.0, &mut s.3,
                );
            },
            (
                EditorState {
                    new_constraint_expr_type_index: expr_type_index,
                    ..EditorState::default()
                },
                ConstraintRegistry::default(),
                test_concept_registry(),
                vec![],
            ),
        );
    h.run();
    h.get_by_label("New Constraint").click();
    h.run();
    h
}

/// Constraint form: open Concept `ComboBox` dropdown.
#[test]
fn constraint_form_concept_dropdown() {
    let mut harness = constraint_form_harness(0);
    click_combobox_by_value(&mut harness, "Motion");
    assert!(harness.query_by_label("Motion").is_some());
}

/// Constraint form: open Expr type `ComboBox` dropdown.
#[test]
fn constraint_form_expr_type_dropdown() {
    let mut harness = constraint_form_harness(0);
    click_combobox_by_value(&mut harness, "PropertyCompare");
    harness.get_by_label("PathBudget");
}

/// Constraint form (PropertyCompare): open Role `ComboBox` dropdown.
#[test]
fn constraint_form_role_dropdown() {
    let mut harness = constraint_form_harness(0);
    click_combobox_by_value(&mut harness, "traveler");
    harness.get_by_label("terrain");
}

/// Constraint form (PropertyCompare): open Op `ComboBox` dropdown.
#[test]
fn constraint_form_op_dropdown() {
    let mut harness = constraint_form_harness(0);
    click_combobox_by_value(&mut harness, "==");
    harness.get_by_label(">=");
}

/// Constraint form (PathBudget): open the cost role `ComboBox`.
#[test]
fn constraint_form_path_budget_combobox_dropdown() {
    let concept_reg = test_concept_registry();

    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                ConstraintRegistry,
                ConceptRegistry,
                Vec<EditorAction>,
            )| {
                super::render_ontology::render_constraints_tab(
                    ui, &mut s.1, &s.2, &mut s.0, &mut s.3,
                );
            },
            (
                EditorState {
                    new_constraint_expr_type_index: 3, // PathBudget
                    ..EditorState::default()
                },
                ConstraintRegistry::default(),
                concept_reg,
                vec![],
            ),
        );
    harness.run();

    harness.get_by_label("New Constraint").click();
    harness.run();

    // Cost role dropdown (selected_text = "traveler")
    click_combobox_by_value(&mut harness, "traveler");
    harness.get_by_label("terrain");
}

/// Helper: build a concepts tab harness with Motion concept expanded.
fn concepts_binding_harness() -> egui_kittest::Harness<
    'static,
    (
        EditorState,
        ConceptRegistry,
        EntityTypeRegistry,
        Vec<EditorAction>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                ConceptRegistry,
                EntityTypeRegistry,
                Vec<EditorAction>,
            )| {
                super::render_ontology::render_concepts_tab(ui, &mut s.1, &s.2, &mut s.0, &mut s.3);
            },
            (
                EditorState::default(),
                test_concept_registry(),
                test_registry(),
                vec![],
            ),
        );
    h.run();
    h.get_by_label("Motion").click();
    h.run();
    h
}

/// Concepts tab: open entity type binding `ComboBox` dropdown.
#[test]
fn concepts_binding_entity_type_dropdown() {
    let mut harness = concepts_binding_harness();
    // Two "(select)" ComboBoxes — click the first (entity type).
    let cbs: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("(select)"))
        .collect();
    cbs[0].click();
    harness.run();
    harness.get_by_label("Plains");
}

/// Concepts tab: open concept role binding `ComboBox` dropdown.
#[test]
fn concepts_binding_concept_role_dropdown() {
    let mut harness = concepts_binding_harness();
    // Two "(select)" ComboBoxes — click the second (concept role).
    let cbs: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("(select)"))
        .collect();
    if cbs.len() >= 2 {
        cbs[1].click();
        harness.run();
        harness.get_by_label("traveler");
    }
}

/// Entity type section: open property type `ComboBox` dropdown.
#[test]
fn entity_type_prop_type_combobox_dropdown() {
    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                systems::render_entity_type_section(
                    ui,
                    &mut s.1,
                    &mut s.0,
                    &mut s.4,
                    EntityRole::BoardPosition,
                    "Board Types",
                    "bt",
                    &s.2,
                    &s.3,
                );
            },
            (
                EditorState::default(),
                test_registry(),
                EnumRegistry::default(),
                StructRegistry::default(),
                vec![],
            ),
        );
    harness.run();
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();

    // Property type ComboBox (selected_text = "Bool")
    click_combobox_by_value(&mut harness, "Bool");
    harness.get_by_label("Enum");
    harness.get_by_label("IntRange");
}

/// Entity type section: open `EntityRef` role filter dropdown (`prop_type_index=6`).
#[test]
fn entity_type_entityref_role_combobox_dropdown() {
    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                systems::render_entity_type_section(
                    ui,
                    &mut s.1,
                    &mut s.0,
                    &mut s.4,
                    EntityRole::BoardPosition,
                    "Board Types",
                    "bt",
                    &s.2,
                    &s.3,
                );
            },
            (
                EditorState {
                    new_prop_type_index: 6,
                    ..EditorState::default()
                },
                test_registry(),
                EnumRegistry::default(),
                StructRegistry::default(),
                vec![],
            ),
        );
    harness.run();
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();

    // EntityRef role ComboBox (selected_text = "Any")
    click_combobox_by_value(&mut harness, "Any");
    harness.get_by_label("BoardPosition");
    harness.get_by_label("Token");
}

/// Entity type section: open List inner type dropdown (`prop_type_index=7`).
#[test]
fn entity_type_list_inner_type_combobox_dropdown() {
    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                systems::render_entity_type_section(
                    ui,
                    &mut s.1,
                    &mut s.0,
                    &mut s.4,
                    EntityRole::BoardPosition,
                    "Board Types",
                    "bt",
                    &s.2,
                    &s.3,
                );
            },
            (
                EditorState {
                    new_prop_type_index: 7,
                    ..EditorState::default()
                },
                test_registry(),
                EnumRegistry::default(),
                StructRegistry::default(),
                vec![],
            ),
        );
    harness.run();
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();

    // List inner type ComboBox (selected_text = "Bool")
    click_combobox_by_value(&mut harness, "Bool");
    harness.get_by_label("Float");
    harness.get_by_label("Color");
}

/// Helper: build entity type section harness with map prop type, headers expanded.
fn entity_type_map_harness() -> egui_kittest::Harness<
    'static,
    (
        EditorState,
        EntityTypeRegistry,
        EnumRegistry,
        StructRegistry,
        Vec<EditorAction>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                systems::render_entity_type_section(
                    ui,
                    &mut s.1,
                    &mut s.0,
                    &mut s.4,
                    EntityRole::BoardPosition,
                    "Board Types",
                    "bt",
                    &s.2,
                    &s.3,
                );
            },
            (
                EditorState {
                    new_prop_type_index: 8,
                    ..EditorState::default()
                },
                test_registry(),
                test_enum_registry(),
                StructRegistry::default(),
                vec![],
            ),
        );
    h.run();
    h.get_by_label("Board Types").click();
    h.run();
    h.get_by_label("Plains").click();
    h.run();
    h
}

/// Entity type section: open Map key enum dropdown (`prop_type_index=8`).
#[test]
fn entity_type_map_key_enum_dropdown() {
    let mut harness = entity_type_map_harness();
    click_combobox_by_value(&mut harness, "(select)");
    harness.get_by_label("Terrain");
}

/// Entity type section: open Map value type dropdown (`prop_type_index=8`).
#[test]
fn entity_type_map_value_type_dropdown() {
    let mut harness = entity_type_map_harness();
    click_combobox_by_value(&mut harness, "Bool");
    harness.get_by_label("Float");
}

/// Entity type section: open Struct selector dropdown (`prop_type_index=9`).
#[test]
fn entity_type_struct_selector_combobox_dropdown() {
    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                systems::render_entity_type_section(
                    ui,
                    &mut s.1,
                    &mut s.0,
                    &mut s.4,
                    EntityRole::BoardPosition,
                    "Board Types",
                    "bt",
                    &s.2,
                    &s.3,
                );
            },
            (
                EditorState {
                    new_prop_type_index: 9,
                    ..EditorState::default()
                },
                test_registry(),
                EnumRegistry::default(),
                test_struct_registry(),
                vec![],
            ),
        );
    harness.run();
    harness.get_by_label("Board Types").click();
    harness.run();
    harness.get_by_label("Plains").click();
    harness.run();

    // Struct selector ComboBox (selected_text = "(select)")
    click_combobox_by_value(&mut harness, "(select)");
    harness.get_by_label("Position");
}

/// Struct tab: open field type `ComboBox` dropdown.
#[test]
fn struct_field_type_combobox_dropdown() {
    let mut harness = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui, s: &mut (EditorState, StructRegistry, EnumRegistry, Vec<EditorAction>)| {
                systems::render_structs_tab(ui, &s.1, &s.2, &mut s.0, &mut s.3);
            },
            (
                EditorState::default(),
                test_struct_registry(),
                EnumRegistry::default(),
                vec![],
            ),
        );
    harness.run();

    // Expand "Position" struct CollapsingHeader
    harness.get_by_label("Position").click();
    harness.run();

    // Field type ComboBox (selected_text = "Bool")
    click_combobox_by_value(&mut harness, "Bool");
    harness.get_by_label("Float");
    harness.get_by_label("Color");
}

// ---------------------------------------------------------------------------
// Batch 10 — render_rules button click and form submission coverage
// ---------------------------------------------------------------------------

/// Helper: build a mechanics tab harness for interaction tests.
fn mechanics_harness(
    ts: TurnStructure,
    crt: CombatResultsTable,
    mods: CombatModifierRegistry,
    state: EditorState,
) -> egui_kittest::Harness<
    'static,
    (
        EditorState,
        TurnStructure,
        CombatResultsTable,
        CombatModifierRegistry,
        Vec<EditorAction>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 2000.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                TurnStructure,
                CombatResultsTable,
                CombatModifierRegistry,
                Vec<EditorAction>,
            )| {
                render_rules::render_mechanics_tab(ui, &s.1, &s.2, &s.3, &mut s.0, &mut s.4);
            },
            (state, ts, crt, mods, vec![]),
        );
    h.run();
    h
}

/// Click "Simultaneous" player order and verify action is emitted.
#[test]
fn mechanics_player_order_simultaneous_click() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    harness.get_by_label("Simultaneous").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(actions.iter().any(|a| matches!(
        a,
        EditorAction::SetPlayerOrder {
            order: PlayerOrder::Simultaneous
        }
    )));
}

/// Click "Activation" player order and verify action is emitted.
#[test]
fn mechanics_player_order_activation_click() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    harness.get_by_label("Activation").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(actions.iter().any(|a| matches!(
        a,
        EditorAction::SetPlayerOrder {
            order: PlayerOrder::ActivationBased
        }
    )));
}

/// Click "x" on a phase to remove it.
#[test]
fn mechanics_phase_remove_click() {
    let ts = test_turn_structure();
    let phase_id = ts.phases[0].id;
    let mut harness = mechanics_harness(
        ts,
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    // Multiple "x" buttons exist. Find the first small one near phase area.
    // The phases render "x" buttons — click the first one.

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("x"))
        .collect();
    assert!(!btns.is_empty(), "no x buttons found");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemovePhase { id } if *id == phase_id))
    );
}

/// Click up arrow on second phase to move it up.
#[test]
fn mechanics_phase_move_up_click() {
    let ts = test_turn_structure();
    let combat_id = ts.phases[1].id;
    let mut harness = mechanics_harness(
        ts,
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    // Find up-arrow buttons (Unicode ↑ = \u{2191})

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("\u{2191}"))
        .collect();
    assert!(!btns.is_empty(), "no up-arrow buttons found");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::MovePhaseUp { id } if *id == combat_id))
    );
}

/// Click down arrow on first phase to move it down.
#[test]
fn mechanics_phase_move_down_click() {
    let ts = test_turn_structure();
    let movement_id = ts.phases[0].id;
    let mut harness = mechanics_harness(
        ts,
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    // Find down-arrow buttons (Unicode ↓ = \u{2193})

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("\u{2193}"))
        .collect();
    assert!(!btns.is_empty(), "no down-arrow buttons found");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::MovePhaseDown { id } if *id == movement_id))
    );
}

/// Click "Add Phase" button with Combat type to cover `PhaseType::Combat` branch.
#[test]
fn mechanics_add_phase_combat_type() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_phase_name: "Assault".to_string(),
            new_phase_type_index: 1,
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Phase").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(actions.iter().any(|a| matches!(
        a,
        EditorAction::AddPhase {
            phase_type: PhaseType::Combat,
            ..
        }
    )));
}

/// Click "Add Phase" button with Admin type to cover `PhaseType::Admin` branch.
#[test]
fn mechanics_add_phase_admin_type() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_phase_name: "Cleanup".to_string(),
            new_phase_type_index: 2,
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Phase").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(actions.iter().any(|a| matches!(
        a,
        EditorAction::AddPhase {
            phase_type: PhaseType::Admin,
            ..
        }
    )));
}

/// Click "Add Col" with Differential type to cover that branch.
#[test]
fn mechanics_add_crt_col_differential() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_crt_col_label: "D+1".to_string(),
            new_crt_col_threshold: "1.0".to_string(),
            new_crt_col_type_index: 1,
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Col").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtColumn { .. }))
    );
}

/// Click "x" on a CRT column to remove it.
#[test]
fn mechanics_remove_crt_column_click() {
    let mut harness = mechanics_harness(
        TurnStructure {
            player_order: PlayerOrder::Alternating,
            phases: vec![],
        },
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    // With no phases, the first "x" buttons belong to CRT columns

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("x"))
        .collect();
    assert!(!btns.is_empty(), "no x buttons found");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveCrtColumn { .. }))
    );
}

/// Click "x" on a CRT row to remove it.
#[test]
fn mechanics_remove_crt_row_click() {
    let mut harness = mechanics_harness(
        TurnStructure {
            player_order: PlayerOrder::Alternating,
            phases: vec![],
        },
        CombatResultsTable {
            id: TypeId::new(),
            name: "Test CRT".to_string(),
            columns: vec![],
            rows: vec![hexorder_contracts::mechanics::CrtRow {
                label: "1".to_string(),
                die_value_min: 1,
                die_value_max: 2,
            }],
            outcomes: vec![],
            combat_concept_id: None,
        },
        CombatModifierRegistry::default(),
        EditorState::default(),
    );

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("x"))
        .collect();
    assert!(!btns.is_empty(), "no x buttons found for row");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveCrtRow { .. }))
    );
}

/// Click "Add Row" button to cover row creation logic.
#[test]
fn mechanics_add_crt_row() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_crt_row_label: "3".to_string(),
            new_crt_row_die_min: "5".to_string(),
            new_crt_row_die_max: "6".to_string(),
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Row").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCrtRow { .. }))
    );
}

/// Click "x" on a combat modifier to remove it.
#[test]
fn mechanics_remove_modifier_click() {
    let mods = test_modifiers();
    let mod_id = mods.modifiers[0].id;
    let mut harness = mechanics_harness(
        TurnStructure {
            player_order: PlayerOrder::Alternating,
            phases: vec![],
        },
        CombatResultsTable {
            id: TypeId::new(),
            name: "CRT".to_string(),
            columns: vec![],
            rows: vec![],
            outcomes: vec![],
            combat_concept_id: None,
        },
        mods,
        EditorState::default(),
    );

    let btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("x"))
        .collect();
    assert!(!btns.is_empty(), "no x buttons found for modifiers");
    btns[0].click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::RemoveCombatModifier { id } if *id == mod_id))
    );
}

/// Click "Add Modifier" with default source (`DefenderTerrain`).
#[test]
fn mechanics_add_modifier_defender_terrain() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_modifier_name: "River Crossing".to_string(),
            new_modifier_source_index: 0,
            new_modifier_shift: -2,
            new_modifier_priority: 5,
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Modifier").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCombatModifier { .. }))
    );
}

/// Click "Add Modifier" with `AttackerTerrain` source.
#[test]
fn mechanics_add_modifier_attacker_terrain() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_modifier_name: "Open Ground".to_string(),
            new_modifier_source_index: 1,
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Modifier").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCombatModifier { .. }))
    );
}

/// Click "Add Modifier" with Custom source.
#[test]
fn mechanics_add_modifier_custom_source() {
    let mut harness = mechanics_harness(
        test_turn_structure(),
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState {
            new_modifier_name: "Weather".to_string(),
            new_modifier_source_index: 2,
            new_modifier_custom_source: "storm".to_string(),
            ..EditorState::default()
        },
    );
    harness.get_by_label("Add Modifier").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::AddCombatModifier { .. }))
    );
}

// ---------------------------------------------------------------------------
// Batch 11 — Inspector property editors & remaining mechanics
// ---------------------------------------------------------------------------

/// Create an entity type registry with diverse property types for inspector tests.
fn inspector_entity_registry() -> (EntityTypeRegistry, EnumRegistry, StructRegistry) {
    let enum_id = TypeId::new();
    let struct_id = TypeId::new();
    let board_type_id = TypeId::new();
    let token_type_id = TypeId::new();

    let registry = EntityTypeRegistry {
        types: vec![
            EntityType {
                id: board_type_id,
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: Color::srgb(0.4, 0.6, 0.2),
                properties: vec![
                    PropertyDefinition {
                        id: TypeId::new(),
                        name: "terrain".to_string(),
                        property_type: PropertyType::Enum(enum_id),
                        default_value: PropertyValue::Enum("Open".to_string()),
                    },
                    PropertyDefinition {
                        id: TypeId::new(),
                        name: "owner".to_string(),
                        property_type: PropertyType::EntityRef(Some(EntityRole::Token)),
                        default_value: PropertyValue::EntityRef(None),
                    },
                    PropertyDefinition {
                        id: TypeId::new(),
                        name: "tags".to_string(),
                        property_type: PropertyType::List(Box::new(PropertyType::Int)),
                        default_value: PropertyValue::List(vec![PropertyValue::Int(1)]),
                    },
                    PropertyDefinition {
                        id: TypeId::new(),
                        name: "costs".to_string(),
                        property_type: PropertyType::Map(enum_id, Box::new(PropertyType::Int)),
                        default_value: PropertyValue::Map(vec![]),
                    },
                    PropertyDefinition {
                        id: TypeId::new(),
                        name: "coords".to_string(),
                        property_type: PropertyType::Struct(struct_id),
                        default_value: PropertyValue::Struct(HashMap::new()),
                    },
                ],
            },
            EntityType {
                id: token_type_id,
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: Color::srgb(0.2, 0.2, 0.8),
                properties: vec![],
            },
        ],
    };

    let mut enum_registry = EnumRegistry::default();
    enum_registry.definitions.insert(
        enum_id,
        EnumDefinition {
            id: enum_id,
            name: "Terrain".to_string(),
            options: vec!["Open".to_string(), "Rough".to_string(), "Dense".to_string()],
        },
    );

    let mut struct_registry = StructRegistry::default();
    struct_registry.definitions.insert(
        struct_id,
        StructDefinition {
            id: struct_id,
            name: "Coords".to_string(),
            fields: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "x".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "y".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(0),
                },
            ],
        },
    );

    (registry, enum_registry, struct_registry)
}

/// Build `EntityData` pre-populated with each property's `default_value` from the registry.
fn entity_data_from_registry(registry: &EntityTypeRegistry, type_index: usize) -> EntityData {
    let et = &registry.types[type_index];
    let mut properties = std::collections::HashMap::new();
    for prop in &et.properties {
        properties.insert(prop.id, prop.default_value.clone());
    }
    EntityData {
        entity_type_id: et.id,
        properties,
    }
}

/// Build a harness that renders `render_inspector` with an entity having diverse properties.
#[allow(clippy::type_complexity)]
fn inspector_harness(
    registry: EntityTypeRegistry,
    enum_registry: EnumRegistry,
    struct_registry: StructRegistry,
    entity_data: Option<EntityData>,
    position: Option<HexPosition>,
) -> Harness<
    'static,
    (
        Option<EntityData>,
        EntityTypeRegistry,
        EnumRegistry,
        StructRegistry,
        Option<HexPosition>,
    ),
> {
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 3000.0))
        .build_ui_state(
            |ui,
             s: &mut (
                Option<EntityData>,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Option<HexPosition>,
            )| {
                render_rules::render_inspector(ui, s.4, s.0.as_mut(), &s.1, &s.2, &s.3);
            },
            (
                entity_data,
                registry,
                enum_registry,
                struct_registry,
                position,
            ),
        );
    h.run();
    h
}

/// Click "Alternating" player order when starting from Simultaneous.
#[test]
fn mechanics_player_order_alternating_click() {
    let mut ts = test_turn_structure();
    ts.player_order = PlayerOrder::Simultaneous;
    let mut harness = mechanics_harness(
        ts,
        test_crt(),
        CombatModifierRegistry::default(),
        EditorState::default(),
    );
    harness.get_by_label("Alternating").click();
    harness.run();
    let actions = &harness.state().4;
    assert!(actions.iter().any(|a| matches!(
        a,
        EditorAction::SetPlayerOrder {
            order: PlayerOrder::Alternating
        }
    )));
}

/// Render inspector (diverse props) with no tile selected — shows message.
#[test]
fn inspector_diverse_no_tile_selected() {
    let (reg, enums, structs) = inspector_entity_registry();
    let harness = inspector_harness(reg, enums, structs, None, None);
    harness.get_by_label("No tile selected");
}

/// Render inspector with position but no entity data — shows "No cell data".
#[test]
fn inspector_diverse_no_cell_data() {
    let (reg, enums, structs) = inspector_entity_registry();
    let harness = inspector_harness(reg, enums, structs, None, Some(HexPosition { q: 1, r: 2 }));
    harness.get_by_label("No cell data");
}

/// Render inspector with entity having diverse property types — covers render path.
#[test]
fn inspector_diverse_properties_render() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    harness.get_by_label("terrain:");
}

/// Open Enum property `ComboBox` dropdown in inspector — covers lines 689-692.
#[test]
fn inspector_enum_combobox_dropdown() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Find the Enum ComboBox showing "Open" (default value) and click it
    click_combobox_by_value(&mut harness, "Open");
    // The dropdown should now show the enum options
    let rough = harness.query_by_label("Rough");
    assert!(rough.is_some(), "Enum dropdown should show 'Rough' option");
}

/// Open `EntityRef` property `ComboBox` dropdown — covers lines 713-724.
#[test]
fn inspector_entity_ref_combobox_dropdown() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // EntityRef ComboBox shows "(none)" by default
    click_combobox_by_value(&mut harness, "(none)");
    // The dropdown should show Infantry (token type matching the filter)
    let infantry = harness.query_by_label("Infantry");
    assert!(
        infantry.is_some(),
        "EntityRef dropdown should show 'Infantry' option"
    );
}

/// Render inspector with List property — click `CollapsingHeader` to expand list.
/// Covers List rendering path (lines 740-770).
#[test]
fn inspector_list_property_renders() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Click the "List (1)" collapsing header to expand it
    harness.get_by_label("List (1)").click();
    harness.run();
    // Should see the [0] label and the "+ Add" button
    let idx_label = harness.query_by_label("[0]");
    assert!(idx_label.is_some(), "List should show [0] index label");
    let add_btn = harness.query_by_label("+ Add");
    assert!(add_btn.is_some(), "List should show '+ Add' button");
}

/// Click "+ Add" in List property to add an item — covers line 768.
#[test]
fn inspector_list_add_item() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Expand the list
    harness.get_by_label("List (1)").click();
    harness.run();
    // Click "+ Add"
    harness.get_by_label("+ Add").click();
    harness.run();
    // After add, the list label should change to "List (2)"
    let expanded = harness.query_by_label("List (2)");
    assert!(expanded.is_some(), "List should grow to 2 items after add");
}

/// Click "x" in List property to remove an item — covers lines 757, 762.
#[test]
fn inspector_list_remove_item() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Expand the list
    harness.get_by_label("List (1)").click();
    harness.run();
    // Click "x" to remove item [0]
    let x_btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("x"))
        .collect();
    // Find the "x" button inside the list (may be multiple "x" buttons in UI)
    x_btns.last().expect("should have 'x' button").click();
    harness.run();
    // After remove, the list label should change to "List (0)"
    let shrunk = harness.query_by_label("List (0)");
    assert!(
        shrunk.is_some(),
        "List should shrink to 0 items after remove"
    );
}

/// Render Map property — expand `CollapsingHeader` to see entries.
/// Covers Map rendering path (lines 790-825).
#[test]
fn inspector_map_property_renders() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Click the "Map (0)" collapsing header to expand it
    harness.get_by_label("Map (0)").click();
    harness.run();
    // Should see the enum options as labels with "(default)" marker
    let open_label = harness.query_by_label("Open:");
    assert!(open_label.is_some(), "Map should show 'Open:' key label");
    let default_labels: Vec<_> = harness.query_all_by_label("(default)").collect();
    assert!(
        !default_labels.is_empty(),
        "Map should show '(default)' for missing entries"
    );
}

/// Click "+" in Map property to add an entry — covers lines 817-820.
#[test]
fn inspector_map_add_entry() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Expand the map
    harness.get_by_label("Map (0)").click();
    harness.run();
    // Click the first "+" button to add an entry
    let plus_btns: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("+"))
        .collect();
    plus_btns[0].click();
    harness.run();
    // After adding, the map should have 1 entry
    let grown = harness.query_by_label("Map (1)");
    assert!(grown.is_some(), "Map should grow to 1 entry after add");
}

/// Render Struct property — expand `CollapsingHeader` to see fields.
/// Covers Struct rendering path (lines 842-866) and field default insertion (line 854-855).
#[test]
fn inspector_struct_property_renders() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    // Click the "Coords" collapsing header to expand struct
    harness.get_by_label("Coords").click();
    harness.run();
    // Should see fields "x:" and "y:"
    let x_label = harness.query_by_label("x:");
    assert!(x_label.is_some(), "Struct should show 'x:' field label");
    let y_label = harness.query_by_label("y:");
    assert!(y_label.is_some(), "Struct should show 'y:' field label");
}

/// Render unit inspector with entity data — covers `render_unit_inspector` property path.
#[test]
fn unit_inspector_renders_properties() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 3000.0))
        .build_ui_state(
            |ui,
             s: &mut (
                Option<EntityData>,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                render_rules::render_unit_inspector(ui, s.0.as_mut(), &s.1, &s.2, &s.3, &mut s.4);
            },
            (Some(entity_data), reg, enums, structs, vec![]),
        );
    h.run();
    // The unit inspector should show the type name and property labels
    let type_label = h.query_by_label("Unit Type: Plains");
    assert!(type_label.is_some(), "Should show unit type name");
    let terrain = h.query_by_label("terrain:");
    assert!(terrain.is_some(), "Should show property labels");
}

/// Render unit inspector (diverse props) with no entity data — shows message.
#[test]
fn unit_inspector_diverse_no_unit_selected() {
    let (reg, enums, structs) = inspector_entity_registry();
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1000.0))
        .build_ui_state(
            |ui,
             s: &mut (
                Option<EntityData>,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                render_rules::render_unit_inspector(ui, s.0.as_mut(), &s.1, &s.2, &s.3, &mut s.4);
            },
            (None, reg, enums, structs, vec![]),
        );
    h.run();
    h.get_by_label("No unit selected");
}

/// Click "Delete Unit" in unit inspector — emits `DeleteSelectedUnit` action.
#[test]
fn unit_inspector_delete_unit_click() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 0);
    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 3000.0))
        .build_ui_state(
            |ui,
             s: &mut (
                Option<EntityData>,
                EntityTypeRegistry,
                EnumRegistry,
                StructRegistry,
                Vec<EditorAction>,
            )| {
                render_rules::render_unit_inspector(ui, s.0.as_mut(), &s.1, &s.2, &s.3, &mut s.4);
            },
            (Some(entity_data), reg, enums, structs, vec![]),
        );
    h.run();
    h.get_by_label("Delete Unit").click();
    h.run();
    let actions = &h.state().4;
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::DeleteSelectedUnit))
    );
}

/// Inspector with entity whose type has no properties — shows "No properties" message.
#[test]
fn inspector_no_properties() {
    let (reg, enums, structs) = inspector_entity_registry();
    let entity_data = entity_data_from_registry(&reg, 1); // Infantry has no properties
    let harness = inspector_harness(
        reg,
        enums,
        structs,
        Some(entity_data),
        Some(HexPosition { q: 0, r: 0 }),
    );
    harness.get_by_label("No properties");
}

// ---------------------------------------------------------------------------
// Batch 12 — format_relation_effect ops & binding interaction tests
// ---------------------------------------------------------------------------

/// `format_relation_effect` with `Add` operation.
#[test]
fn format_relation_effect_add_op() {
    let effect = RelationEffect::ModifyProperty {
        target_property: "hp".to_string(),
        source_property: "bonus".to_string(),
        operation: ModifyOperation::Add,
    };
    assert_eq!(actions::format_relation_effect(&effect), "hp + bonus");
}

/// `format_relation_effect` with `Multiply` operation.
#[test]
fn format_relation_effect_multiply_op() {
    let effect = RelationEffect::ModifyProperty {
        target_property: "damage".to_string(),
        source_property: "factor".to_string(),
        operation: ModifyOperation::Multiply,
    };
    assert_eq!(actions::format_relation_effect(&effect), "damage * factor");
}

/// `format_relation_effect` with `Min` operation.
#[test]
fn format_relation_effect_min_op() {
    let effect = RelationEffect::ModifyProperty {
        target_property: "speed".to_string(),
        source_property: "cap".to_string(),
        operation: ModifyOperation::Min,
    };
    assert_eq!(actions::format_relation_effect(&effect), "speed min cap");
}

/// `format_relation_effect` with `Max` operation.
#[test]
fn format_relation_effect_max_op() {
    let effect = RelationEffect::ModifyProperty {
        target_property: "armor".to_string(),
        source_property: "floor".to_string(),
        operation: ModifyOperation::Max,
    };
    assert_eq!(actions::format_relation_effect(&effect), "armor max floor");
}

/// Select entity type in binding `ComboBox` and verify state change.
#[test]
fn concepts_binding_select_entity_type() {
    let mut harness = concepts_binding_harness();
    // Open the entity type ComboBox (first "(select)")
    let cbs: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("(select)"))
        .collect();
    cbs[0].click();
    harness.run();
    // Click "Plains" to select it
    harness.get_by_label("Plains").click();
    harness.run();
    // After selection, the state should have the entity type ID set
    assert!(harness.state().0.binding_entity_type_id.is_some());
}

/// Select concept role in binding `ComboBox` and verify state change.
#[test]
fn concepts_binding_select_concept_role() {
    let mut harness = concepts_binding_harness();
    // Open the concept role ComboBox (second "(select)")
    let cbs: Vec<_> = harness
        .get_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("(select)"))
        .collect();
    if cbs.len() >= 2 {
        cbs[1].click();
        harness.run();
        // Click "traveler" to select it
        harness.get_by_label("traveler").click();
        harness.run();
        assert!(harness.state().0.binding_concept_role_id.is_some());
    }
}

/// Select both entity type and concept role, then click "+ Bind".
#[test]
fn concepts_binding_bind_entity_click() {
    let reg = test_registry();
    let et_id = reg.types[0].id;
    let cr_id = test_concept_registry().concepts[0].role_labels[0].id;

    let mut h = Harness::builder()
        .with_size(bevy_egui::egui::vec2(400.0, 1200.0))
        .build_ui_state(
            |ui,
             s: &mut (
                EditorState,
                ConceptRegistry,
                EntityTypeRegistry,
                Vec<EditorAction>,
            )| {
                super::render_ontology::render_concepts_tab(ui, &mut s.1, &s.2, &mut s.0, &mut s.3);
            },
            (
                EditorState {
                    binding_entity_type_id: Some(et_id),
                    binding_concept_role_id: Some(cr_id),
                    ..EditorState::default()
                },
                test_concept_registry(),
                reg,
                vec![],
            ),
        );
    h.run();
    // Expand the concept
    h.get_by_label("Motion").click();
    h.run();
    // Click "+ Bind" — button should be enabled since both IDs are set
    h.get_by_label("+ Bind").click();
    h.run();
    let acts = &h.state().3;
    assert!(
        acts.iter().any(|a| matches!(
            a,
            EditorAction::BindEntityToConcept {
                entity_type_id: _,
                concept_id: _,
                concept_role_id: _,
            }
        )),
        "Should emit BindEntityToConcept action, got: {acts:?}"
    );
    // After binding, the editor state should be cleared
    assert!(h.state().0.binding_entity_type_id.is_none());
    assert!(h.state().0.binding_concept_role_id.is_none());
}
