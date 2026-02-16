#![allow(clippy::used_underscore_binding)]
//! Shared mechanics types. See `docs/contracts/mechanics.md`.
//!
//! Defines turn structure, combat resolution (CRT), combat modifiers,
//! and combat execution state. These are the core wargame mechanic
//! primitives for 0.9.0.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::game_system::TypeId;

// ---------------------------------------------------------------------------
// Turn Structure
// ---------------------------------------------------------------------------

/// How players alternate within a turn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum PlayerOrder {
    /// One player completes all phases, then the next (classic IGOUGO).
    #[default]
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
    /// Designer notes for this phase.
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
/// Only meaningful in Play mode. Not persisted.
#[derive(Resource, Debug, Default, Reflect)]
pub struct TurnState {
    /// The current game turn number (1-indexed).
    pub turn_number: u32,
    /// Index into `TurnStructure.phases` for the current phase.
    pub current_phase_index: usize,
    /// Whether the turn is actively running (Play mode is active).
    pub is_active: bool,
}

// ---------------------------------------------------------------------------
// Combat Results Table
// ---------------------------------------------------------------------------

/// How a CRT column is calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CrtColumnType {
    /// Columns represent attacker:defender strength ratios (e.g., 1:2, 1:1, 2:1).
    OddsRatio,
    /// Columns represent attacker - defender strength differentials (e.g., -3, +2).
    Differential,
}

/// A single column header in the CRT.
/// Each column carries its own column type, allowing ratio and differential
/// columns to coexist in a single CRT.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CrtColumn {
    /// Display label (e.g., "3:1" or "+2").
    pub label: String,
    /// Whether this column uses odds ratio or differential calculation.
    pub column_type: CrtColumnType,
    /// For `OddsRatio`: the minimum ratio as a float (e.g., 3.0 for "3:1").
    /// For `Differential`: the minimum differential (e.g., 2.0 for "+2").
    /// Columns are ordered left to right by ascending threshold.
    pub threshold: f64,
}

/// A single row header in the CRT (die roll result).
/// Fully designer-defined — no auto-generation from dice config.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CrtRow {
    /// Display label (e.g., "1", "2", "3-4").
    pub label: String,
    /// The minimum die roll value this row matches.
    pub die_value_min: u32,
    /// The maximum die roll value this row matches (inclusive).
    /// For a single value, same as `die_value_min`.
    pub die_value_max: u32,
}

/// A structured effect that can be partially automated.
/// When present, the system highlights valid actions for designer confirmation.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum OutcomeEffect {
    /// No effect on either side.
    NoEffect,
    /// Defender retreats N hexes.
    Retreat { hexes: u32 },
    /// Defender loses N steps.
    StepLoss { steps: u32 },
    /// Attacker loses N steps.
    AttackerStepLoss { steps: u32 },
    /// Both sides lose steps (exchange).
    Exchange {
        attacker_steps: u32,
        defender_steps: u32,
    },
    /// Attacker eliminated.
    AttackerEliminated,
    /// Defender eliminated.
    DefenderEliminated,
}

/// A combat outcome: designer label + optional structured effect.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatOutcome {
    /// Designer-defined display label (e.g., "AE", "DR", "NE").
    pub label: String,
    /// Optional structured effect for partial automation.
    pub effect: Option<OutcomeEffect>,
}

/// The Combat Results Table: a 2D grid of combat outcomes.
///
/// Column types are per-column (mixed ratio/differential in one CRT).
/// Rows are fully custom (no auto-generation from dice config).
/// Strength comes from concept binding (ontology integration).
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatResultsTable {
    pub id: TypeId,
    pub name: String,
    pub columns: Vec<CrtColumn>,
    pub rows: Vec<CrtRow>,
    /// Outcome grid indexed as `[row_index][column_index]`.
    pub outcomes: Vec<Vec<CombatOutcome>>,
    /// Reference to the Combat concept in the ontology.
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

// ---------------------------------------------------------------------------
// Combat Modifiers
// ---------------------------------------------------------------------------

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
/// Modifiers have priority for ordered evaluation.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatModifierDefinition {
    pub id: TypeId,
    pub name: String,
    pub source: ModifierSource,
    /// Signed column shift. Positive = shift right (favor attacker).
    /// Negative = shift left (favor defender).
    pub column_shift: i32,
    /// Evaluation priority. Higher values are evaluated first.
    pub priority: i32,
    /// Optional cap on the running total column shift after this modifier.
    pub cap: Option<i32>,
    /// Optional: only applies when a specific entity type is the defender terrain.
    pub terrain_type_filter: Option<TypeId>,
}

/// Registry of all combat modifier definitions.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct CombatModifierRegistry {
    pub modifiers: Vec<CombatModifierDefinition>,
}

// ---------------------------------------------------------------------------
// Combat Execution (runtime, Play mode only)
// ---------------------------------------------------------------------------

