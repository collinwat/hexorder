# Feature: Scripting

## Summary

Embedded Lua (LuaJIT) scripting layer for game rule definitions and integration test automation.
Provides read-only access to game system registries from Lua scripts.

## Plugin

- Module: `src/scripting/`
- Plugin struct: `ScriptingPlugin`
- Schedule: `Startup` (Lua VM initialization)

## Dependencies

- **Contracts consumed**: `game_system`, `ontology`, `validation`
- **Contracts produced**: none (read-only for 0.5.0)
- **Crate dependencies**: `mlua = { version = "0.11", features = ["luajit", "vendored"] }`

## Requirements

1. [REQ-1] Initialize an embedded Lua VM on startup and store it as a NonSend resource.
2. [REQ-2] Expose a global `hexorder` module table with a `version` field.
3. [REQ-3] Provide functions to convert `EntityTypeRegistry` to Lua tables.
4. [REQ-4] Provide functions to convert `ConceptRegistry` to Lua tables.
5. [REQ-5] Provide functions to convert `RelationRegistry` to Lua tables.
6. [REQ-6] Provide functions to convert `ConstraintRegistry` to Lua tables.
7. [REQ-7] Provide functions to convert `SchemaValidation` to Lua tables.
8. [REQ-8] All Lua API access is read-only (no mutations from Lua side).

## Success Criteria

- [x] [SC-1] `lua_hexorder_module_has_version` test passes
- [x] [SC-2] `lua_can_query_entity_types` test passes
- [x] [SC-3] `lua_entity_type_has_correct_fields` test passes
- [x] [SC-4] `lua_entity_type_properties` test passes
- [x] [SC-5] `lua_filters_entity_types_by_role` test passes
- [x] [SC-6] `lua_can_query_concepts` test passes
- [x] [SC-7] `lua_can_query_relations` test passes
- [x] [SC-8] `lua_can_query_constraints` test passes
- [x] [SC-9] `lua_schema_validation_valid` test passes
- [x] [SC-10] `lua_schema_validation_with_errors` test passes
- [x] [SC-11] `lua_script_error_handling` test passes
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes (all tests)
- [x] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- `mlua::Lua` is `!Send` — must be stored as `NonSend` resource
- Lua tables are used instead of `UserData` to keep the API stable across refactors
- Write access deferred to a future release
