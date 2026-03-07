#![allow(clippy::used_underscore_binding)]
//! Shared mechanics types. See `docs/contracts/mechanics.md`.
//!
//! Defines turn structure, combat resolution (CRT), combat modifiers,
//! and combat execution state. Table lookup is delegated to the generic
//! `ResolutionTable` primitives in `simulation.rs` (ADR-005).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game_system::TypeId;
use crate::simulation::{ResolutionTable, find_table_column, find_table_row};

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
    /// Actions remaining in the current phase (None = unlimited).
    pub phase_actions_remaining: Option<u32>,
}

/// An action the user can take to control phase progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PhaseAction {
    /// Advance to the next phase (or next turn if at the last phase).
    Advance,
    /// Go back to the previous phase (or previous turn if at the first phase).
    Rewind,
    /// Skip the current phase without executing it.
    Skip,
}

/// The result of a phase transition attempt.
#[derive(Debug, Clone)]
pub struct PhaseTransitionResult {
    /// Whether the turn number changed.
    pub turn_changed: bool,
    /// The phase index before the transition.
    pub from_phase: usize,
    /// The phase index after the transition.
    pub to_phase: usize,
    /// The turn number after the transition.
    pub turn_number: u32,
}

// ---------------------------------------------------------------------------
// Combat Results Table
// ---------------------------------------------------------------------------

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

/// The Combat Results Table: domain-specific wrapper around a `ResolutionTable`.
///
/// The `table` field holds the generic column/row structure for lookup.
/// The `outcomes` field holds combat-specific results (a parallel grid).
/// Resolution uses generic `find_table_column` + `find_table_row` to get
/// indices, then looks up `outcomes[row][col]` for the combat result.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct CombatResultsTable {
    pub id: TypeId,
    pub name: String,
    /// Generic table structure providing columns and rows for lookup.
    pub table: ResolutionTable,
    /// Combat outcome grid indexed as `outcomes[row_index][column_index]`.
    pub outcomes: Vec<Vec<CombatOutcome>>,
    /// Reference to the Combat concept in the ontology.
    pub combat_concept_id: Option<TypeId>,
}

