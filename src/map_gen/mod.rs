//! Procedural hex map generation plugin.
//!
//! Generates heightmap-based terrain using layered Perlin noise and
//! a configurable biome table that maps elevation ranges to cell types.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::contracts::persistence::AppScreen;

mod biome;
mod components;
mod heightmap;
mod systems;
mod ui;

#[cfg(test)]
mod tests;

/// Plugin that provides procedural map generation.
#[derive(Debug)]
pub struct MapGenPlugin;

impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::MapGenParams>()
            .init_resource::<components::BiomeTable>()
            .init_resource::<components::MapGenPanelVisible>()
            .add_systems(
                Update,
                systems::run_generation.run_if(in_state(AppScreen::Editor)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                ui::map_gen_panel.run_if(in_state(AppScreen::Editor)),
            );
    }
}
