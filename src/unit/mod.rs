//! Unit feature plugin.
//!
//! Handles placing user-defined unit types onto hex tiles, unit selection,
//! movement, deletion, and visual sync. Unit type definitions come from
//! the game_system plugin's registry.

use bevy::prelude::*;

mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages unit materials, placement, movement, and visual sync.
#[derive(Debug)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_unit_visuals)
            .add_systems(
                Update,
                (
                    systems::delete_selected_unit,
                    systems::sync_unit_materials,
                    systems::sync_unit_visuals,
                )
                    .chain(),
            )
            .add_observer(systems::handle_unit_placement)
            .add_observer(systems::handle_unit_interaction);
    }
}