impl Default for CombatResultsTable {
    fn default() -> Self {
        Self {
            id: TypeId::new(),
            name: "Combat Results Table".to_string(),
            table: ResolutionTable {
                id: TypeId::new(),
                name: "CRT Lookup".to_string(),
                columns: Vec::new(),
                rows: Vec::new(),
                outcomes: Vec::new(),
            },
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

/// Resolves a complete CRT lookup by delegating to generic table functions.
///
/// Uses `find_table_column` for column lookup and `find_table_row` for row
/// lookup, then maps the intersection to a domain-specific `CombatOutcome`.
///
/// Returns `None` if the column or row cannot be resolved, or if the outcome
/// grid doesn't have the expected dimensions.
#[must_use]
pub fn resolve_crt(
    crt: &CombatResultsTable,
    attacker_strength: f64,
    defender_strength: f64,
    die_roll: u32,
) -> Option<CrtResolution> {
    let col_idx = find_table_column(attacker_strength, defender_strength, &crt.table.columns)?;
    let row_idx = find_table_row(die_roll, &crt.table.rows)?;

    let outcome = crt.outcomes.get(row_idx).and_then(|row| row.get(col_idx))?;

    Some(CrtResolution {
        column_index: col_idx,
        row_index: row_idx,
        column_label: crt.table.columns[col_idx].label.clone(),
        row_label: crt.table.rows[row_idx].label.clone(),
        outcome: outcome.clone(),
    })
}

// ---------------------------------------------------------------------------
// Phase Sequencer Functions
// ---------------------------------------------------------------------------

/// Check whether a phase action is legal given the current turn state.
#[must_use]
pub fn is_phase_action_legal(
    action: PhaseAction,
    turn_state: &TurnState,
    turn_structure: &TurnStructure,
) -> bool {
    if !turn_state.is_active || turn_structure.phases.is_empty() {
        return false;
    }
    match action {
        PhaseAction::Advance | PhaseAction::Skip => true,
        PhaseAction::Rewind => {
            // Can rewind if not at the very start (turn 1, phase 0).
            turn_state.turn_number > 1 || turn_state.current_phase_index > 0
        }
    }
}

/// Execute a phase action, updating turn state and returning the transition result.
/// Returns `None` if the action is not legal.
pub fn execute_phase_action(
    action: PhaseAction,
    turn_state: &mut TurnState,
    turn_structure: &TurnStructure,
) -> Option<PhaseTransitionResult> {
    if !is_phase_action_legal(action, turn_state, turn_structure) {
        return None;
    }

    let from_phase = turn_state.current_phase_index;
    let phase_count = turn_structure.phases.len();
    let mut turn_changed = false;

    match action {
        PhaseAction::Advance | PhaseAction::Skip => {
            let next = turn_state.current_phase_index + 1;
            if next >= phase_count {
                turn_state.turn_number += 1;
                turn_state.current_phase_index = 0;
                turn_changed = true;
            } else {
                turn_state.current_phase_index = next;
            }
        }
        PhaseAction::Rewind => {
            if turn_state.current_phase_index > 0 {
                turn_state.current_phase_index -= 1;
            } else if turn_state.turn_number > 1 {
                turn_state.turn_number -= 1;
                turn_state.current_phase_index = phase_count - 1;
                turn_changed = true;
            }
        }
    }

    // Reset actions remaining for the new phase.
    turn_state.phase_actions_remaining = None;

    Some(PhaseTransitionResult {
        turn_changed,
        from_phase,
        to_phase: turn_state.current_phase_index,
        turn_number: turn_state.turn_number,
    })
}

/// Get the current phase from the turn structure, if valid.
#[must_use]
pub fn current_phase<'a>(
    turn_state: &TurnState,
    turn_structure: &'a TurnStructure,
) -> Option<&'a Phase> {
    turn_structure.phases.get(turn_state.current_phase_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{ColumnType, TableColumn, TableRow};

    fn test_crt() -> CombatResultsTable {
        CombatResultsTable {
            id: TypeId::new(),
            name: "Test CRT".to_string(),
            table: ResolutionTable {
                id: TypeId::new(),
                name: "Test CRT Lookup".to_string(),
                columns: vec![
                    TableColumn {
                        label: "1:2".to_string(),
                        column_type: ColumnType::Ratio,
                        threshold: 0.5,
                    },
                    TableColumn {
                        label: "1:1".to_string(),
                        column_type: ColumnType::Ratio,
                        threshold: 1.0,
                    },
                    TableColumn {
                        label: "2:1".to_string(),
                        column_type: ColumnType::Ratio,
                        threshold: 2.0,
                    },
                ],
                rows: vec![
                    TableRow {
                        label: "1-2".to_string(),
                        value_min: 1,
                        value_max: 2,
                    },
                    TableRow {
                        label: "3-4".to_string(),
                        value_min: 3,
                        value_max: 4,
                    },
                    TableRow {
                        label: "5-6".to_string(),
                        value_min: 5,
                        value_max: 6,
                    },
                ],
                outcomes: Vec::new(),
            },
            outcomes: vec![
                vec![
                    CombatOutcome {
                        label: "AE".to_string(),
                        effect: Some(OutcomeEffect::AttackerEliminated),
                    },
                    CombatOutcome {
                        label: "NE".to_string(),
                        effect: Some(OutcomeEffect::NoEffect),
                    },
                    CombatOutcome {
                        label: "DR".to_string(),
                        effect: Some(OutcomeEffect::Retreat { hexes: 1 }),
                    },
                ],
                vec![
                    CombatOutcome {
                        label: "NE".to_string(),
                        effect: None,
                    },
                    CombatOutcome {
                        label: "DR".to_string(),
                        effect: Some(OutcomeEffect::Retreat { hexes: 2 }),
                    },
                    CombatOutcome {
                        label: "DE".to_string(),
                        effect: Some(OutcomeEffect::DefenderEliminated),
                    },
                ],
                vec![
                    CombatOutcome {
                        label: "EX".to_string(),
                        effect: Some(OutcomeEffect::Exchange {
                            attacker_steps: 1,
                            defender_steps: 1,
                        }),
                    },
                    CombatOutcome {
                        label: "SL".to_string(),
                        effect: Some(OutcomeEffect::StepLoss { steps: 1 }),
                    },
                    CombatOutcome {
                        label: "ASL".to_string(),
                        effect: Some(OutcomeEffect::AttackerStepLoss { steps: 1 }),
                    },
                ],
            ],
            combat_concept_id: None,
        }
    }

    #[test]
    fn resolve_crt_full() {
        let crt = test_crt();
        // 6 vs 2 = 3:1 → column 2; die roll 3 → row 1 → "DE"
        let result = resolve_crt(&crt, 6.0, 2.0, 3);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.column_index, 2);
        assert_eq!(r.row_index, 1);
        assert_eq!(r.outcome.label, "DE");
    }

    #[test]
    fn resolve_crt_no_column() {
        let crt = test_crt();
        let result = resolve_crt(&crt, 1.0, 10.0, 3);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_crt_no_row() {
        let crt = test_crt();
        let result = resolve_crt(&crt, 6.0, 2.0, 99);
        assert!(result.is_none());
    }

    #[test]
    fn turn_structure_default() {
        let ts = TurnStructure::default();
        assert!(ts.phases.is_empty());
        assert_eq!(ts.player_order, PlayerOrder::Alternating);
    }

    #[test]
    fn combat_results_table_default() {
        let crt = CombatResultsTable::default();
        assert_eq!(crt.name, "Combat Results Table");
        assert!(crt.table.columns.is_empty());
        assert!(crt.table.rows.is_empty());
        assert!(crt.outcomes.is_empty());
    }

    #[test]
    fn modifier_source_variants() {
        let sources = [
            ModifierSource::DefenderTerrain,
            ModifierSource::AttackerTerrain,
            ModifierSource::AttackerProperty("str".to_string()),
            ModifierSource::DefenderProperty("def".to_string()),
            ModifierSource::Custom("rule".to_string()),
        ];
        for s in &sources {
            assert!(!format!("{s:?}").is_empty());
        }
    }

    #[test]
    fn outcome_effect_all_variants_debug() {
        let effects = [
            OutcomeEffect::NoEffect,
            OutcomeEffect::Retreat { hexes: 2 },
            OutcomeEffect::StepLoss { steps: 1 },
            OutcomeEffect::AttackerStepLoss { steps: 1 },
            OutcomeEffect::Exchange {
                attacker_steps: 1,
                defender_steps: 2,
            },
            OutcomeEffect::AttackerEliminated,
            OutcomeEffect::DefenderEliminated,
        ];
        for e in &effects {
            assert!(!format!("{e:?}").is_empty());
        }
    }

    #[test]
    fn crt_ron_round_trip() {
        let crt = test_crt();
        let ron_str = ron::to_string(&crt).expect("serialize");
        let deserialized: CombatResultsTable = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.table.columns.len(), 3);
        assert_eq!(deserialized.table.rows.len(), 3);
        assert_eq!(deserialized.outcomes.len(), 3);
    }

    // --- Phase sequencer tests ---

    fn three_phase_structure() -> TurnStructure {
        TurnStructure {
            phases: vec![
                Phase {
                    id: TypeId::new(),
                    name: "Movement".to_string(),
                    phase_type: PhaseType::Movement,
                    description: String::new(),
                },
                Phase {
                    id: TypeId::new(),
                    name: "Combat".to_string(),
                    phase_type: PhaseType::Combat,
                    description: String::new(),
                },
                Phase {
                    id: TypeId::new(),
                    name: "Supply".to_string(),
                    phase_type: PhaseType::Admin,
                    description: String::new(),
                },
            ],
            player_order: PlayerOrder::Alternating,
        }
    }

    #[test]
    fn advance_phase_within_turn() {
        let structure = three_phase_structure();
        let mut state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        let result = execute_phase_action(PhaseAction::Advance, &mut state, &structure);
        assert!(result.is_some());
        let r = result.expect("result");
        assert!(!r.turn_changed);
        assert_eq!(r.from_phase, 0);
        assert_eq!(r.to_phase, 1);
        assert_eq!(state.current_phase_index, 1);
        assert_eq!(state.turn_number, 1);
    }

    #[test]
    fn advance_past_last_phase_wraps_turn() {
        let structure = three_phase_structure();
        let mut state = TurnState {
            turn_number: 1,
            current_phase_index: 2,
            is_active: true,
            phase_actions_remaining: None,
        };

        let result = execute_phase_action(PhaseAction::Advance, &mut state, &structure);
        let r = result.expect("result");
        assert!(r.turn_changed);
        assert_eq!(r.to_phase, 0);
        assert_eq!(state.turn_number, 2);
    }

    #[test]
    fn rewind_phase_within_turn() {
        let structure = three_phase_structure();
        let mut state = TurnState {
            turn_number: 1,
            current_phase_index: 2,
            is_active: true,
            phase_actions_remaining: None,
        };

        let result = execute_phase_action(PhaseAction::Rewind, &mut state, &structure);
        let r = result.expect("result");
        assert!(!r.turn_changed);
        assert_eq!(r.to_phase, 1);
    }

    #[test]
    fn rewind_past_first_phase_wraps_turn() {
        let structure = three_phase_structure();
        let mut state = TurnState {
            turn_number: 2,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        let result = execute_phase_action(PhaseAction::Rewind, &mut state, &structure);
        let r = result.expect("result");
        assert!(r.turn_changed);
        assert_eq!(r.to_phase, 2);
        assert_eq!(state.turn_number, 1);
    }

    #[test]
    fn rewind_at_start_of_game_illegal() {
        let structure = three_phase_structure();
        let state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        assert!(!is_phase_action_legal(
            PhaseAction::Rewind,
            &state,
            &structure
        ));
    }

    #[test]
    fn actions_illegal_when_inactive() {
        let structure = three_phase_structure();
        let state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: false,
            phase_actions_remaining: None,
        };

        assert!(!is_phase_action_legal(
            PhaseAction::Advance,
            &state,
            &structure
        ));
        assert!(!is_phase_action_legal(
            PhaseAction::Rewind,
            &state,
            &structure
        ));
        assert!(!is_phase_action_legal(
            PhaseAction::Skip,
            &state,
            &structure
        ));
    }

    #[test]
    fn actions_illegal_with_no_phases() {
        let structure = TurnStructure::default();
        let state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        assert!(!is_phase_action_legal(
            PhaseAction::Advance,
            &state,
            &structure
        ));
    }

    #[test]
    fn skip_behaves_like_advance() {
        let structure = three_phase_structure();
        let mut state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        let result = execute_phase_action(PhaseAction::Skip, &mut state, &structure);
        let r = result.expect("result");
        assert_eq!(r.to_phase, 1);
        assert!(!r.turn_changed);
    }

    #[test]
    fn current_phase_returns_correct_phase() {
        let structure = three_phase_structure();
        let state = TurnState {
            turn_number: 1,
            current_phase_index: 1,
            is_active: true,
            phase_actions_remaining: None,
        };

        let phase = current_phase(&state, &structure);
        assert!(phase.is_some());
        assert_eq!(phase.expect("phase").name, "Combat");
        assert_eq!(phase.expect("phase").phase_type, PhaseType::Combat);
    }

    #[test]
    fn current_phase_out_of_bounds_returns_none() {
        let structure = three_phase_structure();
        let state = TurnState {
            turn_number: 1,
            current_phase_index: 99,
            is_active: true,
            phase_actions_remaining: None,
        };

        assert!(current_phase(&state, &structure).is_none());
    }
}
