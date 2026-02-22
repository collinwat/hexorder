//! Shared Persistence types. See `docs/contracts/persistence.md`.
//!
//! Types for saving and loading game system definitions and board state
//! to `.hexorder` (RON) files.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::game_system::{
    EntityTypeRegistry, EnumRegistry, GameSystem, PropertyValue, StructRegistry, TypeId,
};
use super::hex_grid::HexPosition;
use super::mechanics::{CombatModifierRegistry, CombatResultsTable, TurnStructure};
use super::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};

/// Current file format version. Increment when the schema changes.
pub const FORMAT_VERSION: u32 = 5;

// ---------------------------------------------------------------------------
// Application State
// ---------------------------------------------------------------------------

/// Application screen state. Controls which systems run.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum AppScreen {
    /// Startup screen — create new or open existing project.
    #[default]
    Launcher,
    /// Main editor — all editing tools active.
    Editor,
    /// Play mode — step through turns, resolve combat.
    Play,
}

/// Tool-level session state for the currently open project.
/// Initialized on `NewProjectEvent` and `LoadRequestEvent`.
/// Reset on `CloseProjectEvent` / return-to-launcher.
#[derive(Resource, Debug, Clone)]
pub struct Workspace {
    /// Human-readable project name (display only, not an identifier).
    pub name: String,
    /// Path to the last-saved file. `None` if never saved.
    pub file_path: Option<PathBuf>,
    /// Whether the project has unsaved changes.
    /// TODO(#111): not actively tracked yet — see unsaved changes issue.
    pub dirty: bool,
    /// Active workspace preset identifier (e.g. `map_editing`, `playtesting`).
    /// Empty string means default (Map Editing).
    pub workspace_preset: String,
    /// Editor font size (points). Default 15.0, range 10–24.
    pub font_size_base: f32,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_path: None,
            dirty: false,
            workspace_preset: String::new(),
            font_size_base: 15.0,
        }
    }
}

/// Temporary resource for deferred board state application after load.
#[derive(Resource, Debug)]
pub struct PendingBoardLoad {
    pub tiles: Vec<TileSaveData>,
    pub units: Vec<UnitSaveData>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Triggers a save operation.
#[derive(Event, Debug)]
pub struct SaveRequestEvent {
    /// If true, always show the file dialog even if a path is known.
    pub save_as: bool,
}

/// Triggers a load operation.
#[derive(Event, Debug)]
pub struct LoadRequestEvent;

/// Triggers creation of a new empty project with the given name.
#[derive(Event, Debug)]
pub struct NewProjectEvent {
    /// Display name for the new workspace.
    pub name: String,
}

/// Triggers closing the current project and returning to the launcher.
#[derive(Event, Debug)]
pub struct CloseProjectEvent;

// ---------------------------------------------------------------------------
// File Container
// ---------------------------------------------------------------------------

/// Top-level file container for a saved game system + board state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSystemFile {
    /// File format version (for future migration).
    pub format_version: u32,
    /// Workspace display name. Empty in v2 files (derived from filename on load).
    #[serde(default)]
    pub name: String,
    /// The game system definitions.
    pub game_system: GameSystem,
    /// Entity type registry.
    pub entity_types: EntityTypeRegistry,
    /// Enum definitions registry (0.7.0).
    pub enums: EnumRegistry,
    /// Struct definitions registry (0.7.0).
    pub structs: StructRegistry,
    /// Ontology data.
    pub concepts: ConceptRegistry,
    pub relations: RelationRegistry,
    pub constraints: ConstraintRegistry,
    /// Turn structure definition (0.9.0).
    #[serde(default)]
    pub turn_structure: TurnStructure,
    /// Combat Results Table (0.9.0).
    #[serde(default)]
    pub combat_results_table: CombatResultsTable,
    /// Combat modifier definitions (0.9.0).
    #[serde(default)]
    pub combat_modifiers: CombatModifierRegistry,
    /// Board configuration.
    pub map_radius: u32,
    /// Board state: per-tile cell data.
    pub tiles: Vec<TileSaveData>,
    /// Board state: placed units.
    pub units: Vec<UnitSaveData>,
    /// Active workspace preset identifier (v4+, e.g. `map_editing`).
    #[serde(default)]
    pub workspace_preset: String,
    /// Editor font size in points (v5+). Default 15.0.
    #[serde(default = "default_font_size")]
    pub font_size_base: f32,
}

fn default_font_size() -> f32 {
    15.0
}

/// Serialized form of a hex tile's cell data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSaveData {
    pub position: HexPosition,
    pub entity_type_id: TypeId,
    pub properties: HashMap<TypeId, PropertyValue>,
}

/// Serialized form of a placed unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSaveData {
    pub position: HexPosition,
    pub entity_type_id: TypeId,
    pub properties: HashMap<TypeId, PropertyValue>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Error type for save/load operations.
#[derive(Debug)]
pub enum PersistenceError {
    /// File system error.
    Io(std::io::Error),
    /// RON serialization failure.
    Serialize(ron::Error),
    /// RON deserialization failure.
    Deserialize(ron::error::SpannedError),
    /// File was written by a newer version of hexorder.
    UnsupportedVersion { found: u32, max: u32 },
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Serialize(e) => write!(f, "serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "deserialization error: {e}"),
            Self::UnsupportedVersion { found, max } => {
                write!(
                    f,
                    "unsupported file format version {found} (max supported: {max})"
                )
            }
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_default_has_empty_name_and_no_path() {
        let ws = Workspace::default();
        assert!(ws.name.is_empty());
        assert!(ws.file_path.is_none());
        assert!(!ws.dirty);
    }
}
