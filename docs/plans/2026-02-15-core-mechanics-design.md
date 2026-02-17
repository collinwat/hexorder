# Design: Core Mechanic Primitives (#77)

> **Historical** — Implemented in v0.9.0. The code, plugin specs, and contracts are now
> authoritative. This document is preserved for context on why decisions were made.

> Turn structure and combat resolution for Hexorder 0.9.0. Big Batch, Cycle 4.

## Open Questions

These need user input before building begins. They are ordered by impact on implementation.

### Q1. Turn Structure: Resource or State Machine?

The pitch describes a turn as an "ordered sequence of named phases." Two approaches:

- **Resource-only (data)**: `TurnStructure` is a resource holding a `Vec<Phase>` definition plus a
  `current_phase_index`. The `game_system` plugin advances the index. No Bevy `States` involved.
  Systems that care about the current phase read the resource. Simpler, more flexible for
  designer-defined phase sequences.
- **SubState (Bevy state machine)**: A `PhaseState` SubState derived from an `AppScreen::Play`
  parent state. Each phase becomes a state variant with `OnEnter`/`OnExit` scheduling. More
  structured, but requires knowing the phase set at compile time (contradicts designer-defined
  phases).

**Recommendation**: Resource-only approach. Designer-defined phases cannot be known at compile time,
so Bevy `States` are a poor fit. Systems use run conditions that check the current phase type
(movement, combat, admin) rather than a specific state variant.

**Needs confirmation**: Is `AppScreen::Play` the right name for the play-mode state? Or should it be
`AppScreen::Simulation` / `AppScreen::Playtest`?

> **DECIDED**: ACCEPTED recommendation. Resource-only approach. `AppScreen::Play` is the confirmed
> name for the play-mode state. No implementation changes from the recommendation.

### Q2. Play Mode vs Design Mode

The pitch says "in play mode during combat phase." Currently Hexorder only has `AppScreen::Launcher`
and `AppScreen::Editor`. This pitch implies a third mode where the designer can step through turns.

- Does the user switch between Editor and Play via a toolbar button?
- In Play mode, are definition-editing panels hidden (read-only)?
- Can the user switch back to Editor mid-turn (pausing the simulation)?

**Recommendation**: Add `AppScreen::Play` to the `AppScreen` enum. Play mode shows a turn tracker,
combat execution panel, and a simplified inspector. The user toggles between Editor and Play via a
toolbar button. Switching to Editor pauses the turn; switching back resumes. Definitions are
read-only in Play mode.

> **DECIDED**: ACCEPTED recommendation. `AppScreen::Play` added. Toggle between Editor and Play via
> toolbar button. Definitions are read-only in Play mode. Switching to Editor pauses the turn. No
> implementation changes from the recommendation.

### Q3. CRT Column Type: Ratio vs Differential vs Designer Choice?

The pitch mentions "odds ratios or differentials." Should the tool support both column types in the
same CRT, or is it one or the other per CRT?

**Recommendation**: Each CRT has a `column_type` field (`OddsRatio` or `Differential`). The column
headers and odds calculation logic change accordingly. A game system can have multiple CRTs with
different column types (e.g., one ratio CRT for ground combat, one differential for bombardment).
One CRT per game system is sufficient for the first piece; multiple CRTs can follow.

> **DECIDED**: REJECTED recommendation. A single CRT can mix both ratio and differential columns.
> Each `CrtColumn` carries its own `column_type` field instead of the CRT having a single top-level
> `column_type`. This means the CRT data model changes: remove `column_type` from
> `CombatResultsTable`, add `column_type: CrtColumnType` to each `CrtColumn`. The column lookup
> logic must handle per-column types, calculating the appropriate value (ratio or differential) for
> each column and finding the best match. This is more complex than the recommendation but gives
> designers full flexibility to blend column types in one table.

### Q4. Die Roll Configuration

The pitch does not specify die types. Classic wargames use 1d6 or 2d6. Should the CRT define its die
configuration (number of dice, sides per die), or is this fixed?

**Recommendation**: The CRT stores `dice_count: u8` and `die_sides: u8`. Row labels are
auto-generated from the range (e.g., 1d6 produces rows 1-6; 2d6 produces rows 2-12). The designer
can override row labels for custom die schemes. Default: 1d6.

> **DECIDED**: REJECTED recommendation. Fully custom rows with maximum flexibility. The designer
> defines arbitrary row labels and value ranges manually. No auto-generation from `dice_count` /
> `die_sides`. Remove `dice_count` and `die_sides` from `CombatResultsTable`. The `CrtRow` type
> keeps `label`, `die_value_min`, `die_value_max` but all values are designer-entered. This supports
> non-standard dice, custom probability curves, or any lookup table the designer invents. The UI
> provides Add Row / Remove Row buttons and the designer fills in every row.

