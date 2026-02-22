//! Unit tests for map generation.

use std::collections::HashMap;

use super::biome::{apply_biome_table, lookup_biome, validate_biome_table};
use crate::contracts::map_gen::MapGenParams;

use super::components::{BiomeEntry, BiomeTable};
use super::heightmap::generate_heightmap;
use crate::contracts::hex_grid::HexPosition;

fn default_layout() -> hexx::HexLayout {
    hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        scale: bevy::math::Vec2::splat(1.0),
        origin: bevy::math::Vec2::ZERO,
    }
}

#[test]
fn default_biome_table_is_valid() {
    let table = BiomeTable::default();
    assert!(
        validate_biome_table(&table).is_ok(),
        "Default biome table should pass validation"
    );
}

#[test]
fn empty_biome_table_fails_validation() {
    let table = BiomeTable {
        entries: Vec::new(),
    };
    assert!(
        validate_biome_table(&table).is_err(),
        "Empty biome table should fail validation"
    );
}

#[test]
fn lookup_biome_covers_full_range() {
    let table = BiomeTable::default();
    assert_eq!(lookup_biome(&table, 0.0), Some("Low"));
    assert_eq!(lookup_biome(&table, 0.1), Some("Low"));
    assert_eq!(lookup_biome(&table, 0.2), Some("Mid-Low"));
    assert_eq!(lookup_biome(&table, 0.5), Some("Mid"));
    assert_eq!(lookup_biome(&table, 0.7), Some("Mid-High"));
    assert_eq!(lookup_biome(&table, 0.9), Some("High"));
    assert_eq!(lookup_biome(&table, 1.0), Some("High"));
}

#[test]
fn heightmap_deterministic_with_same_seed() {
    let layout = default_layout();
    let positions = vec![
        HexPosition::new(0, 0),
        HexPosition::new(1, 0),
        HexPosition::new(0, 1),
    ];
    let params = MapGenParams::default();

    let result1 = generate_heightmap(&params, &positions, &layout);
    let result2 = generate_heightmap(&params, &positions, &layout);

    for pos in &positions {
        assert_eq!(
            result1.get(pos),
            result2.get(pos),
            "Same seed should produce same elevation at {pos:?}"
        );
    }
}

#[test]
fn heightmap_values_in_unit_range() {
    let layout = default_layout();
    let positions: Vec<_> = hexx::shapes::hexagon(hexx::Hex::ZERO, 3)
        .map(HexPosition::from_hex)
        .collect();
    let params = MapGenParams::default();

    let result = generate_heightmap(&params, &positions, &layout);

    for (&pos, &elevation) in &result {
        assert!(
            (0.0..=1.0).contains(&elevation),
            "Elevation at {pos:?} should be in [0.0, 1.0], got {elevation}"
        );
    }
}

#[test]
fn biome_table_gap_detected() {
    let table = BiomeTable {
        entries: vec![
            BiomeEntry {
                min_elevation: 0.0,
                max_elevation: 0.3,
                terrain_name: "Low".to_string(),
            },
            // Gap: 0.3 to 0.5 not covered
            BiomeEntry {
                min_elevation: 0.5,
                max_elevation: 1.0,
                terrain_name: "High".to_string(),
            },
        ],
    };
    assert!(
        validate_biome_table(&table).is_err(),
        "Table with gap should fail validation"
    );
}

#[test]
fn map_gen_params_default_has_expected_seed() {
    let params = MapGenParams::default();
    assert_eq!(params.seed, 42);
    assert_eq!(params.octaves, 6);
    // Use amplitude to verify it's accessible
    assert_eq!(params.amplitude, 1.0);
}

#[test]
fn heightmap_different_seeds_differ() {
    let layout = default_layout();
    let positions: Vec<_> = hexx::shapes::hexagon(hexx::Hex::ZERO, 3)
        .map(HexPosition::from_hex)
        .collect();

    let params_a = MapGenParams {
        seed: 42,
        ..MapGenParams::default()
    };
    let params_b = MapGenParams {
        seed: 43,
        ..MapGenParams::default()
    };

    let result_a = generate_heightmap(&params_a, &positions, &layout);
    let result_b = generate_heightmap(&params_b, &positions, &layout);

    let differences = positions
        .iter()
        .filter(|pos| {
            let a = result_a[pos];
            let b = result_b[pos];
            (a - b).abs() > 0.001
        })
        .count();

    assert!(
        differences > 0,
        "Different seeds should produce at least some different elevations"
    );
}

