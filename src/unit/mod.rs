//! Unit plugin.
//!
//! Handles placing user-defined unit types onto hex tiles, unit selection,
//! movement, deletion, and visual sync. Unit type definitions come from
//! the `game_system` plugin's registry.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages unit materials, placement, movement, and visual sync.
#[derive(Debug)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreen::Editor), systems::setup_unit_visuals)
            .add_systems(
                Update,
                (
                    systems::delete_selected_unit,
                    systems::sync_unit_materials,
                    systems::sync_unit_visuals,
                )
                    .chain()
                    .run_if(in_state(AppScreen::Editor).or(in_state(AppScreen::Play))),
            )
            .add_observer(systems::handle_unit_placement)
            .add_observer(systems::handle_unit_interaction);
    }
}
