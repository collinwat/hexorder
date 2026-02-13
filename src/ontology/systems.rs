//! Ontology systems: auto-constraint generation and schema validation.

use bevy::prelude::*;

use crate::contracts::game_system::{EntityTypeRegistry, TypeId};
use crate::contracts::ontology::{
    CompareOp, ConceptRegistry, Constraint, ConstraintExpr, ConstraintRegistry, ModifyOperation,
    RelationEffect, RelationRegistry,
};
use crate::contracts::validation::{SchemaError, SchemaErrorCategory, SchemaValidation};

/// Auto-generates companion constraints for `Subtract` relations.
///
/// For each relation with [`RelationEffect::ModifyProperty`] where `operation`
/// is [`ModifyOperation::Subtract`], ensures a non-negative budget constraint
/// exists. The constraint is marked `auto_generated = true` and references
/// the relation by ID.
///
/// - If a `Subtract` relation is added and no auto-constraint exists, one is
///   created.
/// - If a `Subtract` relation is removed, its auto-generated constraint is
///   removed (unless the designer has already modified it, i.e.
///   `auto_generated` is `false`).
/// - If a `Subtract` relation is modified, its auto-generated constraint is
///   regenerated.
pub fn auto_generate_constraints(
    relations: Res<RelationRegistry>,
    mut constraints: ResMut<ConstraintRegistry>,
) {
    // Only run when either registry has changed.
    if !relations.is_changed() && !constraints.is_changed() {
        return;
    }

    // Collect the set of relation IDs that require an auto-generated constraint
    // (Subtract operations on ModifyProperty).
    let subtract_relations: Vec<_> = relations
        .relations
        .iter()
        .filter(|r| {
            matches!(
                &r.effect,
                RelationEffect::ModifyProperty {
                    operation: ModifyOperation::Subtract,
                    ..
                }
            )
        })
        .collect();

    // Remove auto-generated constraints whose source relation no longer exists
    // or is no longer a Subtract relation.
    let relation_ids_with_subtract: Vec<TypeId> = subtract_relations.iter().map(|r| r.id).collect();

    constraints.constraints.retain(|c| {
        if !c.auto_generated {
            return true;
        }
        // Keep if its relation_id is in the current subtract set.
        match c.relation_id {
            Some(rel_id) => relation_ids_with_subtract.contains(&rel_id),
            None => true, // Keep auto-generated constraints not tied to a relation.
        }
    });

    // For each subtract relation, ensure an up-to-date auto-generated
    // constraint exists.
    for relation in &subtract_relations {
        let target_property = match &relation.effect {
            RelationEffect::ModifyProperty {
                target_property, ..
            } => target_property.clone(),
            _ => continue,
        };

        // Find existing auto-generated constraint for this relation.
        let existing = constraints
            .constraints
            .iter()
            .find(|c| c.auto_generated && c.relation_id == Some(relation.id));

        let expected_expr = ConstraintExpr::PropertyCompare {
            role_id: relation.subject_role_id,
            property_name: target_property.clone(),
            operator: CompareOp::Ge,
            value: crate::contracts::game_system::PropertyValue::Int(0),
        };

        match existing {
            Some(c) if c.expression == expected_expr => {
                // Already up to date -- nothing to do.
            }
            Some(_) => {
                // Expression is stale -- regenerate.
                // Remove the old one and push a new one.
                let rel_id = relation.id;
                constraints
                    .constraints
                    .retain(|c| !(c.auto_generated && c.relation_id == Some(rel_id)));

                constraints.constraints.push(Constraint {
                    id: TypeId::new(),
                    name: format!("[auto] {target_property} >= 0"),
                    description: format!(
                        "Auto-generated: ensures {target_property} does not go negative from relation \"{}\"",
                        relation.name
                    ),
                    concept_id: relation.concept_id,
                    relation_id: Some(relation.id),
                    expression: expected_expr,
                    auto_generated: true,
                });
            }
            None => {
                // No auto-constraint exists yet -- create one.
                constraints.constraints.push(Constraint {
                    id: TypeId::new(),
                    name: format!("[auto] {target_property} >= 0"),
                    description: format!(
                        "Auto-generated: ensures {target_property} does not go negative from relation \"{}\"",
                        relation.name
                    ),
                    concept_id: relation.concept_id,
                    relation_id: Some(relation.id),
                    expression: expected_expr,
                    auto_generated: true,
                });
            }
        }
    }
}

