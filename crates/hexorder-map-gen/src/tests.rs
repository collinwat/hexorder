//! Unit tests for map generation.

use std::collections::HashMap;

use super::biome::{apply_biome_table, lookup_biome, validate_biome_table};
use hexorder_contracts::map_gen::MapGenParams;

use super::components::{BiomeEntry, BiomeTable};
use super::heightmap::generate_heightmap;
use hexorder_contracts::hex_grid::HexPosition;

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

// ---------------------------------------------------------------------------
// Additional coverage: biome.rs -- lookup_biome_index, apply_biome_table_indexed
// ---------------------------------------------------------------------------

use super::biome::{BiomeTableError, apply_biome_table_indexed, lookup_biome_index};

#[test]
fn lookup_biome_index_covers_full_range() {
    let table = BiomeTable::default();
    assert_eq!(lookup_biome_index(&table, 0.0), Some(0));
    assert_eq!(lookup_biome_index(&table, 0.1), Some(0)); // Low
    assert_eq!(lookup_biome_index(&table, 0.2), Some(1)); // Mid-Low
    assert_eq!(lookup_biome_index(&table, 0.5), Some(2)); // Mid
    assert_eq!(lookup_biome_index(&table, 0.7), Some(3)); // Mid-High
    assert_eq!(lookup_biome_index(&table, 0.9), Some(4)); // High
    assert_eq!(lookup_biome_index(&table, 1.0), Some(4)); // High (last entry inclusive max)
}

#[test]
fn lookup_biome_index_boundary_values() {
    let table = BiomeTable::default();
    // Just below 0.2: still index 0 (Low)
    assert_eq!(lookup_biome_index(&table, 0.199_999_999), Some(0));
    // Exactly at 0.2: index 1 (Mid-Low)
    assert_eq!(lookup_biome_index(&table, 0.2), Some(1));
    // At 0.4: index 2 (Mid)
    assert_eq!(lookup_biome_index(&table, 0.4), Some(2));
    // At 0.6: index 3 (Mid-High)
    assert_eq!(lookup_biome_index(&table, 0.6), Some(3));
    // At 0.8: index 4 (High)
    assert_eq!(lookup_biome_index(&table, 0.8), Some(4));
}

#[test]
fn lookup_biome_index_returns_none_for_out_of_range() {
    let table = BiomeTable::default();
    // Below 0.0
    assert_eq!(lookup_biome_index(&table, -0.1), None);
    // Above 1.0
    assert_eq!(lookup_biome_index(&table, 1.1), None);
}

#[test]
fn apply_biome_table_indexed_maps_all_positions() {
    let table = BiomeTable::default();
    let mut heightmap = HashMap::new();
    heightmap.insert(HexPosition::new(0, 0), 0.1); // Low -> index 0
    heightmap.insert(HexPosition::new(1, 0), 0.5); // Mid -> index 2
    heightmap.insert(HexPosition::new(0, 1), 0.9); // High -> index 4

    let result = apply_biome_table_indexed(&heightmap, &table);

    assert_eq!(result.len(), 3, "All 3 positions should be mapped");
    assert_eq!(result[&HexPosition::new(0, 0)], 0);
    assert_eq!(result[&HexPosition::new(1, 0)], 2);
    assert_eq!(result[&HexPosition::new(0, 1)], 4);
}

#[test]
fn apply_biome_table_indexed_omits_out_of_range() {
    let table = BiomeTable::default();
    let mut heightmap = HashMap::new();
    heightmap.insert(HexPosition::new(0, 0), 0.5); // In range
    heightmap.insert(HexPosition::new(1, 0), -0.5); // Out of range

    let result = apply_biome_table_indexed(&heightmap, &table);

    assert_eq!(result.len(), 1, "Out-of-range elevation should be omitted");
    assert!(result.contains_key(&HexPosition::new(0, 0)));
    assert!(!result.contains_key(&HexPosition::new(1, 0)));
}

// ---------------------------------------------------------------------------
// Additional coverage: biome.rs -- validate_biome_table error variants
// ---------------------------------------------------------------------------

