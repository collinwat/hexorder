//! Tests for the `scripting` plugin.

use mlua::Lua;

use hexorder_contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType, PropertyValue,
    TypeId,
};
use hexorder_contracts::ontology::{
    Concept, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr, ConstraintRegistry,
    Relation, RelationEffect, RelationRegistry, RelationTrigger,
};
use hexorder_contracts::validation::{SchemaError, SchemaErrorCategory, SchemaValidation};

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
                operation: hexorder_contracts::ontology::ModifyOperation::Subtract,
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
                operator: hexorder_contracts::ontology::CompareOp::Ge,
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

// ---------------------------------------------------------------------------
// Additional coverage: property value conversions (lua_api.rs)
// ---------------------------------------------------------------------------

/// Test Bool property value converts to Lua boolean.
#[test]
fn lua_property_value_bool() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "is_passable".to_string(),
                property_type: PropertyType::Bool,
                default_value: PropertyValue::Bool(true),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: bool = prop.get("default_value").expect("default_value");
    assert!(val);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "bool");
}

/// Test Float property value converts to Lua number.
#[test]
fn lua_property_value_float() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "speed".to_string(),
                property_type: PropertyType::Float,
                default_value: PropertyValue::Float(2.5),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: f64 = prop.get("default_value").expect("default_value");
    assert!((val - 2.5).abs() < f64::EPSILON);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "float");
}

/// Test String property value converts to Lua string.
#[test]
fn lua_property_value_string() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "label".to_string(),
                property_type: PropertyType::String,
                default_value: PropertyValue::String("hello".to_string()),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: String = prop.get("default_value").expect("default_value");
    assert_eq!(val, "hello");
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "string");
}

/// Test Color property value converts to Lua table with r/g/b/a.
#[test]
fn lua_property_value_color() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::BoardPosition,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "tint".to_string(),
                property_type: PropertyType::Color,
                default_value: PropertyValue::Color(bevy::color::Color::srgb(1.0, 0.0, 0.0)),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let color_table: mlua::Table = prop.get("default_value").expect("default_value");
    let r: f32 = color_table.get("r").expect("r");
    let g: f32 = color_table.get("g").expect("g");
    let b: f32 = color_table.get("b").expect("b");
    let a: f32 = color_table.get("a").expect("a");
    assert!(r > 0.0);
    assert!(g < 0.01);
    assert!(b < 0.01);
    assert!((a - 1.0).abs() < f32::EPSILON);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "color");
}

/// Test Enum property value converts to Lua string.
#[test]
fn lua_property_value_enum() {
    let enum_id = TypeId::new();
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "movement_mode".to_string(),
                property_type: PropertyType::Enum(enum_id),
                default_value: PropertyValue::Enum("Foot".to_string()),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: String = prop.get("default_value").expect("default_value");
    assert_eq!(val, "Foot");
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "enum");
}

/// `EntityRef` property value: `Some` converts to string, `None` to nil.
#[test]
#[allow(clippy::similar_names)]
fn lua_property_value_entity_ref() {
    let ref_id = TypeId::new();
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "target".to_string(),
                    property_type: PropertyType::EntityRef(None),
                    default_value: PropertyValue::EntityRef(Some(ref_id)),
                },
                PropertyDefinition {
                    id: TypeId::new(),
                    name: "empty_ref".to_string(),
                    property_type: PropertyType::EntityRef(Some(EntityRole::Token)),
                    default_value: PropertyValue::EntityRef(None),
                },
            ],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let properties: mlua::Table = entity.get("properties").expect("properties");

    // First property: EntityRef(Some(id)) -> string
    let first_prop: mlua::Table = properties.get(1).expect("first prop");
    let val1: String = first_prop.get("default_value").expect("default_value");
    assert!(!val1.is_empty());
    let type_str1: String = first_prop.get("type").expect("type");
    assert_eq!(type_str1, "entity_ref");

    // Second property: EntityRef(None) -> nil
    let second_prop: mlua::Table = properties.get(2).expect("second prop");
    let val2: mlua::Value = second_prop.get("default_value").expect("default_value");
    assert!(matches!(val2, mlua::Value::Nil));
}

