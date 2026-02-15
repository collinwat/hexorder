# Plugin Log: game_system

## Status: in-progress

## Decision Log

| Date       | Decision                                                                                           | Rationale                                                                                                                                                                                  |
| ---------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 2026-02-09 | Contract types already existed in `src/contracts/game_system.rs` -- no contract changes needed     | Types were defined ahead of consumers as part of spec process                                                                                                                              |
| 2026-02-09 | Three chained Startup systems: setup_game_system, setup_cell_type_registry, setup_active_cell_type | Chain ensures registry is available when active cell type reads it                                                                                                                         |
| 2026-02-09 | Moved resource creation from Startup systems to `build()` using `app.insert_resource()`            | Deferred `commands.insert_resource()` in Startup caused cross-plugin ordering crash — CellPlugin's Startup expected resources before they were flushed. Factory functions replace systems. |
| 2026-02-09 | Active cell type defaults to first registry entry using `registry.first().map(...)`                | Avoids unwrap; returns None if registry happens to be empty                                                                                                                                |
| 2026-02-09 | Plugin placed before cell/editor_ui in main.rs load order                                          | Required by coordination.md -- cell and editor_ui plugins depend on game_system resources                                                                                                  |

### 2026-02-15 — Registry-based type system architecture (0.7.0)

**Context**: Pitch #81 requires compound property types (EntityRef, List, Map, Struct, IntRange,
FloatRange), reflection-driven form generation, and enum consolidation. Two approaches considered.

**Decision**: Registry-based type system with recursive rendering (Approach B). Separate
`EnumRegistry` and `StructRegistry` resources. Recursive match-based property renderer with 3-level
depth cap. New Enums/Structs editor tabs.

**Rationale**: Reusable definitions (enum used as Map keys and standalone, struct shared across
entity types), clean separation from EntityTypeRegistry, consistent with project's registry pattern
(ConceptRegistry, RelationRegistry, etc.), recursive renderer keeps code in one place.

**Alternatives rejected**: Flat extensions (add variants, keep enums in EntityTypeRegistry, extend
hand-coded editor). Rejected because no definition reuse, editor code grows linearly with type
count, enums stay coupled to entity registry.

**Full design**: `docs/plans/2026-02-15-property-system-design.md`

## Test Results

| Date       | Command                       | Result | Notes                                       |
| ---------- | ----------------------------- | ------ | ------------------------------------------- |
| 2026-02-09 | `cargo build`                 | PASS   | Clean compilation with plugin registered    |
| 2026-02-09 | `cargo clippy -- -D warnings` | PASS   | Zero warnings                               |
| 2026-02-09 | `cargo test`                  | PASS   | 54/54 tests pass (10 new game_system tests) |

### Tests implemented (10):

1. `game_system_resource_exists_after_startup` -- GameSystem has non-empty id and version "0.1.0"
2. `registry_has_starter_types` -- CellTypeRegistry has 5 types
3. `starter_types_have_names` -- all starter types have non-empty names
4. `starter_types_have_distinct_ids` -- all IDs are unique
5. `active_cell_type_defaults_to_first` -- ActiveCellType references first type
6. `property_type_variants_are_distinct` -- all 6 PropertyType variants distinguishable
7. `property_value_default_for_each_type` -- PropertyValue::default_for returns correct variant
8. `type_id_generates_unique` -- two TypeId::new() calls produce different values
9. `registry_get_by_id` -- CellTypeRegistry::get() finds a type by id
10. `registry_get_enum_by_id` -- CellTypeRegistry::get_enum() works

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
| (none)  |            |        |          |

## Status Updates

| Date       | Status      | Notes                                            |
| ---------- | ----------- | ------------------------------------------------ |
| 2026-02-08 | speccing    | Initial spec created for M2                      |
| 2026-02-09 | complete    | Plugin implemented, all tests pass, clippy clean |
| 2026-02-15 | in-progress | 0.7.0 property system foundation (pitch #81)     |

### 2026-02-15 — 0.7.0 Build Results

| Command                      | Result | Notes                             |
| ---------------------------- | ------ | --------------------------------- |
| `mise check:audit`           | PASS   | Full constitution audit clean     |
| `cargo test`                 | PASS   | 151 tests pass (11 new)           |
| `cargo clippy --all-targets` | PASS   | Zero warnings                     |
| `mise check:boundary`        | PASS   | No cross-plugin import violations |
| `mise check:unwrap`          | PASS   | No unwrap() in production code    |

**New tests added (11):**

1. `enum_registry_insert_and_get` — CRUD on EnumRegistry
2. `enum_registry_ron_round_trip` — RON serialization for EnumRegistry
3. `struct_registry_insert_and_get` — CRUD on StructRegistry
4. `struct_registry_ron_round_trip` — RON serialization for StructRegistry
5. `property_type_new_variants_are_distinct` — 6 new PropertyType variants are distinguishable
6. `compound_property_value_ron_round_trip` — RON round-trip for compound PropertyValue variants
7. `nested_compound_type_ron_round_trip` — 3-level nested compound type serialization
8. `property_value_default_for_new_types` — default_for() returns correct defaults for new types
9. `enum_registry_exists_after_startup` — EnumRegistry resource inserted with starter data
10. `struct_registry_exists_after_startup` — StructRegistry resource inserted
11. `v2_save_and_load_includes_registries` — persistence v2 round-trip with registries
