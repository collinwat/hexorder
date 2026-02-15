# Property System Foundation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Extend the property type system with compound types (EntityRef, List, Map, Struct,
IntRange, FloatRange), extract enums and structs into standalone registries, build a recursive
property editor, and migrate persistence to v2.

**Architecture:** Registry-based type system. `EnumRegistry` and `StructRegistry` are standalone
Bevy resources. `PropertyType`/`PropertyValue` gain 6 compound variants each. The editor uses a
recursive match-based renderer with a 3-level depth cap. Persistence migrates from format v1 to v2.

**Tech Stack:** Rust, Bevy 0.18, bevy_egui 0.39, RON serialization, uuid

**Design doc:** `docs/plans/2026-02-15-property-system-design.md`

**Key files to understand before starting:**

- `src/contracts/game_system.rs` — PropertyType, PropertyValue, EnumDefinition, EntityTypeRegistry
- `src/game_system/systems.rs` — factory functions for starter data
- `src/game_system/tests.rs` — existing property system tests
- `src/editor_ui/systems.rs` — `render_property_value_editor` (line 1603), `format_property_type`
  (line 1905), `index_to_property_type` (line 1916)
- `src/editor_ui/components.rs` — `EditorAction`, `EditorState`, `OntologyTab`
- `src/contracts/persistence.rs` — `GameSystemFile`, `FORMAT_VERSION`
- `src/scripting/lua_api.rs` — PropertyType match at line 96

**Test command:** `cargo test` **Lint command:** `cargo clippy --all-targets` **Full check:**
`mise check`

---

## Task 1: Add EnumRegistry and StructRegistry contract types

These are additive — no existing code changes, no breakage.

**Files:**

- Modify: `src/contracts/game_system.rs`

**Step 1: Write failing tests for EnumRegistry**

Add to the bottom of the `#[cfg(test)] mod tests` block in `src/contracts/game_system.rs`:

```rust
#[test]
fn enum_registry_insert_and_get() {
    let mut reg = EnumRegistry::default();
    let id = TypeId::new();
    let def = EnumDefinition {
        id,
        name: "Terrain".to_string(),
        options: vec!["Grass".to_string(), "Sand".to_string()],
    };
    reg.definitions.insert(id, def);
    assert_eq!(reg.definitions.len(), 1);
    assert_eq!(reg.get(id).expect("should find").name, "Terrain");
    assert!(reg.get(TypeId::new()).is_none());
}

#[test]
fn enum_registry_ron_round_trip() {
    let mut reg = EnumRegistry::default();
    let id = TypeId::new();
    reg.definitions.insert(
        id,
        EnumDefinition {
            id,
            name: "Side".to_string(),
            options: vec!["Axis".to_string(), "Allied".to_string()],
        },
    );
    let ron_str = ron::to_string(&reg).expect("serialize");
    let loaded: EnumRegistry = ron::from_str(&ron_str).expect("deserialize");
    assert_eq!(loaded.definitions.len(), 1);
    assert_eq!(loaded.get(id).expect("should find").options.len(), 2);
}
```

**Step 2: Run tests — expect compile error (EnumRegistry not defined)**

Run: `cargo test --lib contracts::game_system` Expected: FAIL — `EnumRegistry` not found

**Step 3: Implement EnumRegistry**

Add after the `EnumDefinition` struct in `src/contracts/game_system.rs`:

```rust
/// Standalone registry of all designer-defined enum definitions.
/// Replaces `EntityTypeRegistry.enum_definitions` (0.7.0).
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct EnumRegistry {
    pub definitions: HashMap<TypeId, EnumDefinition>,
}

impl EnumRegistry {
    /// Look up an enum definition by its ID.
    pub fn get(&self, id: TypeId) -> Option<&EnumDefinition> {
        self.definitions.get(&id)
    }

    /// Look up a mutable enum definition by its ID.
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut EnumDefinition> {
        self.definitions.get_mut(&id)
    }

    /// Insert or replace an enum definition.
    pub fn insert(&mut self, def: EnumDefinition) {
        self.definitions.insert(def.id, def);
    }

    /// Remove an enum definition by ID. Returns the removed definition.
    pub fn remove(&mut self, id: TypeId) -> Option<EnumDefinition> {
        self.definitions.remove(&id)
    }
}
```

**Step 4: Run tests — expect pass**

Run: `cargo test --lib contracts::game_system` Expected: PASS

**Step 5: Write failing tests for StructRegistry**

Add to the same test module:

```rust
#[test]
fn struct_registry_insert_and_get() {
    let mut reg = StructRegistry::default();
    let id = TypeId::new();
    let def = StructDefinition {
        id,
        name: "CombatProfile".to_string(),
        fields: vec![
            PropertyDefinition {
                id: TypeId::new(),
                name: "attack".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(0),
            },
            PropertyDefinition {
                id: TypeId::new(),
                name: "defense".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(0),
            },
        ],
    };
    reg.definitions.insert(id, def);
    assert_eq!(reg.definitions.len(), 1);
    assert_eq!(reg.get(id).expect("should find").name, "CombatProfile");
    assert_eq!(reg.get(id).expect("should find").fields.len(), 2);
    assert!(reg.get(TypeId::new()).is_none());
}

#[test]
fn struct_registry_ron_round_trip() {
    let mut reg = StructRegistry::default();
    let id = TypeId::new();
    reg.definitions.insert(
        id,
        StructDefinition {
            id,
            name: "Stats".to_string(),
            fields: vec![PropertyDefinition {
                id: TypeId::new(),
                name: "hp".to_string(),
                property_type: PropertyType::Int,
                default_value: PropertyValue::Int(10),
            }],
        },
    );
    let ron_str = ron::to_string(&reg).expect("serialize");
    let loaded: StructRegistry = ron::from_str(&ron_str).expect("deserialize");
    assert_eq!(loaded.definitions.len(), 1);
    assert_eq!(loaded.get(id).expect("should find").fields.len(), 1);
}
```