/// Test List property value converts to Lua table array.
#[test]
fn lua_property_value_list() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "tags".to_string(),
                property_type: PropertyType::List(Box::new(PropertyType::Int)),
                default_value: PropertyValue::List(vec![
                    PropertyValue::Int(10),
                    PropertyValue::Int(20),
                ]),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let list: mlua::Table = prop.get("default_value").expect("default_value");
    let first: i64 = list.get(1).expect("first element");
    let second: i64 = list.get(2).expect("second element");
    assert_eq!(first, 10);
    assert_eq!(second, 20);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "list");
}

/// Test Map property value converts to Lua table.
#[test]
fn lua_property_value_map() {
    let enum_id = TypeId::new();
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "costs".to_string(),
                property_type: PropertyType::Map(enum_id, Box::new(PropertyType::Int)),
                default_value: PropertyValue::Map(vec![
                    ("plains".to_string(), PropertyValue::Int(1)),
                    ("forest".to_string(), PropertyValue::Int(3)),
                ]),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let map: mlua::Table = prop.get("default_value").expect("default_value");
    let plains: i64 = map.get("plains").expect("plains");
    let forest: i64 = map.get("forest").expect("forest");
    assert_eq!(plains, 1);
    assert_eq!(forest, 3);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "map");
}

/// Test Struct property value converts to Lua table with field IDs as keys.
#[test]
fn lua_property_value_struct() {
    use std::collections::HashMap;

    let struct_id = TypeId::new();
    let field_id = TypeId::new();
    let lua = Lua::new();

    let mut fields = HashMap::new();
    fields.insert(field_id, PropertyValue::Int(42));

    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "stats".to_string(),
                property_type: PropertyType::Struct(struct_id),
                default_value: PropertyValue::Struct(fields),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let struct_val: mlua::Table = prop.get("default_value").expect("default_value");
    // The key is the field_id UUID as a string
    let val: i64 = struct_val.get(field_id.0.to_string()).expect("field value");
    assert_eq!(val, 42);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "struct");
}

/// `IntRange` property value converts to Lua integer.
#[test]
fn lua_property_value_int_range() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "attack".to_string(),
                property_type: PropertyType::IntRange { min: 0, max: 10 },
                default_value: PropertyValue::IntRange(5),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: i64 = prop.get("default_value").expect("default_value");
    assert_eq!(val, 5);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "int_range");
}

/// `FloatRange` property value converts to Lua number.
#[test]
fn lua_property_value_float_range() {
    let lua = Lua::new();
    let registry = EntityTypeRegistry {
        types: vec![EntityType {
            id: TypeId::new(),
            name: "TestType".to_string(),
            role: EntityRole::Token,
            color: bevy::color::Color::srgb(0.5, 0.5, 0.5),
            properties: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "morale".to_string(),
                property_type: PropertyType::FloatRange { min: 0.0, max: 1.0 },
                default_value: PropertyValue::FloatRange(0.75),
            }],
        }],
    };
    let result = lua_api::entity_types_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let entity: mlua::Table = table.get(1).expect("first");
    let props: mlua::Table = entity.get("properties").expect("properties");
    let prop: mlua::Table = props.get(1).expect("first prop");
    let val: f64 = prop.get("default_value").expect("default_value");
    assert!((val - 0.75).abs() < f64::EPSILON);
    let type_str: String = prop.get("type").expect("type");
    assert_eq!(type_str, "float_range");
}

