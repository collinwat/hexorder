//! Shared Storage types. See `docs/contracts/storage.md`.
//!
//! Defines the storage abstraction layer: a trait for I/O backends,
//! configuration resolved from build target, and a Bevy resource wrapper.

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use super::persistence::{GameSystemFile, PersistenceError};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// How the base directory was determined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageSource {
    /// `macos` feature flag — `~/Library/Application Support/hexorder/`.
    MacOs,
    /// `xdg` feature flag — `$XDG_DATA_HOME/hexorder/`.
    Xdg,
    /// Project-local `.state/{version}/` (default dev mode).
    ProjectLocal,
}

/// Resolved storage configuration. Describes where project data lives.
#[derive(Resource, Debug, Clone)]
pub struct StorageConfig {
    /// The base directory for saved projects.
    pub base_dir: PathBuf,
    /// How the base directory was determined.
    pub source: StorageSource,
}

// ---------------------------------------------------------------------------
// Provider Trait
// ---------------------------------------------------------------------------

/// Metadata about a saved project on disk.
#[derive(Debug, Clone)]
pub struct ProjectEntry {
    /// Human-readable project name (derived from filename stem).
    pub name: String,
    /// Full path to the `.hexorder` file.
    pub path: PathBuf,
}

/// Trait for storage backends. Object-safe, `Send + Sync`.
///
/// Systems use this trait through the [`Storage`] resource instead of
/// calling file I/O helpers directly.
pub trait StorageProvider: Send + Sync + std::fmt::Debug {
    /// Save a game system file to the base directory, returning the written path.
    /// The provider derives the filename from `name`.
    fn save(&self, name: &str, data: &GameSystemFile) -> Result<PathBuf, PersistenceError>;

    /// Save a game system file to a specific path (Save As / overwrite).
    fn save_at(&self, path: &Path, data: &GameSystemFile) -> Result<(), PersistenceError>;

    /// Load a game system file from a specific path.
    fn load(&self, path: &Path) -> Result<GameSystemFile, PersistenceError>;

    /// List all `.hexorder` projects in the base directory.
    fn list(&self) -> Result<Vec<ProjectEntry>, PersistenceError>;

    /// Delete a saved project by path.
    fn delete(&self, path: &Path) -> Result<(), PersistenceError>;

    /// The base directory this provider operates on.
    fn base_dir(&self) -> &Path;
}

// ---------------------------------------------------------------------------
// Bevy Resource
// ---------------------------------------------------------------------------

/// Bevy resource wrapping a boxed [`StorageProvider`].
///
/// Inserted by `PersistencePlugin` during startup. Systems access storage
/// through `Res<Storage>`.
#[derive(Resource, Debug)]
pub struct Storage {
    provider: Box<dyn StorageProvider>,
}

impl Storage {
    /// Create a new `Storage` resource from a provider.
    pub fn new(provider: Box<dyn StorageProvider>) -> Self {
        Self { provider }
    }

    /// Access the underlying storage provider.
    pub fn provider(&self) -> &dyn StorageProvider {
        &*self.provider
    }
}
