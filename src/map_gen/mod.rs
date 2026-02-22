//! Procedural hex map generation plugin.
//!
//! Generates heightmap-based terrain using layered Perlin noise and
//! a configurable biome table that maps elevation ranges to cell types.

use bevy::prelude::*;

use crate::contracts::persistence::AppScreen;

#[allow(dead_code)]
mod biome;
mod components;
mod heightmap;
mod systems;

#[cfg(test)]
mod tests;

/// Plugin that provides procedural map generation.
#[derive(Debug)]
pub struct MapGenPlugin;

impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::MapGenParams>()
            .init_resource::<components::BiomeTable>()
            .add_systems(
                Update,
                systems::run_generation.run_if(in_state(AppScreen::Editor)),
            );
    }
}