#[test]
fn validate_biome_table_gap_at_start() {
    let table = BiomeTable {
        entries: vec![BiomeEntry {
            min_elevation: 0.3, // Does not start at 0.0
            max_elevation: 1.0,
            terrain_name: "Only".to_string(),
        }],
    };
    let err = validate_biome_table(&table).expect_err("should fail");
    assert!(
        matches!(err, BiomeTableError::GapAtStart(_)),
        "Expected GapAtStart, got {err:?}"
    );
}

#[test]
fn validate_biome_table_gap_at_end() {
    let table = BiomeTable {
        entries: vec![BiomeEntry {
            min_elevation: 0.0,
            max_elevation: 0.8, // Does not reach 1.0
            terrain_name: "Only".to_string(),
        }],
    };
    let err = validate_biome_table(&table).expect_err("should fail");
    assert!(
        matches!(err, BiomeTableError::GapAtEnd(_)),
        "Expected GapAtEnd, got {err:?}"
    );
}

#[test]
fn validate_biome_table_gap_between_entries() {
    let table = BiomeTable {
        entries: vec![
            BiomeEntry {
                min_elevation: 0.0,
                max_elevation: 0.4,
                terrain_name: "Low".to_string(),
            },
            BiomeEntry {
                min_elevation: 0.6, // Gap between 0.4 and 0.6
                max_elevation: 1.0,
                terrain_name: "High".to_string(),
            },
        ],
    };
    let err = validate_biome_table(&table).expect_err("should fail");
    assert!(
        matches!(err, BiomeTableError::Gap { .. }),
        "Expected Gap, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// BiomeTableError Display
// ---------------------------------------------------------------------------

#[test]
fn biome_table_error_display_empty() {
    let err = BiomeTableError::Empty;
    let msg = format!("{err}");
    assert_eq!(msg, "biome table is empty");
}

#[test]
fn biome_table_error_display_gap_at_start() {
    let err = BiomeTableError::GapAtStart(0.3);
    let msg = format!("{err}");
    assert!(msg.contains("does not start at 0.0"));
    assert!(msg.contains("0.3"));
}

#[test]
fn biome_table_error_display_gap_at_end() {
    let err = BiomeTableError::GapAtEnd(0.8);
    let msg = format!("{err}");
    assert!(msg.contains("does not reach 1.0"));
    assert!(msg.contains("0.8"));
}

#[test]
fn biome_table_error_display_gap_between() {
    let err = BiomeTableError::Gap {
        after: "Low".to_string(),
        before: "High".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("gap between"));
    assert!(msg.contains("Low"));
    assert!(msg.contains("High"));
}

// ---------------------------------------------------------------------------
// Additional coverage: map_gen/systems.rs -- run_generation system
// ---------------------------------------------------------------------------

use bevy::prelude::*;
use hexorder_contracts::game_system::{
    EntityData, EntityRole, EntityType, EntityTypeRegistry, PropertyValue, TypeId,
};
use hexorder_contracts::hex_grid::{HexGridConfig, HexTile};
use hexorder_contracts::map_gen::GenerateMap;
use hexorder_contracts::persistence::AppScreen;
use hexorder_contracts::undo_redo::UndoStack;

use super::systems;

/// Helper: create a minimal test app for map generation system tests.
fn test_app_for_generation() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<MapGenParams>();
    app.init_resource::<BiomeTable>();
    app
}

#[test]
fn run_generation_does_nothing_without_generate_map_resource() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });
    app.init_resource::<EntityTypeRegistry>();

    // Spawn a tile to verify it's untouched.
    let tile_type_id = TypeId::new();
    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: tile_type_id,
            properties: HashMap::new(),
        },
    ));

    // No GenerateMap resource => system should do nothing.
    app.add_systems(Update, systems::run_generation);
    app.update();

    // Tile should still have original type_id.
    let mut query = app.world_mut().query::<&EntityData>();
    let data: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].entity_type_id, tile_type_id);
}

