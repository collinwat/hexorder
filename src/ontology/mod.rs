//! Ontology plugin.
//!
//! Manages the game ontology framework: concepts, relations, and constraints.
//! Auto-generates companion constraints for `Subtract` relations and runs
//! schema validation to check the game system definition for internal
//! consistency.

use bevy::prelude::*;

use hexorder_contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::validation::SchemaValidation;

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that initializes ontology registries and wires up the
/// auto-generation and schema validation systems.
#[derive(Debug)]
pub struct OntologyPlugin;

impl Plugin for OntologyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConceptRegistry>();
        app.init_resource::<RelationRegistry>();
        app.init_resource::<ConstraintRegistry>();
        app.init_resource::<SchemaValidation>();

        app.add_systems(
            Update,
            (
                systems::auto_generate_constraints,
                systems::run_schema_validation,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
    }
}