/// Test empty registries produce empty Lua tables.
#[test]
fn lua_empty_registries() {
    let lua = Lua::new();

    let empty_et = EntityTypeRegistry { types: vec![] };
    let result = lua_api::entity_types_to_lua(&lua, &empty_et).expect("convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 0);

    let empty_concepts = ConceptRegistry {
        concepts: vec![],
        bindings: vec![],
    };
    let result = lua_api::concepts_to_lua(&lua, &empty_concepts).expect("convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 0);

    let empty_relations = RelationRegistry { relations: vec![] };
    let result = lua_api::relations_to_lua(&lua, &empty_relations).expect("convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 0);

    let empty_constraints = ConstraintRegistry {
        constraints: vec![],
    };
    let result = lua_api::constraints_to_lua(&lua, &empty_constraints).expect("convert");
    let table = result.as_table().expect("table");
    assert_eq!(table.len().expect("len"), 0);
}

/// Concept role details: id, name, `allowed_entity_roles`.
#[test]
fn lua_concept_role_details() {
    let lua = Lua::new();
    let registry = test_concept_registry();
    let result = lua_api::concepts_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let concept: mlua::Table = table.get(1).expect("first concept");

    // Check concept fields
    let _id: String = concept.get("id").expect("concept id");
    let description: String = concept.get("description").expect("description");
    assert_eq!(description, "Movement across the board");

    // Check role details
    let roles: mlua::Table = concept.get("roles").expect("roles");
    let first_role: mlua::Table = roles.get(1).expect("first role");
    let role_name: String = first_role.get("name").expect("role name");
    assert_eq!(role_name, "traveler");
    let _role_id: String = first_role.get("id").expect("role id");
    let allowed: mlua::Table = first_role.get("allowed_entity_roles").expect("allowed");
    let first_allowed: String = allowed.get(1).expect("first allowed");
    assert_eq!(first_allowed, "token");
}

/// Test relation fields are correctly converted.
#[test]
fn lua_relation_fields() {
    let lua = Lua::new();
    let registry = test_relation_registry();
    let result = lua_api::relations_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let relation: mlua::Table = table.get(1).expect("first relation");

    let _id: String = relation.get("id").expect("id");
    let _concept_id: String = relation.get("concept_id").expect("concept_id");
    let _subject_role_id: String = relation.get("subject_role_id").expect("subject_role_id");
    let _object_role_id: String = relation.get("object_role_id").expect("object_role_id");
}

/// Constraint fields including `auto_generated` = true.
#[test]
fn lua_constraint_auto_generated_true() {
    let lua = Lua::new();
    let registry = ConstraintRegistry {
        constraints: vec![Constraint {
            id: TypeId::new(),
            name: "Auto constraint".to_string(),
            description: "Auto-generated".to_string(),
            concept_id: TypeId::new(),
            relation_id: None,
            expression: ConstraintExpr::PropertyCompare {
                role_id: TypeId::new(),
                property_name: "x".to_string(),
                operator: hexorder_contracts::ontology::CompareOp::Ge,
                value: PropertyValue::Int(0),
            },
            auto_generated: true,
        }],
    };
    let result = lua_api::constraints_to_lua(&lua, &registry).expect("convert");
    let table = result.as_table().expect("table");
    let constraint: mlua::Table = table.get(1).expect("first");
    let auto: bool = constraint.get("auto_generated").expect("auto_generated");
    assert!(auto);
    let _id: String = constraint.get("id").expect("id");
    let _concept_id: String = constraint.get("concept_id").expect("concept_id");
    let desc: String = constraint.get("description").expect("description");
    assert_eq!(desc, "Auto-generated");
}

// ---------------------------------------------------------------------------
// Additional coverage: scripting/mod.rs (ScriptingPlugin) and systems.rs
// ---------------------------------------------------------------------------

/// Test that `ScriptingPlugin` builds without panicking and `init_lua`
/// creates the `LuaState` non-send resource with the hexorder module.
#[test]
fn scripting_plugin_registers_init_lua_system() {
    use bevy::prelude::*;
    use hexorder_contracts::persistence::AppScreen;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    // Start in Launcher, then transition to Editor to trigger OnEnter.
    app.insert_state(AppScreen::Launcher);
    app.add_plugins(super::ScriptingPlugin);

    // First update to initialize.
    app.update();

    // LuaState should NOT exist yet (we are in Launcher, not Editor).
    assert!(
        app.world()
            .get_non_send_resource::<super::systems::LuaState>()
            .is_none(),
        "LuaState should not exist before entering Editor"
    );

    // Transition to Editor.
    app.world_mut()
        .resource_mut::<NextState<AppScreen>>()
        .set(AppScreen::Editor);
    app.update();

    // LuaState should now exist.
    let lua_state = app
        .world()
        .get_non_send_resource::<super::systems::LuaState>();
    assert!(
        lua_state.is_some(),
        "LuaState should exist after entering Editor"
    );
}

/// `LuaState` debug impl works.
#[test]
fn lua_state_debug_impl() {
    let state = super::systems::LuaState {
        lua: mlua::Lua::new(),
    };
    let debug = format!("{state:?}");
    assert!(debug.contains("LuaState"));
}

/// `init_lua` creates a working Lua VM with the hexorder global.
#[test]
fn init_lua_registers_hexorder_global() {
    use bevy::prelude::*;

    let mut world = World::new();
    super::systems::init_lua(&mut world);

    let lua_state = world
        .get_non_send_resource::<super::systems::LuaState>()
        .expect("LuaState should be inserted");

    // Verify the hexorder global is set with a version field.
    let version: String = lua_state
        .lua
        .load("return hexorder.version")
        .eval()
        .expect("should eval version");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}