#[test]
fn run_generation_removes_generate_map_marker() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    // Create a registry with at least one BoardPosition type.
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: TypeId::new(),
        name: "Plains".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::WHITE,
        properties: vec![],
    });
    app.insert_resource(registry);

    // Spawn a tile.
    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    // Insert GenerateMap marker.
    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    // GenerateMap should be consumed (removed).
    assert!(
        app.world().get_resource::<GenerateMap>().is_none(),
        "GenerateMap should be removed after generation completes"
    );
}

#[test]
fn run_generation_assigns_entity_type_to_tiles() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    // Create a registry with BoardPosition types.
    let type_id_a = TypeId::new();
    let type_id_b = TypeId::new();
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id_a,
        name: "TypeA".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::WHITE,
        properties: vec![],
    });
    registry.types.push(EntityType {
        id: type_id_b,
        name: "TypeB".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
        properties: vec![],
    });
    app.insert_resource(registry);

    // Spawn some tiles with placeholder data.
    let positions: Vec<_> = hexx::shapes::hexagon(hexx::Hex::ZERO, 2)
        .map(HexPosition::from_hex)
        .collect();
    for pos in &positions {
        app.world_mut().spawn((
            HexTile,
            *pos,
            EntityData {
                entity_type_id: TypeId::new(), // Placeholder
                properties: HashMap::new(),
            },
        ));
    }

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    // All tiles should now have one of the BoardPosition type IDs.
    let valid_ids = [type_id_a, type_id_b];
    let mut query = app.world_mut().query::<&EntityData>();
    for data in query.iter(app.world()) {
        assert!(
            valid_ids.contains(&data.entity_type_id),
            "Tile should have a BoardPosition entity type after generation, got {:?}",
            data.entity_type_id
        );
    }
}

#[test]
fn run_generation_records_undo_command() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: TypeId::new(),
        name: "Land".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::WHITE,
        properties: vec![],
    });
    app.insert_resource(registry);
    app.insert_resource(UndoStack::default());

    // Spawn a tile.
    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    let stack = app.world().resource::<UndoStack>();
    assert!(
        stack.can_undo(),
        "UndoStack should have a command after generation"
    );
    assert_eq!(
        stack.undo_description(),
        Some("Generate Map".to_string()),
        "Undo description should be 'Generate Map'"
    );
}

#[test]
fn run_generation_skips_on_invalid_biome_table() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    // Insert an INVALID biome table (empty).
    app.insert_resource(BiomeTable {
        entries: Vec::new(),
    });
    app.init_resource::<EntityTypeRegistry>();

    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    // GenerateMap should still be consumed even when biome table is invalid.
    assert!(
        app.world().get_resource::<GenerateMap>().is_none(),
        "GenerateMap should be removed even on validation failure"
    );
}

#[test]
fn run_generation_skips_on_empty_board_types() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    // Registry has only Token types, no BoardPosition.
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: TypeId::new(),
        name: "Infantry".to_string(),
        role: EntityRole::Token,
        color: bevy::color::Color::WHITE,
        properties: vec![],
    });
    app.insert_resource(registry);

    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    // GenerateMap should be consumed.
    assert!(
        app.world().get_resource::<GenerateMap>().is_none(),
        "GenerateMap should be removed even with no BoardPosition types"
    );
}

#[test]
fn run_generation_skips_on_no_tiles() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });
    app.init_resource::<EntityTypeRegistry>();

    // No tiles spawned.
    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    assert!(
        app.world().get_resource::<GenerateMap>().is_none(),
        "GenerateMap should be removed even with no tiles"
    );
}

#[test]
fn run_generation_without_undo_stack_does_not_panic() {
    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: TypeId::new(),
        name: "Land".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::WHITE,
        properties: vec![],
    });
    app.insert_resource(registry);
    // No UndoStack inserted -- should still work.

    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update(); // Should not panic
}

// ---------------------------------------------------------------------------
// Additional coverage: map_gen/mod.rs -- MapGenPlugin build
// ---------------------------------------------------------------------------

