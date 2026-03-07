//! Rules Engine plugin.
//!
//! Evaluates ontology constraints against board state. Computes valid
//! moves for selected units via BFS with constraint evaluation.

use bevy::prelude::*;
use hexorder_sdk::{HexorderPlugin, PluginId};

use hexorder_contracts::hex_grid::{InfluenceMap, InfluenceRuleRegistry, StackingRule};
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::validation::ValidMoveSet;

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that initializes the `ValidMoveSet` resource and wires up
/// the valid-move computation system.
#[derive(Debug)]
pub struct RulesEnginePlugin;

impl HexorderPlugin for RulesEnginePlugin {
    fn id(&self) -> PluginId {
        PluginId("hexorder-rules-engine")
    }

    fn plugin_name(&self) -> &'static str {
        "RulesEngine"
    }

    fn build(&self, app: &mut App) {
        app.init_resource::<ValidMoveSet>();
        app.init_resource::<InfluenceRuleRegistry>();
        app.init_resource::<InfluenceMap>();
        app.init_resource::<StackingRule>();
        app.add_systems(
            Update,
            systems::compute_valid_moves.run_if(in_state(AppScreen::Editor)),
        );
    }
}

impl Plugin for RulesEnginePlugin {
    fn build(&self, app: &mut App) {
        HexorderPlugin::build(self, app);
    }
}
