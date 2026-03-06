//! Deferred action application and helper functions.

use bevy::prelude::*;
use bevy_egui::egui;

use hexorder_contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityType, EntityTypeRegistry,
    EnumDefinition, EnumRegistry, PropertyDefinition, PropertyType, PropertyValue, SelectedUnit,
    StructDefinition, StructRegistry, TypeId, UnitInstance,
};
use hexorder_contracts::map_gen::GenerateMap;
use hexorder_contracts::mechanic_reference::{MechanicCatalog, ScaffoldAction};
use hexorder_contracts::mechanics::{
    CombatModifierDefinition, CombatModifierRegistry, CombatOutcome, CombatResultsTable,
    ModifierSource, Phase, PhaseType, TurnStructure,
};
use hexorder_contracts::ontology::{
    CompareOp, ConceptBinding, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, Relation, RelationEffect, RelationRegistry,
};
use hexorder_contracts::simulation::{ColumnType, TableColumn, TableRow};

use super::components::{BrandTheme, EditorAction, EditorState};

// ---------------------------------------------------------------------------
// Deferred Action Application
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_actions(
    actions: Vec<EditorAction>,
    registry: &mut EntityTypeRegistry,
    enum_registry: &mut EnumRegistry,
    struct_registry: &mut StructRegistry,
    tile_data_query: &mut Query<&mut EntityData, Without<UnitInstance>>,
    active_board: &mut ActiveBoardType,
    active_token: &mut ActiveTokenType,
    selected_unit: &mut SelectedUnit,
    editor_state: &EditorState,
    commands: &mut Commands,
    concept_registry: &mut ConceptRegistry,
    relation_registry: &mut RelationRegistry,
    constraint_registry: &mut ConstraintRegistry,
    turn_structure: &mut TurnStructure,
    combat_results_table: &mut CombatResultsTable,
    combat_modifiers: &mut CombatModifierRegistry,
    mechanic_catalog: &MechanicCatalog,
) {
    for action in actions {
        match action {
            EditorAction::CreateEntityType { name, role, color } => {
                registry.types.push(EntityType {
                    id: TypeId::new(),
                    name,
                    role,
                    color,
                    properties: Vec::new(),
                });
            }
            EditorAction::DeleteEntityType { id } => {
                // Determine the role of the type being deleted.
                let role = registry.get(id).map(|et| et.role);

                match role {
                    Some(EntityRole::BoardPosition) => {
                        // Find a fallback BoardPosition type.
                        let fallback_id = registry
                            .types_by_role(EntityRole::BoardPosition)
                            .iter()
                            .find(|et| et.id != id)
                            .map(|et| et.id);
                        if let Some(fallback) = fallback_id {
                            for mut ed in tile_data_query.iter_mut() {
                                if ed.entity_type_id == id {
                                    ed.entity_type_id = fallback;
                                    ed.properties.clear();
                                }
                            }
                            if active_board.entity_type_id == Some(id) {
                                active_board.entity_type_id = Some(fallback);
                            }
                        }
                    }
                    Some(EntityRole::Token) => {
                        let fallback_id = registry
                            .types_by_role(EntityRole::Token)
                            .iter()
                            .find(|et| et.id != id)
                            .map(|et| et.id);
                        if let Some(fallback) = fallback_id
                            && active_token.entity_type_id == Some(id)
                        {
                            active_token.entity_type_id = Some(fallback);
                        }
                    }
                    None => {}
                }

                registry.types.retain(|et| et.id != id);
            }
            EditorAction::AddProperty {
                type_id,
                name,
                prop_type,
                enum_options,
            } => {
                let final_type = match &prop_type {
                    PropertyType::Enum(_) => {
                        let enum_id = TypeId::new();
                        let options: Vec<String> = enum_options
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        enum_registry.insert(EnumDefinition {
                            id: enum_id,
                            name: name.clone(),
                            options,
                        });
                        PropertyType::Enum(enum_id)
                    }
                    PropertyType::EntityRef(_) => {
                        let role = match editor_state.new_prop_entity_ref_role {
                            1 => Some(EntityRole::BoardPosition),
                            2 => Some(EntityRole::Token),
                            _ => None,
                        };
                        PropertyType::EntityRef(role)
                    }
                    PropertyType::List(_) => {
                        let inner = match editor_state.new_prop_list_inner_type {
                            1 => PropertyType::Int,
                            2 => PropertyType::Float,
                            3 => PropertyType::String,
                            4 => PropertyType::Color,
                            _ => PropertyType::Bool,
                        };
                        PropertyType::List(Box::new(inner))
                    }
                    PropertyType::Map(_, _) => {
                        let enum_id = editor_state.new_prop_map_enum_id.unwrap_or_default();
                        let val_type = match editor_state.new_prop_map_value_type {
                            1 => PropertyType::Int,
                            2 => PropertyType::Float,
                            3 => PropertyType::String,
                            4 => PropertyType::Color,
                            _ => PropertyType::Bool,
                        };
                        PropertyType::Map(enum_id, Box::new(val_type))
                    }
                    PropertyType::Struct(_) => {
                        let sid = editor_state.new_prop_struct_id.unwrap_or_default();
                        PropertyType::Struct(sid)
                    }
                    PropertyType::IntRange { .. } => PropertyType::IntRange {
                        min: editor_state.new_prop_int_range_min,
                        max: editor_state.new_prop_int_range_max,
                    },
                    PropertyType::FloatRange { .. } => PropertyType::FloatRange {
                        min: editor_state.new_prop_float_range_min,
                        max: editor_state.new_prop_float_range_max,
                    },
                    other => other.clone(),
                };

                let default_value = PropertyValue::default_for(&final_type);
                if let Some(et) = registry.types.iter_mut().find(|et| et.id == type_id) {
                    et.properties.push(PropertyDefinition {
                        id: TypeId::new(),
                        name,
                        property_type: final_type,
                        default_value,
                    });
                }
            }
            EditorAction::RemoveProperty { type_id, prop_id } => {
                // Determine role to know which query to clean up.
                let role = registry.get(type_id).map(|et| et.role);

                if let Some(et) = registry.types.iter_mut().find(|et| et.id == type_id) {
                    et.properties.retain(|p| p.id != prop_id);
                }

                if role == Some(EntityRole::BoardPosition) {
                    for mut ed in tile_data_query.iter_mut() {
                        if ed.entity_type_id == type_id {
                            ed.properties.remove(&prop_id);
                        }
                    }
                }
                // Token and unknown roles: unit_data_query is not passed to
                // apply_actions; units with removed properties get defaults on
                // next inspector render (consistent with 0.3.0 behavior).
            }
            EditorAction::DeleteSelectedUnit => {
                if let Some(entity) = selected_unit.entity {
                    commands.entity(entity).despawn();
                    selected_unit.entity = None;
                }
            }
            EditorAction::CreateConcept { name, description } => {
                concept_registry
                    .concepts
                    .push(hexorder_contracts::ontology::Concept {
                        id: TypeId::new(),
                        name,
                        description,
                        role_labels: Vec::new(),
                    });
            }
            EditorAction::DeleteConcept { id } => {
                concept_registry.concepts.retain(|c| c.id != id);
                concept_registry.bindings.retain(|b| b.concept_id != id);
                relation_registry.relations.retain(|r| r.concept_id != id);
                constraint_registry
                    .constraints
                    .retain(|c| c.concept_id != id);
            }
            EditorAction::AddConceptRole {
                concept_id,
                name,
                allowed_roles,
            } => {
                if let Some(concept) = concept_registry
                    .concepts
                    .iter_mut()
                    .find(|c| c.id == concept_id)
                {
                    concept.role_labels.push(ConceptRole {
                        id: TypeId::new(),
                        name,
                        allowed_entity_roles: allowed_roles,
                    });
                }
            }
            EditorAction::RemoveConceptRole {
                concept_id,
                role_id,
            } => {
                if let Some(concept) = concept_registry
                    .concepts
                    .iter_mut()
                    .find(|c| c.id == concept_id)
                {
                    concept.role_labels.retain(|r| r.id != role_id);
                }
                concept_registry
                    .bindings
                    .retain(|b| !(b.concept_id == concept_id && b.concept_role_id == role_id));
            }
            EditorAction::BindEntityToConcept {
                entity_type_id,
                concept_id,
                concept_role_id,
            } => {
                concept_registry.bindings.push(ConceptBinding {
                    id: TypeId::new(),
                    entity_type_id,
                    concept_id,
                    concept_role_id,
                    property_bindings: Vec::new(),
                });
            }
            EditorAction::UnbindEntityFromConcept {
                concept_id: _,
                binding_id,
            } => {
                concept_registry.bindings.retain(|b| b.id != binding_id);
            }
            EditorAction::CreateRelation {
                name,
                concept_id,
                subject_role_id,
                object_role_id,
                trigger,
                effect,
            } => {
                relation_registry.relations.push(Relation {
                    id: TypeId::new(),
                    name,
                    concept_id,
                    subject_role_id,
                    object_role_id,
                    trigger,
                    effect,
                });
            }
            EditorAction::DeleteRelation { id } => {
                relation_registry.relations.retain(|r| r.id != id);
                constraint_registry
                    .constraints
                    .retain(|c| c.relation_id != Some(id));
            }
            EditorAction::CreateConstraint {
                name,
                description,
                concept_id,
                expression,
            } => {
                constraint_registry.constraints.push(Constraint {
                    id: TypeId::new(),
                    name,
                    description,
                    concept_id,
                    relation_id: None,
                    expression,
                    auto_generated: false,
                });
            }
            EditorAction::DeleteConstraint { id } => {
                constraint_registry.constraints.retain(|c| c.id != id);
            }
            EditorAction::CreateEnum { name, options } => {
                enum_registry.insert(EnumDefinition {
                    id: TypeId::new(),
                    name,
                    options,
                });
            }
            EditorAction::DeleteEnum { id } => {
                enum_registry.remove(id);
            }
            EditorAction::AddEnumOption { enum_id, option } => {
                if let Some(def) = enum_registry.get_mut(enum_id) {
                    def.options.push(option);
                }
            }
            EditorAction::RemoveEnumOption { enum_id, option } => {
                if let Some(def) = enum_registry.get_mut(enum_id) {
                    def.options.retain(|o| o != &option);
                }
            }
            EditorAction::CreateStruct { name } => {
                struct_registry.insert(StructDefinition {
                    id: TypeId::new(),
                    name,
                    fields: Vec::new(),
                });
            }
            EditorAction::DeleteStruct { id } => {
                struct_registry.remove(id);
            }
            EditorAction::AddStructField {
                struct_id,
                name,
                prop_type,
            } => {
                if let Some(def) = struct_registry.get_mut(struct_id) {
                    let default_value = PropertyValue::default_for(&prop_type);
                    def.fields.push(PropertyDefinition {
                        id: TypeId::new(),
                        name,
                        property_type: prop_type,
                        default_value,
                    });
                }
            }
            EditorAction::RemoveStructField {
                struct_id,
                field_id,
            } => {
                if let Some(def) = struct_registry.get_mut(struct_id) {
                    def.fields.retain(|f| f.id != field_id);
                }
            }
            // -- Mechanics actions --
            EditorAction::SetPlayerOrder { order } => {
                turn_structure.player_order = order;
            }
            EditorAction::AddPhase { name, phase_type } => {
                turn_structure.phases.push(Phase {
                    id: TypeId::new(),
                    name,
                    phase_type,
                    description: String::new(),
                });
            }
            EditorAction::RemovePhase { id } => {
                turn_structure.phases.retain(|p| p.id != id);
            }
            EditorAction::MovePhaseUp { id } => {
                if let Some(idx) = turn_structure.phases.iter().position(|p| p.id == id)
                    && idx > 0
                {
                    turn_structure.phases.swap(idx, idx - 1);
                }
            }
            EditorAction::MovePhaseDown { id } => {
                if let Some(idx) = turn_structure.phases.iter().position(|p| p.id == id)
                    && idx + 1 < turn_structure.phases.len()
                {
                    turn_structure.phases.swap(idx, idx + 1);
                }
            }
            EditorAction::AddCrtColumn {
                label,
                column_type,
                threshold,
            } => {
                combat_results_table.table.columns.push(TableColumn {
                    label,
                    column_type,
                    threshold,
                });
                // Extend each existing row with a default outcome.
                for row_outcomes in &mut combat_results_table.outcomes {
                    row_outcomes.push(CombatOutcome {
                        label: "--".to_string(),
                        effect: None,
                    });
                }
            }
            EditorAction::RemoveCrtColumn { index } => {
                if index < combat_results_table.table.columns.len() {
                    combat_results_table.table.columns.remove(index);
                    for row_outcomes in &mut combat_results_table.outcomes {
                        if index < row_outcomes.len() {
                            row_outcomes.remove(index);
                        }
                    }
                }
            }
            EditorAction::AddCrtRow {
                label,
                die_min,
                die_max,
            } => {
                combat_results_table.table.rows.push(TableRow {
                    label,
                    value_min: die_min,
                    value_max: die_max,
                });
                // Add a row of default outcomes.
                let num_cols = combat_results_table.table.columns.len();
                combat_results_table.outcomes.push(
                    (0..num_cols)
                        .map(|_| CombatOutcome {
                            label: "--".to_string(),
                            effect: None,
                        })
                        .collect(),
                );
            }
            EditorAction::RemoveCrtRow { index } => {
                if index < combat_results_table.table.rows.len() {
                    combat_results_table.table.rows.remove(index);
                    if index < combat_results_table.outcomes.len() {
                        combat_results_table.outcomes.remove(index);
                    }
                }
            }
            EditorAction::SetCrtOutcome { row, col, label } => {
                if let Some(row_outcomes) = combat_results_table.outcomes.get_mut(row)
                    && let Some(outcome) = row_outcomes.get_mut(col)
                {
                    outcome.label = label;
                }
            }
            EditorAction::AddCombatModifier {
                name,
                source,
                shift,
                priority,
            } => {
                combat_modifiers.modifiers.push(CombatModifierDefinition {
                    id: TypeId::new(),
                    name,
                    source,
                    column_shift: shift,
                    priority,
                    cap: None,
                    terrain_type_filter: None,
                });
            }
            EditorAction::RemoveCombatModifier { id } => {
                combat_modifiers.modifiers.retain(|m| m.id != id);
            }
            // -- Mechanic Reference --
            EditorAction::ApplyTemplate { template_id } => {
                if let Some(recipe) = mechanic_catalog.get_template(&template_id) {
                    apply_scaffold_recipe(
                        &recipe,
                        registry,
                        enum_registry,
                        turn_structure,
                        combat_results_table,
                        combat_modifiers,
                    );
                }
            }
            // -- Map Generation --
            EditorAction::GenerateMap => {
                commands.insert_resource(GenerateMap);
            }
        }
    }

    // Suppress unused warning.
    let _ = editor_state;
}

