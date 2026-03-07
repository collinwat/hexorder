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
  `CombatModifierRegistry`, `ActiveCombat`, `AreaMarkerRegistry` resources at startup

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
    pub phase_actions_remaining: Option<u32>,
}

/// An action the user can take to control phase progression.
pub enum PhaseAction {
    Advance,
    Rewind,
    Skip,
}

/// The result of a phase transition attempt.
pub struct PhaseTransitionResult {
    pub turn_changed: bool,
    pub from_phase: usize,
    pub to_phase: usize,
    pub turn_number: u32,
}
```

### Combat Results Table

The CRT delegates its column/row structure to `ResolutionTable` from the `simulation` contract (see
`docs/contracts/simulation.md`). Column types (`ColumnType`), column headers (`TableColumn`), and
row headers (`TableRow`) are defined there. The CRT adds domain-specific outcome semantics.

```rust
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

/// The Combat Results Table: wraps a generic ResolutionTable with combat outcomes.
#[derive(Resource, Debug, Clone, Default)]
pub struct CombatResultsTable {
    pub id: TypeId,
    pub name: String,
    /// Generic 2D table structure (columns + rows).
    pub table: ResolutionTable,
    /// Domain-specific outcomes indexed as [row_index][column_index].
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

### CRT Resolution Functions

Pure functions on contract types. Column/row lookup and modifier evaluation are delegated to the
generic functions in the `simulation` contract (`find_table_column`, `find_table_row`,
`evaluate_column_modifiers`, `apply_column_shift`). The CRT layer adds outcome retrieval.

```rust
/// Result of a full CRT resolution.
pub struct CrtResolution {
    pub column_index: usize,
    pub row_index: usize,
    pub column_label: String,
    pub row_label: String,
    pub outcome: CombatOutcome,
}

/// Full CRT resolution: delegates column/row lookup to simulation primitives,
/// then retrieves the domain-specific combat outcome.
pub fn resolve_crt(crt: &CombatResultsTable, atk: f64, def: f64, die_roll: u32)
    -> Option<CrtResolution>;
```

### Post-Resolution Movement

```rust
/// What happens to combatants after resolution completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostResolutionAction {
    /// Attacker advances into the defender's hex.
    Advance,
    /// Defender retreats away from the attacker.
    Retreat,
    /// No movement — units stay in place.
    Hold,
}

/// A rule that triggers post-resolution movement based on the combat outcome.
#[derive(Debug, Clone)]
pub struct PostResolutionRule {
    pub action: PostResolutionAction,
    /// Which OutcomeEffect variants trigger this rule. Empty means "always".
    pub trigger_effects: Vec<String>,
    /// Maximum movement range in hexes (for Retreat).
    pub movement_range: u32,
}

/// The pending movement resulting from post-resolution rule evaluation.
#[derive(Debug, Clone)]
pub struct PendingMovement {
    pub entity: Entity,
    pub action: PostResolutionAction,
    pub movement_range: u32,
}

/// Evaluate post-resolution rules against a combat outcome.
/// Returns pending movements for the attacker, defender, or both.
pub fn evaluate_post_resolution(
    rules: &[PostResolutionRule],
    outcome: &CombatOutcome,
    attacker: Entity,
    defender: Entity,
) -> Vec<PendingMovement>;
```

### Area-Effect Modifiers

```rust
/// How long an area marker remains active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerDuration {
    Permanent,
    PerTurn { turns_remaining: u32 },
    UntilRemoved,
}

/// An effect applied to hexes within a marker's radius.
#[derive(Debug, Clone)]
pub enum AreaEffect {
    ColumnShift { shift: i32 },
    CostModifier { extra_cost: i64 },
    ActionRestriction { restriction: String },
}

/// A spatial marker that applies effects to hexes within a radius.
#[derive(Debug, Clone)]
pub struct AreaMarker {
    pub marker_type: String,
    pub center: HexPosition,
    pub radius: u32,
    pub effects: Vec<AreaEffect>,
    pub duration: MarkerDuration,
}

/// Registry of active area markers on the board.
#[derive(Resource, Debug, Clone, Default)]
pub struct AreaMarkerRegistry {
    pub markers: Vec<AreaMarker>,
}

/// Collect all column shifts from area markers affecting a position.
pub fn collect_area_column_shifts(
    registry: &AreaMarkerRegistry, position: HexPosition,
) -> i32;

/// Collect extra movement cost from area markers for a position.
pub fn collect_area_cost_modifiers(
    registry: &AreaMarkerRegistry, position: HexPosition,
) -> i64;

/// Check if a specific action restriction applies at a position.
pub fn is_action_restricted(
    registry: &AreaMarkerRegistry, position: HexPosition, restriction_name: &str,
) -> bool;
```

### Phase Sequencer Functions

```rust
/// Check whether a phase action is legal given the current turn state.
pub fn is_phase_action_legal(
    action: PhaseAction, turn_state: &TurnState, turn_structure: &TurnStructure,
) -> bool;

/// Execute a phase action, updating turn state. Returns None if illegal.
pub fn execute_phase_action(
    action: PhaseAction, turn_state: &mut TurnState, turn_structure: &TurnStructure,
) -> Option<PhaseTransitionResult>;

/// Get the current phase from the turn structure, if valid.
pub fn current_phase(turn_state: &TurnState, turn_structure: &TurnStructure) -> Option<&Phase>;
```

**Removed functions** (now in `simulation` contract as generic equivalents):

- `calculate_odds_ratio` / `calculate_differential` — inlined at call sites or use column thresholds
- `find_crt_column` → `simulation::find_table_column`
- `find_crt_row` → `simulation::find_table_row`
- `evaluate_modifiers_prioritized` → `simulation::evaluate_column_modifiers`
- `apply_column_shift` → `simulation::apply_column_shift`

## Invariants

- `TurnStructure` is inserted at startup; may be empty or contain a default phase sequence
- `TurnState` is runtime-only (not persisted); reset when entering Play mode
- `CombatResultsTable.outcomes` dimensions must match `[table.rows.len()][table.columns.len()]`
- Column/row ordering invariants are inherited from the `simulation` contract's `ResolutionTable`
- `ActiveCombat` is runtime-only (not persisted); cleared when exiting Play mode
- `CombatModifierRegistry` modifiers are evaluated in priority order (highest first)
- Column shifts are clamped to `[0, columns.len() - 1]` after all modifiers applied
- `AreaMarkerRegistry` is inserted at startup; starts empty
- Area effects stack additively — multiple markers affecting the same hex sum their shifts/costs
- `collect_area_column_shifts` / `collect_area_cost_modifiers` use `hex_distance` for radius check

## Changelog

| Date       | Change                            | Reason                               |
| ---------- | --------------------------------- | ------------------------------------ |
| 2026-03-07 | Area-effect modifier types        | 0.21.0 Combat & Resolution (#235)    |
| 2026-03-07 | Post-resolution movement types    | 0.21.0 Combat & Resolution (#235)    |
| 2026-03-07 | Phase sequencer types + functions | 0.20.0 Simulation runtime (#234)     |
| 2026-03-05 | CRT → ResolutionTable delegation  | 0.17.0 CRT migration (#225)          |
| 2026-02-16 | Initial definition                | 0.9.0 Core mechanic primitives (#77) |
