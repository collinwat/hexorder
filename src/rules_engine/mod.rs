//! Rules Engine plugin.
//!
//! Evaluates ontology constraints against board state. Computes valid
//! moves for selected units via BFS with constraint evaluation.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;
use crate::contracts::validation::ValidMoveSet;

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that initializes the `ValidMoveSet` resource and wires up
/// the valid-move computation system.
#[derive(Debug)]
pub struct RulesEnginePlugin;

impl Plugin for RulesEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ValidMoveSet>();
        app.add_systems(
            Update,
            systems::compute_valid_moves.run_if(in_state(AppScreen::Editor)),
        );
    }
}
