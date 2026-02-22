//! Pure heightmap generation using layered Perlin noise.

use std::collections::HashMap;

use noise::{NoiseFn, Perlin};

use crate::contracts::hex_grid::HexPosition;

use crate::contracts::map_gen::MapGenParams;

/// Generate a heightmap for the given hex positions.
///
/// Returns elevation values normalized to [0.0, 1.0] for each position.
/// Uses layered Perlin noise (fractal Brownian motion) sampled at
/// world-space coordinates derived from the hex layout.
pub fn generate_heightmap(
    params: &MapGenParams,
    positions: &[HexPosition],
    layout: &hexx::HexLayout,
) -> HashMap<HexPosition, f64> {
    let perlin = Perlin::new(params.seed);
    let mut result = HashMap::with_capacity(positions.len());

    for &pos in positions {
        let world = layout.hex_to_world_pos(pos.to_hex());
        let value = fbm_sample(
            &perlin,
            f64::from(world.x) * params.frequency,
            f64::from(world.y) * params.frequency,
            params,
        );
        // Normalize from roughly [-1, 1] to [0, 1]
        let normalized = (value + 1.0) * 0.5;
        result.insert(pos, normalized.clamp(0.0, 1.0));
    }

    result
}

/// Fractal Brownian motion: layer multiple octaves of Perlin noise.
fn fbm_sample(noise: &Perlin, x: f64, y: f64, params: &MapGenParams) -> f64 {
    let mut total = 0.0;
    let mut freq = 1.0;
    let mut amp = params.amplitude;
    let mut max_amp = 0.0;

    for _ in 0..params.octaves {
        total += noise.get([x * freq, y * freq]) * amp;
        max_amp += amp;
        freq *= params.lacunarity;
        amp *= params.persistence;
    }

    // Normalize by max possible amplitude so output stays in [-1, 1]
    if max_amp > 0.0 { total / max_amp } else { 0.0 }
}