/// Converts a `ScaffoldRecipe` into concrete registry mutations.
///
/// String-based scaffold actions are resolved to typed values here so that
/// the `mechanic_reference` contract stays decoupled from `game_system` types.
pub(super) fn apply_scaffold_recipe(
    recipe: &hexorder_contracts::mechanic_reference::ScaffoldRecipe,
    registry: &mut EntityTypeRegistry,
    enum_registry: &mut EnumRegistry,
    turn_structure: &mut TurnStructure,
    combat_results_table: &mut CombatResultsTable,
    combat_modifiers: &mut CombatModifierRegistry,
) {
    for action in &recipe.actions {
        match action {
            ScaffoldAction::CreateEntityType { name, role, color } => {
                let entity_role = match role.as_str() {
                    "Token" => EntityRole::Token,
                    _ => EntityRole::BoardPosition,
                };
                registry.types.push(EntityType {
                    id: TypeId::new(),
                    name: name.clone(),
                    role: entity_role,
                    color: Color::srgb(color[0], color[1], color[2]),
                    properties: Vec::new(),
                });
            }
            ScaffoldAction::AddProperty {
                entity_name,
                prop_name,
                prop_type,
            } => {
                let property_type = parse_scaffold_prop_type(prop_type, enum_registry);
                let default_value = PropertyValue::default_for(&property_type);
                if let Some(et) = registry.types.iter_mut().find(|et| et.name == *entity_name) {
                    et.properties.push(PropertyDefinition {
                        id: TypeId::new(),
                        name: prop_name.clone(),
                        property_type,
                        default_value,
                    });
                }
            }
            ScaffoldAction::CreateEnum { name, options } => {
                enum_registry.insert(EnumDefinition {
                    id: TypeId::new(),
                    name: name.clone(),
                    options: options.clone(),
                });
            }
            ScaffoldAction::AddCrtColumn {
                label,
                column_type,
                threshold,
            } => {
                let col_type = match column_type.as_str() {
                    "Differential" => ColumnType::Differential,
                    _ => ColumnType::Ratio,
                };
                combat_results_table.table.columns.push(TableColumn {
                    label: label.clone(),
                    column_type: col_type,
                    threshold: *threshold,
                });
                for row_outcomes in &mut combat_results_table.outcomes {
                    row_outcomes.push(CombatOutcome {
                        label: "--".to_string(),
                        effect: None,
                    });
                }
            }
            ScaffoldAction::AddCrtRow {
                label,
                die_min,
                die_max,
            } => {
                combat_results_table.table.rows.push(TableRow {
                    label: label.clone(),
                    value_min: *die_min,
                    value_max: *die_max,
                });
                let num_cols = combat_results_table.table.columns.len();
                combat_results_table.outcomes.push(
                    (0..num_cols)
                        .map(|_| CombatOutcome {
                            label: "--".to_string(),
                            effect: None,
                        })
                        .collect(),
                );
            }
            ScaffoldAction::SetCrtOutcome { row, col, label } => {
                if let Some(row_outcomes) = combat_results_table.outcomes.get_mut(*row)
                    && let Some(outcome) = row_outcomes.get_mut(*col)
                {
                    outcome.label.clone_from(label);
                }
            }
            ScaffoldAction::AddPhase { name, phase_type } => {
                let pt = match phase_type.as_str() {
                    "Combat" => PhaseType::Combat,
                    "Admin" => PhaseType::Admin,
                    _ => PhaseType::Movement,
                };
                turn_structure.phases.push(Phase {
                    id: TypeId::new(),
                    name: name.clone(),
                    phase_type: pt,
                    description: String::new(),
                });
            }
            ScaffoldAction::AddCombatModifier {
                name,
                source,
                shift,
                priority,
            } => {
                let modifier_source = match source.as_str() {
                    "DefenderTerrain" => ModifierSource::DefenderTerrain,
                    "AttackerTerrain" => ModifierSource::AttackerTerrain,
                    other => ModifierSource::Custom(other.to_string()),
                };
                combat_modifiers.modifiers.push(CombatModifierDefinition {
                    id: TypeId::new(),
                    name: name.clone(),
                    source: modifier_source,
                    column_shift: *shift,
                    priority: *priority,
                    cap: None,
                    terrain_type_filter: None,
                });
            }
        }
    }
}

