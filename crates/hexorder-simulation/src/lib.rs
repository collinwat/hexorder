//! Simulation plugin.
//!
//! Hosts the `SimulationRng` resource and provides observer events
//! for die rolls and table resolutions. All types and pure functions
//! live in `hexorder_contracts::simulation`.

use bevy::prelude::*;
use hexorder_contracts::simulation::SimulationRng;
use hexorder_sdk::{HexorderPlugin, PluginId};

pub mod events;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that provides simulation primitives: seeded RNG and
/// table resolution runtime support.
#[derive(Debug)]
pub struct SimulationPlugin;

impl HexorderPlugin for SimulationPlugin {
    fn id(&self) -> PluginId {
        PluginId("hexorder-simulation")
    }

    fn plugin_name(&self) -> &'static str {
        "Simulation"
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationRng::new_random());
        app.add_observer(systems::on_die_rolled);
        app.add_observer(systems::on_table_resolved);
    }
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        HexorderPlugin::build(self, app);
    }
}
