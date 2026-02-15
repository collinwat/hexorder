# Design: Property System Foundation

**Pitch**: #81 — Property system foundation: type extensions, reflection forms, enum consolidation
**Branch**: `0.7.0/property-system` **Date**: 2026-02-15 **Appetite**: Big Batch (full cycle)

## Problem

The property type system supports 6 basic types (Bool, Int, Float, String, Color, Enum). Wargames
require compound types: EntityRef (unit -> weapon type), List (special abilities), Map<Enum, Int>
(terrain movement costs, CRTs), and Struct (combat profiles). Property editing is hand-coded per
type. Enum definitions are coupled to EntityTypeRegistry rather than being a first-class resource.

## Decision: Registry-Based Type System with Recursive Rendering

### Approach Rejected: Flat Extensions

Add new variants to `PropertyType`/`PropertyValue` with minimal new types. Keep enums in
EntityTypeRegistry. Extend the existing editor match arms.

- Pros: fast to build, low ceremony
- Cons: no reuse for enum/struct definitions across entity types, editor code becomes unwieldy, enum
  definitions stay coupled to entity registry

### Approach Chosen: Registry-Based Type System (Approach B)

Introduce `EnumRegistry` and `StructRegistry` as standalone Bevy resources. PropertyType and
PropertyValue gain compound variants referencing these registries by TypeId. The property editor
becomes a recursive match-based renderer. New editor tabs manage enums and structs independently.

- Pros: reusable definitions, clean separation of concerns, recursive renderer handles arbitrary
  nesting, registries can be consumed by validation/rules without going through EntityTypeRegistry
- Cons: more types to manage, persistence migration needed
- Rationale: aligns with the project's existing pattern of typed registries (EntityTypeRegistry,
  ConceptRegistry, etc.) and gives the type system room to grow without rearchitecting

## Contract Changes

### New Resources

#### EnumRegistry

```rust
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct EnumRegistry {
    pub definitions: HashMap<TypeId, EnumDefinition>,
}
```

Standalone resource replacing `EntityTypeRegistry.enum_definitions`. `EnumDefinition` struct is
unchanged (id, name, options).

#### StructRegistry

```rust
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct StructRegistry {
    pub definitions: HashMap<TypeId, StructDefinition>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct StructDefinition {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<PropertyDefinition>,
}
```

Centrally defined, reusable struct schemas. A "CombatProfile" struct is defined once and referenced
by multiple entity types.

### Extended PropertyType (6 new variants)

```rust
pub enum PropertyType {
    // Existing (unchanged)
    Bool,
    Int,
    Float,
    String,
    Color,
    Enum(TypeId),

    // New
    EntityRef(Option<EntityRole>),     // reference to entity type, optional role filter
    List(Box<PropertyType>),           // ordered collection of inner type
    Map(TypeId, Box<PropertyType>),    // enum TypeId keys, inner type values
    Struct(TypeId),                    // references StructDefinition
    IntRange { min: i64, max: i64 },   // bounded integer
    FloatRange { min: f64, max: f64 }, // bounded float
}
```

### Extended PropertyValue (6 matching variants)

```rust
pub enum PropertyValue {
    // Existing (unchanged)
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(bevy::color::Color),
    Enum(String),

    // New
    EntityRef(Option<TypeId>),                 // selected entity type id or None
    List(Vec<PropertyValue>),                  // ordered values
    Map(Vec<(String, PropertyValue)>),         // enum key name -> value pairs
    Struct(HashMap<TypeId, PropertyValue>),     // field id -> value
    IntRange(i64),                             // value within [min, max]
    FloatRange(f64),                           // value within [min, max]
}
```

### EntityTypeRegistry Changes

- `enum_definitions` field removed
- `get_enum()` method removed (callers use `EnumRegistry` directly)
- All other fields and methods unchanged

### Consumers Update

Plugins that currently read `EntityTypeRegistry.enum_definitions` or call `get_enum()`:

- **editor_ui**: switch to `Res<EnumRegistry>` for enum dropdowns, Map key sets
- **persistence**: serialize/deserialize `EnumRegistry` and `StructRegistry` as top-level fields
- **validation**: read `EnumRegistry` for enum option validation

## Data Flow

### Initialization Order (in GameSystemPlugin::build)

1. `EnumRegistry` inserted with starter enums (e.g., "Terrain Type", "Movement Mode")
2. `StructRegistry` inserted (empty or with optional starter like "CombatProfile")
3. `EntityTypeRegistry` inserted (references enum TypeIds from step 1, no longer holds definitions)
4. Active type resources set as before

