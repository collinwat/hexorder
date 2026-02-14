//! Tests for the `scripting` plugin.

use mlua::Lua;

use crate::contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType, PropertyValue,
    TypeId,
};
use crate::contracts::ontology::{
    Concept, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr, ConstraintRegistry,
    Relation, RelationEffect, RelationRegistry, RelationTrigger,
};
use crate::contracts::validation::{SchemaError, SchemaErrorCategory, SchemaValidation};

use super::lua_api;

/// Helper: create a minimal entity type registry with one board type and one token type.
fn test_registry() -> EntityTypeRegistry {
    EntityTypeRegistry {
        types: vec![
            EntityType {
                id: TypeId::new(),
                name: "Plains".to_string(),
                role: EntityRole::BoardPosition,
                color: bevy::color::Color::srgb(0.4, 0.6, 0.2),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "movement_cost".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(1),
                }],
            },
            EntityType {
                id: TypeId::new(),
                name: "Infantry".to_string(),
                role: EntityRole::Token,
                color: bevy::color::Color::srgb(0.2, 0.2, 0.8),
                properties: vec![PropertyDefinition {
                    id: TypeId::new(),
                    name: "movement_points".to_string(),
                    property_type: PropertyType::Int,
                    default_value: PropertyValue::Int(3),
                }],
            },
        ],
        enum_definitions: vec![],
    }
}

/// Helper: create a concept registry with one concept.
fn test_concept_registry() -> ConceptRegistry {
    let concept_id = TypeId::new();
    ConceptRegistry {
        concepts: vec![Concept {
            id: concept_id,
            name: "Motion".to_string(),
            description: "Movement across the board".to_string(),
            role_labels: vec![
                ConceptRole {
                    id: TypeId::new(),
                    name: "traveler".to_string(),
                    allowed_entity_roles: vec![EntityRole::Token],
                },
                ConceptRole {
                    id: TypeId::new(),
                    name: "terrain".to_string(),
                    allowed_entity_roles: vec![EntityRole::BoardPosition],
                },
            ],
        }],
        bindings: vec![],
    }
}

/// Helper: create a relation registry with one relation.
fn test_relation_registry() -> RelationRegistry {
    let concept_id = TypeId::new();
    RelationRegistry {
        relations: vec![Relation {
            id: TypeId::new(),
            name: "Terrain Cost".to_string(),
            concept_id,
            subject_role_id: TypeId::new(),
            object_role_id: TypeId::new(),
            trigger: RelationTrigger::OnEnter,
            effect: RelationEffect::ModifyProperty {
                target_property: "budget".to_string(),
                source_property: "cost".to_string(),
                operation: crate::contracts::ontology::ModifyOperation::Subtract,
            },
        }],
    }
}

/// Helper: create a constraint registry with one constraint.
fn test_constraint_registry() -> ConstraintRegistry {
    ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Budget >= 0".to_string(),
            description: "Traveler must have non-negative budget".to_string(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: TypeId::new(),
                property_name: "budget".to_string(),
                operator: crate::contracts::ontology::CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: false,
        }],
    }
}

#[test]
fn lua_can_query_entity_types() {
    let lua = Lua::new();
    let registry = test_registry();
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("should be a table");
    assert_eq!(table.len().expect("len"), 2);
}

#[test]
fn lua_entity_type_has_correct_fields() {
    let lua = Lua::new();
    let registry = test_registry();
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("table");
    let first: mlua::Table = table.get(1).expect("first element");
    let name: String = first.get("name").expect("name field");
    let role: String = first.get("role").expect("role field");
    assert_eq!(name, "Plains");
    assert_eq!(role, "board_position");
}

#[test]
fn lua_entity_type_properties() {
    let lua = Lua::new();
    let registry = test_registry();
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("table");
    let first: mlua::Table = table.get(1).expect("first element");
    let props: mlua::Table = first.get("properties").expect("properties field");
    assert_eq!(props.len().expect("props len"), 1);

    let prop: mlua::Table = props.get(1).expect("first property");
    let prop_name: String = prop.get("name").expect("prop name");
    let prop_type: String = prop.get("type").expect("prop type");
    let default_val: i64 = prop.get("default_value").expect("default value");
    assert_eq!(prop_name, "movement_cost");
    assert_eq!(prop_type, "int");
    assert_eq!(default_val, 1);
}

