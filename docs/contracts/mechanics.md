# Contract: mechanics

## Purpose

Defines shared types for turn structure, combat resolution (CRT), combat modifiers, and combat
execution state. These are the core wargame mechanic primitives that enable designers to define how
turns are structured and how combat is resolved.

## Consumers

- `game_system` — inserts default resources at startup
- `rules_engine` — combat resolution logic, modifier evaluation
- `unit` — combat selection in Play mode
- `editor_ui` — turn structure editor, CRT editor, combat execution panel
- `persistence` — save/load turn structure, CRT, and modifiers

## Producers

- `game_system` — inserts `TurnStructure`, `TurnState`, `CombatResultsTable`,
  `CombatModifierRegistry`, `ActiveCombat` resources at startup

## Types

### Turn Structure

```rust
/// How players alternate within a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerOrder {
    /// One player completes all phases, then the next (classic IGOUGO).
    Alternating,
    /// Both players act simultaneously in each phase.
    Simultaneous,
    /// Players alternate activating individual units or groups.
    ActivationBased,
}

/// The category of actions allowed during a phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseType {
    /// Units may move (movement budget consumed).
    Movement,
    /// Combat may be declared and resolved.
    Combat,
    /// Administrative actions (reinforcements, supply, victory checks).
    Admin,
}

/// A single named phase within a turn sequence.
#[derive(Debug, Clone)]
pub struct Phase {
    pub id: TypeId,
    pub name: String,
    pub phase_type: PhaseType,
    pub description: String,
}

/// The designer-defined turn structure for the game system.
#[derive(Resource, Debug, Clone, Default)]
pub struct TurnStructure {
    pub phases: Vec<Phase>,
    pub player_order: PlayerOrder,
}

/// Runtime state tracking the current position within a turn (Play mode only).
#[derive(Resource, Debug, Default)]
pub struct TurnState {
    pub turn_number: u32,
    pub current_phase_index: usize,
    pub is_active: bool,
}
```

### Combat Results Table

```rust
/// How a CRT column is calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrtColumnType {
    /// Attacker:defender strength ratios (e.g., 1:2, 1:1, 2:1, 3:1).
    OddsRatio,
    /// Attacker - defender strength differentials (e.g., -3, -2, ..., +3).
    Differential,
}

/// A single column header in the CRT. Each column carries its own column type,
/// allowing ratio and differential columns to coexist in a single CRT.
#[derive(Debug, Clone)]
pub struct CrtColumn {
    pub label: String,
    pub column_type: CrtColumnType,
    /// For OddsRatio: minimum ratio (e.g., 3.0 for "3:1").
    /// For Differential: minimum differential (e.g., 2.0 for "+2").
    pub threshold: f64,
}

/// A single row header in the CRT (die roll result). Fully designer-defined.
#[derive(Debug, Clone)]
pub struct CrtRow {
    pub label: String,
    pub die_value_min: u32,
    pub die_value_max: u32,
}

/// A structured effect that can be partially automated.
#[derive(Debug, Clone)]
pub enum OutcomeEffect {
    NoEffect,
    Retreat { hexes: u32 },
    StepLoss { steps: u32 },
    AttackerStepLoss { steps: u32 },
    Exchange { attacker_steps: u32, defender_steps: u32 },
    AttackerEliminated,
    DefenderEliminated,
}

/// A combat outcome: designer label + optional structured effect.
#[derive(Debug, Clone)]
pub struct CombatOutcome {
    pub label: String,
    pub effect: Option<OutcomeEffect>,
}

/// The Combat Results Table: a 2D grid of combat outcomes.
#[derive(Resource, Debug, Clone, Default)]
pub struct CombatResultsTable {
    pub id: TypeId,
    pub name: String,
    pub columns: Vec<CrtColumn>,
    pub rows: Vec<CrtRow>,
    /// Indexed as [row_index][column_index].
    pub outcomes: Vec<Vec<CombatOutcome>>,
    /// Reference to the Combat concept in the ontology.
    pub combat_concept_id: Option<TypeId>,
}
```

### Combat Modifiers

```rust
/// The source of a combat modifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierSource {
    DefenderTerrain,
    AttackerTerrain,
    AttackerProperty(String),
    DefenderProperty(String),
    Custom(String),
}

/// A combat modifier: a signed column shift with priority-based evaluation.
#[derive(Debug, Clone)]
pub struct CombatModifierDefinition {
    pub id: TypeId,
    pub name: String,
    pub source: ModifierSource,
    pub column_shift: i32,
    pub priority: i32,
    pub cap: Option<i32>,
    pub terrain_type_filter: Option<TypeId>,
}

/// Registry of all combat modifier definitions.
#[derive(Resource, Debug, Clone, Default)]
pub struct CombatModifierRegistry {
    pub modifiers: Vec<CombatModifierDefinition>,
}
```

### Combat Execution (runtime, Play mode only)

```rust
/// Tracks the in-progress combat being resolved.
#[derive(Resource, Debug, Default)]
pub struct ActiveCombat {
    pub attacker: Option<Entity>,
    pub defender: Option<Entity>,
    pub raw_value: Option<f64>,
    pub total_shift: i32,
    pub applied_modifiers: Vec<(String, i32)>,
    pub resolved_column: Option<usize>,
    pub die_roll: Option<u32>,
    pub resolved_row: Option<usize>,
    pub outcome: Option<CombatOutcome>,
}
```

### Events

```rust
/// Fired when the turn advances to the next phase.
#[derive(Event, Debug)]
pub struct PhaseAdvancedEvent {
    pub turn_number: u32,
    pub phase_index: usize,
    pub phase_name: String,
    pub phase_type: PhaseType,
}

/// Fired when a combat is fully resolved.
#[derive(Event, Debug)]
pub struct CombatResolvedEvent {
    pub attacker: Entity,
    pub defender: Entity,
    pub outcome: CombatOutcome,
    pub die_roll: u32,
    pub column_label: String,
}
```

## Invariants

- `TurnStructure` is inserted at startup; may be empty or contain a default phase sequence
- `TurnState` is runtime-only (not persisted); reset when entering Play mode
- `CombatResultsTable.outcomes` dimensions must match `[rows.len()][columns.len()]`
- `CrtColumn` entries should be ordered by ascending threshold
- `CrtRow` entries should have non-overlapping `die_value_min..=die_value_max` ranges
- `ActiveCombat` is runtime-only (not persisted); cleared when exiting Play mode
- `CombatModifierRegistry` modifiers are evaluated in priority order (highest first)
- Column shifts are clamped to `[0, columns.len() - 1]` after all modifiers applied

## Changelog

| Date       | Change             | Reason                               |
| ---------- | ------------------ | ------------------------------------ |
| 2026-02-16 | Initial definition | 0.9.0 Core mechanic primitives (#77) |
