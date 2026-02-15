//! Unit tests for the `game_system` plugin.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityRole, EntityTypeRegistry, EnumDefinition, GameSystem,
    PropertyType, PropertyValue, SelectedUnit, TypeId,
};
use crate::contracts::persistence::AppScreen;

/// Helper: create a minimal App with the `GameSystemPlugin`.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(AppScreen::Editor);
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
fn registry_has_starter_board_types() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let board_types = registry.types_by_role(EntityRole::BoardPosition);
    assert_eq!(
        board_types.len(),
        5,
        "Registry should have exactly 5 BoardPosition types"
    );
}

#[test]
fn registry_has_starter_token_types() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let token_types = registry.types_by_role(EntityRole::Token);
    assert_eq!(
        token_types.len(),
        3,
        "Registry should have exactly 3 Token types"
    );
}

#[test]
fn starter_types_have_names() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    for et in &registry.types {
        assert!(
            !et.name.is_empty(),
            "Each starter entity type should have a non-empty name"
        );
    }
}

#[test]
fn starter_types_have_distinct_ids() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let mut ids = HashSet::new();
    for et in &registry.types {
        assert!(
            ids.insert(et.id),
            "Duplicate entity type id found: {:?}",
            et.id
        );
    }

    assert_eq!(ids.len(), 8, "All 8 ids should be unique");
}

#[test]
fn active_board_type_defaults_to_first() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let first_id = registry
        .first_by_role(EntityRole::BoardPosition)
        .expect("Registry should have at least one BoardPosition type")
        .id;

    let active = app
        .world()
        .get_resource::<ActiveBoardType>()
        .expect("ActiveBoardType should exist");

    assert_eq!(
        active.entity_type_id,
        Some(first_id),
        "ActiveBoardType should reference the first BoardPosition type's id"
    );
}

#[test]
fn active_token_type_defaults_to_first() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let first_id = registry
        .first_by_role(EntityRole::Token)
        .expect("Registry should have at least one Token type")
        .id;

    let active = app
        .world()
        .get_resource::<ActiveTokenType>()
        .expect("ActiveTokenType should exist");

    assert_eq!(
        active.entity_type_id,
        Some(first_id),
        "ActiveTokenType should reference the first Token type's id"
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
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let first = registry
        .first_by_role(EntityRole::BoardPosition)
        .expect("Registry should have at least one BoardPosition type");
    let found = registry.get(first.id);

    assert!(
        found.is_some(),
        "get() should find the first entity type by id"
    );
    assert_eq!(
        found.expect("already checked is_some").name,
        first.name,
        "get() should return the correct entity type"
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

    let registry = EntityTypeRegistry {
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

#[test]
fn infantry_has_movement_points_property() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let infantry = registry
        .types
        .iter()
        .find(|t| t.name == "Infantry")
        .expect("Infantry type should exist");

    assert_eq!(infantry.role, EntityRole::Token);
    assert_eq!(
        infantry.properties.len(),
        1,
        "Infantry should have 1 property"
    );
    assert_eq!(infantry.properties[0].name, "Movement Points");
    assert_eq!(infantry.properties[0].property_type, PropertyType::Int);
    assert_eq!(infantry.properties[0].default_value, PropertyValue::Int(4));
}

#[test]
fn mountain_has_movement_cost_property() {
    let mut app = test_app();
    app.update();

    let registry = app
        .world()
        .get_resource::<EntityTypeRegistry>()
        .expect("EntityTypeRegistry should exist");

    let mountain = registry
        .types
        .iter()
        .find(|t| t.name == "Mountain")
        .expect("Mountain type should exist");

    assert_eq!(mountain.role, EntityRole::BoardPosition);
    assert_eq!(
        mountain.properties.len(),
        1,
        "Mountain should have 1 property"
    );
    assert_eq!(mountain.properties[0].name, "Movement Cost");
    assert_eq!(mountain.properties[0].property_type, PropertyType::Int);
    assert_eq!(mountain.properties[0].default_value, PropertyValue::Int(3));
}

#[test]
fn property_value_default_for_new_types() {
    let enum_id = TypeId::new();
    let struct_id = TypeId::new();

    assert_eq!(
        PropertyValue::default_for(&PropertyType::EntityRef(None)),
        PropertyValue::EntityRef(None)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::List(Box::new(PropertyType::Int))),
        PropertyValue::List(Vec::new())
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Map(enum_id, Box::new(PropertyType::Int))),
        PropertyValue::Map(Vec::new())
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::Struct(struct_id)),
        PropertyValue::Struct(std::collections::HashMap::new())
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::IntRange { min: 1, max: 10 }),
        PropertyValue::IntRange(1)
    );
    assert_eq!(
        PropertyValue::default_for(&PropertyType::FloatRange { min: 0.0, max: 1.0 }),
        PropertyValue::FloatRange(0.0)
    );
}
