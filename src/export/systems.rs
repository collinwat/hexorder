//! Systems for the export plugin.

use std::path::Path;

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task, block_on, poll_once};

use hexorder_contracts::editor_ui::{ToastEvent, ToastKind};
use hexorder_contracts::game_system::{EntityData, EntityTypeRegistry, UnitInstance};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use hexorder_contracts::shortcuts::{CommandExecutedEvent, CommandId};

use super::counter_sheet::PrintAndPlayExporter;
use super::hex_map::HexMapExporter;
use super::{ExportData, ExportError, ExportTarget, collect_export_data};

// ---------------------------------------------------------------------------
// Async Export Dialog
// ---------------------------------------------------------------------------

/// Holds the in-flight async folder picker and the export data collected
/// before the dialog was opened. Only one export dialog at a time.
#[derive(Resource)]
pub(crate) struct PendingExport {
    pub data: ExportData,
    pub task: Task<Option<std::path::PathBuf>>,
}

// Manual Debug impl because Task<T> does not implement Debug.
impl std::fmt::Debug for PendingExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingExport")
            .field("data", &self.data)
            .field("task", &"<Task>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Handles the export command. Collects game state from ECS, spawns an
/// async folder picker dialog, and stores the pending export. The actual
/// file writing happens in `poll_pending_export` when the dialog completes.
pub(crate) fn handle_export_command(trigger: On<CommandExecutedEvent>, mut commands: Commands) {
    if trigger.command_id != CommandId("file.export_pnp") {
        return;
    }

    commands.queue(|world: &mut World| {
        // Guard: only one export dialog at a time.
        if world.get_resource::<PendingExport>().is_some() {
            return;
        }

        // Not in Editor state — grid not yet initialized.
        if world.get_resource::<HexGridConfig>().is_none() {
            return;
        }

        let tiles: Vec<_> = world
            .query_filtered::<(&HexPosition, &EntityData), (With<HexTile>, Without<UnitInstance>)>()
            .iter(world)
            .map(|(pos, data)| (*pos, data.clone()))
            .collect();
        let tokens: Vec<_> = world
            .query_filtered::<(&HexPosition, &EntityData), With<UnitInstance>>()
            .iter(world)
            .map(|(pos, data)| (*pos, data.clone()))
            .collect();

        let entity_types = world.resource::<EntityTypeRegistry>();
        let grid_config = world.resource::<HexGridConfig>();
        let export_data = collect_export_data(entity_types, grid_config, &tiles, &tokens);

        info!(
            "Export: collected {} entity types, {} tiles, {} tokens (map radius {})",
            export_data.entity_types.len(),
            export_data.board_entities.len(),
            export_data.token_entities.len(),
            export_data.grid_config.map_radius,
        );

        let task = IoTaskPool::get().spawn(async move {
            let dialog = rfd::AsyncFileDialog::new().set_title("Export Print-and-Play PDFs");
            dialog.pick_folder().await.map(|h| h.path().to_path_buf())
        });

        world.insert_resource(PendingExport {
            data: export_data,
            task,
        });
    });
}

/// Polls the in-flight export folder picker each frame.
///
/// Uses `block_on(poll_once(...))` which is zero-cost when the future is
/// not yet ready — it returns `None` immediately without blocking.
///
/// When the task completes, runs the exporters and writes files to the
/// chosen directory.
pub(crate) fn poll_pending_export(world: &mut World) {
    let result = {
        let Some(mut pending) = world.get_resource_mut::<PendingExport>() else {
            return;
        };

        let Some(result) = block_on(poll_once(&mut pending.task)) else {
            return; // Task still in progress.
        };

        result
    }; // pending borrow released here.

    let pending = world
        .remove_resource::<PendingExport>()
        .expect("checked above");

    let Some(output_dir) = result else {
        return; // User cancelled.
    };

    run_export(&pending.data, &output_dir, world);
}

/// Run all exporters and write output files to the given directory.
fn run_export(data: &ExportData, output_dir: &Path, world: &mut World) {
    let exporters: Vec<Box<dyn ExportTarget>> = vec![
        Box::new(PrintAndPlayExporter::default()),
        Box::new(HexMapExporter::default()),
    ];

    let mut written = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for exporter in &exporters {
        match exporter.export(data) {
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
                if matches!(e, ExportError::EmptyGameSystem) {
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
        world.trigger(ToastEvent {
            message: msg,
            kind: if written > 0 {
                ToastKind::Success
            } else {
                ToastKind::Info
            },
        });
    } else {
        world.trigger(ToastEvent {
            message: format!("Export errors: {}", errors.join("; ")),
            kind: ToastKind::Error,
        });
    }
}
