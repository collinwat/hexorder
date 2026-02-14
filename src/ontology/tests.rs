//! Ontology plugin unit tests.

use bevy::prelude::*;

use crate::contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType, PropertyValue,
    TypeId,
};
use crate::contracts::ontology::{
    CompareOp, Concept, ConceptBinding, ConceptRegistry, ConceptRole, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, PropertyBinding, Relation, RelationEffect,
    RelationRegistry, RelationTrigger,
};
use crate::contracts::persistence::AppScreen;
use crate::contracts::validation::{SchemaErrorCategory, SchemaValidation};
use crate::ontology::OntologyPlugin;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Builds a minimal headless app with `OntologyPlugin` and a pre-populated
/// `EntityTypeRegistry` containing one `BoardPosition` type and one `Token`
/// type, each with a single integer property.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.insert_resource(test_entity_type_registry());
    app.add_plugins(OntologyPlugin);
    app
}

/// Returns the test "Grassland" entity type ID.
fn board_type_id() -> TypeId {
    // Deterministic UUID so we can reference it across helpers.
    TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

/// Returns the test "Infantry" entity type ID.
fn token_type_id() -> TypeId {
    TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]))
}

/// Property ID for `movement_cost` on Grassland.
fn movement_cost_prop_id() -> TypeId {
    TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x03,
    ]))
}

/// Property ID for `movement_points` on Infantry.
fn movement_points_prop_id() -> TypeId {
    TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04,
    ]))
}

fn test_entity_type_registry() -> EntityTypeRegistry {
    EntityTypeRegistry {
        types: vec![
            EntityType {
                id: board_type_id(),
                name: "Grassland".to_string(),
                role: EntityRole::BoardPosition,
                color: bevy::color::Color::srgb(0.2, 0.6, 0.2),
                properties: vec![PropertyDefinition {
                    id: movement_cost_prop_id(),
                    name: "movement_cost".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(1),
                }],
            },
            EntityType {
                id: token_type_id(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: bevy::color::Color::srgb(0.8, 0.2, 0.2),
                properties: vec![PropertyDefinition {
                    id: movement_points_prop_id(),
                    name: "movement_points".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(4),
                }],
            },
        ],
        enum_definitions: vec![],
    }
}

/// Creates a "Motion" concept with "traveler" (Token) and "terrain"
/// (`BoardPosition`) roles. Returns (concept, `traveler_role_id`, `terrain_role_id`).
fn motion_concept() -> (Concept, TypeId, TypeId) {
    let traveler_role_id = TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x10,
    ]));
    let terrain_role_id = TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x11,
    ]));
    let concept_id = TypeId(uuid::Uuid::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x20,
    ]));

    let concept = Concept {
        id: concept_id,
        name: "Motion".to_string(),
        description: "Movement across terrain".to_string(),
        role_labels: vec![
            ConceptRole {
                id: traveler_role_id,
                name: "traveler".to_string(),
                allowed_entity_roles: vec![EntityRole::Token],
            },
            ConceptRole {
                id: terrain_role_id,
                name: "terrain".to_string(),
                allowed_entity_roles: vec![EntityRole::BoardPosition],
            },
        ],
    };

    (concept, traveler_role_id, terrain_role_id)
}

/// Creates a subtract relation for motion concept.
fn subtract_relation(
    concept_id: TypeId,
    subject_role_id: TypeId,
    object_role_id: TypeId,
) -> Relation {
    Relation {
        id: TypeId(uuid::Uuid::from_bytes([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x30,
        ])),
        name: "Terrain Movement Cost".to_string(),
        concept_id,
        subject_role_id,
        object_role_id,
        trigger: RelationTrigger::OnEnter,
        effect: RelationEffect::ModifyProperty {
            target_property: "budget".to_string(),
            source_property: "cost".to_string(),
            operation: ModifyOperation::Subtract,
        },
    }
}

// ---------------------------------------------------------------------------
// SC-1: Registries available at startup
// ---------------------------------------------------------------------------

#[test]
fn registries_available_at_startup() {
    let app = test_app();

    assert!(
        app.world().get_resource::<ConceptRegistry>().is_some(),
        "ConceptRegistry should exist"
    );
    assert!(
        app.world().get_resource::<RelationRegistry>().is_some(),
        "RelationRegistry should exist"
    );
    assert!(
        app.world().get_resource::<ConstraintRegistry>().is_some(),
        "ConstraintRegistry should exist"
    );
    assert!(
        app.world().get_resource::<SchemaValidation>().is_some(),
        "SchemaValidation should exist"
    );
}

// ---------------------------------------------------------------------------
// SC-2: Auto-constraint on subtract relation
// ---------------------------------------------------------------------------

