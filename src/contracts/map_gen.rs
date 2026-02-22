//! Shared types for procedural map generation.
//!
//! The `map_gen` plugin owns the generation systems; `editor_ui` renders
//! parameter controls as a dock tab and triggers generation.

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
    /// Initial amplitude for the first noise octave. Controls terrain roughness:
    /// higher values make the first octave dominant (smoother terrain), lower
    /// values let higher-frequency octaves show through more.
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

/// Marker resource that triggers map generation when inserted.
/// Consumed (removed) after generation completes.
#[derive(Resource, Debug)]
pub struct GenerateMap;