/// Parses a scaffold property type string into a `PropertyType`.
///
/// Supports: `"Bool"`, `"Int"`, `"Float"`, `"String"`, `"Color"`,
/// `"Enum(EnumName)"` (looks up by name in the registry),
/// `"IntRange(min,max)"`, `"FloatRange(min,max)"`.
pub(super) fn parse_scaffold_prop_type(s: &str, enum_registry: &EnumRegistry) -> PropertyType {
    match s {
        "Bool" => PropertyType::Bool,
        "Int" => PropertyType::Int,
        "Float" => PropertyType::Float,
        "String" => PropertyType::String,
        "Color" => PropertyType::Color,
        other => {
            if let Some(inner) = other
                .strip_prefix("Enum(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let enum_id = enum_registry
                    .definitions
                    .values()
                    .find(|e| e.name == inner)
                    .map(|e| e.id)
                    .unwrap_or_default();
                PropertyType::Enum(enum_id)
            } else if let Some(inner) = other
                .strip_prefix("IntRange(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let parts: Vec<&str> = inner.split(',').collect();
                let min = parts
                    .first()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                let max = parts
                    .get(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(100);
                PropertyType::IntRange { min, max }
            } else if let Some(inner) = other
                .strip_prefix("FloatRange(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let parts: Vec<&str> = inner.split(',').collect();
                let min = parts
                    .first()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0.0);
                let max = parts
                    .get(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(1.0);
                PropertyType::FloatRange { min, max }
            } else {
                PropertyType::String
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(super) fn format_property_type(pt: &PropertyType) -> &'static str {
    match pt {
        PropertyType::Bool => "Bool",
        PropertyType::Int => "Int",
        PropertyType::Float => "Float",
        PropertyType::String => "String",
        PropertyType::Color => "Color",
        PropertyType::Enum(_) => "Enum",
        PropertyType::EntityRef(_) => "EntityRef",
        PropertyType::List(_) => "List",
        PropertyType::Map(_, _) => "Map",
        PropertyType::Struct(_) => "Struct",
        PropertyType::IntRange { .. } => "IntRange",
        PropertyType::FloatRange { .. } => "FloatRange",
    }
}

pub(super) fn index_to_property_type(index: usize) -> PropertyType {
    match index {
        1 => PropertyType::Int,
        2 => PropertyType::Float,
        3 => PropertyType::String,
        4 => PropertyType::Color,
        5 => PropertyType::Enum(TypeId::new()),
        6 => PropertyType::EntityRef(None),
        7 => PropertyType::List(Box::new(PropertyType::Int)),
        8 => PropertyType::Map(TypeId::new(), Box::new(PropertyType::Int)),
        9 => PropertyType::Struct(TypeId::new()),
        10 => PropertyType::IntRange { min: 0, max: 100 },
        11 => PropertyType::FloatRange { min: 0.0, max: 1.0 },
        _ => PropertyType::Bool,
    }
}

pub(super) fn bevy_color_to_egui(color: Color) -> egui::Color32 {
    match color {
        Color::Srgba(c) => egui::Color32::from_rgba_unmultiplied(
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
            (c.alpha * 255.0) as u8,
        ),
        Color::LinearRgba(c) => {
            let srgba: bevy::color::Srgba = c.into();
            egui::Color32::from_rgba_unmultiplied(
                (srgba.red * 255.0) as u8,
                (srgba.green * 255.0) as u8,
                (srgba.blue * 255.0) as u8,
                (srgba.alpha * 255.0) as u8,
            )
        }
        _ => BrandTheme::TEXT_SECONDARY,
    }
}

pub(super) fn egui_color_to_bevy(color: egui::Color32) -> Color {
    let [r, g, b, _] = color.to_array();
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub(super) fn rgb_to_color32(rgb: [f32; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
    )
}

pub(super) fn color32_to_rgb(c: egui::Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

pub(super) fn format_relation_effect(effect: &RelationEffect) -> String {
    match effect {
        RelationEffect::ModifyProperty {
            target_property,
            source_property,
            operation,
        } => {
            let op = match operation {
                ModifyOperation::Add => "+",
                ModifyOperation::Subtract => "-",
                ModifyOperation::Multiply => "*",
                ModifyOperation::Min => "min",
                ModifyOperation::Max => "max",
            };
            format!("{target_property} {op} {source_property}")
        }
        RelationEffect::Block { .. } => "Block".to_string(),
        RelationEffect::Allow { .. } => "Allow".to_string(),
    }
}

pub(super) fn format_constraint_expr(expr: &ConstraintExpr) -> String {
    match expr {
        ConstraintExpr::PropertyCompare {
            property_name,
            operator,
            value,
            ..
        } => {
            let op = format_compare_op(*operator);
            format!("{property_name} {op} {value:?}")
        }
        ConstraintExpr::CrossCompare {
            left_property,
            right_property,
            operator,
            ..
        } => {
            let op = format_compare_op(*operator);
            format!("{left_property} {op} {right_property}")
        }
        ConstraintExpr::IsType { .. } => "is type".to_string(),
        ConstraintExpr::IsNotType { .. } => "is not type".to_string(),
        ConstraintExpr::PathBudget {
            cost_property,
            budget_property,
            ..
        } => {
            format!("sum(path.{cost_property}) <= {budget_property}")
        }
        ConstraintExpr::All(exprs) => {
            let parts: Vec<String> = exprs.iter().map(format_constraint_expr).collect();
            format!("({})", parts.join(" AND "))
        }
        ConstraintExpr::Any(exprs) => {
            let parts: Vec<String> = exprs.iter().map(format_constraint_expr).collect();
            format!("({})", parts.join(" OR "))
        }
        ConstraintExpr::Not(expr) => {
            format!("NOT ({})", format_constraint_expr(expr))
        }
    }
}

pub(super) fn format_compare_op(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "==",
        CompareOp::Ne => "!=",
        CompareOp::Lt => "<",
        CompareOp::Le => "<=",
        CompareOp::Gt => ">",
        CompareOp::Ge => ">=",
    }
}

pub(super) fn index_to_modify_operation(index: usize) -> ModifyOperation {
    match index {
        1 => ModifyOperation::Subtract,
        2 => ModifyOperation::Multiply,
        3 => ModifyOperation::Min,
        4 => ModifyOperation::Max,
        _ => ModifyOperation::Add,
    }
}

pub(super) fn index_to_compare_op(index: usize) -> CompareOp {
    match index {
        1 => CompareOp::Ne,
        2 => CompareOp::Lt,
        3 => CompareOp::Le,
        4 => CompareOp::Gt,
        5 => CompareOp::Ge,
        _ => CompareOp::Eq,
    }
}

pub(super) fn build_constraint_expression(
    editor_state: &EditorState,
    roles: &[ConceptRole],
) -> ConstraintExpr {
    match editor_state.new_constraint_expr_type_index {
        0 => {
            // PropertyCompare
            let role_id = roles
                .get(editor_state.new_constraint_role_index)
                .map_or_else(TypeId::new, |r| r.id);
            let value = editor_state
                .new_constraint_value_str
                .trim()
                .parse::<i64>()
                .map_or(PropertyValue::Int(0), PropertyValue::Int);
            ConstraintExpr::PropertyCompare {
                role_id,
                property_name: editor_state.new_constraint_property.trim().to_string(),
                operator: index_to_compare_op(editor_state.new_constraint_op_index),
                value,
            }
        }
        3 => {
            // PathBudget
            let cost_role_id = roles
                .get(editor_state.new_constraint_role_index)
                .map_or_else(TypeId::new, |r| r.id);
            // For PathBudget, use the first role as budget role if different, or same role
            let budget_role_idx =
                usize::from(roles.len() > 1 && editor_state.new_constraint_role_index == 0);
            let budget_role_id = roles
                .get(budget_role_idx)
                .map_or_else(TypeId::new, |r| r.id);
            ConstraintExpr::PathBudget {
                concept_id: TypeId::new(), // Will be set from the concept
                cost_property: editor_state.new_constraint_property.trim().to_string(),
                cost_role_id,
                budget_property: editor_state.new_constraint_value_str.trim().to_string(),
                budget_role_id,
            }
        }
        _ => {
            // TODO(#17): CrossCompare and IsType constraint expressions
            ConstraintExpr::All(Vec::new())
        }
    }
}