/// Validates the ontology registries against the entity type registry and
/// produces a [`SchemaValidation`] resource with any errors found.
///
/// Runs when any ontology registry or [`EntityTypeRegistry`] changes.
///
/// Checks performed:
/// 1. `ConceptBinding` references valid entity type, concept, and concept role.
/// 2. Entity type's `EntityRole` matches `ConceptRole`'s `allowed_entity_roles`.
/// 3. `PropertyBinding` references valid property on the bound entity type.
/// 4. `Relation` references valid concept and distinct roles within that concept.
/// 5. `Constraint` expression references valid roles and concept-local property names.
/// 6. Concept roles have at least one binding (warning as `MissingBinding`).
pub fn run_schema_validation(
    concepts: Res<ConceptRegistry>,
    relations: Res<RelationRegistry>,
    constraints: Res<ConstraintRegistry>,
    entity_types: Res<EntityTypeRegistry>,
    mut validation: ResMut<SchemaValidation>,
) {
    // Only run when something has changed.
    if !concepts.is_changed()
        && !relations.is_changed()
        && !constraints.is_changed()
        && !entity_types.is_changed()
    {
        return;
    }

    let mut errors = Vec::new();

    // --- Check 1, 2, 3: ConceptBinding validation ---
    for binding in &concepts.bindings {
        // Check 1a: entity type exists
        let entity_type = entity_types.get(binding.entity_type_id);
        if entity_type.is_none() {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "ConceptBinding references non-existent entity type {:?}",
                    binding.entity_type_id
                ),
                source_id: binding.id,
            });
        }

        // Check 1b: concept exists
        let concept = concepts
            .concepts
            .iter()
            .find(|c| c.id == binding.concept_id);
        if concept.is_none() {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "ConceptBinding references non-existent concept {:?}",
                    binding.concept_id
                ),
                source_id: binding.id,
            });
        }

        // Check 1c: concept role exists within the concept
        let concept_role = concept.and_then(|c| {
            c.role_labels
                .iter()
                .find(|r| r.id == binding.concept_role_id)
        });
        if concept.is_some() && concept_role.is_none() {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "ConceptBinding references non-existent concept role {:?} within concept",
                    binding.concept_role_id
                ),
                source_id: binding.id,
            });
        }

        // Check 2: entity role matches concept role's allowed_entity_roles
        if let (Some(et), Some(cr)) = (entity_type, concept_role)
            && !cr.allowed_entity_roles.contains(&et.role)
        {
            errors.push(SchemaError {
                category: SchemaErrorCategory::RoleMismatch,
                message: format!(
                    "Entity type \"{}\" has role {:?} but concept role \"{}\" only allows {:?}",
                    et.name, et.role, cr.name, cr.allowed_entity_roles
                ),
                source_id: binding.id,
            });
        }

        // Check 3: property bindings reference valid properties
        if let Some(et) = entity_type {
            for pb in &binding.property_bindings {
                let prop_exists = et.properties.iter().any(|p| p.id == pb.property_id);
                if !prop_exists {
                    errors.push(SchemaError {
                        category: SchemaErrorCategory::PropertyMismatch,
                        message: format!(
                            "PropertyBinding references non-existent property {:?} on entity type \"{}\"",
                            pb.property_id, et.name
                        ),
                        source_id: binding.id,
                    });
                }
            }
        }
    }

    // --- Check 4: Relation validation ---
    for relation in &relations.relations {
        let concept = concepts
            .concepts
            .iter()
            .find(|c| c.id == relation.concept_id);
        let Some(concept) = concept else {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "Relation \"{}\" references non-existent concept {:?}",
                    relation.name, relation.concept_id
                ),
                source_id: relation.id,
            });
            continue;
        };

        let subject_exists = concept
            .role_labels
            .iter()
            .any(|r| r.id == relation.subject_role_id);
        let object_exists = concept
            .role_labels
            .iter()
            .any(|r| r.id == relation.object_role_id);

        if !subject_exists {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "Relation \"{}\" references non-existent subject role {:?} in concept \"{}\"",
                    relation.name, relation.subject_role_id, concept.name
                ),
                source_id: relation.id,
            });
        }
        if !object_exists {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "Relation \"{}\" references non-existent object role {:?} in concept \"{}\"",
                    relation.name, relation.object_role_id, concept.name
                ),
                source_id: relation.id,
            });
        }
        if subject_exists && object_exists && relation.subject_role_id == relation.object_role_id {
            errors.push(SchemaError {
                category: SchemaErrorCategory::InvalidExpression,
                message: format!(
                    "Relation \"{}\" has the same role for subject and object",
                    relation.name
                ),
                source_id: relation.id,
            });
        }
    }

    // --- Check 5: Constraint expression validation ---
    for constraint in &constraints.constraints {
        let concept = concepts
            .concepts
            .iter()
            .find(|c| c.id == constraint.concept_id);
        let Some(concept) = concept else {
            errors.push(SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: format!(
                    "Constraint \"{}\" references non-existent concept {:?}",
                    constraint.name, constraint.concept_id
                ),
                source_id: constraint.id,
            });
            continue;
        };

        validate_constraint_expr(
            &constraint.expression,
            concept,
            &concepts.bindings,
            constraint.id,
            &mut errors,
        );
    }

    // --- Check 6: Concept roles have at least one binding (warning) ---
    for concept in &concepts.concepts {
        for role in &concept.role_labels {
            let has_binding = concepts
                .bindings
                .iter()
                .any(|b| b.concept_id == concept.id && b.concept_role_id == role.id);
            if !has_binding {
                errors.push(SchemaError {
                    category: SchemaErrorCategory::MissingBinding,
                    message: format!(
                        "Concept role \"{}\" in concept \"{}\" has no entity type bindings",
                        role.name, concept.name
                    ),
                    source_id: concept.id,
                });
            }
        }
    }

    let is_valid = errors.is_empty();
    validation.errors = errors;
    validation.is_valid = is_valid;
}

