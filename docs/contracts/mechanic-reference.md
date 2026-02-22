# Contract: mechanic_reference

## Purpose

Defines the read-only mechanic reference catalog types. The catalog is populated at startup by the
`mechanic_reference` plugin and consumed by `editor_ui` for display and scaffolding. Scaffolding
templates use string-based action types to avoid coupling to `game_system` internals.

## Types

### Category taxonomy

```rust
/// The six areas of the Engelstein taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanicCategory {
    CoreUniversal,
    AdvancedCommon,
    BespokeUnusual,
    GameSystemArchitecture,
    DigitalImplementation,
    GenreEvolution,
}
```

### Template availability

```rust
/// Whether a mechanic entry has a scaffolding template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateAvailability {
    None,
    Available { template_id: String, preview: String },
}
```

### Scaffolding actions

```rust
/// A single scaffolding instruction produced by a mechanic template.
/// Uses string-based types to avoid cross-contract dependencies.
#[derive(Debug, Clone)]
pub enum ScaffoldAction {
    CreateEntityType { name: String, role: String, color: [f32; 3] },
    AddProperty { entity_name: String, prop_name: String, prop_type: String },
    CreateEnum { name: String, options: Vec<String> },
    AddCrtColumn { label: String, column_type: String, threshold: f64 },
    AddCrtRow { label: String, die_min: u32, die_max: u32 },
    SetCrtOutcome { row: usize, col: usize, label: String },
    AddPhase { name: String, phase_type: String },
    AddCombatModifier { name: String, source: String, shift: i32, priority: i32 },
}

/// A complete scaffolding recipe.
#[derive(Debug, Clone)]
pub struct ScaffoldRecipe {
    pub template_id: String,
    pub description: String,
    pub actions: Vec<ScaffoldAction>,
}
```

### Catalog entry and resource

```rust
/// A single entry in the mechanic reference catalog.
#[derive(Debug, Clone)]
pub struct MechanicEntry {
    pub name: String,
    pub category: MechanicCategory,
    pub description: String,
    pub example_games: Vec<String>,
    pub design_considerations: String,
    pub template: TemplateAvailability,
}

/// Resource holding the full mechanic reference catalog.
#[derive(Resource, Debug, Default)]
pub struct MechanicCatalog {
    pub entries: Vec<MechanicEntry>,
    pub templates: Vec<ScaffoldRecipe>,
}
```

## Consumers

- editor_ui (reads `MechanicCatalog` for browsable panel, applies `ScaffoldRecipe` actions)

## Producers

- mechanic_reference (populates `MechanicCatalog` at startup via `insert_resource`)

## Invariants

- `MechanicCatalog` is populated once at startup and is read-only thereafter
- Every `TemplateAvailability::Available::template_id` has a matching `ScaffoldRecipe` in
  `MechanicCatalog.templates`
- `ScaffoldAction` strings are resolved to typed values at application time by `editor_ui`, not by
  this contract
- Scaffolded elements are standard registry objects with no link back to the template

## Changelog

| Date       | Change             | Reason                                |
| ---------- | ------------------ | ------------------------------------- |
| 2026-02-22 | Initial definition | Pitch #100 mechanic reference library |
