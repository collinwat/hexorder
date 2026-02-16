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