#[test]
fn lua_filters_entity_types_by_role() {
    let lua = Lua::new();
    let registry = test_registry();
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("should convert");

    // Use Lua script to filter by role
    lua.globals().set("types", result).expect("set global");

    let count: i64 = lua
        .load(
            r#"
            local count = 0
            for _, t in ipairs(types) do
                if t.role == "token" then count = count + 1 end
            end
            return count
        "#,
        )
        .eval()
        .expect("eval");

    assert_eq!(count, 1);
}

#[test]
fn lua_can_query_concepts() {
    let lua = Lua::new();
    let registry = test_concept_registry();
    let result = lua_api::concepts_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 1);

    let concept: mlua::Table = table.get(1).expect("first concept");
    let name: String = concept.get("name").expect("name");
    assert_eq!(name, "Motion");

    let roles: mlua::Table = concept.get("roles").expect("roles");
    assert_eq!(roles.len().expect("roles len"), 2);
}

#[test]
fn lua_can_query_relations() {
    let lua = Lua::new();
    let registry = test_relation_registry();
    let result = lua_api::relations_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 1);

    let relation: mlua::Table = table.get(1).expect("first relation");
    let name: String = relation.get("name").expect("name");
    assert_eq!(name, "Terrain Cost");
}

#[test]
fn lua_can_query_constraints() {
    let lua = Lua::new();
    let registry = test_constraint_registry();
    let result = lua_api::constraints_to_lua(&lua, &registry).expect("should convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 1);

    let constraint: mlua::Table = table.get(1).expect("first constraint");
    let name: String = constraint.get("name").expect("name");
    let auto: bool = constraint.get("auto_generated").expect("auto_generated");
    assert_eq!(name, "Budget >= 0");
    assert!(!auto);
}

#[test]
fn lua_schema_validation_valid() {
    let lua = Lua::new();
    let sv = SchemaValidation {
        errors: vec![],
        is_valid: true,
    };
    let result = lua_api::schema_validation_to_lua(&lua, &sv).expect("should convert");
    let table = result.as_table().expect("table");
    let is_valid: bool = table.get("is_valid").expect("is_valid");
    let error_count: i64 = table.get("error_count").expect("error_count");
    assert!(is_valid);
    assert_eq!(error_count, 0);
}

#[test]
fn lua_schema_validation_with_errors() {
    let lua = Lua::new();
    let sv = SchemaValidation {
        errors: vec![
            SchemaError {
                category: SchemaErrorCategory::DanglingReference,
                message: "Missing concept".to_string(),
                source_id: TypeId::new(),
            },
            SchemaError {
                category: SchemaErrorCategory::RoleMismatch,
                message: "Wrong role".to_string(),
                source_id: TypeId::new(),
            },
        ],
        is_valid: false,
    };
    let result = lua_api::schema_validation_to_lua(&lua, &sv).expect("should convert");
    let table = result.as_table().expect("table");
    let is_valid: bool = table.get("is_valid").expect("is_valid");
    let error_count: i64 = table.get("error_count").expect("error_count");
    assert!(!is_valid);
    assert_eq!(error_count, 2);

    let errors: mlua::Table = table.get("errors").expect("errors");
    let first_err: mlua::Table = errors.get(1).expect("first error");
    let msg: String = first_err.get("message").expect("message");
    assert_eq!(msg, "Missing concept");
}

#[test]
fn lua_script_error_handling() {
    let lua = Lua::new();
    let result: Result<i64, mlua::Error> = lua.load("return 1 + nil").eval();
    assert!(
        result.is_err(),
        "Lua should return error for invalid arithmetic"
    );
}

#[test]
fn lua_hexorder_module_has_version() {
    let lua = Lua::new();
    let module = super::lua_api::create_hexorder_module(&lua).expect("create module");
    let version: String = module.get("version").expect("version");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}
