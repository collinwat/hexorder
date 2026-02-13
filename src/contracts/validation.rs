//! Shared Validation types. See `.specs/contracts/validation.md`.
//!
//! Types for schema-level validation (is the game system definition
//! internally consistent?) and state-level validation (given a board
//! state, are constraints satisfied?).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::game_system::TypeId;
use super::hex_grid::HexPosition;

// ---------------------------------------------------------------------------
// Schema Validation
// ---------------------------------------------------------------------------

/// Category of schema-level error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaErrorCategory {
    /// A reference points to a type/concept/role/property that doesn't exist.
    DanglingReference,
    /// An entity type's `EntityRole` doesn't match the `ConceptRole`'s
    /// `allowed_entity_roles`.
    RoleMismatch,
    /// A property binding references a property that doesn't exist on the
    /// entity type, or the property type is incompatible with the
    /// constraint/relation usage.
    PropertyMismatch,
    /// A concept has roles but no entity types are bound to them.
    MissingBinding,
    /// A constraint expression references invalid roles or properties.
    InvalidExpression,
}

/// A single schema-level validation error.
#[derive(Debug, Clone)]
pub struct SchemaError {
    pub category: SchemaErrorCategory,
    /// Human-readable error message.
    pub message: String,
    /// The ID of the offending definition (concept, relation, constraint, or binding).
    pub source_id: TypeId,
}

/// Schema-level validation results for the entire game system definition.
/// Updated by the `rules_engine` when ontology resources change.
#[derive(Resource, Debug, Default)]
pub struct SchemaValidation {
    pub errors: Vec<SchemaError>,
    pub is_valid: bool,
}

// ---------------------------------------------------------------------------
// State Validation
// ---------------------------------------------------------------------------

/// The result of evaluating a single constraint against a specific board position.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub constraint_id: TypeId,
    pub constraint_name: String,
    pub satisfied: bool,
    /// Human-readable explanation of why the constraint passed or failed.
    pub explanation: String,
}

/// The computed set of valid moves for a selected entity.
/// Produced by the `rules_engine` and consumed by `hex_grid` for visual overlay.
#[derive(Resource, Debug, Default)]
pub struct ValidMoveSet {
    /// Hex positions the selected entity can move to.
    pub valid_positions: HashSet<HexPosition>,
    /// For each invalid position within range, the reasons it's blocked.
    pub blocked_explanations: HashMap<HexPosition, Vec<ValidationResult>>,
    /// The entity this move set was computed for (None when no unit is selected).
    pub for_entity: Option<Entity>,
}