**Step 6: Run tests — expect compile error**

Run: `cargo test --lib contracts::game_system` Expected: FAIL — `StructRegistry`, `StructDefinition`
not found

**Step 7: Implement StructRegistry and StructDefinition**

Add after `EnumRegistry` in `src/contracts/game_system.rs`:

```rust
/// A named composite type — a list of typed, named fields.
/// Registered centrally so multiple entity types can reference the same struct schema.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct StructDefinition {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<PropertyDefinition>,
}

/// Standalone registry of all designer-defined struct definitions (0.7.0).
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct StructRegistry {
    pub definitions: HashMap<TypeId, StructDefinition>,
}

impl StructRegistry {
    /// Look up a struct definition by its ID.
    pub fn get(&self, id: TypeId) -> Option<&StructDefinition> {
        self.definitions.get(&id)
    }

    /// Look up a mutable struct definition by its ID.
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut StructDefinition> {
        self.definitions.get_mut(&id)
    }

    /// Insert or replace a struct definition.
    pub fn insert(&mut self, def: StructDefinition) {
        self.definitions.insert(def.id, def);
    }

    /// Remove a struct definition by ID. Returns the removed definition.
    pub fn remove(&mut self, id: TypeId) -> Option<StructDefinition> {
        self.definitions.remove(&id)
    }
}
```

**Step 8: Run tests — expect pass**

Run: `cargo test --lib contracts::game_system` Expected: PASS

**Step 9: Run full test suite**

Run: `cargo test` Expected: PASS (all 140+ tests)

**Step 10: Commit**

```bash
git add src/contracts/game_system.rs
git commit -m "feat(contracts): add EnumRegistry and StructRegistry types"
```

---

## Task 2: Extend PropertyType with 6 new variants

This is the big breaking change — every `match` on PropertyType must be updated atomically.

**Files:**

- Modify: `src/contracts/game_system.rs` — add variants + update `default_for`
- Modify: `src/editor_ui/systems.rs` — update `format_property_type`
- Modify: `src/scripting/lua_api.rs` — update PropertyType match

**Step 1: Write failing test for new PropertyType variants**

Add to `src/contracts/game_system.rs` test module:

```rust
#[test]
fn property_type_new_variants_are_distinct() {
    let enum_id = TypeId::new();
    let struct_id = TypeId::new();
    let variants: Vec<PropertyType> = vec![
        PropertyType::EntityRef(None),
        PropertyType::EntityRef(Some(EntityRole::Token)),
        PropertyType::List(Box::new(PropertyType::Int)),
        PropertyType::Map(enum_id, Box::new(PropertyType::Int)),
        PropertyType::Struct(struct_id),
        PropertyType::IntRange { min: 0, max: 10 },
        PropertyType::FloatRange { min: 0.0, max: 1.0 },
    ];
    // Each new variant is distinct from every other.
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Variants {i} and {j} should differ");
            }
        }
    }
}
```

**Step 2: Run — expect compile error**

Run: `cargo test --lib contracts::game_system` Expected: FAIL — new variants not defined

**Step 3: Add the 6 new variants to PropertyType**

In `src/contracts/game_system.rs`, extend the `PropertyType` enum:

```rust
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    String,
    Color,
    /// References an `EnumDefinition` by its `TypeId`.
    Enum(TypeId),
    /// Reference to an entity type, optionally filtered by role.
    EntityRef(Option<EntityRole>),
    /// Ordered collection of a single inner property type.
    List(Box<PropertyType>),
    /// Map with enum keys (by TypeId) and typed values.
    Map(TypeId, Box<PropertyType>),
    /// Named composite referencing a `StructDefinition` by TypeId.
    Struct(TypeId),
    /// Bounded integer with min/max validation.
    IntRange { min: i64, max: i64 },
    /// Bounded float with min/max validation.
    FloatRange { min: f64, max: f64 },
}
```

**Step 4: Fix all match arms on PropertyType across the codebase**

The compiler will guide you. The exhaustive list of match sites:

1. `src/contracts/game_system.rs` — `PropertyValue::default_for()` (line ~84): add arms for each new
   variant:

    ```rust
    PropertyType::EntityRef(_) => PropertyValue::EntityRef(None),
    PropertyType::List(_) => PropertyValue::List(Vec::new()),
    PropertyType::Map(_, _) => PropertyValue::Map(Vec::new()),
    PropertyType::Struct(_) => PropertyValue::Struct(HashMap::new()),
    PropertyType::IntRange { min, .. } => PropertyValue::IntRange(*min),
    PropertyType::FloatRange { min, .. } => PropertyValue::FloatRange(*min),
    ```

    (This will fail to compile until PropertyValue also has the new variants — do Task 3 step 3
    atomically with this.)

