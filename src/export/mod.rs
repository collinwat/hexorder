//! Export plugin.
//!
//! Provides a generic export pipeline for getting game system designs out of
//! Hexorder. The `ExportTarget` trait defines the interface; implementations
//! produce specific output formats (PDF, JSON, etc.).
//!
//! The first export target is print-and-play PDF (counter sheets + hex maps).

use bevy::prelude::*;

use hexorder_contracts::game_system::{EntityData, EntityType, EntityTypeRegistry};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition};
use hexorder_contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};

pub(crate) mod counter_sheet;
pub(crate) mod hex_map;
mod systems;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Export Trait and Data Types
// ---------------------------------------------------------------------------

/// Snapshot of the current game state, collected from ECS resources and queries.
/// Exporters receive this immutable snapshot — they never access the ECS directly.
#[derive(Debug, Clone)]
pub struct ExportData {
    /// All entity type definitions from the registry.
    pub entity_types: Vec<EntityType>,
    /// Board position entities (hex tiles) with their position and data.
    pub board_entities: Vec<(HexPosition, EntityData)>,
    /// Token entities (units) with their position and data.
    pub token_entities: Vec<(HexPosition, EntityData)>,
    /// Grid configuration (layout, map radius).
    pub grid_config: GridSnapshot,
}

/// Minimal grid configuration snapshot (avoids carrying non-Clone Bevy types).
#[derive(Debug, Clone)]
pub struct GridSnapshot {
    /// Map radius in hex tiles from center.
    pub map_radius: u32,
    /// Whether the layout is pointy-top (true) or flat-top (false).
    pub pointy_top: bool,
}

/// A single file produced by an export target.
#[derive(Debug, Clone)]
pub struct ExportFile {
    /// Suggested filename without extension (e.g., "counter-sheet").
    pub name: String,
    /// File extension without dot (e.g., "pdf").
    pub extension: String,
    /// Raw file contents.
    pub data: Vec<u8>,
}

/// Output from an export operation — one or more files.
#[derive(Debug, Clone)]
pub struct ExportOutput {
    pub files: Vec<ExportFile>,
}

/// Errors that can occur during export.
#[derive(Debug)]
pub enum ExportError {
    /// No data to export (empty game system).
    EmptyGameSystem,
    /// The export target encountered an error producing output.
    GenerationFailed(String),
    /// I/O error writing files to disk.
    IoError(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyGameSystem => write!(f, "Nothing to export — game system is empty"),
            Self::GenerationFailed(msg) => write!(f, "Export failed: {msg}"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Trait for export targets. Each implementation produces a different output format.
///
/// Exporters are stateless — they receive an `ExportData` snapshot and produce
/// `ExportOutput`. Configuration (page size, counter size, etc.) is carried on
/// the implementing struct.
pub trait ExportTarget: Send + Sync {
    /// Human-readable name of this export format (e.g., "Print-and-Play PDF").
    fn name(&self) -> &str;

    /// File extension this target produces (e.g., "pdf").
    #[allow(dead_code)]
    fn extension(&self) -> &str;

    /// Export the game state snapshot to one or more files.
    fn export(&self, data: &ExportData) -> Result<ExportOutput, ExportError>;
}

// ---------------------------------------------------------------------------
// Data Collection
// ---------------------------------------------------------------------------

/// Collect an `ExportData` snapshot from the current ECS state.
pub(crate) fn collect_export_data(
    entity_types: &EntityTypeRegistry,
    grid_config: &HexGridConfig,
    tiles: &[(HexPosition, EntityData)],
    tokens: &[(HexPosition, EntityData)],
) -> ExportData {
    ExportData {
        entity_types: entity_types.types.clone(),
        board_entities: tiles.to_vec(),
        token_entities: tokens.to_vec(),
        grid_config: GridSnapshot {
            map_radius: grid_config.map_radius,
            pointy_top: grid_config.layout.orientation == hexx::HexOrientation::Pointy,
        },
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that provides the export pipeline.
///
/// Registers export commands in the shortcut registry and handles
/// `CommandExecutedEvent` to trigger exports.
#[derive(Debug)]
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world_mut().resource_mut::<ShortcutRegistry>();
        register_shortcuts(&mut registry);

        app.add_observer(systems::handle_export_command);
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    registry.register(CommandEntry {
        id: CommandId("file.export_pnp"),
        name: "Export Print-and-Play".to_string(),
        description: "Export counter sheets and hex map as PDF".to_string(),
        bindings: vec![KeyBinding::new(
            bevy::input::keyboard::KeyCode::KeyE,
            Modifiers::CMD_SHIFT,
        )],
        category: CommandCategory::File,
        continuous: false,
    });
}
