//! Bevy systems for map generation.

use std::collections::HashMap;

use bevy::prelude::*;

use hexorder_contracts::game_system::{EntityData, EntityRole, EntityTypeRegistry, PropertyValue};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use hexorder_contracts::map_gen::{GenerateMap, MapGenParams};
use hexorder_contracts::undo_redo::{CompoundCommand, SetTerrainCommand, UndoStack};

use super::biome::{apply_biome_table_indexed, validate_biome_table};
use super::components::BiomeTable;
use super::heightmap::generate_heightmap;

/// System that runs when `GenerateMap` resource is present.
/// Generates a heightmap, applies the biome table, and writes
/// `EntityData` to all tile entities. Records a compound undo command
/// so the entire generation can be reversed. Removes `GenerateMap` when done.
#[allow(clippy::too_many_arguments)]
pub fn run_generation(
    mut commands: Commands,
    params: Res<MapGenParams>,
    biome_table: Res<BiomeTable>,
    grid_config: Res<HexGridConfig>,
    registry: Res<EntityTypeRegistry>,
    generate: Option<Res<GenerateMap>>,
    mut tiles: Query<(Entity, &HexPosition, &mut EntityData), With<HexTile>>,
    mut undo_stack: Option<ResMut<UndoStack>>,
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
    let positions: Vec<HexPosition> = tiles.iter().map(|(_, pos, _)| *pos).collect();

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

    // Write EntityData to tiles, collecting undo commands for each changed tile.
    let mut tile_commands: Vec<Box<dyn hexorder_contracts::undo_redo::UndoableCommand>> =
        Vec::new();

    for (entity, pos, mut entity_data) in &mut tiles {
        if let Some(&biome_index) = biome_indices.get(pos) {
            // Clamp index to available types (wraps if more biome entries than types).
            let type_index = biome_index % board_types.len();
            let entity_type = board_types[type_index];

            let new_properties: HashMap<_, _> = entity_type
                .properties
                .iter()
                .map(|pd| (pd.id, PropertyValue::default_for(&pd.property_type)))
                .collect();

            // Snapshot old state before mutation.
            let old_type_id = entity_data.entity_type_id;
            let old_properties = entity_data.properties.clone();

            entity_data.entity_type_id = entity_type.id;
            entity_data.properties = new_properties.clone();

            tile_commands.push(Box::new(SetTerrainCommand {
                entity,
                old_type_id,
                old_properties,
                new_type_id: entity_type.id,
                new_properties,
                label: format!("Generate terrain at ({}, {})", pos.q, pos.r),
            }));
        }
    }

    // Record compound undo command if any tiles were changed.
    if !tile_commands.is_empty() {
        if let Some(ref mut stack) = undo_stack {
            stack.record(Box::new(CompoundCommand {
                commands: tile_commands,
                label: "Generate Map".to_string(),
            }));
        }
    }

    // Remove the marker to prevent re-running.
    commands.remove_resource::<GenerateMap>();
}
