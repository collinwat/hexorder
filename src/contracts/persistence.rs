//! Shared Persistence types. See `docs/contracts/persistence.md`.
//!
//! Types for saving and loading game system definitions and board state
//! to `.hexorder` (RON) files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::game_system::{EntityTypeRegistry, GameSystem, PropertyValue, TypeId};
use super::hex_grid::HexPosition;
use super::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};

/// Current file format version. Increment when the schema changes.
pub const FORMAT_VERSION: u32 = 1;

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
}

/// Tracks the path to the currently open file.
#[derive(Resource, Debug, Default, Reflect)]
pub struct CurrentFilePath {
    pub path: Option<PathBuf>,
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

/// Triggers creation of a new empty project.
#[derive(Event, Debug)]
pub struct NewProjectEvent;

// ---------------------------------------------------------------------------
// File Container
// ---------------------------------------------------------------------------

/// Top-level file container for a saved game system + board state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSystemFile {
    /// File format version (for future migration).
    pub format_version: u32,
    /// The game system definitions.
    pub game_system: GameSystem,
    /// Entity type registry.
    pub entity_types: EntityTypeRegistry,
    /// Ontology data.
    pub concepts: ConceptRegistry,
    pub relations: RelationRegistry,
    pub constraints: ConstraintRegistry,
    /// Board configuration.
    pub map_radius: u32,
    /// Board state: per-tile cell data.
    pub tiles: Vec<TileSaveData>,
    /// Board state: placed units.
    pub units: Vec<UnitSaveData>,
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
// File I/O
// ---------------------------------------------------------------------------

/// Serialize a `GameSystemFile` to RON and write to disk.
pub fn save_to_file(path: &Path, data: &GameSystemFile) -> Result<(), PersistenceError> {
    let config = ron::ser::PrettyConfig::default();
    let ron_str = ron::ser::to_string_pretty(data, config).map_err(PersistenceError::Serialize)?;
    std::fs::write(path, ron_str)?;
    Ok(())
}

/// Read a RON file from disk and deserialize to `GameSystemFile`.
pub fn load_from_file(path: &Path) -> Result<GameSystemFile, PersistenceError> {
    let contents = std::fs::read_to_string(path)?;
    let file: GameSystemFile = ron::from_str(&contents).map_err(PersistenceError::Deserialize)?;

    if file.format_version > FORMAT_VERSION {
        return Err(PersistenceError::UnsupportedVersion {
            found: file.format_version,
            max: FORMAT_VERSION,
        });
    }

    Ok(file)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, GameSystem, TypeId,
    };

    /// Helper: create a minimal `GameSystemFile` for testing.
    fn test_file() -> GameSystemFile {
        let type_id = TypeId::new();
        GameSystemFile {
            format_version: FORMAT_VERSION,
            game_system: GameSystem {
                id: "test-id".to_string(),
                version: "0.1.0".to_string(),
            },
            entity_types: EntityTypeRegistry {
                types: vec![EntityType {
                    id: type_id,
                    name: "Plains".to_string(),
                    role: EntityRole::BoardPosition,
                    color: bevy::color::Color::srgb(0.6, 0.8, 0.4),
                    properties: Vec::new(),
                }],
            },
            concepts: ConceptRegistry::default(),
            relations: RelationRegistry::default(),
            constraints: ConstraintRegistry::default(),
            map_radius: 10,
            tiles: vec![TileSaveData {
                position: HexPosition::new(0, 0),
                entity_type_id: type_id,
                properties: HashMap::new(),
            }],
            units: Vec::new(),
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join("hexorder_test_round_trip.hexorder");
        let data = test_file();

        save_to_file(&dir, &data).expect("save should succeed");
        let loaded = load_from_file(&dir).expect("load should succeed");

        assert_eq!(loaded.format_version, FORMAT_VERSION);
        assert_eq!(loaded.game_system.id, "test-id");
        assert_eq!(loaded.entity_types.types.len(), 1);
        assert_eq!(loaded.entity_types.types[0].name, "Plains");
        assert_eq!(loaded.map_radius, 10);
        assert_eq!(loaded.tiles.len(), 1);
        assert_eq!(loaded.tiles[0].position, HexPosition::new(0, 0));
        assert!(loaded.units.is_empty());

        // Clean up.
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn load_nonexistent_file_returns_io_error() {
        let result = load_from_file(Path::new("/nonexistent/path.hexorder"));
        assert!(matches!(result, Err(PersistenceError::Io(_))));
    }

    #[test]
    fn load_malformed_ron_returns_deserialize_error() {
        let dir = std::env::temp_dir().join("hexorder_test_malformed.hexorder");
        std::fs::write(&dir, "this is not valid RON").expect("write");
        let result = load_from_file(&dir);
        assert!(matches!(result, Err(PersistenceError::Deserialize(_))));
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn load_unsupported_version_returns_error() {
        let dir = std::env::temp_dir().join("hexorder_test_version.hexorder");
        let mut data = test_file();
        data.format_version = 999;

        // Write the future-version file manually.
        let config = ron::ser::PrettyConfig::default();
        let ron_str = ron::ser::to_string_pretty(&data, config).expect("serialize");
        std::fs::write(&dir, ron_str).expect("write");

        let result = load_from_file(&dir);
        assert!(matches!(
            result,
            Err(PersistenceError::UnsupportedVersion { found: 999, max: 1 })
        ));
        let _ = std::fs::remove_file(&dir);
    }
}
