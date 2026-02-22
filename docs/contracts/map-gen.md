# Contract: map_gen

## Purpose

Defines the shared types for procedural map generation. The `map_gen` plugin owns the generation
systems; `editor_ui` renders the parameter controls as a dock tab and triggers generation.

## Types

### Generation parameters

```rust
/// Parameters controlling heightmap noise generation.
#[derive(Resource, Debug, Clone)]
pub struct MapGenParams {
    /// Random seed for noise generation. Same seed = same output.
    pub seed: u32,
    /// Number of noise octaves layered together. More octaves = more detail.
    pub octaves: usize,
    /// Base frequency of the noise. Lower = larger terrain features.
    pub frequency: f64,
    /// Initial amplitude for the first noise octave.
    pub amplitude: f64,
    /// Frequency multiplier per octave. Typical: 2.0.
    pub lacunarity: f64,
    /// Amplitude multiplier per octave. Typical: 0.5.
    pub persistence: f64,
}
```

### Generation trigger

```rust
/// Marker resource that triggers map generation when inserted.
/// Consumed (removed) after generation completes.
#[derive(Resource, Debug)]
pub struct GenerateMap;
```

## Producers

- `editor_ui` — inserts `GenerateMap` resource when the user clicks "Generate Map"
- `editor_ui` — mutates `MapGenParams` via dock tab controls

## Consumers

- `map_gen` — reads `MapGenParams` during generation, removes `GenerateMap` after completion