#[test]
fn auto_constraint_on_subtract_relation() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let relation = subtract_relation(concept.id, traveler_role_id, terrain_role_id);
    let relation_id = relation.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(relation);
    }

    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert_eq!(
        constraint_reg.constraints.len(),
        1,
        "Expected exactly one auto-generated constraint"
    );

    let c = &constraint_reg.constraints[0];
    assert!(c.auto_generated, "Constraint should be auto_generated");
    assert_eq!(
        c.relation_id,
        Some(relation_id),
        "Constraint should reference the source relation"
    );
    assert_eq!(
        c.expression,
        ConstraintExpr::PropertyCompare {
            role_id: traveler_role_id,
            property_name: "budget".to_string(),
            operator: CompareOp::Ge,
            value: PropertyValue::Int(0),
        },
        "Auto-generated constraint should be budget >= 0"
    );
}

// ---------------------------------------------------------------------------
// SC-3: Auto-constraint deleted with relation
// ---------------------------------------------------------------------------

#[test]
fn auto_constraint_deleted_with_relation() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let relation = subtract_relation(concept.id, traveler_role_id, terrain_role_id);

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(relation);
    }

    // First update: auto-generates the constraint.
    app.update();

    {
        let constraint_reg = app.world().resource::<ConstraintRegistry>();
        assert_eq!(
            constraint_reg.constraints.len(),
            1,
            "Constraint should exist after first update"
        );
    }

    // Remove the relation.
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.clear();
    }

    // Second update: removes the auto-generated constraint.
    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert!(
        constraint_reg.constraints.is_empty(),
        "Auto-generated constraint should be removed when its relation is deleted"
    );
}

// ---------------------------------------------------------------------------
// SC-4: Auto-constraint regenerated on relation change
// ---------------------------------------------------------------------------

#[test]
fn auto_constraint_regenerated_on_relation_change() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let relation = subtract_relation(concept.id, traveler_role_id, terrain_role_id);
    let relation_id = relation.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(relation);
    }

    // First update: generates constraint with target_property = "budget".
    app.update();

    {
        let constraint_reg = app.world().resource::<ConstraintRegistry>();
        assert_eq!(constraint_reg.constraints.len(), 1);
        match &constraint_reg.constraints[0].expression {
            ConstraintExpr::PropertyCompare { property_name, .. } => {
                assert_eq!(property_name, "budget");
            }
            other => panic!("Expected PropertyCompare, got {other:?}"),
        }
    }

    // Modify the relation's target property.
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        if let Some(r) = relation_reg
            .relations
            .iter_mut()
            .find(|r| r.id == relation_id)
        {
            r.effect = RelationEffect::ModifyProperty {
                target_property: "stamina".to_string(),
                source_property: "cost".to_string(),
                operation: ModifyOperation::Subtract,
            };
        }
    }

    // Second update: regenerates the constraint with the new property name.
    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert_eq!(
        constraint_reg.constraints.len(),
        1,
        "Should still have exactly one auto-generated constraint"
    );
    let c = &constraint_reg.constraints[0];
    assert!(c.auto_generated);
    match &c.expression {
        ConstraintExpr::PropertyCompare { property_name, .. } => {
            assert_eq!(
                property_name, "stamina",
                "Regenerated constraint should reference the new property"
            );
        }
        other => panic!("Expected PropertyCompare, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// SC-5: Schema validation catches dangling reference
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_dangling_reference() {
    let mut app = test_app();

    let (concept, _traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Add a binding referencing a non-existent entity type.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: TypeId::new(), // does not exist in EntityTypeRegistry
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(
        !validation.is_valid,
        "Validation should not be valid with a dangling reference"
    );
    let dangling_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::DanglingReference)
        .collect();
    assert!(
        !dangling_errors.is_empty(),
        "Should have at least one DanglingReference error"
    );
}

// ---------------------------------------------------------------------------
// SC-6: Schema validation catches role mismatch
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_role_mismatch() {
    let mut app = test_app();

    let (concept, _traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Bind a Token entity type (Infantry) to the "terrain" role, which
        // only allows BoardPosition.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(), // Token role
            concept_id,
            concept_role_id: terrain_role_id, // allows only BoardPosition
            property_bindings: vec![],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(
        !validation.is_valid,
        "Validation should not be valid with a role mismatch"
    );
    let mismatch_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::RoleMismatch)
        .collect();
    assert!(
        !mismatch_errors.is_empty(),
        "Should have at least one RoleMismatch error"
    );
}

// ---------------------------------------------------------------------------
// SC-7: Schema validation catches property mismatch
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_property_mismatch() {
    let mut app = test_app();

    let (concept, traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Bind Infantry (Token) to the "traveler" role (correct role match),
        // but reference a non-existent property ID.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: TypeId::new(), // does not exist on Infantry
                concept_local_name: "budget".to_string(),
            }],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(
        !validation.is_valid,
        "Validation should not be valid with a property mismatch"
    );
    let prop_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::PropertyMismatch)
        .collect();
    assert!(
        !prop_errors.is_empty(),
        "Should have at least one PropertyMismatch error"
    );
}