/// Recursively validates a [`ConstraintExpr`] tree, checking that all role IDs
/// exist within the concept and all property names are valid concept-local names.
fn validate_constraint_expr(
    expr: &ConstraintExpr,
    concept: &crate::contracts::ontology::Concept,
    bindings: &[crate::contracts::ontology::ConceptBinding],
    source_id: TypeId,
    errors: &mut Vec<SchemaError>,
) {
    match expr {
        ConstraintExpr::PropertyCompare {
            role_id,
            property_name,
            ..
        } => {
            validate_role_and_property(
                *role_id,
                property_name,
                concept,
                bindings,
                source_id,
                errors,
            );
        }
        ConstraintExpr::CrossCompare {
            left_role_id,
            left_property,
            right_role_id,
            right_property,
            ..
        } => {
            validate_role_and_property(
                *left_role_id,
                left_property,
                concept,
                bindings,
                source_id,
                errors,
            );
            validate_role_and_property(
                *right_role_id,
                right_property,
                concept,
                bindings,
                source_id,
                errors,
            );
        }
        ConstraintExpr::IsType { role_id, .. } | ConstraintExpr::IsNotType { role_id, .. } => {
            if !concept.role_labels.iter().any(|r| r.id == *role_id) {
                errors.push(SchemaError {
                    category: SchemaErrorCategory::InvalidExpression,
                    message: format!(
                        "Constraint expression references non-existent role {:?} in concept \"{}\"",
                        role_id, concept.name
                    ),
                    source_id,
                });
            }
        }
        ConstraintExpr::PathBudget {
            cost_role_id,
            cost_property,
            budget_role_id,
            budget_property,
            ..
        } => {
            validate_role_and_property(
                *cost_role_id,
                cost_property,
                concept,
                bindings,
                source_id,
                errors,
            );
            validate_role_and_property(
                *budget_role_id,
                budget_property,
                concept,
                bindings,
                source_id,
                errors,
            );
        }
        ConstraintExpr::All(exprs) | ConstraintExpr::Any(exprs) => {
            for sub in exprs {
                validate_constraint_expr(sub, concept, bindings, source_id, errors);
            }
        }
        ConstraintExpr::Not(sub) => {
            validate_constraint_expr(sub, concept, bindings, source_id, errors);
        }
    }
}

/// Validates that a role ID exists within a concept and that a property name
/// is a valid concept-local name in any binding for that role.
fn validate_role_and_property(
    role_id: TypeId,
    property_name: &str,
    concept: &crate::contracts::ontology::Concept,
    bindings: &[crate::contracts::ontology::ConceptBinding],
    source_id: TypeId,
    errors: &mut Vec<SchemaError>,
) {
    if !concept.role_labels.iter().any(|r| r.id == role_id) {
        errors.push(SchemaError {
            category: SchemaErrorCategory::InvalidExpression,
            message: format!(
                "Constraint expression references non-existent role {:?} in concept \"{}\"",
                role_id, concept.name
            ),
            source_id,
        });
        return;
    }

    // Check that at least one binding for this concept+role has a
    // PropertyBinding with the given concept_local_name.
    let bindings_for_role: Vec<_> = bindings
        .iter()
        .filter(|b| b.concept_id == concept.id && b.concept_role_id == role_id)
        .collect();

    // If there are no bindings for this role, the MissingBinding check (check 6)
    // will catch it. We skip the property name check here to avoid duplicate errors.
    if bindings_for_role.is_empty() {
        return;
    }

    let has_property = bindings_for_role.iter().any(|b| {
        b.property_bindings
            .iter()
            .any(|pb| pb.concept_local_name == property_name)
    });

    if !has_property {
        errors.push(SchemaError {
            category: SchemaErrorCategory::InvalidExpression,
            message: format!(
                "Constraint expression references unknown concept-local property \"{}\" for role {:?} in concept \"{}\"",
                property_name, role_id, concept.name
            ),
            source_id,
        });
    }
}
