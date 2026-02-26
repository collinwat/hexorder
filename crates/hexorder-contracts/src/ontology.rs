//! Shared Ontology types. See `docs/contracts/ontology.md`.
//!
//! Defines the game ontology framework: concepts, relations, and constraints.
//! These are designer-defined abstractions that give meaning to entity types
//! and their properties without hardcoding any game terms.

// bevy_reflect derive macros generate underscore-prefixed bindings internally
#![allow(clippy::used_underscore_binding)]

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game_system::{EntityRole, PropertyValue, TypeId};

// ---------------------------------------------------------------------------
// Concepts
// ---------------------------------------------------------------------------

/// A designer-defined abstract category that groups related behaviors.
/// Concepts provide the vocabulary for relations between entity types.
/// Example: "Motion" is a concept; "Defense" is another concept.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Concept {
    pub id: TypeId,
    pub name: String,
    pub description: String,
    /// Named role slots within this concept. Entity types bind to these roles.
    pub role_labels: Vec<ConceptRole>,
}

/// A named slot within a concept. Entity types can bind to this role.
/// Example: The "Motion" concept has roles "traveler" (`Token`) and
/// "terrain" (`BoardPosition`).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ConceptRole {
    pub id: TypeId,
    pub name: String,
    /// Which `EntityRole`(s) can fill this concept role.
    /// E.g., the "terrain" role in Motion is limited to `BoardPosition`.
    pub allowed_entity_roles: Vec<EntityRole>,
}

// ---------------------------------------------------------------------------
// Concept Bindings
// ---------------------------------------------------------------------------

/// Binds an entity type to a concept role and declares which of its
/// properties are relevant to that concept.
/// Example: Infantry binds to Motion's "traveler" role, mapping its
/// `movement_points` property as concept-local name "budget".
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ConceptBinding {
    pub id: TypeId,
    pub entity_type_id: TypeId,
    pub concept_id: TypeId,
    pub concept_role_id: TypeId,
    /// Which properties of this entity type participate in this concept.
    pub property_bindings: Vec<PropertyBinding>,
}

/// Maps an entity type's property to a concept-local semantic name.
/// The concept-local name is what relations and constraints reference.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct PropertyBinding {
    pub property_id: TypeId,
    /// The name used within this concept. E.g., "budget", "cost", "passable".
    pub concept_local_name: String,
}

// ---------------------------------------------------------------------------
// Relations
// ---------------------------------------------------------------------------

/// When a relation is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum RelationTrigger {
    /// Evaluated when an entity enters a hex position.
    OnEnter,
    /// Evaluated when an entity exits a hex position.
    OnExit,
    /// Continuously true while entities coexist at a position.
    WhilePresent,
}

/// How to apply a numeric modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum ModifyOperation {
    Add,
    Subtract,
    Multiply,
    Min,
    Max,
}

/// What effect a relation has when triggered.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum RelationEffect {
    /// Modifies a numeric property value.
    /// E.g., terrain cost subtracts from movement budget.
    ModifyProperty {
        /// Concept-local name of the property being modified (on the subject).
        target_property: String,
        /// Concept-local name of the property providing the modifier (on the object).
        source_property: String,
        /// How to apply the modification.
        operation: ModifyOperation,
    },
    /// Blocks the subject from the position when the condition is met.
    /// If condition is None, the block is unconditional.
    Block { condition: Option<ConstraintExpr> },
    /// Permits the subject at the position when the condition is met.
    /// Used for allowlisting in combination with default-deny rules.
    Allow { condition: Option<ConstraintExpr> },
}

/// A designer-defined relation between two concept roles.
/// Example: "Terrain Movement Cost" — when a traveler enters terrain,
/// subtract the terrain's cost from the traveler's budget.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Relation {
    pub id: TypeId,
    pub name: String,
    pub concept_id: TypeId,
    /// The role that initiates or is the subject of the relation (e.g., traveler).
    pub subject_role_id: TypeId,
    /// The role that is the object or target of the relation (e.g., terrain).
    pub object_role_id: TypeId,
    pub trigger: RelationTrigger,
    pub effect: RelationEffect,
}

// ---------------------------------------------------------------------------
// Constraints
// ---------------------------------------------------------------------------

