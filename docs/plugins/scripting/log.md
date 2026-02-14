# Plugin Log: Scripting

## Status: complete

## Decision Log

### 2026-02-13 — Use plain Lua tables instead of UserData

**Context**: Need to expose game registries to Lua. Two approaches: Lua tables (copy data) vs
UserData (zero-copy references). **Decision**: Plain Lua tables for all conversions. **Rationale**:
Tables are simple, stable across refactors, and avoid lifetime coupling between Lua and Rust.
Read-only access means no synchronization needed. **Alternatives rejected**: UserData coupling —
more complex, ties Lua types to Rust struct layout.

### 2026-02-13 — NonSend resource for Lua VM

**Context**: `mlua::Lua` is `!Send`, cannot be a normal Bevy `Resource`. **Decision**: Store as
`NonSend<LuaState>` using `world.insert_non_send_resource()`. **Rationale**: Standard pattern for
`!Send` types in Bevy. The `init_lua` system takes `&mut World` for exclusive access.

### 2026-02-13 — LuaJIT with vendored feature

**Context**: Need to choose Lua version and build strategy. **Decision**: `mlua` with `luajit` and
`vendored` features. **Rationale**: LuaJIT is fast and widely used for game scripting. `vendored`
bundles the source so no system dependency is needed. MIT license (allowed).

## Test Results

### 2026-02-13 — Initial implementation

```
11 tests pass: lua_can_query_entity_types, lua_entity_type_has_correct_fields,
lua_entity_type_properties, lua_filters_entity_types_by_role, lua_can_query_concepts,
lua_can_query_relations, lua_can_query_constraints, lua_schema_validation_valid,
lua_schema_validation_with_errors, lua_script_error_handling, lua_hexorder_module_has_version
```

**Result**: pass

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Status Updates

| Date       | Status   | Notes                                     |
| ---------- | -------- | ----------------------------------------- |
| 2026-02-13 | complete | 11 tests pass, clippy clean, build passes |
