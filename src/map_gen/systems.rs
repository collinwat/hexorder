//! Bevy systems for map generation.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::contracts::game_system::{EntityData, EntityRole, EntityTypeRegistry, PropertyValue};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};

use super::biome::{apply_biome_table, validate_biome_table};
use super::components::{BiomeTable, GenerateMap, MapGenParams};
use super::heightmap::generate_heightmap;

/// System that runs when `GenerateMap` resource is present.
/// Generates a heightmap, applies the biome table, and writes
/// `EntityData` to all tile entities. Removes `GenerateMap` when done.
#[allow(clippy::too_many_arguments)]
pub fn run_generation(
    mut commands: Commands,
    params: Res<MapGenParams>,
    biome_table: Res<BiomeTable>,
    grid_config: Res<HexGridConfig>,
    registry: Res<EntityTypeRegistry>,
    generate: Option<Res<GenerateMap>>,
    mut tiles: Query<(&HexPosition, &mut EntityData), With<HexTile>>,
) {
    // Only run when GenerateMap marker resource is present.
    if generate.is_none() {
        return;
    }

    // Validate the biome table before using it.
    if let Err(err) = validate_biome_table(&biome_table) {
        warn!("Biome table validation failed: {err} -- skipping map generation");
        commands.remove_resource::<GenerateMap>();
        return;
    }

    // Collect all tile positions.
    let positions: Vec<HexPosition> = tiles.iter().map(|(pos, _)| *pos).collect();

    if positions.is_empty() {
        commands.remove_resource::<GenerateMap>();
        return;
    }

    // Generate heightmap.
    let heightmap = generate_heightmap(&params, &positions, &grid_config.layout);

    // Apply biome table to get terrain names.
    let terrain_names = apply_biome_table(&heightmap, &biome_table);

    // Build name-to-TypeId lookup for BoardPosition entity types.
    let name_to_type: HashMap<&str, _> = registry
        .types_by_role(EntityRole::BoardPosition)
        .into_iter()
        .map(|et| (et.name.as_str(), et))
        .collect();

    // Write EntityData to tiles.
    for (pos, mut entity_data) in &mut tiles {
        if let Some(terrain_name) = terrain_names.get(pos) {
            if let Some(entity_type) = name_to_type.get(terrain_name.as_str()) {
                let new_properties: HashMap<_, _> = entity_type
                    .properties
                    .iter()
                    .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
                    .collect();

                entity_data.entity_type_id = entity_type.id;
                entity_data.properties = new_properties;
            } else {
                warn!(
                    "Biome table references terrain '{}' not found in registry",
                    terrain_name
                );
            }
        }
    }

    // Remove the marker to prevent re-running.
    commands.remove_resource::<GenerateMap>();
}
