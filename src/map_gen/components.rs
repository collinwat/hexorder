//! Plugin-local resources for map generation.

use bevy::prelude::*;

/// Parameters controlling heightmap noise generation.
#[derive(Resource, Debug, Clone)]
pub struct MapGenParams {
    /// Random seed for noise generation. Same seed = same output.
    pub seed: u32,
    /// Number of noise octaves layered together. More octaves = more detail.
    pub octaves: usize,
    /// Base frequency of the noise. Lower = larger terrain features.
    pub frequency: f64,
    /// Controls the overall height range of the noise output.
    #[allow(dead_code)]
    pub amplitude: f64,
    /// Frequency multiplier per octave. Typical: 2.0.
    pub lacunarity: f64,
    /// Amplitude multiplier per octave. Typical: 0.5.
    pub persistence: f64,
}

impl Default for MapGenParams {
    fn default() -> Self {
        Self {
            seed: 42,
            octaves: 6,
            frequency: 0.03,
            amplitude: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

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
    fn default() -> Self {
        Self {
            entries: vec![
                BiomeEntry {
                    min_elevation: 0.0,
                    max_elevation: 0.2,
                    terrain_name: "Water".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.2,
                    max_elevation: 0.4,
                    terrain_name: "Plains".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.4,
                    max_elevation: 0.6,
                    terrain_name: "Forest".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.6,
                    max_elevation: 0.8,
                    terrain_name: "Hills".to_string(),
                },
                BiomeEntry {
                    min_elevation: 0.8,
                    max_elevation: 1.0,
                    terrain_name: "Mountains".to_string(),
                },
            ],
        }
    }
}

/// Marker resource that triggers map generation when inserted.
/// Consumed (removed) after generation completes.
#[derive(Resource, Debug)]
pub struct GenerateMap;
