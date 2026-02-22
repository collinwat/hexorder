//! Storage provider implementation and configuration resolution.
//!
//! Contains `FilesystemProvider` (the default storage backend) and
//! `resolve_storage_config()` which determines the base directory
//! from compile-time feature flags.

#[cfg(all(feature = "xdg", feature = "macos"))]
compile_error!("Features `xdg` and `macos` are mutually exclusive.");

use std::path::{Path, PathBuf};

use crate::contracts::persistence::{FORMAT_VERSION, GameSystemFile, PersistenceError};
use crate::contracts::storage::{ProjectEntry, StorageConfig, StorageProvider, StorageSource};

// ---------------------------------------------------------------------------
// Configuration Resolution
// ---------------------------------------------------------------------------

/// Resolve storage configuration from compile-time feature flags.
///
/// Resolution order:
/// 1. `macos` feature → `~/Library/Application Support/hexorder/`
/// 2. `xdg` feature → `$XDG_DATA_HOME/hexorder/`
/// 3. Default → `{CARGO_MANIFEST_DIR}/.state/{CARGO_PKG_VERSION}`
///
/// Callers that need a custom path can insert a [`StorageConfig`] resource
/// before adding `PersistencePlugin` — the plugin will use it instead.
pub fn resolve_storage_config() -> StorageConfig {
    #[cfg(feature = "macos")]
    {
        if let Some(data) = dirs::data_dir() {
            return StorageConfig {
                base_dir: data.join("hexorder"),
                source: StorageSource::MacOs,
            };
        }
    }

    #[cfg(feature = "xdg")]
    {
        if let Some(data) = dirs::data_dir() {
            return StorageConfig {
                base_dir: data.join("hexorder"),
                source: StorageSource::Xdg,
            };
        }
    }

    // Default: project-local dev path, anchored to the manifest directory
    // (stable across cwd changes and per-worktree).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    StorageConfig {
        base_dir: manifest_dir.join(".state").join(version),
        source: StorageSource::ProjectLocal,
    }
}

// ---------------------------------------------------------------------------
// Filesystem Provider
// ---------------------------------------------------------------------------

/// Storage backend that reads and writes `.hexorder` files on the local
/// filesystem, using RON serialization.
#[derive(Debug)]
pub struct FilesystemProvider {
    config: StorageConfig,
}

impl FilesystemProvider {
    /// Create a new filesystem provider with the given configuration.
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }
}

impl StorageProvider for FilesystemProvider {
    fn save(&self, name: &str, data: &GameSystemFile) -> Result<PathBuf, PersistenceError> {
        let sanitized = super::systems::sanitize_filename(name);
        let path = self.config.base_dir.join(format!("{sanitized}.hexorder"));
        std::fs::create_dir_all(&self.config.base_dir)?;
        self.save_at(&path, data)?;
        Ok(path)
    }