### Q5. Combat Outcome Semantics

The pitch says "outcomes are designer-defined strings." This means AE, AR, DR, EX, NE are just
labels with no hardcoded behavior. But the pitch also says "applies result" during combat execution.

- What does "applies result" mean if outcomes are just strings?
- Should there be an optional structured outcome type (e.g., "retreat N hexes", "lose N steps")
  alongside the string label?

**Recommendation**: For this cycle, outcomes are purely designer-defined strings displayed in the
combat result panel. No automated application. The designer manually interprets the result and
moves/removes units. Future cycles can add structured outcome types with automated effects. This
aligns with the pitch's No Go: "no automated retreat."

> **DECIDED**: REJECTED recommendation. Structured outcomes with string labels. Each outcome has
> both a designer-defined string label AND an optional structured type. Structured types include
> retreat N hexes, lose N steps, exchange, no effect, etc. The string label is always displayed; the
> structured type enables partial automation this cycle. When a structured outcome is present, the
> system can highlight valid retreat hexes or mark step losses, but the designer still confirms.
> This deviates from the "no automated retreat" No Go -- partial automation (highlight + confirm) is
> permitted, but fully automated application is not. The CRT cell type changes from `String` to a
> new `CombatOutcome` struct with `label: String` and `effect: Option<OutcomeEffect>`.

### Q6. Modifier Precedence and Stacking

When multiple modifiers apply (e.g., defender in forest + across river + entrenched), how do they
combine? Simple addition of column shifts? Are there caps?

**Recommendation**: Modifiers are signed column shifts that sum additively. The total shift is
clamped to the CRT's column range (cannot shift beyond the leftmost or rightmost column). No
multiplicative modifiers in this cycle. The modifier list is displayed in the combat resolution
panel so the designer can verify correctness.

> **DECIDED**: REJECTED recommendation. Prioritized/ordered modifiers. Each modifier has a
> `priority: i32` field. Modifiers are evaluated in priority order (highest first). Higher-priority
> modifiers can override or cap lower-priority ones. This replaces simple additive sums with a
> priority-based evaluation pipeline. `CombatModifierDefinition` gains `priority: i32` and an
> optional `cap: Option<i32>` field (caps the running total column shift). The modifier evaluation
> system sorts by priority descending, then applies shifts and caps in order. The final total is
> still clamped to column bounds. The combat resolution panel displays modifiers sorted by priority
> to show the evaluation order.

### Q7. Attacker/Defender Strength Source

Where does combat strength come from? The pitch assumes units have attack/defense values, but the
current entity type system uses generic properties. Should combat strength be:

- A well-known property name (e.g., "Attack Strength", "Defense Strength")?
- A concept binding (e.g., a "Combat" concept with "attacker" and "defender" roles)?
- Explicitly selected by the designer per CRT?

**Recommendation**: The CRT definition includes `attacker_strength_property: String` and
`defender_strength_property: String`. These reference property names on Token entity types. The
combat execution system looks up these properties on the selected attacker/defender units. This
avoids hardcoding property names while giving the designer explicit control. A concept binding for
combat could be added later for richer integration with the ontology.

> **DECIDED**: REJECTED recommendation. Concept binding for combat strength. A "Combat" concept with
> attacker/defender roles, integrated with the ontology system. Remove `attacker_strength_property`
> and `defender_strength_property` from `CombatResultsTable`. Instead, the CRT references a
> `combat_concept_id: TypeId` pointing to a concept in the ontology. The concept defines roles
> (attacker, defender) with role bindings that map to entity type properties. The combat execution
> system resolves strength by looking up the concept, finding the role binding for the relevant role
> (attacker/defender), and reading the bound property from the entity. This is more complex but
> integrates combat with the existing ontology system rather than creating a parallel
> property-reference mechanism. Requires coordination with the ontology contract.

---

## Overview

This pitch adds four capabilities to Hexorder:

1. **Turn structure definition**: The designer defines a sequence of named phases with types. A turn
   tracker resource manages phase progression.
2. **Combat Results Table (CRT) editor**: A spreadsheet-like grid in the editor UI where the
   designer defines odds columns, die roll rows, and outcome cells.
3. **Combat execution**: In play mode during a combat phase, the user selects attacker/defender
   units, the system calculates odds, highlights the CRT column, rolls dice, and displays the
   result.
4. **Modifier system**: Combat modifiers from terrain, unit status, or designer-defined sources are
   registered as signed column shifts that adjust the CRT lookup.

The implementation touches four plugins (`game_system`, `rules_engine`, `unit`, `editor_ui`) and
introduces a new contract (`mechanics`). No new plugins are created; this scope extends existing
ones.

