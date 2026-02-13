//! Shared Ontology types. See `.specs/contracts/ontology.md`.
//!
//! Defines the game ontology framework: concepts, relations, and constraints.
//! These are designer-defined abstractions that give meaning to entity types
//! and their properties without hardcoding any game terms.

use bevy::prelude::*;

use super::game_system::{EntityRole, PropertyValue, TypeId};

// ---------------------------------------------------------------------------
// Concepts
// ---------------------------------------------------------------------------

/// A designer-defined abstract category that groups related behaviors.
/// Concepts provide the vocabulary for relations between entity types.
/// Example: "Motion" is a concept; "Defense" is another concept.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PropertyBinding {
    pub property_id: TypeId,
    /// The name used within this concept. E.g., "budget", "cost", "passable".
    pub concept_local_name: String,
}

// ---------------------------------------------------------------------------
// Relations
// ---------------------------------------------------------------------------

/// When a relation is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationTrigger {
    /// Evaluated when an entity enters a hex position.
    OnEnter,
    /// Evaluated when an entity exits a hex position.
    OnExit,
    /// Continuously true while entities coexist at a position.
    WhilePresent,
}

/// How to apply a numeric modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOperation {
    Add,
    Subtract,
    Multiply,
    Min,
    Max,
}

/// What effect a relation has when triggered.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// A structured constraint expression. Deliberately limited for M4:
/// property comparisons, cross-entity comparisons, path budgets,
/// type checks, and boolean logic. Not a full DSL.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
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
#[derive(Resource, Debug, Default)]
pub struct ConceptRegistry {
    pub concepts: Vec<Concept>,
    pub bindings: Vec<ConceptBinding>,
}

/// Registry of all relations.
#[derive(Resource, Debug, Default)]
pub struct RelationRegistry {
    pub relations: Vec<Relation>,
}

/// Registry of all constraints.
#[derive(Resource, Debug, Default)]
pub struct ConstraintRegistry {
    pub constraints: Vec<Constraint>,
}
