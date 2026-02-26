//! Ontology plugin unit tests.

use bevy::prelude::*;

use crate::ontology::OntologyPlugin;
use hexorder_contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType, PropertyValue,
    TypeId,
};
use hexorder_contracts::ontology::{
    CompareOp, Concept, ConceptBinding, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, PropertyBinding, Relation, RelationEffect,
    RelationRegistry, RelationTrigger,
};
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::validation::{SchemaErrorCategory, SchemaValidation};

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

// ---------------------------------------------------------------------------
// Auto-constraint: non-subtract relations are ignored
// ---------------------------------------------------------------------------

#[test]
fn auto_constraint_ignores_add_relation() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(Relation {
            id: TypeId::new(),
            name: "Add Relation".to_string(),
            concept_id: motion_concept().0.id,
            subject_role_id: traveler_role_id,
            object_role_id: terrain_role_id,
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::ModifyProperty {
                target_property: "budget".to_string(),
                source_property: "bonus".to_string(),
                operation: ModifyOperation::Add,
            },
        });
    }

    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert!(
        constraint_reg.constraints.is_empty(),
        "Add relations should not produce auto-generated constraints"
    );
}

// ---------------------------------------------------------------------------
// Auto-constraint: manual constraints are retained when relation removed
// ---------------------------------------------------------------------------

#[test]
fn manual_constraint_retained_when_relation_removed() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    // Add a manual (non-auto-generated) constraint.
    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Manual constraint".to_string(),
            description: "Designer-created constraint".to_string(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: traveler_role_id,
                property_name: "budget".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        });
    }

    // Add and remove a subtract relation.
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(subtract_relation(
            concept_id,
            traveler_role_id,
            terrain_role_id,
        ));
    }
    app.update();
    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.clear();
    }
    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert_eq!(
        constraint_reg.constraints.len(),
        1,
        "Manual constraint should be retained"
    );
    assert!(
        !constraint_reg.constraints[0].auto_generated,
        "Retained constraint should be the manual one"
    );
}

// ---------------------------------------------------------------------------
// Auto-constraint: auto-generated constraint without relation_id is retained
// ---------------------------------------------------------------------------

