//! Hex grid plugin.
//!
//! Spawns a hexagonal grid on the XZ ground plane, handles tile selection
//! via mouse click, and provides hover feedback.

use bevy::prelude::*;
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};

use crate::contracts::persistence::AppScreen;

mod components;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that spawns and manages the hex grid.
#[derive(Debug)]
pub struct HexGridPlugin;

impl Plugin for HexGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppScreen::Editor),
            (
                systems::setup_grid_config,
                systems::setup_materials,
                systems::spawn_grid,
                systems::setup_indicators,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                systems::update_hover.run_if(not(egui_wants_any_pointer_input)),
                systems::handle_click.run_if(not(egui_wants_any_pointer_input)),
                systems::deselect_on_escape.run_if(not(egui_wants_any_keyboard_input)),
                systems::update_indicators,
                systems::sync_move_overlays,
            )
                .chain()
                .run_if(in_state(AppScreen::Editor)),
        );
    }
}
