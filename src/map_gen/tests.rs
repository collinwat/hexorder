//! Unit tests for map generation.

use super::biome::{lookup_biome, validate_biome_table};
use super::components::{BiomeEntry, BiomeTable, MapGenParams};
use super::heightmap::generate_heightmap;
use crate::contracts::hex_grid::HexPosition;

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
    assert_eq!(lookup_biome(&table, 0.0), Some("Water"));
    assert_eq!(lookup_biome(&table, 0.1), Some("Water"));
    assert_eq!(lookup_biome(&table, 0.2), Some("Plains"));
    assert_eq!(lookup_biome(&table, 0.5), Some("Forest"));
    assert_eq!(lookup_biome(&table, 0.7), Some("Hills"));
    assert_eq!(lookup_biome(&table, 0.9), Some("Mountains"));
    assert_eq!(lookup_biome(&table, 1.0), Some("Mountains"));
}

#[test]
fn heightmap_deterministic_with_same_seed() {
    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        scale: bevy::math::Vec2::splat(1.0),
        origin: bevy::math::Vec2::ZERO,
    };
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
    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        scale: bevy::math::Vec2::splat(1.0),
        origin: bevy::math::Vec2::ZERO,
    };
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