---

## New Contracts

### `contracts::mechanics` (new file: `src/contracts/mechanics.rs`)

This contract defines all shared types for turn structure and combat resolution. It is consumed by
`game_system`, `rules_engine`, `unit`, and `editor_ui`.

```rust
// ---------- Turn Structure ----------

/// How players alternate within a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum PlayerOrder {
    /// One player completes all phases, then the next (classic IGOUGO).
    Alternating,
    /// Both players act simultaneously in each phase.
    Simultaneous,
    /// Players alternate activating individual units or groups.
    ActivationBased,
}

/// The category of actions allowed during a phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum PhaseType {
    /// Units may move (movement budget consumed).
    Movement,
    /// Combat may be declared and resolved.
    Combat,
    /// Administrative actions (reinforcements, supply, victory checks).
    Admin,
}

/// A single named phase within a turn sequence.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Phase {
    pub id: TypeId,
    pub name: String,
    pub phase_type: PhaseType,
    /// Designer notes for this phase (e.g., "Attacker moves all units").
    pub description: String,
}

/// The designer-defined turn structure for the game system.
/// An ordered sequence of phases that repeats each game turn.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TurnStructure {
    pub phases: Vec<Phase>,
    pub player_order: PlayerOrder,
}

impl Default for TurnStructure {
    fn default() -> Self {
        Self {
            phases: Vec::new(),
            player_order: PlayerOrder::Alternating,
        }
    }
}

/// Runtime state tracking the current position within a turn.
/// Only meaningful in Play mode.
#[derive(Resource, Debug, Default, Reflect)]
pub struct TurnState {
    /// The current game turn number (1-indexed).
    pub turn_number: u32,
    /// Index into TurnStructure.phases for the current phase.
    pub current_phase_index: usize,
    /// Whether the turn is actively running (Play mode is active).
    pub is_active: bool,
}

// ---------- Combat Results Table ----------

/// How the CRT columns are calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CrtColumnType {
    /// Columns represent attacker:defender strength ratios (e.g., 1:2, 1:1, 2:1, 3:1).
    OddsRatio,
    /// Columns represent attacker - defender strength differentials (e.g., -3, -2, ..., +3).
    Differential,
}

/// A single column header in the CRT.
/// Each column carries its own column type, allowing ratio and differential
/// columns to coexist in a single CRT (Q3 decision).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CrtColumn {
    /// Display label (e.g., "3:1" or "+2").
    pub label: String,
    /// Whether this column uses odds ratio or differential calculation.
    pub column_type: CrtColumnType,
    /// For OddsRatio: the minimum ratio as a float (e.g., 3.0 for "3:1").
    /// For Differential: the minimum differential as a float (e.g., 2.0 for "+2").
    /// Columns are ordered left to right; the lookup finds the highest column
    /// whose threshold the calculated value meets or exceeds.
    pub threshold: f64,
}

/// A single row header in the CRT (die roll result).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CrtRow {
    /// Display label (e.g., "1", "2", "3-4").
    pub label: String,
    /// The die roll value this row matches. For ranges, the minimum value.
    pub die_value_min: u32,
    /// The maximum die roll value this row matches (inclusive).
    /// For a single value, same as die_value_min.
    pub die_value_max: u32,
}

/// A structured effect that can be partially automated (Q5 decision).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum OutcomeEffect {
    /// No effect on either side.
    NoEffect,
    /// Defender retreats N hexes (highlights valid retreat hexes for confirmation).
    Retreat { hexes: u32 },
    /// Defender loses N steps (marks step losses for confirmation).
    StepLoss { steps: u32 },
    /// Attacker loses N steps.
    AttackerStepLoss { steps: u32 },
    /// Both sides lose steps (exchange).
    Exchange { attacker_steps: u32, defender_steps: u32 },
    /// Attacker eliminated.
    AttackerEliminated,
    /// Defender eliminated.
    DefenderEliminated,
}

/// A combat outcome: designer label + optional structured effect (Q5 decision).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatOutcome {
    /// Designer-defined display label (e.g., "AE", "DR", "NE").
    pub label: String,
    /// Optional structured effect for partial automation.
    /// When present, the system can highlight valid actions (e.g., retreat hexes)
    /// but the designer must confirm before application.
    pub effect: Option<OutcomeEffect>,
}

/// The Combat Results Table: a 2D grid of combat outcomes.
/// Column types are per-column (Q3: mixed ratio/differential in one CRT).
/// Rows are fully custom (Q4: no auto-generation from dice config).
/// Strength comes from concept binding (Q7: ontology integration).
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatResultsTable {
    pub id: TypeId,
    pub name: String,
    /// Note: no top-level column_type -- each CrtColumn has its own (Q3 decision).
    pub columns: Vec<CrtColumn>,
    /// Fully designer-defined rows (Q4 decision). No dice_count/die_sides.
    pub rows: Vec<CrtRow>,
    /// Outcome structs indexed as [row_index][column_index] (Q5 decision).
    /// Each cell has a label and optional structured effect.
    pub outcomes: Vec<Vec<CombatOutcome>>,
    /// Reference to the Combat concept in the ontology (Q7 decision).
    /// The concept defines attacker/defender roles with property bindings.
    pub combat_concept_id: Option<TypeId>,
}

impl Default for CombatResultsTable {
    fn default() -> Self {
        Self {
            id: TypeId::new(),
            name: "Combat Results Table".to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
            outcomes: Vec::new(),
            combat_concept_id: None,
        }
    }
}

// ---------- Combat Modifiers ----------

/// The source of a combat modifier.
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum ModifierSource {
    /// Modifier comes from the terrain the defender occupies.
    DefenderTerrain,
    /// Modifier comes from the terrain the attacker occupies.
    AttackerTerrain,
    /// Modifier comes from a property on the attacking unit.
    AttackerProperty(String),
    /// Modifier comes from a property on the defending unit.
    DefenderProperty(String),
    /// Modifier comes from a designer-defined rule (manual).
    Custom(String),
}

/// A combat modifier definition: a signed column shift applied during CRT lookup.
/// Modifiers have priority for ordered evaluation (Q6 decision).
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatModifierDefinition {
    pub id: TypeId,
    pub name: String,
    pub source: ModifierSource,
    /// Signed column shift. Positive = shift right (favor attacker).
    /// Negative = shift left (favor defender).
    pub column_shift: i32,
    /// Evaluation priority. Higher values are evaluated first.
    /// Higher-priority modifiers can override or cap lower-priority ones.
    pub priority: i32,
    /// Optional cap on the running total column shift after this modifier is applied.
    /// If set, the running total is clamped to [-cap, +cap] after this modifier's
    /// shift is added. This allows high-priority modifiers to limit the impact of
    /// lower-priority ones.
    pub cap: Option<i32>,
    /// Optional: only applies when a specific entity type is the defender terrain.
    pub terrain_type_filter: Option<TypeId>,
}

/// Registry of all combat modifier definitions.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct CombatModifierRegistry {
    pub modifiers: Vec<CombatModifierDefinition>,
}

// ---------- Combat Execution (runtime, Play mode only) ----------

/// Tracks the in-progress combat being resolved.
#[derive(Resource, Debug, Default, Reflect)]
pub struct ActiveCombat {
    /// The attacking unit entity.
    pub attacker: Option<Entity>,
    /// The defending unit entity.
    pub defender: Option<Entity>,
    /// Calculated raw odds ratio or differential before modifiers.
    pub raw_value: Option<f64>,
    /// Total column shift from all applicable modifiers.
    pub total_shift: i32,
    /// List of modifier names and their shifts (for display).
    pub applied_modifiers: Vec<(String, i32)>,
    /// The final CRT column index after applying shifts.
    pub resolved_column: Option<usize>,
    /// The die roll result (sum of all dice).
    pub die_roll: Option<u32>,
    /// The resolved CRT row index.
    pub resolved_row: Option<usize>,
    /// The resolved combat outcome (label + optional structured effect).
    pub outcome: Option<CombatOutcome>,
}

// ---------- Events ----------

/// Fired when the turn advances to the next phase.
#[derive(Event, Debug, Reflect)]
pub struct PhaseAdvancedEvent {
    pub turn_number: u32,
    pub phase_index: usize,
    pub phase_name: String,
    pub phase_type: PhaseType,
}

/// Fired when a combat is fully resolved (die rolled, outcome determined).
#[derive(Event, Debug, Reflect)]
pub struct CombatResolvedEvent {
    pub attacker: Entity,
    pub defender: Entity,
    /// The resolved outcome with label and optional structured effect.
    pub outcome: CombatOutcome,
    pub die_roll: u32,
    pub column_label: String,
}
```

