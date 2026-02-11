//! Game System feature plugin.
//!
//! Provides the Game System container, cell type registry, unit type registry,
//! and their active-selection resources. This is the root design artifact that
//! holds all user-defined definitions for a hex board game system.

use bevy::prelude::*;

use crate::contracts::game_system::{ActiveCellType, ActiveUnitType, SelectedUnit};

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that initializes the Game System, registries, and active-selection
/// resources at build time so they are immediately available to downstream plugins.
#[derive(Debug)]
pub struct GameSystemPlugin;

impl Plugin for GameSystemPlugin {
    fn build(&self, app: &mut App) {
        let cell_registry = systems::create_cell_type_registry();
        let first_cell_id = cell_registry.first().map(|ct| ct.id);

        let unit_registry = systems::create_unit_type_registry();
        let first_unit_id = unit_registry.first().map(|ut| ut.id);

        app.insert_resource(systems::create_game_system());
        app.insert_resource(cell_registry);
        app.insert_resource(ActiveCellType {
            cell_type_id: first_cell_id,
        });
        app.insert_resource(unit_registry);
        app.insert_resource(ActiveUnitType {
            unit_type_id: first_unit_id,
        });
        app.insert_resource(SelectedUnit::default());
    }
}
