//! Bevy systems for map generation.

use std::collections::HashMap;

use bevy::prelude::*;

use hexorder_contracts::game_system::{EntityData, EntityRole, EntityTypeRegistry, PropertyValue};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};

use hexorder_contracts::map_gen::{GenerateMap, MapGenParams};

use super::biome::{apply_biome_table_indexed, validate_biome_table};
use super::components::BiomeTable;
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

    // Apply biome table to get entry indices per position.
    let biome_indices = apply_biome_table_indexed(&heightmap, &biome_table);

    // Collect BoardPosition entity types in a stable order for ordinal mapping.
    // The Nth biome entry maps to the Nth BoardPosition type.
    let board_types: Vec<_> = registry.types_by_role(EntityRole::BoardPosition);

    if board_types.is_empty() {
        warn!("No BoardPosition entity types in registry -- skipping map generation");
        commands.remove_resource::<GenerateMap>();
        return;
    }

    // Write EntityData to tiles.
    for (pos, mut entity_data) in &mut tiles {
        if let Some(&biome_index) = biome_indices.get(pos) {
            // Clamp index to available types (wraps if more biome entries than types).
            let type_index = biome_index % board_types.len();
            let entity_type = board_types[type_index];

            let new_properties: HashMap<_, _> = entity_type
                .properties
                .iter()
                .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
                .collect();

            entity_data.entity_type_id = entity_type.id;
            entity_data.properties = new_properties;
        }
    }

    // Remove the marker to prevent re-running.
    commands.remove_resource::<GenerateMap>();
}