### Contract spec: `docs/contracts/mechanics.md`

Must be created before implementation. Mirrors the types above with Purpose, Consumers, Producers,
Invariants sections per the contract template.

**Consumers**: `game_system`, `rules_engine`, `unit`, `editor_ui`, `persistence`

**Producers**: `game_system` (inserts `TurnStructure`, `TurnState`, `CombatResultsTable`,
`CombatModifierRegistry`, `ActiveCombat` at startup)

### Changes to Existing Contracts

#### `contracts::persistence` (modified)

`GameSystemFile` gains new fields for turn structure and combat data:

```rust
pub struct GameSystemFile {
    // ... existing fields ...

    /// Turn structure definition (0.9.0).
    pub turn_structure: TurnStructure,
    /// Combat Results Table (0.9.0).
    pub combat_results_table: CombatResultsTable,
    /// Combat modifier definitions (0.9.0).
    pub combat_modifiers: CombatModifierRegistry,
}
```

`FORMAT_VERSION` bumps to `3`.

#### `contracts::persistence::AppScreen` (modified)

```rust
pub enum AppScreen {
    #[default]
    Launcher,
    Editor,
    /// Play mode: step through turns, resolve combat (0.9.0).
    Play,
}
```

#### `contracts::editor_ui` (modified)

`EditorTool` gains a combat-related variant for Play mode:

```rust
pub enum EditorTool {
    Select,
    Paint,
    Place,
    /// In Play mode: select attacker/defender for combat.
    CombatSelect,
}
```

---

## Plugin Changes

### game_system

**New responsibilities**: Initialize turn structure and CRT resources.

**Changes to `build()`**:

- Insert `TurnStructure::default()` resource
- Insert `TurnState::default()` resource
- Insert `CombatResultsTable::default()` resource
- Insert `CombatModifierRegistry::default()` resource
- Insert `ActiveCombat::default()` resource

**New factory functions in `systems.rs`**:

- `create_default_turn_structure()` -- Returns a starter 5-phase turn structure (Reinforcement,
  Movement, Combat, Supply, Victory Check) with appropriate `PhaseType` values. This gives the
  designer something to edit immediately rather than starting from blank.
- `create_default_crt()` -- Returns a starter CRT with standard odds-ratio columns (1:2, 1:1, 2:1,
  3:1, 4:1, 5:1, 6:1) and 6 custom rows (labeled "1" through "6", each with matching
  die_value_min/max), populated with classic `CombatOutcome` values (label + structured effect).
  Starter entity types gain "Attack Strength" and "Defense Strength" properties on Token types. A
  default "Combat" concept is created in the ontology with attacker/defender roles bound to these
  properties.

**Starter property additions**: Infantry, Cavalry, Artillery gain:

- `Attack Strength` (Int, defaults: Infantry=4, Cavalry=6, Artillery=8)
- `Defense Strength` (Int, defaults: Infantry=4, Cavalry=3, Artillery=2)

### rules_engine

**New responsibilities**: Combat resolution logic and modifier evaluation.

**New systems**:

- `resolve_combat_odds` (Update, Play state) -- When `ActiveCombat` has both attacker and defender
  set, resolves strength via the Combat concept binding in the ontology (Q7 decision). Looks up the
  concept, finds the role bindings for attacker/defender, reads the bound properties from the
  entities. For each CRT column, calculates the appropriate value (ratio or differential) based on
  that column's `column_type` (Q3 decision). Finds the best matching column. Populates
  `ActiveCombat.raw_value` and `ActiveCombat.resolved_column`.

- `evaluate_combat_modifiers` (Update, Play state, runs after `resolve_combat_odds`) -- Scans
  `CombatModifierRegistry` for applicable modifiers given the attacker, defender, and defender's
  terrain. Sorts applicable modifiers by priority descending (Q6 decision). Evaluates in priority
  order: applies each shift to a running total, then applies the modifier's cap (if any) to clamp
  the running total. Final total is clamped to column bounds. Populates
  `ActiveCombat.applied_modifiers` (sorted by priority) and adjusts `ActiveCombat.resolved_column`.

- `apply_die_roll` (Update, Play state, runs after `evaluate_combat_modifiers`) -- When the user
  triggers a die roll (via UI button), generates a random result. The die roll value is entered or
  generated externally (no built-in dice config -- Q4 decision). Finds the matching CRT row by
  checking `die_value_min`/`die_value_max` ranges. Reads the `CombatOutcome` from
  `outcomes[row][column]`. If the outcome has a structured effect, prepares partial automation data
  (e.g., valid retreat hexes). Sets `ActiveCombat.outcome`. Fires `CombatResolvedEvent`.

**Existing system changes**:

- `compute_valid_moves` -- No changes needed. Movement validation continues to work as before. In
  Play mode during non-movement phases, movement systems are gated by phase type, so `ValidMoveSet`
  is not consulted.

**New helper functions**:

- `calculate_odds_ratio(attacker_strength: f64, defender_strength: f64) -> f64`
- `calculate_differential(attacker_strength: f64, defender_strength: f64) -> f64`
- `find_crt_column(attacker: f64, defender: f64, columns: &[CrtColumn]) -> Option<usize>` -- Handles
  mixed column types (Q3): calculates ratio or differential per column based on each column's
  `column_type` field, then finds the best match.
