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
}