#[test]
fn map_gen_plugin_builds_without_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);

    // Pre-insert resources that the plugin systems need.
    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });
    app.init_resource::<EntityTypeRegistry>();

    app.add_plugins(super::MapGenPlugin);
    app.update(); // Should not panic

    // MapGenParams and BiomeTable should be initialized.
    assert!(
        app.world().get_resource::<MapGenParams>().is_some(),
        "MapGenParams should be initialized by plugin"
    );
    assert!(
        app.world().get_resource::<BiomeTable>().is_some(),
        "BiomeTable should be initialized by plugin"
    );
}

// ---------------------------------------------------------------------------
// Additional coverage: biome.rs -- lookup_biome returns None for out of range
// ---------------------------------------------------------------------------

#[test]
fn lookup_biome_returns_none_below_range() {
    let table = BiomeTable::default();
    assert_eq!(lookup_biome(&table, -0.1), None);
}

#[test]
fn lookup_biome_returns_none_above_range() {
    let table = BiomeTable::default();
    assert_eq!(lookup_biome(&table, 1.1), None);
}

// ---------------------------------------------------------------------------
// BiomeTable default contents
// ---------------------------------------------------------------------------

#[test]
fn biome_table_default_has_five_entries() {
    let table = BiomeTable::default();
    assert_eq!(
        table.entries.len(),
        5,
        "Default biome table should have 5 entries"
    );
}

#[test]
fn biome_table_default_entries_cover_full_range() {
    let table = BiomeTable::default();
    assert!(
        (table.entries[0].min_elevation - 0.0).abs() < f64::EPSILON,
        "First entry should start at 0.0"
    );
    assert!(
        (table.entries[4].max_elevation - 1.0).abs() < f64::EPSILON,
        "Last entry should end at 1.0"
    );
}

// ---------------------------------------------------------------------------
// run_generation with properties on entity types
// ---------------------------------------------------------------------------

#[test]
fn run_generation_populates_default_properties() {
    use hexorder_contracts::game_system::{PropertyDefinition, PropertyType};

    let mut app = test_app_for_generation();

    let layout = hexx::HexLayout {
        orientation: hexx::HexOrientation::Pointy,
        ..hexx::HexLayout::default()
    }
    .with_hex_size(1.0);
    app.insert_resource(HexGridConfig {
        layout,
        map_radius: 2,
    });

    let prop_id = TypeId::new();
    let type_id = TypeId::new();
    let mut registry = EntityTypeRegistry::default();
    registry.types.push(EntityType {
        id: type_id,
        name: "Land".to_string(),
        role: EntityRole::BoardPosition,
        color: bevy::color::Color::WHITE,
        properties: vec![PropertyDefinition {
            id: prop_id,
            name: "Movement Cost".to_string(),
            property_type: PropertyType::Int,
            default_value: PropertyValue::Int(1),
        }],
    });
    app.insert_resource(registry);

    app.world_mut().spawn((
        HexTile,
        HexPosition::new(0, 0),
        EntityData {
            entity_type_id: TypeId::new(),
            properties: HashMap::new(),
        },
    ));

    app.insert_resource(GenerateMap);
    app.add_systems(Update, systems::run_generation);
    app.update();

    // Verify the tile has properties set.
    let mut query = app.world_mut().query::<&EntityData>();
    for data in query.iter(app.world()) {
        assert!(
            data.properties.contains_key(&prop_id),
            "Generated tile should have property from entity type definition"
        );
        assert_eq!(
            data.properties[&prop_id],
            PropertyValue::Int(0), // default_for(Int) is 0
            "Property should have default value for its type"
        );
    }
}

// ---------------------------------------------------------------------------
// validate_biome_table: unsorted entries still validated correctly
// ---------------------------------------------------------------------------

#[test]
fn validate_biome_table_sorts_entries_before_checking() {
    // Entries are out of order but form a valid contiguous range.
    let table = BiomeTable {
        entries: vec![
            BiomeEntry {
                min_elevation: 0.5,
                max_elevation: 1.0,
                terrain_name: "High".to_string(),
            },
            BiomeEntry {
                min_elevation: 0.0,
                max_elevation: 0.5,
                terrain_name: "Low".to_string(),
            },
        ],
    };
    assert!(
        validate_biome_table(&table).is_ok(),
        "Unsorted but contiguous entries should pass validation"
    );
}