- `find_crt_row(die_roll: u32, rows: &[CrtRow]) -> Option<usize>`
- `resolve_strength_from_concept(concept_id: TypeId, role: &str, entity: Entity, ...) -> Option<f64>`
  -- Looks up combat strength via the ontology concept binding (Q7 decision).
- `evaluate_modifiers_prioritized(modifiers: &[CombatModifierDefinition]) -> (i32, Vec<(String, i32)>)`
  -- Sorts by priority descending, applies shifts and caps in order (Q6 decision).

### unit

**New responsibilities**: Attacker/defender selection in Play mode.

**New system**:

- `handle_combat_selection` (observer on `HexSelectedEvent`, Play state + Combat phase +
  `CombatSelect` tool) -- When a hex with a unit is clicked:
    - If `ActiveCombat.attacker` is None, set this unit as attacker.
    - If attacker is set but defender is None, set this unit as defender (must be a different unit
      from the attacker). Trigger odds calculation.
    - If both are set, reset and start over.
    - Validate that attacker and defender are on adjacent or nearby hexes (designer-defined range,
      or no range check for this cycle per simplicity).

**Existing system changes**:

- `handle_unit_interaction` -- Add a guard: skip unit movement logic when in Play mode during
  non-movement phases. During movement phases in Play mode, movement still works as normal.
- `handle_unit_placement` -- Guard: skip placement when in Play mode (units are placed only in
  Editor mode).

### editor_ui

**New responsibilities**: Turn structure editor, CRT editor, combat execution panel, play mode
toggle.

**New UI panels** (all in `editor_ui/systems.rs`):

#### Turn Structure Editor (Editor mode, new tab)

- Renders in a new "Mechanics" tab alongside Types, Enums, Structs, Concepts, etc.
- Shows the ordered list of phases with drag-to-reorder (or up/down buttons).
- For each phase: name (text input), type (dropdown: Movement/Combat/Admin), description.
- Add Phase / Remove Phase buttons.
- Player order dropdown (Alternating / Simultaneous / ActivationBased).

#### CRT Editor (Editor mode, under Mechanics tab)

- A collapsible section within the Mechanics tab.
- Column headers: editable labels, thresholds, and per-column type selector (OddsRatio /
  Differential) (Q3 decision: mixed column types in one CRT). Add/Remove column buttons.
- Row headers: fully designer-defined (Q4 decision). Add Row / Remove Row buttons. Each row has
  editable label, die_value_min, and die_value_max fields. No auto-generation.
- Grid of outcome cells. Each cell has a label text input and an optional structured effect dropdown
  (Q5 decision: None, Retreat N, StepLoss N, etc.) with numeric parameters.
- Combat concept selector: dropdown to pick the Combat concept from the ontology (Q7 decision).
  Shows the concept's role bindings (attacker property, defender property) read-only for reference.

#### Combat Modifier Editor (Editor mode, under Mechanics tab)

- List of modifier definitions sorted by priority (highest first), with name, source type, column
  shift, priority, optional cap, and terrain filter (Q6 decision).
- Add/Remove modifier buttons.
- Priority is an integer field. The display order reflects evaluation order.

#### Play Mode Toggle

- A prominent button in the toolbar area: "Play" / "Editor" toggle.
- Triggers `AppScreen` state transition.

#### Turn Tracker (Play mode)

- A top bar showing: Turn N, current phase name, phase type badge, Next Phase button.
- Advancing past the last phase increments the turn number and wraps to phase 0.
- Fires `PhaseAdvancedEvent`.

#### Combat Execution Panel (Play mode, Combat phase)

- Left section: Attacker info (name, position, attack strength resolved via concept binding).
- Right section: Defender info (name, position, defense strength resolved via concept binding,
  terrain).
- Center: Calculated odds, modifier breakdown (table of name + shift + priority, sorted by priority
  descending -- Q6 decision), final column highlight.
- "Roll" button triggers the die roll.
- Result display: die roll value, row label, outcome label (large, prominent). If the outcome has a
  structured effect (Q5 decision), display the effect type and parameters (e.g., "Retreat 2 hexes")
  and highlight valid actions on the hex grid for designer confirmation.
- "Confirm" button (when structured effect present) applies the effect.
- "Clear" button resets `ActiveCombat`.

**New `EditorAction` variants**:

- `AddPhase { name, phase_type, description }`
- `RemovePhase { id }`
- `ReorderPhase { id, new_index }`
- `UpdatePhase { id, name, phase_type, description }`
- `SetPlayerOrder { order }`
- `AddCrtColumn { label, column_type, threshold }` (Q3: per-column type)
- `RemoveCrtColumn { index }`
- `UpdateCrtColumn { index, label, column_type, threshold }` (Q3: per-column type editable)
- `AddCrtRow { label, die_value_min, die_value_max }` (Q4: fully custom rows)
- `RemoveCrtRow { index }` (Q4: manual row management)
- `UpdateCrtRow { index, label, die_value_min, die_value_max }` (Q4: editable row values)
- `UpdateCrtCell { row, column, outcome: CombatOutcome }` (Q5: structured outcomes)
- `SetCrtCombatConcept { concept_id }` (Q7: concept binding)
- `AddCombatModifier { name, source, shift, priority, cap, terrain_filter }` (Q6: prioritized)
- `UpdateCombatModifier { id, name, source, shift, priority, cap, terrain_filter }` (Q6)
- `RemoveCombatModifier { id }`

**New `OntologyTab` variant**: `Mechanics` (or a separate top-level tab outside the ontology
section, since mechanics are not part of the ontology framework).

**New `EditorState` fields**: Turn structure editor state, CRT editor state, combat modifier editor
state (new type name fields, selection indices, etc.).

---

## System Ordering and Data Flow

### Schedule Labels

| System                    | Schedule | Run Condition                                   | Plugin       |
| ------------------------- | -------- | ----------------------------------------------- | ------------ |
| Turn structure init       | Startup  | --                                              | game_system  |
| CRT/modifier init         | Startup  | --                                              | game_system  |
| handle_combat_selection   | Observer | HexSelectedEvent, Play + CombatSelect tool      | unit         |
| resolve_combat_odds       | Update   | in_state(AppScreen::Play), ActiveCombat changed | rules_engine |
| evaluate_combat_modifiers | Update   | in_state(AppScreen::Play), after resolve_odds   | rules_engine |
| apply_die_roll            | Update   | in_state(AppScreen::Play), after evaluate_mods  | rules_engine |
| turn_structure_editor     | EguiPass | in_state(AppScreen::Editor)                     | editor_ui    |
| crt_editor                | EguiPass | in_state(AppScreen::Editor)                     | editor_ui    |
| combat_execution_panel    | EguiPass | in_state(AppScreen::Play)                       | editor_ui    |
| turn_tracker              | EguiPass | in_state(AppScreen::Play)                       | editor_ui    |
| play_mode_toggle          | EguiPass | always (both Editor and Play)                   | editor_ui    |

### System Ordering Within rules_engine (Play mode)

```
resolve_combat_odds -> evaluate_combat_modifiers -> apply_die_roll
```

These chain sequentially. The die roll system only runs when the user clicks the Roll button
(checked via a flag on `ActiveCombat` or a dedicated `RollDiceEvent`).

### Data Flow: Define -> Play -> Observe

```
DEFINE (Editor mode):
  Designer edits TurnStructure via UI  -> TurnStructure resource updated
  Designer edits CRT via grid UI       -> CombatResultsTable resource updated
  Designer adds modifiers via UI       -> CombatModifierRegistry resource updated
  Designer adds Attack/Defense props   -> EntityTypeRegistry + EntityData updated

PLAY (Play mode):
  1. User clicks "Play" button          -> AppScreen::Play state entered
  2. Turn tracker shows phase 1         -> TurnState initialized (turn 1, phase 0)
  3. User advances to Combat phase      -> TurnState.current_phase_index updated
                                           PhaseAdvancedEvent fired
  4. User selects attacker unit         -> ActiveCombat.attacker set
  5. User selects defender unit         -> ActiveCombat.defender set
  6. resolve_combat_odds runs           -> Resolves strength via Combat concept binding (Q7)
                                           Reads CombatResultsTable columns (mixed types, Q3)
                                           Populates ActiveCombat.raw_value, resolved_column
  7. evaluate_combat_modifiers runs     -> Reads CombatModifierRegistry
                                           Reads defender terrain EntityData
                                           Populates ActiveCombat.applied_modifiers, adjusts column
  8. Combat execution panel shows       -> Odds, modifiers, highlighted column displayed
  9. User clicks "Roll"                 -> apply_die_roll generates random result
                                           Finds CRT row
                                           Reads outcomes[row][column]
                                           Sets ActiveCombat.outcome
                                           Fires CombatResolvedEvent
  10. Result displayed                  -> UI shows die roll, outcome string
  11. Designer manually applies result  -> Moves/removes units in Editor or Play mode

OBSERVE:
  Combat result is displayed but not automatically applied.
  Designer interprets the outcome string and acts manually.
```

### Persistence Flow

On save, the persistence plugin reads `TurnStructure`, `CombatResultsTable`, and
`CombatModifierRegistry` resources and includes them in `GameSystemFile`. On load, it restores these
resources. `TurnState` and `ActiveCombat` are NOT persisted (they are runtime-only).

---

## First Piece

### CRT data model + resolution logic + minimal test harness