#[test]
fn auto_generated_constraint_without_relation_id_retained() {
    let mut app = test_app();

    let (concept, traveler_role_id, _terrain_role_id) = motion_concept();

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    // Add an auto_generated constraint that has no relation_id.
    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "[auto] orphan".to_string(),
            description: "Auto-generated without relation".to_string(),
            concept_id: motion_concept().0.id,
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: traveler_role_id,
                property_name: "budget".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: true,
        });
    }

    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert_eq!(
        constraint_reg.constraints.len(),
        1,
        "Auto-generated constraint without relation_id should be retained"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: binding references non-existent concept
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_dangling_concept_on_binding() {
    let mut app = test_app();

    let (concept, _traveler_role_id, terrain_role_id) = motion_concept();

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Binding references a concept that does not exist.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id: TypeId::new(), // non-existent concept
            concept_role_id: terrain_role_id,
            property_bindings: vec![],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("non-existent concept")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect dangling concept reference on binding"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: binding references non-existent concept role
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_dangling_concept_role_on_binding() {
    let mut app = test_app();

    let (concept, _traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Binding references a valid concept but non-existent role.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: TypeId::new(), // non-existent role
            property_bindings: vec![],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("non-existent concept role")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect dangling concept role reference on binding"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: relation references non-existent concept
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_relation_dangling_concept() {
    let mut app = test_app();

    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(Relation {
            id: TypeId::new(),
            name: "Bad Relation".to_string(),
            concept_id: TypeId::new(), // non-existent
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("Relation")
                && e.message.contains("non-existent concept")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect relation referencing non-existent concept"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: relation references invalid subject/object roles
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_relation_invalid_roles() {
    let mut app = test_app();

    let (concept, _traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(Relation {
            id: TypeId::new(),
            name: "Bad Roles".to_string(),
            concept_id,
            subject_role_id: TypeId::new(), // non-existent role
            object_role_id: TypeId::new(),  // non-existent role
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let subject_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("subject role")
        })
        .collect();
    let object_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("object role")
        })
        .collect();
    assert!(
        !subject_errors.is_empty(),
        "Should detect invalid subject role"
    );
    assert!(
        !object_errors.is_empty(),
        "Should detect invalid object role"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: relation with same subject and object role
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_relation_same_subject_object() {
    let mut app = test_app();

    let (concept, traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    {
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        relation_reg.relations.push(Relation {
            id: TypeId::new(),
            name: "Self Relation".to_string(),
            concept_id,
            subject_role_id: traveler_role_id,
            object_role_id: traveler_role_id, // same as subject
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::Block { condition: None },
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("same role for subject and object")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect relation with same subject and object role"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint references non-existent concept
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_constraint_dangling_concept() {
    let mut app = test_app();

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Bad Constraint".to_string(),
            description: "References bad concept".to_string(),
            concept_id: TypeId::new(), // non-existent
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: TypeId::new(),
                property_name: "budget".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::DanglingReference
                && e.message.contains("Constraint")
                && e.message.contains("non-existent concept")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect constraint referencing non-existent concept"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: missing binding for concept role (warning)
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_catches_missing_binding() {
    let mut app = test_app();

    let (concept, _traveler_role_id, _terrain_role_id) = motion_concept();

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        // No bindings added for either role.
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    let missing_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::MissingBinding)
        .collect();
    assert!(
        missing_errors.len() >= 2,
        "Should detect missing bindings for both concept roles (traveler and terrain)"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: valid schema produces no errors
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_valid_schema() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);

        // Correctly bind Infantry (Token) to traveler role with valid property.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_points_prop_id(),
                concept_local_name: "budget".to_string(),
            }],
        });

        // Correctly bind Grassland (BoardPosition) to terrain role.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_cost_prop_id(),
                concept_local_name: "cost".to_string(),
            }],
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(
        validation.is_valid,
        "A correctly wired schema should be valid (errors: {:?})",
        validation.errors
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint expr CrossCompare validation
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_cross_compare_invalid_roles() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        // Add bindings with property_bindings so property checks can work.
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_points_prop_id(),
                concept_local_name: "budget".to_string(),
            }],
        });
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_cost_prop_id(),
                concept_local_name: "cost".to_string(),
            }],
        });
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        let bad_role = TypeId::new();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "CrossCompare bad role".to_string(),
            description: "Uses invalid roles".to_string(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::CrossCompare {
                left_role_id: bad_role,
                left_property: "budget".to_string(),
                operator: CompareOp::Ge,
                right_role_id: bad_role,
                right_property: "cost".to_string(),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::InvalidExpression)
        .collect();
    assert!(
        errors.len() >= 2,
        "Should detect invalid roles on both sides of CrossCompare"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint expr CrossCompare valid roles, bad properties
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_cross_compare_invalid_properties() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_points_prop_id(),
                concept_local_name: "budget".to_string(),
            }],
        });
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_cost_prop_id(),
                concept_local_name: "cost".to_string(),
            }],
        });
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "CrossCompare bad props".to_string(),
            description: "Uses invalid property names".to_string(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::CrossCompare {
                left_role_id: traveler_role_id,
                left_property: "nonexistent_left".to_string(),
                operator: CompareOp::Ge,
                right_role_id: terrain_role_id,
                right_property: "nonexistent_right".to_string(),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("unknown concept-local property")
        })
        .collect();
    assert!(
        errors.len() >= 2,
        "Should detect invalid properties on both sides of CrossCompare"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint expr IsType / IsNotType with bad role
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_is_type_invalid_role() {
    let mut app = test_app();

    let (concept, _traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();

        // IsType with bad role
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "IsType bad role".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::IsType {
                role_id: TypeId::new(), // non-existent
                entity_type_id: token_type_id(),
            },
            auto_generated: false,
        });

        // IsNotType with bad role
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "IsNotType bad role".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::IsNotType {
                role_id: TypeId::new(), // non-existent
                entity_type_id: token_type_id(),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("non-existent role")
        })
        .collect();
    assert!(
        errors.len() >= 2,
        "Should detect invalid roles on IsType and IsNotType"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint expr PathBudget with bad roles/properties
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_path_budget_invalid() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_points_prop_id(),
                concept_local_name: "budget".to_string(),
            }],
        });
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_cost_prop_id(),
                concept_local_name: "cost".to_string(),
            }],
        });
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "PathBudget bad props".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::PathBudget {
                concept_id,
                cost_property: "nonexistent_cost".to_string(),
                cost_role_id: terrain_role_id,
                budget_property: "nonexistent_budget".to_string(),
                budget_role_id: traveler_role_id,
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("unknown concept-local property")
        })
        .collect();
    assert!(
        errors.len() >= 2,
        "Should detect invalid properties on both parts of PathBudget"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: constraint expr All / Any / Not recursion
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_all_any_not_recursion() {
    let mut app = test_app();

    let (concept, _traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
    }

    let bad_role = TypeId::new();

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();

        // All with nested bad IsType
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "All nested".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::All(vec![ConstraintExpr::IsType {
                role_id: bad_role,
                entity_type_id: token_type_id(),
            }]),
            auto_generated: false,
        });

        // Any with nested bad IsNotType
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Any nested".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::Any(vec![ConstraintExpr::IsNotType {
                role_id: bad_role,
                entity_type_id: token_type_id(),
            }]),
            auto_generated: false,
        });

        // Not with nested bad PropertyCompare
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Not nested".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::Not(Box::new(ConstraintExpr::PropertyCompare {
                role_id: bad_role,
                property_name: "x".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            })),
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::InvalidExpression)
        .collect();
    assert!(
        errors.len() >= 3,
        "Should detect invalid roles inside All, Any, and Not expressions"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: PropertyCompare with valid role but bad property
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_constraint_property_compare_bad_property() {
    let mut app = test_app();

    let (concept, traveler_role_id, terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: token_type_id(),
            concept_id,
            concept_role_id: traveler_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_points_prop_id(),
                concept_local_name: "budget".to_string(),
            }],
        });
        concept_reg.bindings.push(ConceptBinding {
            id: TypeId::new(),
            entity_type_id: board_type_id(),
            concept_id,
            concept_role_id: terrain_role_id,
            property_bindings: vec![PropertyBinding {
                property_id: movement_cost_prop_id(),
                concept_local_name: "cost".to_string(),
            }],
        });
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "Bad property name".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: traveler_role_id,
                property_name: "nonexistent".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    assert!(!validation.is_valid);
    let errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("unknown concept-local property")
                && e.message.contains("nonexistent")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "Should detect unknown property name in PropertyCompare"
    );
}

