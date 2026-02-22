//! Systems for the export plugin.

use bevy::prelude::*;

use crate::contracts::game_system::{EntityData, EntityTypeRegistry, UnitInstance};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use crate::contracts::shortcuts::{CommandExecutedEvent, CommandId};

use super::collect_export_data;

/// Handles the export command. Collects game state from ECS and delegates
/// to the appropriate export target.
///
/// Currently logs the collected data summary. The actual PDF generation
/// (Scope 2-3) will replace the log with file writing.
#[allow(clippy::type_complexity)]
pub(crate) fn handle_export_command(
    trigger: On<CommandExecutedEvent>,
    entity_types: Res<EntityTypeRegistry>,
    grid_config: Res<HexGridConfig>,
    tile_query: Query<(&HexPosition, &EntityData), (With<HexTile>, Without<UnitInstance>)>,
    token_query: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
) {
    if trigger.command_id != CommandId("file.export_pnp") {
        return;
    }

    let tiles: Vec<_> = tile_query
        .iter()
        .map(|(pos, data)| (*pos, data.clone()))
        .collect();
    let tokens: Vec<_> = token_query
        .iter()
        .map(|(pos, data)| (*pos, data.clone()))
        .collect();

    let export_data = collect_export_data(&entity_types, &grid_config, &tiles, &tokens);

    info!(
        "Export: collected {} entity types, {} tiles, {} tokens (map radius {})",
        export_data.entity_types.len(),
        export_data.board_entities.len(),
        export_data.token_entities.len(),
        export_data.grid_config.map_radius,
    );
}
