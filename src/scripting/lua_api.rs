//! Lua API: converts game system registries into Lua tables.
//!
//! All access is read-only for 0.5.0. The Lua side receives plain tables
//! (no `UserData` coupling), making the API stable across refactors.

use mlua::{Lua, Result as LuaResult, Table, Value};

use crate::contracts::game_system::{
    EntityRole, EntityType, EntityTypeRegistry, PropertyDefinition, PropertyType, PropertyValue,
};
use crate::contracts::ontology::{
    Concept, ConceptRegistry, Constraint, ConstraintRegistry, Relation, RelationRegistry,
};
use crate::contracts::validation::SchemaValidation;

/// Create the top-level `hexorder` module table with query functions.
///
/// Functions are closures that capture nothing — they receive `World` state
/// at call time through the Lua registry mechanism.
pub fn create_hexorder_module(lua: &Lua) -> LuaResult<Table> {
    let module = lua.create_table()?;
    module.set("version", env!("CARGO_PKG_VERSION"))?;
    Ok(module)
}

/// Convert the `EntityTypeRegistry` into a Lua table array.
pub fn entity_types_to_lua(lua: &Lua, registry: &EntityTypeRegistry) -> LuaResult<Value> {
    let table = lua.create_table()?;
    for (i, entity_type) in registry.types.iter().enumerate() {
        table.set(i + 1, entity_type_to_lua(lua, entity_type)?)?;
    }
    Ok(Value::Table(table))
}

/// Convert a single `EntityType` to a Lua table.
fn entity_type_to_lua(lua: &Lua, et: &EntityType) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("id", et.id.0.to_string())?;
    table.set("name", et.name.clone())?;
    table.set("role", role_to_string(et.role))?;

    let props = lua.create_table()?;
    for (j, prop) in et.properties.iter().enumerate() {
        props.set(j + 1, property_def_to_lua(lua, prop)?)?;
    }
    table.set("properties", props)?;

    Ok(table)
}

/// Convert a `PropertyDefinition` to a Lua table.
fn property_def_to_lua(lua: &Lua, prop: &PropertyDefinition) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("id", prop.id.0.to_string())?;
    table.set("name", prop.name.clone())?;
    table.set("type", property_type_to_string(&prop.property_type))?;
    table.set(
        "default_value",
        property_value_to_lua(lua, &prop.default_value)?,
    )?;
    Ok(table)
}

/// Convert a `PropertyValue` to the closest Lua equivalent.
fn property_value_to_lua(lua: &Lua, val: &PropertyValue) -> LuaResult<Value> {
    match val {
        PropertyValue::Bool(b) => Ok(Value::Boolean(*b)),
        PropertyValue::Int(i) => Ok(Value::Integer(*i)),
        PropertyValue::Float(f) => Ok(Value::Number(*f)),
        PropertyValue::String(s) | PropertyValue::Enum(s) => {
            Ok(Value::String(lua.create_string(s)?))
        }
        PropertyValue::Color(c) => {
            let linear = c.to_linear();
            let t = lua.create_table()?;
            t.set("r", linear.red)?;
            t.set("g", linear.green)?;
            t.set("b", linear.blue)?;
            t.set("a", linear.alpha)?;
            Ok(Value::Table(t))
        }
    }
}

/// Convert `EntityRole` to a string for Lua.
fn role_to_string(role: EntityRole) -> &'static str {
    match role {
        EntityRole::BoardPosition => "board_position",
        EntityRole::Token => "token",
    }
}

/// Convert `PropertyType` to a string for Lua.
fn property_type_to_string(pt: &PropertyType) -> &'static str {
    match pt {
        PropertyType::Bool => "bool",
        PropertyType::Int => "int",
        PropertyType::Float => "float",
        PropertyType::String => "string",
        PropertyType::Color => "color",
        PropertyType::Enum(_) => "enum",
    }
}

/// Convert the `ConceptRegistry` concepts into a Lua table array.
pub fn concepts_to_lua(lua: &Lua, registry: &ConceptRegistry) -> LuaResult<Value> {
    let table = lua.create_table()?;
    for (i, concept) in registry.concepts.iter().enumerate() {
        table.set(i + 1, concept_to_lua(lua, concept)?)?;
    }
    Ok(Value::Table(table))
}

/// Convert a single `Concept` to a Lua table.
fn concept_to_lua(lua: &Lua, c: &Concept) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("id", c.id.0.to_string())?;
    table.set("name", c.name.clone())?;
    table.set("description", c.description.clone())?;

    let roles = lua.create_table()?;
    for (j, role) in c.role_labels.iter().enumerate() {
        let role_table = lua.create_table()?;
        role_table.set("id", role.id.0.to_string())?;
        role_table.set("name", role.name.clone())?;
        let allowed = lua.create_table()?;
        for (k, er) in role.allowed_entity_roles.iter().enumerate() {
            allowed.set(k + 1, role_to_string(*er))?;
        }
        role_table.set("allowed_entity_roles", allowed)?;
        roles.set(j + 1, role_table)?;
    }
    table.set("roles", roles)?;

    Ok(table)
}

/// Convert the `RelationRegistry` into a Lua table array.
pub fn relations_to_lua(lua: &Lua, registry: &RelationRegistry) -> LuaResult<Value> {
    let table = lua.create_table()?;
    for (i, relation) in registry.relations.iter().enumerate() {
        table.set(i + 1, relation_to_lua(lua, relation)?)?;
    }
    Ok(Value::Table(table))
}

/// Convert a single `Relation` to a Lua table.
fn relation_to_lua(lua: &Lua, r: &Relation) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("id", r.id.0.to_string())?;
    table.set("name", r.name.clone())?;
    table.set("concept_id", r.concept_id.0.to_string())?;
    table.set("subject_role_id", r.subject_role_id.0.to_string())?;
    table.set("object_role_id", r.object_role_id.0.to_string())?;
    Ok(table)
}

/// Convert the `ConstraintRegistry` into a Lua table array.
pub fn constraints_to_lua(lua: &Lua, registry: &ConstraintRegistry) -> LuaResult<Value> {
    let table = lua.create_table()?;
    for (i, constraint) in registry.constraints.iter().enumerate() {
        table.set(i + 1, constraint_to_lua(lua, constraint)?)?;
    }
    Ok(Value::Table(table))
}

/// Convert a single `Constraint` to a Lua table.
fn constraint_to_lua(lua: &Lua, c: &Constraint) -> LuaResult<Table> {
    let table = lua.create_table()?;
    table.set("id", c.id.0.to_string())?;
    table.set("name", c.name.clone())?;
    table.set("description", c.description.clone())?;
    table.set("concept_id", c.concept_id.0.to_string())?;
    table.set("auto_generated", c.auto_generated)?;
    Ok(table)
}

/// Convert `SchemaValidation` to a Lua table.
pub fn schema_validation_to_lua(lua: &Lua, sv: &SchemaValidation) -> LuaResult<Value> {
    let table = lua.create_table()?;
    table.set("is_valid", sv.is_valid)?;
    table.set("error_count", sv.errors.len())?;

    let errors = lua.create_table()?;
    for (i, err) in sv.errors.iter().enumerate() {
        let err_table = lua.create_table()?;
        err_table.set("message", err.message.clone())?;
        err_table.set("category", format!("{:?}", err.category))?;
        errors.set(i + 1, err_table)?;
    }
    table.set("errors", errors)?;

    Ok(Value::Table(table))
}