// ---------------------------------------------------------------------------
// Schema validation: PropertyCompare with valid role but no bindings for role
// (skips property check - covered by MissingBinding check 6)
// ---------------------------------------------------------------------------

#[test]
fn schema_validation_skips_property_check_when_no_bindings() {
    let mut app = test_app();

    let (concept, traveler_role_id, _terrain_role_id) = motion_concept();
    let concept_id = concept.id;

    {
        let mut concept_reg = app.world_mut().resource_mut::<ConceptRegistry>();
        concept_reg.concepts.push(concept);
        // No bindings added for traveler role.
    }

    {
        let mut constraint_reg = app.world_mut().resource_mut::<ConstraintRegistry>();
        constraint_reg.constraints.push(Constraint {
            id: TypeId::new(),
            name: "PropertyCompare no bindings".to_string(),
            description: String::new(),
            concept_id,
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: traveler_role_id,
                property_name: "anything".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        });
    }

    app.update();

    let validation = app.world().resource::<SchemaValidation>();
    // Should have MissingBinding warnings but NOT InvalidExpression for property
    let property_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| {
            e.category == SchemaErrorCategory::InvalidExpression
                && e.message.contains("unknown concept-local property")
        })
        .collect();
    assert!(
        property_errors.is_empty(),
        "Should skip property check when no bindings exist for the role"
    );

    let missing_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.category == SchemaErrorCategory::MissingBinding)
        .collect();
    assert!(
        !missing_errors.is_empty(),
        "Should still report MissingBinding for unbound roles"
    );
}

// ---------------------------------------------------------------------------
// Auto-constraint: already up to date is a no-op
// ---------------------------------------------------------------------------

#[test]
fn auto_constraint_already_up_to_date_is_noop() {
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

    // First update: auto-generates the constraint.
    app.update();

    let first_constraint_id = {
        let constraint_reg = app.world().resource::<ConstraintRegistry>();
        assert_eq!(constraint_reg.constraints.len(), 1);
        constraint_reg.constraints[0].id
    };

    // Trigger change detection without modifying the relation.
    {
        // Touching the relation registry marks it as changed.
        let mut relation_reg = app.world_mut().resource_mut::<RelationRegistry>();
        // Re-set the same data to trigger is_changed().
        let relations = relation_reg.relations.clone();
        relation_reg.relations = relations;
    }

    // Second update: the auto-constraint is already up to date.
    app.update();

    let constraint_reg = app.world().resource::<ConstraintRegistry>();
    assert_eq!(
        constraint_reg.constraints.len(),
        1,
        "Should still have exactly one constraint"
    );
    assert_eq!(
        constraint_reg.constraints[0].id, first_constraint_id,
        "Same constraint ID should be preserved (no regeneration)"
    );
    assert_eq!(
        constraint_reg.constraints[0].relation_id,
        Some(relation_id),
        "Constraint should still reference the same relation"
    );
}
