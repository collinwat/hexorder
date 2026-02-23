//! Systems for the export plugin.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::contracts::editor_ui::{ToastEvent, ToastKind};
use crate::contracts::game_system::{EntityData, EntityTypeRegistry, UnitInstance};
use crate::contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use crate::contracts::shortcuts::{CommandExecutedEvent, CommandId};

use super::counter_sheet::PrintAndPlayExporter;
use super::hex_map::HexMapExporter;
use super::{ExportTarget, collect_export_data};

/// Handles the export command. Collects game state from ECS, runs both
/// exporters (counter sheet + hex map), shows a save dialog, and writes
/// the PDF files to the chosen directory.
#[allow(clippy::type_complexity)]
pub(crate) fn handle_export_command(
    trigger: On<CommandExecutedEvent>,
    entity_types: Res<EntityTypeRegistry>,
    grid_config: Option<Res<HexGridConfig>>,
    tile_query: Query<(&HexPosition, &EntityData), (With<HexTile>, Without<UnitInstance>)>,
    token_query: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    mut commands: Commands,
) {
    if trigger.command_id != CommandId("file.export_pnp") {
        return;
    }

    let Some(grid_config) = grid_config else {
        return; // Not in Editor state — grid not yet initialized.
    };

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

    // Ask user for output directory.
    let dialog = rfd::FileDialog::new().set_title("Export Print-and-Play PDFs");
    let output_dir = dialog.pick_folder();
    // Native file dialogs take over the macOS event loop, so key-up events
    // that occur while the dialog is open are never delivered to Bevy.
    // Reset keyboard state to prevent stuck keys after the dialog closes.
    commands.queue(|world: &mut World| {
        if let Some(mut keys) = world.get_resource_mut::<ButtonInput<KeyCode>>() {
            keys.reset_all();
        }
    });
    let Some(output_dir) = output_dir else {
        return; // User cancelled.
    };

    // Run exporters and collect results.
    let exporters: Vec<Box<dyn ExportTarget>> = vec![
        Box::new(PrintAndPlayExporter::default()),
        Box::new(HexMapExporter::default()),
    ];

    let mut written = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for exporter in &exporters {
        match exporter.export(&export_data) {
            Ok(output) => {
                for file in &output.files {
                    let path = output_dir.join(format!("{}.{}", file.name, file.extension));
                    if let Err(e) = std::fs::write(&path, &file.data) {
                        errors.push(format!("{}: {e}", file.name));
                    } else {
                        written += 1;
                        info!("Exported {} to {}", file.name, path.display());
                    }
                }
            }
            Err(e) => {
                let name = exporter.name();
                // Empty game system errors are expected for partial exports.
                if matches!(e, super::ExportError::EmptyGameSystem) {
                    info!("Skipped {name}: no data to export");
                } else {
                    errors.push(format!("{name}: {e}"));
                }
            }
        }
    }

    if errors.is_empty() {
        let msg = if written > 0 {
            format!("Exported {written} file(s) to {}", output_dir.display())
        } else {
            "Nothing to export — add tokens or terrain first".to_string()
        };
        commands.trigger(ToastEvent {
            message: msg,
            kind: if written > 0 {
                ToastKind::Success
            } else {
                ToastKind::Info
            },
        });
    } else {
        commands.trigger(ToastEvent {
            message: format!("Export errors: {}", errors.join("; ")),
            kind: ToastKind::Error,
        });
    }
}