/// Comparison operators for constraint expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// A structured constraint expression. Deliberately limited for 0.4.0:
/// property comparisons, cross-entity comparisons, path budgets,
/// type checks, and boolean logic. Not a full DSL.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub enum ConstraintExpr {
    /// Compare a property value against a literal.
    /// E.g., traveler.budget >= 0
    PropertyCompare {
        role_id: TypeId,
        /// Concept-local property name.
        property_name: String,
        operator: CompareOp,
        value: PropertyValue,
    },
    /// Compare two properties across concept roles.
    /// E.g., traveler.budget >= terrain.cost
    CrossCompare {
        left_role_id: TypeId,
        left_property: String,
        operator: CompareOp,
        right_role_id: TypeId,
        right_property: String,
    },
    /// Check if an entity is of a specific type.
    IsType {
        role_id: TypeId,
        entity_type_id: TypeId,
    },
    /// Check if an entity is NOT of a specific type.
    IsNotType {
        role_id: TypeId,
        entity_type_id: TypeId,
    },
    /// Sum a property along a path and compare against a budget.
    /// E.g., sum(path.terrain.cost) <= traveler.budget
    PathBudget {
        concept_id: TypeId,
        /// Concept-local name of the per-step cost property (on terrain role).
        cost_property: String,
        cost_role_id: TypeId,
        /// Concept-local name of the budget property (on traveler role).
        budget_property: String,
        budget_role_id: TypeId,
    },
    /// All sub-expressions must be true.
    All(Vec<ConstraintExpr>),
    /// At least one sub-expression must be true.
    Any(Vec<ConstraintExpr>),
    /// The sub-expression must be false.
    Not(Box<ConstraintExpr>),
}

/// A named constraint in the game system.
/// Can be auto-generated from a relation or manually created by the designer.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Constraint {
    pub id: TypeId,
    pub name: String,
    pub description: String,
    /// The concept this constraint operates within.
    pub concept_id: TypeId,
    /// If auto-generated, the relation it was derived from.
    pub relation_id: Option<TypeId>,
    /// The condition that must hold for the game state to be valid.
    pub expression: ConstraintExpr,
    /// Whether this constraint was auto-generated (shown with "[auto]" badge in UI).
    pub auto_generated: bool,
}

// ---------------------------------------------------------------------------
// Registries
// ---------------------------------------------------------------------------

/// Registry of all concepts and their bindings.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct ConceptRegistry {
    pub concepts: Vec<Concept>,
    pub bindings: Vec<ConceptBinding>,
}

/// Registry of all relations.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct RelationRegistry {
    pub relations: Vec<Relation>,
}

