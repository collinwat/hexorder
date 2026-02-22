//! Plugin-local resources for map generation.

use bevy::prelude::*;

/// A single entry in the biome table mapping an elevation range to a terrain name.
#[derive(Debug, Clone)]
pub struct BiomeEntry {
    /// Minimum elevation (inclusive).
    pub min_elevation: f64,
    /// Maximum elevation (exclusive, except for the last entry which is inclusive).
    pub max_elevation: f64,
    /// Name of the terrain type, matched against `EntityTypeRegistry` by name.
    pub terrain_name: String,
}

/// Maps elevation ranges to terrain type names. Entries must be sorted by
/// `min_elevation` with no gaps covering the full [0.0, 1.0] range.
#[derive(Resource, Debug, Clone)]
pub struct BiomeTable {
    pub entries: Vec<BiomeEntry>,
}

impl Default for BiomeTable {
    /// Default biome table references the starter `BoardPosition` types
    /// from `create_entity_type_registry`. Designers replace these with
    /// their own types via the UI.
    fn default() -> Self {
        Self {
            entries: vec![
                BiomeEntry {
                    min_elevation: 0.0,
                    max_elevation: 0.2,
                    terrain_name: "Low".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.2,
                    max_elevation: 0.4,
                    terrain_name: "Mid-Low".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.4,
                    max_elevation: 0.6,
                    terrain_name: "Mid".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.6,
                    max_elevation: 0.8,
                    terrain_name: "Mid-High".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.8,
                    max_elevation: 1.0,
                    terrain_name: "High".to_string(),
                },
            ],
        }
    }
}