2. `src/editor_ui/systems.rs` — `format_property_type()` (line ~1905): add arms:

    ```rust
    PropertyType::EntityRef(_) => "EntityRef",
    PropertyType::List(_) => "List",
    PropertyType::Map(_, _) => "Map",
    PropertyType::Struct(_) => "Struct",
    PropertyType::IntRange { .. } => "IntRange",
    PropertyType::FloatRange { .. } => "FloatRange",
    ```

3. `src/scripting/lua_api.rs` — match at line ~96: add arms:

    ```rust
    PropertyType::EntityRef(_) => "entity_ref",
    PropertyType::List(_) => "list",
    PropertyType::Map(_, _) => "map",
    PropertyType::Struct(_) => "struct",
    PropertyType::IntRange { .. } => "int_range",
    PropertyType::FloatRange { .. } => "float_range",
    ```

**NOTE:** Steps 3 and 4 must be done together with Task 3 (PropertyValue extensions) because
`default_for` returns `PropertyValue` variants. Do Tasks 2+3 as a single compile unit.

---

## Task 3: Extend PropertyValue with 6 new variants

Do this atomically with Task 2 step 4 — both enums must be updated together for compilation.

**Files:**

- Modify: `src/contracts/game_system.rs` — add variants

**Step 1: Add new variants to PropertyValue**

```rust
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(bevy::color::Color),
    /// The selected option name from the referenced `EnumDefinition`.
    Enum(String),
    /// Reference to an entity type, or None if unset.
    EntityRef(Option<TypeId>),
    /// Ordered collection of values.
    List(Vec<PropertyValue>),
    /// Enum key name to value pairs (preserves insertion order for display).
    Map(Vec<(String, PropertyValue)>),
    /// Field values keyed by `PropertyDefinition` ID from the `StructDefinition`.
    Struct(HashMap<TypeId, PropertyValue>),
    /// Bounded integer value.
    IntRange(i64),
    /// Bounded float value.
    FloatRange(f64),
}
```

**Step 2: Update `default_for` (already described in Task 2 step 4)**

Ensure the 6 new arms are in the match.

**Step 3: Fix all match arms on PropertyValue**

The compiler will guide you. Known exhaustive match sites:

1. `src/editor_ui/systems.rs` — `render_property_value_editor()` (line ~1609): add placeholder arms
   for now (we'll build proper editors in Task 9):

    ```rust
    PropertyValue::EntityRef(_) => {
        ui.label("(EntityRef editor pending)");
    }
    PropertyValue::List(_) => {
        ui.label("(List editor pending)");
    }
    PropertyValue::Map(_) => {
        ui.label("(Map editor pending)");
    }
    PropertyValue::Struct(_) => {
        ui.label("(Struct editor pending)");
    }
    PropertyValue::IntRange(v) => {
        ui.add(egui::DragValue::new(v));
    }
    PropertyValue::FloatRange(v) => {
        ui.add(egui::DragValue::new(v).speed(0.1));
    }
    ```

    Note: The rules*engine `property_value_as_i64()` and other match sites use `* => ...` wildcards,
    so they won't need changes.

**Step 4: Write test for new default_for variants**

Add to `src/game_system/tests.rs`, extending `property_value_default_for_each_type`:

```rust
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
```

**Step 5: Write RON round-trip test for compound types**

Add to `src/contracts/game_system.rs` test module:

```rust
#[test]
fn compound_property_value_ron_round_trip() {
    use std::collections::HashMap;

    let values: Vec<PropertyValue> = vec![
        PropertyValue::EntityRef(Some(TypeId::new())),
        PropertyValue::EntityRef(None),
        PropertyValue::List(vec![PropertyValue::Int(1), PropertyValue::Int(2)]),
        PropertyValue::Map(vec![
            ("Grass".to_string(), PropertyValue::Int(1)),
            ("Sand".to_string(), PropertyValue::Int(2)),
        ]),
        PropertyValue::Struct({
            let mut m = HashMap::new();
            m.insert(TypeId::new(), PropertyValue::Int(5));
            m.insert(TypeId::new(), PropertyValue::String("test".to_string()));
            m
        }),
        PropertyValue::IntRange(7),
        PropertyValue::FloatRange(0.5),
    ];

    for value in &values {
        let ron_str = ron::to_string(value).expect("serialize");
        let loaded: PropertyValue = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(&loaded, value, "Round-trip failed for {value:?}");
    }
}

#[test]
fn nested_compound_type_ron_round_trip() {
    // Map containing Struct containing List — tests 3-level nesting.
    let field_id = TypeId::new();
    let inner = PropertyValue::Struct({
        let mut m = std::collections::HashMap::new();
        m.insert(
            field_id,
            PropertyValue::List(vec![PropertyValue::Int(1), PropertyValue::Int(2)]),
        );
        m
    });
    let map_val = PropertyValue::Map(vec![("Key".to_string(), inner)]);

    let ron_str = ron::to_string(&map_val).expect("serialize");
    let loaded: PropertyValue = ron::from_str(&ron_str).expect("deserialize");
    assert_eq!(loaded, map_val);
}
```

**Step 6: Run full test suite**

Run: `cargo test` Expected: PASS

**Step 7: Run clippy**

Run: `cargo clippy --all-targets` Expected: PASS (zero warnings)

**Step 8: Commit**

```bash
git add src/contracts/game_system.rs src/game_system/tests.rs src/editor_ui/systems.rs src/scripting/lua_api.rs
git commit -m "feat(contracts): add 6 compound PropertyType and PropertyValue variants"
```

---

## Task 4: Wire up GameSystemPlugin to insert EnumRegistry and StructRegistry

**Files:**

- Modify: `src/game_system/mod.rs` — insert new resources in `build()`
- Modify: `src/game_system/tests.rs` — test new resources exist

**Step 1: Write failing tests**

Add to `src/game_system/tests.rs`:

```rust
use crate::contracts::game_system::EnumRegistry;
use crate::contracts::game_system::StructRegistry;

#[test]
fn enum_registry_exists_after_startup() {
    let mut app = test_app();
    app.update();
    let reg = app
        .world()
        .get_resource::<EnumRegistry>()
        .expect("EnumRegistry should exist");
    assert!(
        !reg.definitions.is_empty(),
        "EnumRegistry should not be empty (starter enums)"
    );
}

#[test]
fn struct_registry_exists_after_startup() {
    let mut app = test_app();
    app.update();
    app.world()
        .get_resource::<StructRegistry>()
        .expect("StructRegistry should exist");
}
```

**Step 2: Run — expect fail**

Run: `cargo test --lib game_system` Expected: FAIL — resources not found

**Step 3: Update GameSystemPlugin::build**

In `src/game_system/mod.rs`, add imports and insert the new resources before the EntityTypeRegistry:

```rust
use crate::contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EnumRegistry, EntityRole, SelectedUnit, StructRegistry,
};
```

In the `build()` method, before the `registry` line:

```rust
app.insert_resource(systems::create_enum_registry());
app.insert_resource(StructRegistry::default());
```

**Step 4: Add `create_enum_registry` factory function**

In `src/game_system/systems.rs`:

```rust
use crate::contracts::game_system::{
    EnumDefinition, EnumRegistry, EntityRole, EntityType, EntityTypeRegistry, GameSystem,
    PropertyDefinition, PropertyType, PropertyValue, TypeId,
};

/// Creates the default `EnumRegistry` with starter enum definitions.
pub fn create_enum_registry() -> EnumRegistry {
    let mut reg = EnumRegistry::default();

    let terrain_id = TypeId::new();
    reg.insert(EnumDefinition {
        id: terrain_id,
        name: "Terrain Type".to_string(),
        options: vec![
            "Open".to_string(),
            "Rough".to_string(),
            "Impassable".to_string(),
        ],
    });

    let movement_id = TypeId::new();
    reg.insert(EnumDefinition {
        id: movement_id,
        name: "Movement Mode".to_string(),
        options: vec![
            "Foot".to_string(),
            "Wheeled".to_string(),
            "Tracked".to_string(),
        ],
    });

    reg
}
```

Update the re-export line in `src/game_system/mod.rs`:

```rust
pub(crate) use systems::{create_entity_type_registry, create_enum_registry, create_game_system};
```

**Step 5: Run tests**

Run: `cargo test --lib game_system` Expected: PASS

**Step 6: Run full test suite**

Run: `cargo test` Expected: PASS

**Step 7: Commit**

```bash
git add src/game_system/mod.rs src/game_system/systems.rs src/game_system/tests.rs
git commit -m "feat(game_system): insert EnumRegistry and StructRegistry at startup"
```

---

## Task 5: Migrate enum_definitions out of EntityTypeRegistry

Remove `enum_definitions` field and `get_enum()` method from `EntityTypeRegistry`. Update all
consumers to use `EnumRegistry` instead.

**Files:**

- Modify: `src/contracts/game_system.rs` — remove field and method
- Modify: `src/game_system/systems.rs` — remove `enum_definitions: Vec::new()` from factory
- Modify: `src/game_system/tests.rs` — rewrite `registry_get_enum_by_id` to use EnumRegistry
- Modify: `src/editor_ui/systems.rs` — switch to `Res<EnumRegistry>`
- Modify: `src/editor_ui/components.rs` — update EditorAction::AddProperty
- Modify: `src/contracts/persistence.rs` — update tests
- Modify: `src/cell/tests.rs` — remove `enum_definitions` from test registries
- Modify: `src/unit/tests.rs` — remove `enum_definitions` from test registries
- Modify: `src/ontology/tests.rs` — remove `enum_definitions` from test registries
- Modify: `src/scripting/tests.rs` — remove `enum_definitions` from test registries
- Modify: `src/persistence/tests.rs` — remove `enum_definitions` from test registries
- Modify: `src/editor_ui/ui_tests.rs` — remove `enum_definitions` from test registries

**Step 1: Remove `enum_definitions` and `get_enum()` from EntityTypeRegistry**

In `src/contracts/game_system.rs`:

- Remove the `pub enum_definitions: Vec<EnumDefinition>` field from `EntityTypeRegistry`
- Remove the `get_enum()` method from `impl EntityTypeRegistry`
- Update the RON round-trip test to not include `enum_definitions`

**Step 2: Fix every compile error**

The compiler will find every site. These are all in test helper code constructing
`EntityTypeRegistry` directly — remove the `enum_definitions: ...` field from each. Known sites:

- `src/game_system/systems.rs:92` — `enum_definitions: Vec::new()` — remove
- `src/game_system/tests.rs:267` — rewrite test to use `EnumRegistry` directly
- `src/cell/tests.rs:54` — remove field
- `src/unit/tests.rs:49` — remove field
- `src/ontology/tests.rs:96` — remove field
- `src/scripting/tests.rs:46` — remove field
- `src/persistence/tests.rs:46` (in `test_file()`) — remove field
- `src/contracts/persistence.rs:200` — remove field
- `src/contracts/game_system.rs:243` — remove field from RON test
- `src/editor_ui/ui_tests.rs:61` — remove field

**Step 3: Update editor_ui to use EnumRegistry**

In `src/editor_ui/systems.rs`:

- Add `enum_registry: Res<EnumRegistry>` parameter to `editor_panel_system`
- Replace `registry.enum_definitions.clone()` with enum_registry lookups
- Pass `&enum_registry` to `render_inspector`, `render_unit_inspector`, and
  `render_property_value_editor`
- In `EditorAction::AddProperty` handler (in `apply_actions`): push new enum to a
  `ResMut<EnumRegistry>` instead of `registry.enum_definitions`

This is the most involved change. The `editor_panel_system` already has many parameters — add
`enum_registry: Res<EnumRegistry>` (or `ResMut<EnumRegistry>` since AddProperty creates enums).

Update function signatures that receive enum_defs:

```rust
// Before:
let enum_defs: Vec<EnumDefinition> = registry.enum_definitions.clone();
// After:
let enum_defs: Vec<EnumDefinition> = enum_registry.definitions.values().cloned().collect();
```

And in `apply_actions`, the AddProperty handler:

```rust
// Before:
registry.enum_definitions.push(EnumDefinition { ... });
// After:
enum_registry.insert(EnumDefinition { ... });
```

**Step 4: Run tests**

Run: `cargo test` Expected: PASS

**Step 5: Run clippy**

Run: `cargo clippy --all-targets` Expected: PASS

**Step 6: Commit**

```bash
git add -A  # Many files touched
git commit -m "refactor(contracts): extract enum_definitions into standalone EnumRegistry"
```

---

## Task 6: Update persistence for v2 format

**Files:**

- Modify: `src/contracts/persistence.rs` — bump version, add fields, migration logic
- Modify: `src/persistence/tests.rs` — update tests

**Step 1: Write failing test for v2 format**

Add to `src/contracts/persistence.rs` test module:

```rust
#[test]
fn v2_save_and_load_includes_registries() {
    use crate::contracts::game_system::{EnumDefinition, EnumRegistry, StructRegistry, TypeId};

    let dir = std::env::temp_dir().join("hexorder_test_v2.hexorder");
    let mut data = test_file();

    // Add enum and struct registries
    let mut enums = EnumRegistry::default();
    let eid = TypeId::new();
    enums.insert(EnumDefinition {
        id: eid,
        name: "Side".to_string(),
        options: vec!["Axis".to_string(), "Allied".to_string()],
    });
    data.enums = enums;
    data.structs = StructRegistry::default();

    save_to_file(&dir, &data).expect("save");
    let loaded = load_from_file(&dir).expect("load");

    assert_eq!(loaded.format_version, FORMAT_VERSION);
    assert_eq!(loaded.enums.definitions.len(), 1);
    assert_eq!(
        loaded.enums.get(eid).expect("should find").name,
        "Side"
    );

    let _ = std::fs::remove_file(&dir);
}
```

**Step 2: Run — expect compile error**

Expected: FAIL — `enums` field not on `GameSystemFile`

**Step 3: Update GameSystemFile and FORMAT_VERSION**

In `src/contracts/persistence.rs`:

```rust
pub const FORMAT_VERSION: u32 = 2;
```

Add to `GameSystemFile` struct:

```rust
/// Enum definitions registry (0.7.0).
pub enums: EnumRegistry,
/// Struct definitions registry (0.7.0).
pub structs: StructRegistry,
```

Add the necessary imports at the top of the file:

```rust
use super::game_system::{EnumRegistry, EntityTypeRegistry, GameSystem, PropertyValue, StructRegistry, TypeId};
```

Update the `test_file()` helper to include the new fields:

```rust
enums: EnumRegistry::default(),
structs: StructRegistry::default(),
```

**Step 4: Update persistence plugin's save/load to handle the new fields**

Check `src/persistence/systems.rs` (or wherever the actual save/load observer systems are) — the
`GameSystemFile` construction during save needs to include `enums` and `structs` from world
resources. The load needs to insert them back.

**Step 5: Run tests**

Run: `cargo test` Expected: PASS

**Step 6: Update the existing version check test**

The `load_unsupported_version_returns_error` test checks for `max: 1` — update to `max: 2`.

**Step 7: Commit**

```bash
git add src/contracts/persistence.rs src/persistence/
git commit -m "feat(contracts): bump persistence to v2 with EnumRegistry and StructRegistry"
```

---

## Task 7: Add Enums editor tab

**Files:**

- Modify: `src/editor_ui/components.rs` — add `OntologyTab::Enums`, new EditorAction variants, new
  EditorState fields
- Modify: `src/editor_ui/systems.rs` — add `render_enums_tab`, wire into tab bar
- Modify: `src/editor_ui/ui_tests.rs` — add UI tests for Enums tab
- Modify: `src/editor_ui/tests.rs` — update tab variant tests

**Step 1: Write failing UI test**

Add to `src/editor_ui/ui_tests.rs`:

```rust
#[test]
fn enums_tab_shows_heading() {
    // Similar pattern to existing tab tests — render the Enums tab and check for "Enums" heading.
    // (Follow the pattern from `concepts_tab_shows_heading` test)
}

#[test]
fn enums_tab_shows_existing_enum_name() {
    // Insert an EnumRegistry with one definition, render Enums tab, check name appears.
}
```

**Step 2: Add OntologyTab::Enums**

In `src/editor_ui/components.rs`, add `Enums` variant to `OntologyTab`:

```rust
pub enum OntologyTab {
    Types,
    Enums,     // NEW
    Structs,   // NEW (for Task 8)
    Concepts,
    Relations,
    Constraints,
    Validation,
}
```

**Step 3: Add EditorState fields for enum editing**

```rust
// Enum editor
pub new_enum_name: String,
pub new_enum_option_text: String,
```

Initialize in `Default` impl.

**Step 4: Add EditorAction variants**

```rust
CreateEnum { name: String, options: Vec<String> },
DeleteEnum { id: TypeId },
AddEnumOption { enum_id: TypeId, option: String },
RemoveEnumOption { enum_id: TypeId, option: String },
RenameEnum { id: TypeId, new_name: String },
```

**Step 5: Implement render_enums_tab**

In `src/editor_ui/systems.rs`:

```rust
pub(crate) fn render_enums_tab(
    ui: &mut egui::Ui,
    enum_registry: &EnumRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(egui::RichText::new("Enums").strong());

    // Create new enum form
    ui.group(|ui| {
        ui.label(egui::RichText::new("New Enum").small());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_state.new_enum_name);
        });
        ui.horizontal(|ui| {
            ui.label("Options:");
            ui.text_edit_singleline(&mut editor_state.new_enum_option_text);
        });
        ui.label(
            egui::RichText::new("(comma-separated)")
                .small()
                .color(egui::Color32::GRAY),
        );
        let name_valid = !editor_state.new_enum_name.trim().is_empty();
        ui.add_enabled_ui(name_valid, |ui| {
            if ui.button("+ Create Enum").clicked() && name_valid {
                let options: Vec<String> = editor_state
                    .new_enum_option_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                actions.push(EditorAction::CreateEnum {
                    name: editor_state.new_enum_name.trim().to_string(),
                    options,
                });
                editor_state.new_enum_name.clear();
                editor_state.new_enum_option_text.clear();
            }
        });
    });

    ui.add_space(4.0);

    // List existing enums
    if enum_registry.definitions.is_empty() {
        ui.label(
            egui::RichText::new("No enums defined")
                .small()
                .color(egui::Color32::GRAY),
        );
        return;
    }

    let enum_snapshots: Vec<_> = enum_registry
        .definitions
        .values()
        .map(|e| (e.id, e.name.clone(), e.options.clone()))
        .collect();

    for (enum_id, name, options) in &enum_snapshots {
        let mut delete = false;

        egui::CollapsingHeader::new(name)
            .id_salt(format!("enum_{enum_id:?}"))
            .show(ui, |ui| {
                // Options list with remove buttons
                for opt in options {
                    ui.horizontal(|ui| {
                        ui.label(format!("  {opt}"));
                        if ui.small_button("x").clicked() {
                            actions.push(EditorAction::RemoveEnumOption {
                                enum_id: *enum_id,
                                option: opt.clone(),
                            });
                        }
                    });
                }

                // Add option inline
                // (Reuses new_enum_option_text — in a real implementation you'd want
                // per-enum state, but this is acceptable for now)

                // Delete button
                ui.add_space(4.0);
                if ui
                    .button(
                        egui::RichText::new("Delete Enum")
                            .color(egui::Color32::from_rgb(200, 80, 80)),
                    )
                    .clicked()
                {
                    delete = true;
                }
            });

        if delete {
            actions.push(EditorAction::DeleteEnum { id: *enum_id });
        }
    }
}
```

**Step 6: Wire into tab bar and editor_panel_system**

Update `render_tab_bar` to include the new tab. Update the match in `editor_panel_system` to call
`render_enums_tab`. Handle the new `EditorAction` variants in `apply_actions`.

**Step 7: Run tests**

Run: `cargo test` Expected: PASS

**Step 8: Commit**

```bash
git add src/editor_ui/
git commit -m "feat(editor_ui): add Enums editor tab with CRUD"
```

---

## Task 8: Add Structs editor tab

Follows the same pattern as Task 7 but for `StructRegistry`.

**Files:**

- Modify: `src/editor_ui/components.rs` — EditorAction variants, EditorState fields
- Modify: `src/editor_ui/systems.rs` — `render_structs_tab`, wire into tab bar

**Step 1: Add EditorAction variants**

```rust
CreateStruct { name: String },
DeleteStruct { id: TypeId },
AddStructField { struct_id: TypeId, name: String, prop_type: PropertyType },
RemoveStructField { struct_id: TypeId, field_id: TypeId },
```

**Step 2: Add EditorState fields**

```rust
pub new_struct_name: String,
pub new_struct_field_name: String,
pub new_struct_field_type_index: usize,
```

**Step 3: Implement render_structs_tab**

Similar pattern to `render_enums_tab`. List existing StructDefinitions with collapsible headers.
Each shows its fields. "Add Field" form with name + type selector (reuse the existing
`index_to_property_type` pattern). Delete button.

**Step 4: Wire into editor, handle actions**

**Step 5: Write UI tests, run full suite**

**Step 6: Commit**

```bash
git add src/editor_ui/
git commit -m "feat(editor_ui): add Structs editor tab with CRUD"
```

---

## Task 9: Update "Add Property" form for new types

Extend the type selector dropdown from 6 to 12 options and add conditional sub-selectors.

**Files:**

- Modify: `src/editor_ui/systems.rs` — `render_entity_type_section`, `index_to_property_type`
- Modify: `src/editor_ui/components.rs` — new EditorState fields for sub-selectors

**Step 1: Update index_to_property_type**

```rust
fn index_to_property_type(index: usize) -> PropertyType {
    match index {
        1 => PropertyType::Int,
        2 => PropertyType::Float,
        3 => PropertyType::String,
        4 => PropertyType::Color,
        5 => PropertyType::Enum(TypeId::new()),
        6 => PropertyType::EntityRef(None),
        7 => PropertyType::List(Box::new(PropertyType::Int)),     // inner type set by sub-selector
        8 => PropertyType::Map(TypeId::new(), Box::new(PropertyType::Int)), // set by sub-selectors
        9 => PropertyType::Struct(TypeId::new()),                 // set by sub-selector
        10 => PropertyType::IntRange { min: 0, max: 100 },       // set by sub-fields
        11 => PropertyType::FloatRange { min: 0.0, max: 1.0 },   // set by sub-fields
        _ => PropertyType::Bool,
    }
}
```

**Step 2: Update the type name array in render_entity_type_section**

```rust
let types = [
    "Bool", "Int", "Float", "String", "Color", "Enum",
    "EntityRef", "List", "Map", "Struct", "IntRange", "FloatRange",
];
```

**Step 3: Add conditional sub-selectors after the type dropdown**

For each compound type, show additional fields:

- **Enum (index 5)**: comma-separated options (existing behavior)
- **EntityRef (index 6)**: role filter dropdown (None / BoardPosition / Token)
- **List (index 7)**: inner type sub-selector (simple dropdown of base types)
- **Map (index 8)**: enum key picker + value type sub-selector
- **Struct (index 9)**: struct picker from StructRegistry
- **IntRange (index 10)**: min and max fields
- **FloatRange (index 11)**: min and max fields

Add EditorState fields for the sub-selectors:

```rust
pub new_prop_entity_ref_role: usize,       // 0=Any, 1=BoardPosition, 2=Token
pub new_prop_list_inner_type: usize,       // index into base types
pub new_prop_map_enum_id: Option<TypeId>,
pub new_prop_map_value_type: usize,
pub new_prop_struct_id: Option<TypeId>,
pub new_prop_int_range_min: i64,
pub new_prop_int_range_max: i64,
pub new_prop_float_range_min: f64,
pub new_prop_float_range_max: f64,
```

**Step 4: Update the AddProperty action handler to use sub-selector values**

Build the correct `PropertyType` from the sub-selector state before pushing the action.

**Step 5: Run tests, commit**

```bash
git add src/editor_ui/
git commit -m "feat(editor_ui): extend Add Property form for 12 types"
```

---

## Task 10: Recursive property value renderer

Replace placeholder editors from Task 3 with full recursive rendering.

**Files:**

- Modify: `src/editor_ui/systems.rs` — rewrite `render_property_value_editor`

**Step 1: Update function signature to include registries and depth**

```rust
fn render_property_value_editor(
    ui: &mut egui::Ui,
    value: &mut PropertyValue,
    prop_type: &PropertyType,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
    entity_registry: &EntityTypeRegistry,
    depth: usize,
)
```

**Step 2: Update all call sites**

The function is called from `render_inspector` and `render_unit_inspector`. Pass the registries
(available as system parameters) and `depth: 0`.

**Step 3: Implement compound type editors**

Replace placeholder arms with recursive implementations:

```rust
PropertyValue::IntRange(v) => {
    if let PropertyType::IntRange { min, max } = prop_type {
        ui.add(egui::DragValue::new(v).range(*min..=*max));
    }
}
PropertyValue::FloatRange(v) => {
    if let PropertyType::FloatRange { min, max } = prop_type {
        ui.add(egui::DragValue::new(v).range(*min..=*max).speed(0.1));
    }
}
PropertyValue::EntityRef(selected) => {
    let role_filter = if let PropertyType::EntityRef(filter) = prop_type {
        *filter
    } else {
        None
    };
    let candidates: Vec<_> = entity_registry
        .types
        .iter()
        .filter(|et| role_filter.is_none() || Some(et.role) == role_filter)
        .map(|et| (et.id, et.name.clone()))
        .collect();
    let selected_name = selected
        .and_then(|id| candidates.iter().find(|(eid, _)| *eid == id))
        .map_or("(none)", |(_, n)| n.as_str())
        .to_string();
    egui::ComboBox::from_id_salt(format!("eref_{depth}"))
        .selected_text(&selected_name)
        .show_ui(ui, |ui| {
            if ui.selectable_label(selected.is_none(), "(none)").clicked() {
                *selected = None;
            }
            for (eid, ename) in &candidates {
                if ui.selectable_label(*selected == Some(*eid), ename).clicked() {
                    *selected = Some(*eid);
                }
            }
        });
}
PropertyValue::List(items) => {
    if depth >= 3 {
        ui.label(egui::RichText::new("(nested limit)").small().color(egui::Color32::GRAY));
        return;
    }
    let inner_type = if let PropertyType::List(inner) = prop_type {
        inner.as_ref()
    } else {
        return;
    };
    egui::CollapsingHeader::new(format!("List ({})", items.len()))
        .id_salt(format!("list_{depth}"))
        .show(ui, |ui| {
            let mut remove_idx = None;
            for (idx, item) in items.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{idx}]"));
                    render_property_value_editor(
                        ui, item, inner_type,
                        enum_registry, struct_registry, entity_registry,
                        depth + 1,
                    );
                    if ui.small_button("x").clicked() {
                        remove_idx = Some(idx);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                items.remove(idx);
            }
            if ui.button("+ Add").clicked() {
                items.push(PropertyValue::default_for(inner_type));
            }
        });
}
PropertyValue::Map(entries) => {
    if depth >= 3 {
        ui.label(egui::RichText::new("(nested limit)").small().color(egui::Color32::GRAY));
        return;
    }
    let (enum_id, value_type) = if let PropertyType::Map(eid, vt) = prop_type {
        (*eid, vt.as_ref())
    } else {
        return;
    };
    let enum_options = enum_registry
        .get(enum_id)
        .map(|ed| ed.options.clone())
        .unwrap_or_default();
    egui::CollapsingHeader::new(format!("Map ({})", entries.len()))
        .id_salt(format!("map_{depth}"))
        .show(ui, |ui| {
            for opt in &enum_options {
                let entry = entries
                    .iter_mut()
                    .find(|(k, _)| k == opt);
                if let Some((_, val)) = entry {
                    ui.horizontal(|ui| {
                        ui.label(format!("{opt}:"));
                        render_property_value_editor(
                            ui, val, value_type,
                            enum_registry, struct_registry, entity_registry,
                            depth + 1,
                        );
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(format!("{opt}:"));
                        ui.label(
                            egui::RichText::new("(default)")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        if ui.small_button("+").clicked() {
                            entries.push((opt.clone(), PropertyValue::default_for(value_type)));
                        }
                    });
                }
            }
        });
}
PropertyValue::Struct(fields) => {
    if depth >= 3 {
        ui.label(egui::RichText::new("(nested limit)").small().color(egui::Color32::GRAY));
        return;
    }
    let struct_id = if let PropertyType::Struct(sid) = prop_type {
        *sid
    } else {
        return;
    };
    let struct_def = struct_registry.get(struct_id);
    egui::CollapsingHeader::new(
        struct_def.map_or("Struct", |sd| sd.name.as_str()).to_string(),
    )
    .id_salt(format!("struct_{depth}"))
    .show(ui, |ui| {
        if let Some(sd) = struct_def {
            for field in &sd.fields {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", field.name));
                    let val = fields
                        .entry(field.id)
                        .or_insert_with(|| PropertyValue::default_for(&field.property_type));
                    render_property_value_editor(
                        ui, val, &field.property_type,
                        enum_registry, struct_registry, entity_registry,
                        depth + 1,
                    );
                });
            }
        }
    });
}
```

**Step 4: Run tests**

Run: `cargo test` Expected: PASS

**Step 5: Run clippy**

Run: `cargo clippy --all-targets` Expected: PASS

**Step 6: Commit**

```bash
git add src/editor_ui/systems.rs
git commit -m "feat(editor_ui): recursive property value renderer with depth cap"
```

---

## Task 11: Update persistence save/load for new registries

Ensure the persistence plugin reads/writes `EnumRegistry` and `StructRegistry` from the world.

**Files:**

- Modify: `src/persistence/systems.rs` (or wherever save/load observers live)
- Modify: `src/persistence/tests.rs`

**Step 1: Write integration test**

Add a test that saves a game system with compound properties, reloads it, and verifies the
registries and property values survived the round-trip.

**Step 2: Update save observer**

When building `GameSystemFile`, read `Res<EnumRegistry>` and `Res<StructRegistry>` from the world
and include them.

**Step 3: Update load observer**

When applying a loaded `GameSystemFile`, insert `EnumRegistry` and `StructRegistry` as resources.

**Step 4: Run full test suite**

Run: `cargo test` Expected: PASS

**Step 5: Commit**

```bash
git add src/persistence/
git commit -m "feat(persistence): save and load EnumRegistry and StructRegistry"
```

---

## Task 12: Final integration testing and cleanup

**Files:**

- All files touched in previous tasks

**Step 1: Run full audit**

Run: `mise check:audit` Expected: PASS

**Step 2: Run boundary check**

Run: `mise check:boundary` Expected: PASS — new types go through `src/contracts/`

**Step 3: Run unwrap check**

Run: `mise check:unwrap` Expected: PASS — no unwrap() in production code

**Step 4: Update contract spec**

Update `docs/contracts/game-system.md` to document the new types: EnumRegistry, StructRegistry,
StructDefinition, and the 6 new PropertyType/PropertyValue variants.

**Step 5: Update plugin spec**

Update `docs/plugins/game-system/spec.md` with new requirements and success criteria for 0.7.0.

**Step 6: Update plugin log**

Update `docs/plugins/game-system/log.md` with test results and final decisions.

**Step 7: Post progress comment on pitch issue**

```bash
gh issue comment 81 --body "Property system foundation implementation complete. EnumRegistry and StructRegistry extracted as standalone resources. 6 new compound PropertyType/PropertyValue variants (EntityRef, List, Map, Struct, IntRange, FloatRange). Recursive property editor with 3-level depth cap. Persistence migrated to v2. All tests pass."
```

**Step 8: Final commit**

```bash
git add docs/
git commit -m "docs(game_system): update contract spec, plugin spec, and log for 0.7.0"
```