/// Registry of all constraints.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct ConstraintRegistry {
    pub constraints: Vec<Constraint>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_system::{EntityRole, PropertyValue, TypeId};

    /// Round-trip: serialize `ConceptRegistry` with bindings.
    #[test]
    fn concept_registry_ron_round_trip() {
        let concept_id = TypeId::new();
        let role_id = TypeId::new();
        let entity_type_id = TypeId::new();
        let prop_id = TypeId::new();

        let registry = ConceptRegistry {
            concepts: vec![Concept {
                id: concept_id,
                name: "Motion".to_string(),
                description: "Movement system".to_string(),
                role_labels: vec![ConceptRole {
                    id: role_id,
                    name: "traveler".to_string(),
                    allowed_entity_roles: vec![EntityRole::Token],
                }],
            }],
            bindings: vec![ConceptBinding {
                id: TypeId::new(),
                entity_type_id,
                concept_id,
                concept_role_id: role_id,
                property_bindings: vec![PropertyBinding {
                    property_id: prop_id,
                    concept_local_name: "budget".to_string(),
                }],
            }],
        };

        let ron_str = ron::to_string(&registry).expect("serialize");
        let deserialized: ConceptRegistry = ron::from_str(&ron_str).expect("deserialize");

        assert_eq!(deserialized.concepts.len(), 1);
        assert_eq!(deserialized.concepts[0].name, "Motion");
        assert_eq!(deserialized.concepts[0].role_labels.len(), 1);
        assert_eq!(deserialized.bindings.len(), 1);
        assert_eq!(deserialized.bindings[0].property_bindings.len(), 1);
        assert_eq!(
            deserialized.bindings[0].property_bindings[0].concept_local_name,
            "budget"
        );
    }

    /// Round-trip: serialize recursive `ConstraintExpr`.
    #[test]
    fn constraint_expr_ron_round_trip() {
        let role_id = TypeId::new();
        let expr = ConstraintExpr::All(vec![
            ConstraintExpr::PropertyCompare {
                role_id,
                property_name: "budget".to_string(),
                operator: CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            ConstraintExpr::Not(Box::new(ConstraintExpr::IsType {
                role_id,
                entity_type_id: TypeId::new(),
            })),
        ]);

        let ron_str = ron::to_string(&expr).expect("serialize");
        let deserialized: ConstraintExpr = ron::from_str(&ron_str).expect("deserialize");

        match &deserialized {
            ConstraintExpr::All(children) => {
                assert_eq!(children.len(), 2);
                assert!(matches!(
                    &children[0],
                    ConstraintExpr::PropertyCompare { .. }
                ));
                assert!(matches!(&children[1], ConstraintExpr::Not(_)));
            }
            other => panic!("expected All, got {other:?}"),
        }
    }

    /// Exercise remaining `ConstraintExpr` variants: `CrossCompare`, `IsNotType`, `PathBudget`, Any.
    #[test]
    fn constraint_expr_cross_compare_ron_round_trip() {
        let expr = ConstraintExpr::CrossCompare {
            left_role_id: TypeId::new(),
            left_property: "budget".to_string(),
            operator: CompareOp::Gt,
            right_role_id: TypeId::new(),
            right_property: "cost".to_string(),
        };
        let ron_str = ron::to_string(&expr).expect("serialize");
        let deserialized: ConstraintExpr = ron::from_str(&ron_str).expect("deserialize");
        assert!(matches!(deserialized, ConstraintExpr::CrossCompare { .. }));
    }

    #[test]
    fn constraint_expr_is_not_type_ron_round_trip() {
        let expr = ConstraintExpr::IsNotType {
            role_id: TypeId::new(),
            entity_type_id: TypeId::new(),
        };
        let ron_str = ron::to_string(&expr).expect("serialize");
        let deserialized: ConstraintExpr = ron::from_str(&ron_str).expect("deserialize");
        assert!(matches!(deserialized, ConstraintExpr::IsNotType { .. }));
    }

    #[test]
    fn constraint_expr_path_budget_ron_round_trip() {
        let expr = ConstraintExpr::PathBudget {
            concept_id: TypeId::new(),
            cost_property: "cost".to_string(),
            cost_role_id: TypeId::new(),
            budget_property: "budget".to_string(),
            budget_role_id: TypeId::new(),
        };
        let ron_str = ron::to_string(&expr).expect("serialize");
        let deserialized: ConstraintExpr = ron::from_str(&ron_str).expect("deserialize");
        assert!(matches!(deserialized, ConstraintExpr::PathBudget { .. }));
    }

    #[test]
    fn constraint_expr_any_ron_round_trip() {
        let expr = ConstraintExpr::Any(vec![ConstraintExpr::PropertyCompare {
            role_id: TypeId::new(),
            property_name: "x".to_string(),
            operator: CompareOp::Eq,
            value: PropertyValue::Int(1),
        }]);
        let ron_str = ron::to_string(&expr).expect("serialize");
        let deserialized: ConstraintExpr = ron::from_str(&ron_str).expect("deserialize");
        match deserialized {
            ConstraintExpr::Any(children) => assert_eq!(children.len(), 1),
            other => panic!("expected Any, got {other:?}"),
        }
    }

    #[test]
    fn relation_registry_ron_round_trip() {
        let reg = RelationRegistry {
            relations: vec![Relation {
                id: TypeId::new(),
                name: "Cost".to_string(),
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
        };
        let ron_str = ron::to_string(&reg).expect("serialize");
        let deserialized: RelationRegistry = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.relations.len(), 1);
        assert_eq!(deserialized.relations[0].trigger, RelationTrigger::OnEnter);
    }

    #[test]
    fn relation_effect_block_allow_ron_round_trip() {
        let block = RelationEffect::Block { condition: None };
        let allow = RelationEffect::Allow {
            condition: Some(ConstraintExpr::All(vec![])),
        };
        let ron_block = ron::to_string(&block).expect("serialize block");
        let ron_allow = ron::to_string(&allow).expect("serialize allow");
        let _: RelationEffect = ron::from_str(&ron_block).expect("deserialize block");
        let _: RelationEffect = ron::from_str(&ron_allow).expect("deserialize allow");
    }

    #[test]
    fn constraint_registry_ron_round_trip() {
        let reg = ConstraintRegistry {
            constraints: vec![Constraint {
                id: TypeId::new(),
                name: "Budget >= 0".to_string(),
                description: "Must have budget".to_string(),
                concept_id: TypeId::new(),
                relation_id: None,
                expression: ConstraintExpr::All(Vec::new()),
                auto_generated: false,
            }],
        };
        let ron_str = ron::to_string(&reg).expect("serialize");
        let deserialized: ConstraintRegistry = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.constraints.len(), 1);
        assert!(!deserialized.constraints[0].auto_generated);
    }

    #[test]
    fn relation_trigger_variants() {
        assert_ne!(RelationTrigger::OnEnter, RelationTrigger::OnExit);
        assert_ne!(RelationTrigger::OnExit, RelationTrigger::WhilePresent);
    }

    #[test]
    fn compare_op_all_variants() {
        let ops = [
            CompareOp::Eq,
            CompareOp::Ne,
            CompareOp::Lt,
            CompareOp::Le,
            CompareOp::Gt,
            CompareOp::Ge,
        ];
        for op in ops {
            assert!(!format!("{op:?}").is_empty());
        }
    }

    #[test]
    fn modify_operation_all_variants() {
        let ops = [
            ModifyOperation::Add,
            ModifyOperation::Subtract,
            ModifyOperation::Multiply,
            ModifyOperation::Min,
            ModifyOperation::Max,
        ];
        for op in ops {
            assert!(!format!("{op:?}").is_empty());
        }
    }

    #[test]
    fn concept_role_construction() {
        let role = ConceptRole {
            id: TypeId::new(),
            name: "terrain".to_string(),
            allowed_entity_roles: vec![EntityRole::BoardPosition],
        };
        assert_eq!(role.name, "terrain");
        assert_eq!(role.allowed_entity_roles.len(), 1);
        assert_eq!(role.allowed_entity_roles[0], EntityRole::BoardPosition);
    }

    #[test]
    fn concept_description_field() {
        let concept = Concept {
            id: TypeId::new(),
            name: "Defense".to_string(),
            description: "Defensive capabilities".to_string(),
            role_labels: vec![],
        };
        assert_eq!(concept.description, "Defensive capabilities");
        assert!(concept.role_labels.is_empty());
    }

    #[test]
    fn property_binding_construction() {
        let pb = PropertyBinding {
            property_id: TypeId::new(),
            concept_local_name: "cost".to_string(),
        };
        assert_eq!(pb.concept_local_name, "cost");
    }

    #[test]
    fn concept_binding_fields() {
        let concept_id = TypeId::new();
        let role_id = TypeId::new();
        let entity_type_id = TypeId::new();
        let binding = ConceptBinding {
            id: TypeId::new(),
            entity_type_id,
            concept_id,
            concept_role_id: role_id,
            property_bindings: vec![],
        };
        assert_eq!(binding.concept_id, concept_id);
        assert_eq!(binding.concept_role_id, role_id);
        assert_eq!(binding.entity_type_id, entity_type_id);
        assert!(binding.property_bindings.is_empty());
    }

    #[test]
    fn relation_fields() {
        let r = Relation {
            id: TypeId::new(),
            name: "Terrain Cost".to_string(),
            concept_id: TypeId::new(),
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::WhilePresent,
            effect: RelationEffect::Block { condition: None },
        };
        assert_eq!(r.name, "Terrain Cost");
        assert_eq!(r.trigger, RelationTrigger::WhilePresent);
    }

    #[test]
    fn constraint_auto_generated_field() {
        let c = Constraint {
            id: TypeId::new(),
            name: "auto".to_string(),
            description: "Auto-generated".to_string(),
            concept_id: TypeId::new(),
            relation_id: Some(TypeId::new()),
            expression: ConstraintExpr::All(vec![]),
            auto_generated: true,
        };
        assert!(c.auto_generated);
        assert!(c.relation_id.is_some());
    }

    #[test]
    fn concept_registry_default_is_empty() {
        let reg = ConceptRegistry::default();
        assert!(reg.concepts.is_empty());
        assert!(reg.bindings.is_empty());
    }

    #[test]
    fn relation_registry_default_is_empty() {
        let reg = RelationRegistry::default();
        assert!(reg.relations.is_empty());
    }

    #[test]
    fn constraint_registry_default_is_empty() {
        let reg = ConstraintRegistry::default();
        assert!(reg.constraints.is_empty());
    }

    #[test]
    fn relation_effect_modify_property_ron_round_trip() {
        let effect = RelationEffect::ModifyProperty {
            target_property: "budget".to_string(),
            source_property: "cost".to_string(),
            operation: ModifyOperation::Add,
        };
        let ron_str = ron::to_string(&effect).expect("serialize");
        let deserialized: RelationEffect = ron::from_str(&ron_str).expect("deserialize");
        assert!(matches!(
            deserialized,
            RelationEffect::ModifyProperty { .. }
        ));
    }
}