**Why this first**:

- The CRT is the most novel, domain-specific piece. It has the highest uncertainty.
- It is a self-contained data structure with pure-function resolution logic.
- It can be fully tested without UI, without play mode, without turn structure.
- It surfaces unknowns early: column lookup edge cases, threshold ordering, ratio vs differential
  calculation, die roll row matching.

**What "end-to-end" means for this piece**:

1. Define `CombatResultsTable` type in `src/contracts/mechanics.rs`
2. Write `calculate_odds_ratio`, `find_crt_column`, `find_crt_row` helper functions in
   `rules_engine/systems.rs`
3. Write unit tests:
    - CRT column lookup for odds ratio (exact match, between columns, below minimum, above maximum)
    - CRT column lookup for differential (same cases)
    - CRT row lookup for single values and ranges
    - Full resolution: given attacker strength, defender strength, and die roll, get outcome string
    - Column shift clamping (shift left/right past bounds)
    - Edge cases: zero defender strength, empty CRT, single-column CRT
4. Add contract spec `docs/contracts/mechanics.md`
5. Register the contract module in `src/contracts/mod.rs`

**Time estimate**: 1-2 days. Pure data structures and logic, no UI or Bevy systems needed.

### Second piece: Turn structure resource + phase advancement

After the CRT logic is proven, add the turn structure types and the phase advancement system. This
is straightforward (a counter incrementing through a list) but connects to the `AppScreen::Play`
state, which requires a small amount of editor_ui work (the Play button).

### Third piece: CRT editor UI

The spreadsheet-like grid is the most complex UI work. Build it after the data model is stable.
egui's `Grid` or `TableBuilder` widgets can render a scrollable CRT grid with editable cells.

### Fourth piece: Combat execution flow

Wire up attacker/defender selection, odds calculation, modifier evaluation, die rolling, and the
combat execution panel. This is the integration piece that ties everything together.

---

## Risk Assessment

### High risk: CRT editor UI complexity

The spreadsheet grid is the most complex UI element Hexorder has built. egui does not have a native
spreadsheet widget. We need a scrollable grid with individually editable text cells, column/row
headers, and dynamic sizing. Mitigation: prototype the grid layout early using `egui::Grid` or
`egui_extras::TableBuilder`. If the UI is too complex, fall back to a simpler list-based CRT editor
(one column at a time).

### Medium risk: AppScreen::Play state integration

Adding a third app state affects multiple plugins. Systems currently gated on
`in_state(AppScreen::Editor)` will not run in Play mode. We need to audit all existing run
conditions to ensure correct behavior in Play mode. Some systems (like cell visual sync, hex grid
rendering) must run in both Editor and Play. Mitigation: identify all `run_if` conditions in
existing plugins before starting, and group them by "Editor-only", "Play-only", and "both".

### Medium risk: Persistence format version bump

Bumping `FORMAT_VERSION` to 3 means old `.hexorder` files need migration handling (the new fields
are absent). Mitigation: use `Option<T>` with `#[serde(default)]` for the new fields in
`GameSystemFile` so that v2 files load cleanly with empty/default turn structure and CRT.

### Low risk: Random number generation

Combat resolution needs a random die roll. Bevy 0.18 may include `GlobalRng` or we may need the
`rand` crate. Mitigation: check Bevy 0.18's random number API; if not available, `rand` is a trivial
dependency.

### Low risk: Turn structure is simple data

The turn structure is just an ordered list of phases with a counter. No complex state machine logic.
This is low risk.

### No risk: Modifier system

The modifier system is additive column shifts. Pure arithmetic with clamping. Well-understood.

---

## Dependency Graph Update

After this pitch, the architecture dependency graph gains:

```
mechanics (contract) --> game_system    (inserts resources)
mechanics (contract) --> rules_engine   (combat resolution logic)
mechanics (contract) --> unit           (combat selection)
mechanics (contract) --> editor_ui      (CRT editor, combat panel, turn tracker)
mechanics (contract) --> persistence    (save/load)
```

The `mechanics` contract depends on `game_system` (uses `TypeId`, `PropertyValue`) and `hex_grid`
(uses `HexPosition`).

---

## Deferred Items (from Pitch No-Gos)

These are explicitly out of scope. Each should be captured as a GitHub Issue if not already present.

- Card-driven / chit-pull activation systems
- Automated retreat (outcome application)
- Multi-hex combat (multiple hexes attacking one)
- Ranged support fire
- Probability visualization (odds calculator)
- ZOC effects on combat
- Structured outcome types (beyond string labels)
- Multiple CRTs per game system (beyond the first one)
- Die roll modifiers (DRMs) separate from column shifts
- Combat advance after combat
- Supply effects on combat strength