### Property Resolution Chain

- `PropertyType::Enum(id)` -> look up in `EnumRegistry`
- `PropertyType::Struct(id)` -> look up in `StructRegistry`
- `PropertyType::Map(enum_id, inner)` -> key set from `EnumRegistry`, value type recurses
- `PropertyType::List(inner)` -> inner type recurses
- `PropertyType::EntityRef(role_filter)` -> candidates from `EntityTypeRegistry`, filtered by role

### Nesting Depth Cap

Maximum 3 levels of nesting in the editor (per pitch rabbit hole). A Struct containing a List of
Structs renders 3 levels deep. Beyond that, a "(nested limit)" placeholder appears. This is a UI cap
only; the data model has no depth restriction.

## Persistence Migration

- Bump `FORMAT_VERSION` from 1 to 2
- `GameSystemFile` gains `enums: EnumRegistry` and `structs: StructRegistry` fields
- `EntityTypeRegistry` serialized form loses `enum_definitions`
- **v1 -> v2 migration**: when loading a v1 file, migrate `entity_types.enum_definitions` into the
  new `EnumRegistry` field, then clear the old field. One-way upgrade.
- All new PropertyValue variants derive `Serialize`/`Deserialize` — RON handles enum-with-data
  natively

## Editor UI Changes

### New Tabs

Add to `OntologyTab` enum:

- **Enums** tab: list all EnumDefinitions, create/edit/delete. Each definition shows name and
  options list with add/remove option buttons.
- **Structs** tab: list all StructDefinitions, create/edit/delete. Each definition shows name and
  fields list. Fields use the same "Add Property" form pattern (name + type selector).

### "Add Property" Form Update

Type selector gains new entries: EntityRef, List, Map, Struct, IntRange, FloatRange.

Conditional sub-selectors:

- **Map**: enum key picker (from EnumRegistry) + value type sub-selector
- **List**: inner type sub-selector
- **Struct**: struct picker (from StructRegistry)
- **IntRange/FloatRange**: min/max number fields
- **EntityRef**: optional role filter dropdown (None / BoardPosition / Token)

### Recursive Property Renderer

Extend `render_property_value_editor` with a depth parameter:

```
fn render_property_value_editor(
    ui, value, prop_type, enum_registry, struct_registry, entity_registry, depth
)
```

New match arms:

- `IntRange(i64)` -> DragValue clamped to min..=max from PropertyType
- `FloatRange(f64)` -> DragValue clamped, speed 0.1
- `EntityRef(Option<TypeId>)` -> ComboBox from EntityTypeRegistry (filtered by role)
- `List(Vec<PropertyValue>)` -> CollapsingHeader, indexed rows, each recurses, + Add / Remove
- `Map(Vec<(String, PropertyValue)>)` -> CollapsingHeader, one row per enum key, value recurses
- `Struct(HashMap<TypeId, PropertyValue>)` -> CollapsingHeader, one row per field, each recurses
- If `depth >= 3`, render "(nested limit)" label instead of recursing

## Testing Strategy

### Contract Layer (unit tests)

- `PropertyValue::default_for` returns correct variant for all 12 types
- RON round-trip for each new PropertyType/PropertyValue variant
- EnumRegistry CRUD: insert, get, remove, list
- StructRegistry CRUD: insert, get, remove, list
- Nested type serialization: Map containing Struct containing List
- Nesting depth: data model accepts arbitrary depth (no restriction)

### Editor UI Layer (unit tests)

- Property type index mapping covers all 12 types
- Format helper functions handle new type display names

### Persistence Layer (integration tests)

- v1 -> v2 migration: load a v1 fixture file, verify enums migrated to EnumRegistry
- v2 round-trip: save and reload a file with compound properties

### Architecture Tests

- Boundary check passes: new registries exposed through contracts only
- No unwrap() in production code

## Dependency Chain

1. **Week 1**: EnumRegistry extraction + StructRegistry + contract changes
2. **Weeks 2-3**: PropertyType/PropertyValue extensions with serialization and validation
3. **Weeks 3-5**: Recursive form renderer + Enums/Structs editor tabs
4. **Weeks 5-6**: Persistence migration + integration testing + polish

## No-Gos (from pitch)

- No Formula/Calculated property type
- No AssetPath property type
- No property inheritance (entity A extends entity B)
- No undo/redo integration
- No drag-and-drop reordering in List editors

## Related Issues

- #14 — PropertyType extensions: EntityRef, List, Map, Struct
- #23 — Reflection-driven form generation
- #24 — Enum registry consolidation
- #28 — Research: bevy-inspector-egui patterns