#[test]
fn heightmap_spatial_coherence() {
    let layout = default_layout();

    let center = hexx::Hex::ZERO;
    let neighbors = center.all_neighbors();

    let mut all_hexes = vec![center];
    all_hexes.extend_from_slice(&neighbors);
    let positions: Vec<_> = all_hexes
        .iter()
        .map(|h| HexPosition::from_hex(*h))
        .collect();

    let params = MapGenParams {
        frequency: 0.01, // Low frequency = large terrain features
        ..MapGenParams::default()
    };

    let result = generate_heightmap(&params, &positions, &layout);

    let center_pos = HexPosition::from_hex(center);
    let center_elev = result[&center_pos];

    for neighbor_hex in &neighbors {
        let neighbor_pos = HexPosition::from_hex(*neighbor_hex);
        let neighbor_elev = result[&neighbor_pos];
        let diff = (center_elev - neighbor_elev).abs();

        assert!(
            diff < 0.5,
            "At low frequency, adjacent hexes should have similar elevations. \
             Center={center_elev:.4}, neighbor at {neighbor_pos:?}={neighbor_elev:.4}, diff={diff:.4}"
        );
    }
}

#[test]
fn biome_lookup_boundary_values() {
    let table = BiomeTable::default();

    // Just below 0.2 boundary: should still be Low
    assert_eq!(
        lookup_biome(&table, 0.199_999_999),
        Some("Low"),
        "Elevation just below 0.2 should be Low"
    );
    // Exactly at 0.2: should transition to Mid-Low (non-last entry has exclusive max)
    assert_eq!(
        lookup_biome(&table, 0.2),
        Some("Mid-Low"),
        "Elevation exactly at 0.2 should be Mid-Low"
    );

    // Just below 0.4 boundary: should still be Mid-Low
    assert_eq!(
        lookup_biome(&table, 0.399_999_999),
        Some("Mid-Low"),
        "Elevation just below 0.4 should be Mid-Low"
    );
    // Exactly at 0.4: should transition to Mid
    assert_eq!(
        lookup_biome(&table, 0.4),
        Some("Mid"),
        "Elevation exactly at 0.4 should be Mid"
    );
}

#[test]
fn biome_table_apply_maps_all_positions() {
    let table = BiomeTable::default();

    let mut heightmap = HashMap::new();
    heightmap.insert(HexPosition::new(0, 0), 0.1); // Low range [0.0, 0.2)
    heightmap.insert(HexPosition::new(1, 0), 0.5); // Mid range [0.4, 0.6)
    heightmap.insert(HexPosition::new(0, 1), 0.9); // High range [0.8, 1.0]

    let result = apply_biome_table(&heightmap, &table);

    assert_eq!(result.len(), 3, "All 3 positions should be mapped");
    assert_eq!(result[&HexPosition::new(0, 0)], "Low");
    assert_eq!(result[&HexPosition::new(1, 0)], "Mid");
    assert_eq!(result[&HexPosition::new(0, 1)], "High");
}

#[test]
fn full_generation_pipeline() {
    // End-to-end test: params -> heightmap -> biome -> terrain names
    let params = MapGenParams::default();
    let layout = default_layout();

    // Generate a larger grid: radius 3 = 37 hexes
    let positions: Vec<HexPosition> = hexx::shapes::hexagon(hexx::Hex::ZERO, 3)
        .map(HexPosition::from_hex)
        .collect();

    let heightmap = generate_heightmap(&params, &positions, &layout);

    assert_eq!(heightmap.len(), positions.len());

    let table = BiomeTable::default();
    let terrain = apply_biome_table(&heightmap, &table);

    // Every position should get a terrain assignment
    assert_eq!(terrain.len(), positions.len());

    // All terrain names should be from the default biome table
    let valid_names: std::collections::HashSet<&str> =
        ["Low", "Mid-Low", "Mid", "Mid-High", "High"]
            .iter()
            .copied()
            .collect();
    for name in terrain.values() {
        assert!(
            valid_names.contains(name.as_str()),
            "Unexpected terrain name: {name}"
        );
    }
}