    fn save_at(&self, path: &Path, data: &GameSystemFile) -> Result<(), PersistenceError> {
        let config = ron::ser::PrettyConfig::default();
        let ron_str =
            ron::ser::to_string_pretty(data, config).map_err(PersistenceError::Serialize)?;
        std::fs::write(path, ron_str)?;
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<GameSystemFile, PersistenceError> {
        let contents = std::fs::read_to_string(path)?;
        let file: GameSystemFile =
            ron::from_str(&contents).map_err(PersistenceError::Deserialize)?;

        if file.format_version > FORMAT_VERSION {
            return Err(PersistenceError::UnsupportedVersion {
                found: file.format_version,
                max: FORMAT_VERSION,
            });
        }

        Ok(file)
    }

    fn list(&self) -> Result<Vec<ProjectEntry>, PersistenceError> {
        let dir = &self.config.base_dir;
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "hexorder") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                entries.push(ProjectEntry { name, path });
            }
        }
        Ok(entries)
    }

    fn delete(&self, path: &Path) -> Result<(), PersistenceError> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn base_dir(&self) -> &Path {
        &self.config.base_dir
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::contracts::game_system::{
        EntityRole, EntityType, EntityTypeRegistry, EnumRegistry, GameSystem, StructRegistry,
        TypeId,
    };
    use crate::contracts::hex_grid::HexPosition;
    use crate::contracts::mechanics::{CombatModifierRegistry, CombatResultsTable, TurnStructure};
    use crate::contracts::ontology::{ConceptRegistry, ConstraintRegistry, RelationRegistry};
    use crate::contracts::persistence::{GameSystemFile, TileSaveData};

    /// Helper: create a minimal `GameSystemFile` for testing.
    fn test_file() -> GameSystemFile {
        let type_id = TypeId::new();
        GameSystemFile {
            format_version: FORMAT_VERSION,
            name: "Test Storage".to_string(),
            game_system: GameSystem {
                id: "storage-test".to_string(),
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
            enums: EnumRegistry::default(),
            structs: StructRegistry::default(),
            concepts: ConceptRegistry::default(),
            relations: RelationRegistry::default(),
            constraints: ConstraintRegistry::default(),
            turn_structure: TurnStructure::default(),
            combat_results_table: CombatResultsTable::default(),
            combat_modifiers: CombatModifierRegistry::default(),
            map_radius: 5,
            tiles: vec![TileSaveData {
                position: HexPosition::new(0, 0),
                entity_type_id: type_id,
                properties: HashMap::new(),
            }],
            units: Vec::new(),
            workspace_preset: String::new(),
        }
    }

    /// Helper: create a `FilesystemProvider` backed by a unique temp directory.
    /// Returns the provider and the directory path (caller should clean up).
    fn temp_provider(name: &str) -> (FilesystemProvider, PathBuf) {
        let dir = std::env::temp_dir().join(format!("hexorder_storage_test_{name}"));
        // Clean up from any prior failed run.
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let config = StorageConfig {
            base_dir: dir.clone(),
            source: StorageSource::ProjectLocal,
        };
        (FilesystemProvider::new(config), dir)
    }

    #[test]
    fn default_config_resolves_to_project_local() {
        let config = resolve_storage_config();
        assert_eq!(config.source, StorageSource::ProjectLocal);
        let path_str = config.base_dir.to_string_lossy();
        assert!(
            path_str.contains(".state"),
            "path should contain .state: {path_str}"
        );
        assert!(
            path_str.contains(env!("CARGO_PKG_VERSION")),
            "path should contain version: {path_str}"
        );
    }

    #[test]
    fn save_load_round_trip() {
        let (provider, dir) = temp_provider("round_trip");
        let data = test_file();

        let path = provider
            .save("My Project", &data)
            .expect("save should succeed");
        assert!(path.exists());
        assert!(path.ends_with("My Project.hexorder"));

        let loaded = provider.load(&path).expect("load should succeed");
        assert_eq!(loaded.game_system.id, "storage-test");
        assert_eq!(loaded.name, "Test Storage");
        assert_eq!(loaded.tiles.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_finds_hexorder_files() {
        let (provider, dir) = temp_provider("list");
        let data = test_file();

        provider.save("Alpha", &data).expect("save alpha");
        provider.save("Beta", &data).expect("save beta");

        let entries = provider.list().expect("list should succeed");
        assert_eq!(entries.len(), 2);

        let mut names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["Alpha", "Beta"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_removes_file() {
        let (provider, dir) = temp_provider("delete");
        let data = test_file();

        let path = provider.save("Doomed", &data).expect("save");
        assert!(path.exists());

        provider.delete(&path).expect("delete should succeed");
        assert!(!path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_base_directory() {
        let parent = std::env::temp_dir().join("hexorder_storage_test_nested");
        let _ = std::fs::remove_dir_all(&parent);
        let nested = parent.join("deeply").join("nested");
        let config = StorageConfig {
            base_dir: nested.clone(),
            source: StorageSource::ProjectLocal,
        };
        let provider = FilesystemProvider::new(config);
        let data = test_file();

        let path = provider.save("Nested", &data).expect("save should succeed");
        assert!(path.exists());
        assert!(nested.exists());

        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn list_returns_empty_for_nonexistent_dir() {
        let config = StorageConfig {
            base_dir: PathBuf::from("/nonexistent/dir/that/does/not/exist"),
            source: StorageSource::ProjectLocal,
        };
        let provider = FilesystemProvider::new(config);

        let entries = provider.list().expect("list should succeed");
        assert!(entries.is_empty());
    }
}