/// Tracks the in-progress combat being resolved.
/// Runtime-only — not persisted.
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
    /// The die roll result.
    pub die_roll: Option<u32>,
    /// The resolved CRT row index.
    pub resolved_row: Option<usize>,
    /// The resolved combat outcome.
    pub outcome: Option<CombatOutcome>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// CRT Resolution Helpers
// ---------------------------------------------------------------------------

/// Result of a full CRT resolution.
#[derive(Debug, Clone)]
pub struct CrtResolution {
    pub column_index: usize,
    pub row_index: usize,
    pub column_label: String,
    pub row_label: String,
    pub outcome: CombatOutcome,
}

/// Calculates the odds ratio of attacker to defender strength.
/// Returns the ratio as a float (e.g., 3.0 for a 3:1 advantage).
/// Returns `f64::INFINITY` if defender strength is zero.
pub fn calculate_odds_ratio(attacker_strength: f64, defender_strength: f64) -> f64 {
    if defender_strength <= 0.0 {
        return f64::INFINITY;
    }
    attacker_strength / defender_strength
}

/// Calculates the differential of attacker minus defender strength.
pub fn calculate_differential(attacker_strength: f64, defender_strength: f64) -> f64 {
    attacker_strength - defender_strength
}

/// Finds the best matching CRT column for the given attacker and defender strengths.
///
/// Each column carries its own `column_type` (ratio or differential), so the
/// lookup calculates the appropriate value per column. Columns are assumed to
/// be ordered by ascending threshold. The function returns the index of the
/// rightmost column whose threshold the calculated value meets or exceeds.
///
/// Returns `None` if the CRT has no columns or the calculated value is below
/// all thresholds.
pub fn find_crt_column(
    attacker_strength: f64,
    defender_strength: f64,
    columns: &[CrtColumn],
) -> Option<usize> {
    let mut best_index: Option<usize> = None;

    for (i, col) in columns.iter().enumerate() {
        let value = match col.column_type {
            CrtColumnType::OddsRatio => calculate_odds_ratio(attacker_strength, defender_strength),
            CrtColumnType::Differential => {
                calculate_differential(attacker_strength, defender_strength)
            }
        };

        if value >= col.threshold {
            best_index = Some(i);
        }
    }

    best_index
}

/// Finds the CRT row matching a given die roll value.
///
/// Searches rows for one whose `[die_value_min, die_value_max]` range
/// includes the given roll. Returns `None` if no row matches.
pub fn find_crt_row(die_roll: u32, rows: &[CrtRow]) -> Option<usize> {
    rows.iter()
        .position(|row| die_roll >= row.die_value_min && die_roll <= row.die_value_max)
}

/// Resolves a complete CRT lookup: given attacker/defender strengths and a die
/// roll, returns the column index, row index, and outcome label.
///
/// Returns `None` if the column or row cannot be resolved, or if the outcome
/// grid doesn't have the expected dimensions.
pub fn resolve_crt(
    crt: &CombatResultsTable,
    attacker_strength: f64,
    defender_strength: f64,
    die_roll: u32,
) -> Option<CrtResolution> {
    let col_idx = find_crt_column(attacker_strength, defender_strength, &crt.columns)?;
    let row_idx = find_crt_row(die_roll, &crt.rows)?;

    let outcome = crt.outcomes.get(row_idx).and_then(|row| row.get(col_idx))?;

    Some(CrtResolution {
        column_index: col_idx,
        row_index: row_idx,
        column_label: crt.columns[col_idx].label.clone(),
        row_label: crt.rows[row_idx].label.clone(),
        outcome: outcome.clone(),
    })
}

/// Evaluates combat modifiers in priority order (highest first).
///
/// Each modifier's column shift is added to a running total. If the modifier
/// has a cap, the running total is clamped to `[-cap, +cap]` after addition.
/// The final total is clamped to the given column bounds.
///
/// Returns the final column shift and a display list of `(name, shift)` pairs
/// in evaluation order (highest priority first).
pub fn evaluate_modifiers_prioritized(
    modifiers: &[CombatModifierDefinition],
    column_count: usize,
) -> (i32, Vec<(String, i32)>) {
    let mut sorted: Vec<&CombatModifierDefinition> = modifiers.iter().collect();
    sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

    let mut total_shift: i32 = 0;
    let mut display: Vec<(String, i32)> = Vec::with_capacity(sorted.len());

    for modifier in &sorted {
        total_shift += modifier.column_shift;

        if let Some(cap) = modifier.cap {
            total_shift = total_shift.clamp(-cap, cap);
        }

        display.push((modifier.name.clone(), modifier.column_shift));
    }

    // Clamp to column bounds.
    if column_count > 0 {
        let max_shift = (column_count - 1) as i32;
        total_shift = total_shift.clamp(-max_shift, max_shift);
    }

    (total_shift, display)
}

/// Applies a column shift to a base column index, clamping to bounds.
pub fn apply_column_shift(base_column: usize, shift: i32, column_count: usize) -> usize {
    if column_count == 0 {
        return 0;
    }
    let shifted = base_column as i32 + shift;
    shifted.clamp(0, (column_count - 1) as i32) as usize
}
