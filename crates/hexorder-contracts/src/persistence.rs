//! Shared Persistence types. See `docs/contracts/persistence.md`.
//!
//! Types for saving and loading game system definitions and board state
//! to `.hexorder` (RON) files.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game_system::{
    EntityTypeRegistry, EnumRegistry, GameSystem, PropertyValue, StructRegistry, TypeId,
};
use crate::hex_grid::{
    HexEdgeRegistry, HexPosition, InfluenceRuleRegistry, MovementCostMatrix, StackingRule,
};
use crate::mechanics::{CombatModifierRegistry, CombatResultsTable, TurnStructure};
use crate::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};

/// Current file format version. Increment when the schema changes.
pub const FORMAT_VERSION: u32 = 6;

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
    /// Tracked via `UndoStack.is_dirty()` — see `sync_dirty_flag` system.
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
    /// Hex edge feature annotations (v6+).
    #[serde(default)]
    pub edge_features: HexEdgeRegistry,
    /// Spatial influence rules (v6+).
    #[serde(default)]
    pub influence_rules: InfluenceRuleRegistry,
    /// Stacking constraint (v6+).
    #[serde(default)]
    pub stacking_rule: StackingRule,
    /// Movement cost matrix — 2D lookup by terrain type and unit classification (v6+).
    #[serde(default)]
    pub movement_cost_matrix: MovementCostMatrix,
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

    #[test]
    fn workspace_default_font_size() {
        let ws = Workspace::default();
        assert!((ws.font_size_base - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn workspace_default_preset_is_empty() {
        let ws = Workspace::default();
        assert!(ws.workspace_preset.is_empty());
    }

    #[test]
    fn format_version_constant() {
        assert_eq!(FORMAT_VERSION, 6);
    }

    #[test]
    fn app_screen_default_is_launcher() {
        let screen = AppScreen::default();
        assert_eq!(screen, AppScreen::Launcher);
    }

    #[test]
    fn app_screen_variants_are_distinct() {
        assert_ne!(AppScreen::Launcher, AppScreen::Editor);
        assert_ne!(AppScreen::Editor, AppScreen::Play);
        assert_ne!(AppScreen::Launcher, AppScreen::Play);
    }

    #[test]
    fn save_request_event_construction() {
        let evt = SaveRequestEvent { save_as: true };
        assert!(evt.save_as);
        let evt2 = SaveRequestEvent { save_as: false };
        assert!(!evt2.save_as);
    }

    #[test]
    fn new_project_event_construction() {
        let evt = NewProjectEvent {
            name: "Test".to_string(),
        };
        assert_eq!(evt.name, "Test");
    }

    #[test]
    fn persistence_error_display_io() {
        let err = PersistenceError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        let msg = format!("{err}");
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn persistence_error_display_unsupported_version() {
        let err = PersistenceError::UnsupportedVersion { found: 99, max: 5 };
        let msg = format!("{err}");
        assert!(msg.contains("99"));
        assert!(msg.contains('5'));
    }

    #[test]
    fn persistence_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let pe: PersistenceError = io_err.into();
        assert!(matches!(pe, PersistenceError::Io(_)));
    }

    #[test]
    fn default_font_size_helper_returns_fifteen() {
        assert!((default_font_size() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn persistence_error_display_serialize() {
        // ron::Error::Message is the simplest public variant to construct.
        let err = ron::Error::Message("bad data".to_string());
        let pe = PersistenceError::Serialize(err);
        let msg = format!("{pe}");
        assert!(msg.contains("serialization error"));
        assert!(msg.contains("bad data"));
    }

    #[test]
    fn persistence_error_display_deserialize() {
        let err = ron::from_str::<i32>("not_valid_ron").unwrap_err();
        let pe = PersistenceError::Deserialize(err);
        let msg = format!("{pe}");
        assert!(msg.contains("deserialization error"));
    }

    #[test]
    fn workspace_with_custom_fields() {
        let ws = Workspace {
            name: "My Project".to_string(),
            file_path: Some(std::path::PathBuf::from("/tmp/test.ron")),
            dirty: true,
            workspace_preset: "playtesting".to_string(),
            font_size_base: 18.0,
        };
        assert!(ws.dirty);
        assert_eq!(ws.name, "My Project");
        assert_eq!(ws.file_path.unwrap().to_str().unwrap(), "/tmp/test.ron");
        assert_eq!(ws.workspace_preset, "playtesting");
    }

    #[test]
    fn tile_save_data_construction() {
        let data = TileSaveData {
            position: HexPosition { q: 1, r: -1 },
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        };
        assert_eq!(data.position.q, 1);
    }

    #[test]
    fn unit_save_data_construction() {
        let data = UnitSaveData {
            position: HexPosition { q: 0, r: 0 },
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        };
        assert_eq!(data.position.r, 0);
    }

    #[test]
    fn pending_board_load_construction() {
        let load = PendingBoardLoad {
            tiles: vec![],
            units: vec![],
        };
        assert!(load.tiles.is_empty());
        assert!(load.units.is_empty());
    }
}
