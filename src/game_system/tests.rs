//! Unit tests for the `game_system` feature plugin.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::contracts::game_system::{
    ActiveCellType, ActiveUnitType, CellTypeRegistry, EnumDefinition, GameSystem, PropertyType,
    PropertyValue, SelectedUnit, TypeId, UnitTypeRegistry,
};

/// Helper: create a minimal App with the `GameSystemPlugin`.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(super::GameSystemPlugin);
    app
}

#[test]
fn game_system_resource_exists_after_startup() {
    let mut app = test_app();
    app.update();

    let gs = app
        .world()
        .get_resource::<GameSystem>()
        .expect("GameSystem should exist");

    assert!(!gs.id.is_empty(), "GameSystem id should be non-empty");
    assert_eq!(gs.version, "0.1.0", "GameSystem version should be 0.1.0");
}

#[test]
fn registry_has_starter_types() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<CellTypeRegistry>()
        .expect("CellTypeRegistry should exist");

    assert_eq!(
        registry.types.len(),
        5,
        "Registry should have exactly 5 starter cell types"
    );
}

#[test]
fn starter_types_have_names() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<CellTypeRegistry>()
        .expect("CellTypeRegistry should exist");

    for vt in &registry.types {
        assert!(
            !vt.name.is_empty(),
            "Each starter cell type should have a non-empty name"
        );
    }
}

#[test]
fn starter_types_have_distinct_ids() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<CellTypeRegistry>()
        .expect("CellTypeRegistry should exist");

    let mut ids = HashSet::new();
    for vt in &registry.types {
        assert!(
            ids.insert(vt.id),
            "Duplicate cell type id found: {:?}",
            vt.id
        );
    }

    assert_eq!(ids.len(), 5, "All 5 ids should be unique");
}

#[test]
fn active_cell_type_defaults_to_first() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<CellTypeRegistry>()
        .expect("CellTypeRegistry should exist");

    let first_id = registry
        .first()
        .expect("Registry should have at least one type")
        .id;

    let active = app
        .world()
        .get_resource::<ActiveCellType>()
        .expect("ActiveCellType should exist");

    assert_eq!(
        active.cell_type_id,
        Some(first_id),
        "ActiveCellType should reference the first cell type's id"
    );
}

#[test]
fn property_type_variants_are_distinct() {
    let dummy_enum_id = TypeId::new();
    let variants: Vec<PropertyType> = vec![
        PropertyType::Bool,
        PropertyType::Int,
        PropertyType::Float,
        PropertyType::String,
        PropertyType::Color,
        PropertyType::Enum(dummy_enum_id),
    ];

    // All 6 variants should be distinct from each other.
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Variants at indices {i} and {j} should differ");
            }
        }
    }
}

#[test]
fn property_value_default_for_each_type() {
    let dummy_enum_id = TypeId::new();

    assert_eq!(
        PropertyValue::default_for(&PropertyType::Bool),
        PropertyValue::Bool(false)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Int),
        PropertyValue::Int(0)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Float),
        PropertyValue::Float(0.0)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::String),
        PropertyValue::String(String::new())
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Color),
        PropertyValue::Color(bevy::color::Color::WHITE)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Enum(dummy_enum_id)),
        PropertyValue::Enum(String::new())
    );
}

#[test]
fn type_id_generates_unique() {
    let id1 = TypeId::new();
    let id2 = TypeId::new();
    assert_ne!(
        id1, id2,
        "Two TypeId::new() calls should produce different values"
    );
}

#[test]
fn registry_get_by_id() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<CellTypeRegistry>()
        .expect("CellTypeRegistry should exist");

    let first = registry
        .first()
        .expect("Registry should have at least one type");
    let found = registry.get(first.id);

    assert!(
        found.is_some(),
        "get() should find the first cell type by id"
    );
    assert_eq!(
        found.expect("already checked is_some").name,
        first.name,
        "get() should return the correct cell type"
    );
}

#[test]
fn registry_get_enum_by_id() {
    // Create a registry with a manually-added enum definition.
    let enum_id = TypeId::new();
    let enum_def = EnumDefinition {
        id: enum_id,
        name: "TestEnum".to_string(),
        options: vec!["A".to_string(), "B".to_string()],
    };

    let registry = CellTypeRegistry {
        types: Vec::new(),
        enum_definitions: vec![enum_def],
    };

    let found = registry.get_enum(enum_id);
    assert!(found.is_some(), "get_enum() should find the enum by id");
    assert_eq!(
        found.expect("already checked is_some").name,
        "TestEnum",
        "get_enum() should return the correct enum definition"
    );

    // Non-existent id returns None.
    let other_id = TypeId::new();
    assert!(
        registry.get_enum(other_id).is_none(),
        "get_enum() should return None for unknown id"
    );
}

// ---------------------------------------------------------------------------
// Unit Type Registry Tests (M3)
// ---------------------------------------------------------------------------

#[test]
fn unit_registry_has_starter_types() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<UnitTypeRegistry>()
        .expect("UnitTypeRegistry should exist");

    assert_eq!(
        registry.types.len(),
        3,
        "Registry should have exactly 3 starter unit types"
    );
}

#[test]
fn unit_starter_types_have_names() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<UnitTypeRegistry>()
        .expect("UnitTypeRegistry should exist");

    for ut in &registry.types {
        assert!(
            !ut.name.is_empty(),
            "Each starter unit type should have a non-empty name"
        );
    }
}

#[test]
fn unit_starter_types_have_distinct_ids() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<UnitTypeRegistry>()
        .expect("UnitTypeRegistry should exist");

    let mut ids = HashSet::new();
    for ut in &registry.types {
        assert!(
            ids.insert(ut.id),
            "Duplicate unit type id found: {:?}",
            ut.id
        );
    }

    assert_eq!(ids.len(), 3, "All 3 ids should be unique");
}

#[test]
fn active_unit_type_defaults_to_first() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<UnitTypeRegistry>()
        .expect("UnitTypeRegistry should exist");

    let first_id = registry
        .first()
        .expect("Registry should have at least one type")
        .id;

    let active = app
        .world()
        .get_resource::<ActiveUnitType>()
        .expect("ActiveUnitType should exist");

    assert_eq!(
        active.unit_type_id,
        Some(first_id),
        "ActiveUnitType should reference the first unit type's id"
    );
}

#[test]
fn selected_unit_defaults_to_none() {
    let mut app = test_app();
    app.update();

    let selected = app
        .world()
        .get_resource::<SelectedUnit>()
        .expect("SelectedUnit should exist");

    assert!(
        selected.entity.is_none(),
        "SelectedUnit should default to None"
    );
}
