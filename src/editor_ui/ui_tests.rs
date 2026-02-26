//! UI interaction tests for the `editor_ui` plugin.
//!
//! Uses `egui_kittest` to test render functions in isolation.
//! Each test creates a minimal `Harness` with the relevant state and
//! verifies that the rendered UI contains the expected labels, that
//! buttons produce the correct `EditorAction`s, and that disabled
//! states are handled correctly.

use bevy::prelude::*;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use hexorder_contracts::editor_ui::EditorTool;
use hexorder_contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityRole, EntityType, EntityTypeRegistry, EnumDefinition,
    EnumRegistry, GameSystem, PropertyDefinition, PropertyType, PropertyValue, StructDefinition,
    StructRegistry, TypeId,
};
use hexorder_contracts::mechanics::{
    CombatModifierDefinition, CombatModifierRegistry, CombatOutcome, CombatResultsTable, CrtColumn,
    CrtColumnType, CrtRow, ModifierSource, Phase, PhaseType, PlayerOrder, TurnStructure,
};
use hexorder_contracts::ontology::{
    CompareOp, Concept, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, Relation, RelationEffect, RelationRegistry,
    RelationTrigger,
};
use hexorder_contracts::persistence::Workspace;
use hexorder_contracts::validation::{SchemaError, SchemaErrorCategory, SchemaValidation};

use super::components::{EditorAction, EditorState};
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
    let registry = test_registry();
    let active = ActiveBoardType::default();

    struct CellPaletteState {
        registry: EntityTypeRegistry,
        active: ActiveBoardType,
    }

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
    let registry = test_registry();
    let active = ActiveTokenType::default();

    struct UnitPaletteState {
        registry: EntityTypeRegistry,
        active: ActiveTokenType,
    }

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
    let enum_registry = EnumRegistry::default();
    let state = EditorState {
        new_enum_name: "Weather".to_string(),
        ..EditorState::default()
    };
    let actions: Vec<EditorAction> = Vec::new();

    struct EnumsState {
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let state = EnumsState {
        enum_registry,
        editor_state: state,
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
    let struct_registry = StructRegistry::default();
    let enum_registry = EnumRegistry::default();
    let state = EditorState {
        new_struct_name: "Coordinate".to_string(),
        ..EditorState::default()
    };
    let actions: Vec<EditorAction> = Vec::new();

    struct StructsState {
        struct_registry: StructRegistry,
        enum_registry: EnumRegistry,
        editor_state: EditorState,
        actions: Vec<EditorAction>,
    }

    let state = StructsState {
        struct_registry,
        enum_registry,
        editor_state: state,
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
