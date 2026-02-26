//! Shared Validation types. See `docs/contracts/validation.md`.
//!
//! Types for schema-level validation (is the game system definition
//! internally consistent?) and state-level validation (given a board
//! state, are constraints satisfied?).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game_system::TypeId;
use crate::hex_grid::HexPosition;

// ---------------------------------------------------------------------------
// Schema Validation
// ---------------------------------------------------------------------------

/// Category of schema-level error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
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
#[derive(Debug, Clone, Reflect)]
pub struct SchemaError {
    pub category: SchemaErrorCategory,
    /// Human-readable error message.
    pub message: String,
    /// The ID of the offending definition (concept, relation, constraint, or binding).
    pub source_id: TypeId,
}

/// Schema-level validation results for the entire game system definition.
/// Updated by the `rules_engine` when ontology resources change.
#[derive(Resource, Debug, Default, Reflect)]
pub struct SchemaValidation {
    pub errors: Vec<SchemaError>,
    pub is_valid: bool,
}

// ---------------------------------------------------------------------------
// State Validation
// ---------------------------------------------------------------------------

/// The result of evaluating a single constraint against a specific board position.
#[derive(Debug, Clone, Reflect)]
pub struct ValidationResult {
    pub constraint_id: TypeId,
    pub constraint_name: String,
    pub satisfied: bool,
    /// Human-readable explanation of why the constraint passed or failed.
    pub explanation: String,
}

/// The computed set of valid moves for a selected entity.
/// Produced by the `rules_engine` and consumed by `hex_grid` for visual overlay.
#[derive(Resource, Debug, Default, Reflect)]
pub struct ValidMoveSet {
    /// Hex positions the selected entity can move to.
    #[reflect(ignore)]
    pub valid_positions: HashSet<HexPosition>,
    /// For each invalid position within range, the reasons it's blocked.
    #[reflect(ignore)]
    pub blocked_explanations: HashMap<HexPosition, Vec<ValidationResult>>,
    /// The entity this move set was computed for (None when no unit is selected).
    pub for_entity: Option<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_error_construction() {
        let error = SchemaError {
            category: SchemaErrorCategory::MissingBinding,
            message: "name is required".to_string(),
            source_id: TypeId::new(),
        };
        assert_eq!(error.category, SchemaErrorCategory::MissingBinding);
        assert_eq!(error.message, "name is required");
    }

    #[test]
    fn schema_error_category_all_variants_debug() {
        // Exercise Debug derive on all variants.
        let cats = [
            SchemaErrorCategory::DanglingReference,
            SchemaErrorCategory::RoleMismatch,
            SchemaErrorCategory::PropertyMismatch,
            SchemaErrorCategory::MissingBinding,
            SchemaErrorCategory::InvalidExpression,
        ];
        for cat in cats {
            let debug = format!("{cat:?}");
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn schema_validation_default_is_valid() {
        let validation = SchemaValidation::default();
        assert!(validation.errors.is_empty());
        assert!(!validation.is_valid);
    }

    #[test]
    fn valid_move_set_default_is_empty() {
        let moves = ValidMoveSet::default();
        assert!(moves.valid_positions.is_empty());
        assert!(moves.blocked_explanations.is_empty());
        assert!(moves.for_entity.is_none());
    }

    #[test]
    fn validation_result_construction() {
        let result = ValidationResult {
            constraint_id: TypeId::new(),
            constraint_name: "Budget >= 0".to_string(),
            satisfied: true,
            explanation: "Budget is 5".to_string(),
        };
        assert!(result.satisfied);
        assert_eq!(result.constraint_name, "Budget >= 0");
    }

    #[test]
    fn validation_result_failed() {
        let result = ValidationResult {
            constraint_id: TypeId::new(),
            constraint_name: "Passable".to_string(),
            satisfied: false,
            explanation: "Terrain is impassable".to_string(),
        };
        assert!(!result.satisfied);
        assert_eq!(result.explanation, "Terrain is impassable");
    }

    #[test]
    fn schema_validation_with_errors() {
        let validation = SchemaValidation {
            errors: vec![SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: "concept 'X' not found".to_string(),
                source_id: TypeId::new(),
            }],
            is_valid: false,
        };
        assert_eq!(validation.errors.len(), 1);
        assert_eq!(
            validation.errors[0].category,
            SchemaErrorCategory::DanglingReference
        );
        assert!(!validation.is_valid);
    }

    #[test]
    fn valid_move_set_with_positions() {
        let mut moves = ValidMoveSet::default();
        let pos = HexPosition { q: 1, r: 2 };
        moves.valid_positions.insert(pos);
        assert!(moves.valid_positions.contains(&pos));
        assert_eq!(moves.valid_positions.len(), 1);
    }

    #[test]
    fn valid_move_set_with_blocked_explanations() {
        let mut moves = ValidMoveSet::default();
        let pos = HexPosition { q: 3, r: -1 };
        let result = ValidationResult {
            constraint_id: TypeId::new(),
            constraint_name: "Passable".to_string(),
            satisfied: false,
            explanation: "Blocked by mountain".to_string(),
        };
        moves.blocked_explanations.insert(pos, vec![result]);
        assert_eq!(moves.blocked_explanations.len(), 1);
        assert_eq!(moves.blocked_explanations[&pos].len(), 1);
        assert!(!moves.blocked_explanations[&pos][0].satisfied);
    }

    #[test]
    fn schema_error_all_categories() {
        let cats = [
            SchemaErrorCategory::DanglingReference,
            SchemaErrorCategory::RoleMismatch,
            SchemaErrorCategory::PropertyMismatch,
            SchemaErrorCategory::MissingBinding,
            SchemaErrorCategory::InvalidExpression,
        ];
        // Verify PartialEq works.
        for (i, a) in cats.iter().enumerate() {
            for (j, b) in cats.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn schema_error_debug() {
        let err = SchemaError {
            category: SchemaErrorCategory::RoleMismatch,
            message: "Role 'X' wrong".to_string(),
            source_id: TypeId::new(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("RoleMismatch"));
    }

    #[test]
    fn validation_result_debug() {
        let result = ValidationResult {
            constraint_id: TypeId::new(),
            constraint_name: "test".to_string(),
            satisfied: true,
            explanation: "ok".to_string(),
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("test"));
    }
}
