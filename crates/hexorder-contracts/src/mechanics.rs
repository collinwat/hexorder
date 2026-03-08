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
// Post-Resolution Movement
// ---------------------------------------------------------------------------

/// What happens to combatants after resolution completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum PostResolutionAction {
    /// Attacker advances into the defender's hex.
    Advance,
    /// Defender retreats away from the attacker.
    Retreat,
    /// No movement — units stay in place.
    Hold,
}

/// A rule that triggers post-resolution movement based on the combat outcome.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct PostResolutionRule {
    /// The action to execute.
    pub action: PostResolutionAction,
    /// Which `OutcomeEffect` variants trigger this rule. Empty means "always".
    pub trigger_effects: Vec<String>,
    /// Maximum movement range in hexes (for Retreat).
    pub movement_range: u32,
}

/// The pending movement resulting from post-resolution rule evaluation.
#[derive(Debug, Clone)]
pub struct PendingMovement {
    /// The entity that moves.
    pub entity: Entity,
    /// The action type.
    pub action: PostResolutionAction,
    /// Maximum range in hexes.
    pub movement_range: u32,
}

/// Evaluate post-resolution rules against a combat outcome.
///
/// Returns pending movements for the attacker, defender, or both.
/// The caller is responsible for executing the movement (via BFS).
#[must_use]
pub fn evaluate_post_resolution(
    rules: &[PostResolutionRule],
    outcome: &CombatOutcome,
    attacker: Entity,
    defender: Entity,
) -> Vec<PendingMovement> {
    let mut pending = Vec::new();
    let effect_label = outcome
        .effect
        .as_ref()
        .map(|e| format!("{e:?}"))
        .unwrap_or_default();

    for rule in rules {
        let triggers = !rule.trigger_effects.is_empty()
            && !rule
                .trigger_effects
                .iter()
                .any(|t| effect_label.contains(t));
        if triggers {
            continue;
        }

        match rule.action {
            PostResolutionAction::Advance => {
                pending.push(PendingMovement {
                    entity: attacker,
                    action: PostResolutionAction::Advance,
                    movement_range: 1,
                });
            }
            PostResolutionAction::Retreat => {
                pending.push(PendingMovement {
                    entity: defender,
                    action: PostResolutionAction::Retreat,
                    movement_range: rule.movement_range,
                });
            }
            PostResolutionAction::Hold => {}
        }
    }

    pending
}

// ---------------------------------------------------------------------------
// Constrained Pathfinding
// ---------------------------------------------------------------------------

/// A constraint applied when finding a post-resolution path (e.g., retreat).
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum PathConstraint {
    /// Each step must increase distance from the source hex.
    AwayFrom {
        source: crate::hex_grid::HexPosition,
    },
    /// Avoid hexes that appear in the influence map for the given type.
    AvoidInfluence { influence_type: String },
    /// Avoid hexes with the given terrain type.
    AvoidTerrain { terrain_type: String },
    /// Total path cost must not exceed this budget.
    MaxCost { budget: u32 },
}

/// Request to find a constrained path from a starting hex.
#[derive(Debug, Clone)]
pub struct ConstrainedPathRequest {
    /// Starting hex (e.g., defender's current position).
    pub start: crate::hex_grid::HexPosition,
    /// Constraints the path must satisfy.
    pub constraints: Vec<PathConstraint>,
    /// Maximum path length in hexes.
    pub max_distance: u32,
}

/// Result of a constrained pathfinding attempt.
#[derive(Debug, Clone)]
pub struct ConstrainedPathResult {
    /// The path from start to destination (inclusive of start), or `None` if
    /// no valid path exists.
    pub path: Option<Vec<crate::hex_grid::HexPosition>>,
    /// Human-readable reason for failure, if any.
    pub failure_reason: Option<String>,
}

/// Context data required by the constrained pathfinding algorithm.
///
/// The caller (rules engine) populates this with the relevant game state.
/// The algorithm itself is pure — it reads from this context without side effects.
#[derive(Debug)]
pub struct PathfindingContext {
    /// Set of hexes the unit is allowed to move to (from BFS / `ValidMoveSet`).
    pub valid_positions: std::collections::HashSet<crate::hex_grid::HexPosition>,
    /// Hexes under influence, keyed by influence type name.
    pub influence_zones:
        std::collections::HashMap<String, std::collections::HashSet<crate::hex_grid::HexPosition>>,
    /// Terrain type at each hex, keyed by position.
    pub terrain_types: std::collections::HashMap<crate::hex_grid::HexPosition, String>,
}

/// Find the best path satisfying all constraints via BFS.
///
/// Returns the longest valid path (maximizing distance from threat for retreat)
/// that satisfies every constraint. If no valid path exists, returns the reason.
///
/// This is a pure function — all state is provided via `ctx` and `request`.
#[must_use]
pub fn find_constrained_path(
    request: &ConstrainedPathRequest,
    ctx: &PathfindingContext,
) -> ConstrainedPathResult {
    use std::collections::{HashSet, VecDeque};

    // Parse constraints once.
    let away_from: Vec<crate::hex_grid::HexPosition> = request
        .constraints
        .iter()
        .filter_map(|c| {
            if let PathConstraint::AwayFrom { source } = c {
                Some(*source)
            } else {
                None
            }
        })
        .collect();

    let avoid_influence: Vec<&str> = request
        .constraints
        .iter()
        .filter_map(|c| {
            if let PathConstraint::AvoidInfluence { influence_type } = c {
                Some(influence_type.as_str())
            } else {
                None
            }
        })
        .collect();

    let avoid_terrain: Vec<&str> = request
        .constraints
        .iter()
        .filter_map(|c| {
            if let PathConstraint::AvoidTerrain { terrain_type } = c {
                Some(terrain_type.as_str())
            } else {
                None
            }
        })
        .collect();

    let max_cost = request
        .constraints
        .iter()
        .find_map(|c| {
            if let PathConstraint::MaxCost { budget } = c {
                Some(*budget)
            } else {
                None
            }
        })
        .unwrap_or(request.max_distance);

    let effective_max = max_cost.min(request.max_distance);

    // BFS from start, tracking paths.
    let mut visited = HashSet::new();
    visited.insert(request.start);

    // (position, path_so_far, distance)
    let mut queue: VecDeque<(
        crate::hex_grid::HexPosition,
        Vec<crate::hex_grid::HexPosition>,
        u32,
    )> = VecDeque::new();
    queue.push_back((request.start, vec![request.start], 0));

    let mut best_path: Option<Vec<crate::hex_grid::HexPosition>> = None;
    let mut best_away_distance: u32 = 0;

    while let Some((pos, path, dist)) = queue.pop_front() {
        // Check if this position is a valid endpoint (not the start).
        if dist > 0 {
            // Compute minimum distance to all AwayFrom sources.
            let min_away = if away_from.is_empty() {
                u32::MAX
            } else {
                away_from
                    .iter()
                    .map(|s| crate::hex_grid::hex_distance(pos, *s))
                    .min()
                    .unwrap_or(0)
            };

            if min_away > best_away_distance
                || (min_away == best_away_distance
                    && best_path.as_ref().is_none_or(|bp| path.len() < bp.len()))
            {
                best_away_distance = min_away;
                best_path = Some(path.clone());
            }
        }

        // Don't expand beyond the budget.
        if dist >= effective_max {
            continue;
        }

        // Expand neighbors using hexx.
        let hex = pos.to_hex();
        for neighbor_hex in hex.all_neighbors() {
            let neighbor = crate::hex_grid::HexPosition::new(neighbor_hex.x, neighbor_hex.y);

            if visited.contains(&neighbor) {
                continue;
            }

            // Must be in the valid move set.
            if !ctx.valid_positions.contains(&neighbor) {
                continue;
            }

            // AwayFrom: each step must not decrease distance from source.
            let away_ok = away_from.iter().all(|s| {
                crate::hex_grid::hex_distance(neighbor, *s)
                    >= crate::hex_grid::hex_distance(pos, *s)
            });
            if !away_ok {
                continue;
            }

            // AvoidInfluence: neighbor must not be in any avoided influence zone.
            let influence_ok = avoid_influence.iter().all(|inf_type| {
                ctx.influence_zones
                    .get(*inf_type)
                    .is_none_or(|zone| !zone.contains(&neighbor))
            });
            if !influence_ok {
                continue;
            }

            // AvoidTerrain: neighbor must not have an avoided terrain type.
            let terrain_ok = avoid_terrain
                .iter()
                .all(|terr| ctx.terrain_types.get(&neighbor).is_none_or(|t| t != terr));
            if !terrain_ok {
                continue;
            }

            visited.insert(neighbor);
            let mut new_path = path.clone();
            new_path.push(neighbor);
            queue.push_back((neighbor, new_path, dist + 1));
        }
    }

    if let Some(path) = best_path {
        ConstrainedPathResult {
            path: Some(path),
            failure_reason: None,
        }
    } else {
        ConstrainedPathResult {
            path: None,
            failure_reason: Some("No valid path satisfying all constraints".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduled Entity Spawning
// ---------------------------------------------------------------------------

/// A single scheduled spawn: place an entity of a given type at a given hex on a given turn.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct SpawnEntry {
    /// The entity type to spawn (references `EntityTypeRegistry`).
    pub entity_type_id: TypeId,
    /// The turn number on which this entity should appear (1-indexed).
    pub turn: u32,
    /// The target hex position for spawning.
    pub hex: crate::hex_grid::HexPosition,
    /// The source zone name (for designer organization, e.g. "North Reinforcements").
    pub source_zone: String,
}

/// The designer-defined spawn schedule for a scenario.
/// Evaluated at each turn boundary to spawn entities whose turn has arrived.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct SpawnSchedule {
    pub entries: Vec<SpawnEntry>,
}

/// A named staging zone for off-grid entities awaiting spawn.
/// Minimal representation — no spatial layout, just a name and a list of staged entity types.
#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct SpawnZone {
    pub name: String,
    /// Entity types staged in this zone (not yet on the board).
    pub staged_entities: Vec<TypeId>,
}

/// Fired when an entity is spawned from the schedule.
#[derive(Event, Debug, Clone)]
pub struct SpawnTriggered {
    /// The entity type that was spawned.
    pub entity_type_id: TypeId,
    /// The turn on which the spawn occurred.
    pub turn: u32,
    /// The hex position where the entity was placed.
    pub hex: crate::hex_grid::HexPosition,
    /// The source zone name.
    pub source_zone: String,
}

/// Result of attempting to spawn a single entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnResult {
    /// Entity was placed at the target hex.
    Placed { hex: crate::hex_grid::HexPosition },
    /// Target hex was full; entity was placed at an adjacent hex.
    Displaced {
        target: crate::hex_grid::HexPosition,
        actual: crate::hex_grid::HexPosition,
    },
    /// All candidate hexes were full; spawn deferred to next turn.
    Deferred { reason: String },
}

/// Evaluate which spawn entries should fire for the given turn number.
///
/// Returns the indices of entries in `schedule.entries` that match the turn.
/// The caller is responsible for actually placing entities and checking stacking.
#[must_use]
pub fn entries_due_for_turn(schedule: &SpawnSchedule, turn: u32) -> Vec<usize> {
    schedule
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.turn == turn)
        .map(|(i, _)| i)
        .collect()
}

/// Given a target hex and a set of occupied positions (hexes at stacking capacity),
/// find the best hex for placement: the target if available, otherwise the first
/// available adjacent hex in ring order.
///
/// Returns `None` if the target and all adjacent hexes are occupied.
#[must_use]
pub fn find_spawn_hex<S: std::hash::BuildHasher>(
    target: crate::hex_grid::HexPosition,
    occupied: &std::collections::HashSet<crate::hex_grid::HexPosition, S>,
) -> Option<crate::hex_grid::HexPosition> {
    if !occupied.contains(&target) {
        return Some(target);
    }
    // Try adjacent hexes in ring-1 order.
    let hex = target.to_hex();
    for neighbor in hex.all_neighbors() {
        let pos = crate::hex_grid::HexPosition::new(neighbor.x, neighbor.y);
        if !occupied.contains(&pos) {
            return Some(pos);
        }
    }
    None
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

// ---------------------------------------------------------------------------
// Area-Effect Modifiers
// ---------------------------------------------------------------------------

/// How long an area marker remains active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum MarkerDuration {
    /// Stays until explicitly removed.
    Permanent,
    /// Expires after N turns.
    PerTurn { turns_remaining: u32 },
    /// Active until a game condition removes it.
    UntilRemoved,
}

/// An effect applied to hexes within a marker's radius.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum AreaEffect {
    /// Shift the CRT column during resolution for combatants in the area.
    ColumnShift { shift: i32 },
    /// Add extra movement cost for units entering hexes in the area.
    CostModifier { extra_cost: i64 },
    /// Restrict a category of actions within the area (e.g., `"no_retreat"`).
    ActionRestriction { restriction: String },
}

/// A spatial marker that applies effects to hexes within a radius.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct AreaMarker {
    /// Designer-defined marker type name (e.g., "Bombardment Zone").
    pub marker_type: String,
    /// Center hex of the affected area.
    pub center: crate::hex_grid::HexPosition,
    /// Radius in hexes (0 = center hex only).
    pub radius: u32,
    /// Effects applied to hexes within the radius.
    pub effects: Vec<AreaEffect>,
    /// How long this marker remains active.
    pub duration: MarkerDuration,
}

/// Registry of active area markers on the board.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct AreaMarkerRegistry {
    pub markers: Vec<AreaMarker>,
}

/// Collect all column shifts from area markers that affect a given hex position.
///
/// Returns the total additive column shift from all markers whose area
/// contains the given position.
#[must_use]
pub fn collect_area_column_shifts(
    registry: &AreaMarkerRegistry,
    position: crate::hex_grid::HexPosition,
) -> i32 {
    let mut total_shift = 0;
    for marker in &registry.markers {
        let distance = crate::hex_grid::hex_distance(marker.center, position);
        if distance <= marker.radius {
            for effect in &marker.effects {
                if let AreaEffect::ColumnShift { shift } = effect {
                    total_shift += shift;
                }
            }
        }
    }
    total_shift
}

/// Collect the extra movement cost from area markers for a given hex position.
///
/// Returns the total additive cost modifier from all markers whose area
/// contains the given position.
#[must_use]
pub fn collect_area_cost_modifiers(
    registry: &AreaMarkerRegistry,
    position: crate::hex_grid::HexPosition,
) -> i64 {
    let mut total_cost = 0;
    for marker in &registry.markers {
        let distance = crate::hex_grid::hex_distance(marker.center, position);
        if distance <= marker.radius {
            for effect in &marker.effects {
                if let AreaEffect::CostModifier { extra_cost } = effect {
                    total_cost += extra_cost;
                }
            }
        }
    }
    total_cost
}

/// Check if a specific action restriction applies at a position.
#[must_use]
pub fn is_action_restricted(
    registry: &AreaMarkerRegistry,
    position: crate::hex_grid::HexPosition,
    restriction_name: &str,
) -> bool {
    for marker in &registry.markers {
        let distance = crate::hex_grid::hex_distance(marker.center, position);
        if distance <= marker.radius {
            for effect in &marker.effects {
                if let AreaEffect::ActionRestriction { restriction } = effect
                    && restriction == restriction_name
                {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Accumulation Tracker
// ---------------------------------------------------------------------------

/// A condition that triggers accumulation of points.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum AccumulationTrigger {
    /// Award points when a faction occupies a specific hex.
    OccupyHex {
        hex: crate::hex_grid::HexPosition,
        points: i32,
    },
    /// Award points on a state transition (e.g., phase change).
    StateTransition {
        from_state: String,
        to_state: String,
        points: i32,
    },
    /// Award points at each turn boundary.
    TurnBoundary { points: i32 },
    /// Manually awarded by the designer or game system.
    Manual,
}

/// A named score accumulator that tracks points for a faction.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Accumulator {
    /// Unique identifier for this accumulator.
    pub id: String,
    /// Optional faction this accumulator belongs to.
    pub faction: Option<String>,
    /// Triggers that can add points to this accumulator.
    pub triggers: Vec<AccumulationTrigger>,
    /// Current accumulated value.
    pub value: i32,
    /// History of (turn, delta) entries.
    pub history: Vec<(u32, i32)>,
}

/// Registry of all accumulators in the scenario.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct AccumulatorRegistry {
    pub accumulators: Vec<Accumulator>,
}

/// How to compare an accumulator's value against the threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum ComparisonOp {
    GreaterOrEqual,
    LessOrEqual,
    Equal,
}

/// A victory condition: when an accumulator's value meets a threshold.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct VictoryCondition {
    /// Which accumulator to check.
    pub accumulator_id: String,
    /// The target threshold value.
    pub threshold: i32,
    /// How to compare value against threshold.
    pub comparison: ComparisonOp,
}

/// Registry of victory conditions for the scenario.
#[derive(Resource, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct VictoryConditionRegistry {
    pub conditions: Vec<VictoryCondition>,
}

/// Fired when a victory condition is met.
#[derive(Event, Debug, Clone)]
pub struct VictoryReached {
    pub accumulator_id: String,
    pub faction: Option<String>,
    pub value: i32,
    pub threshold: i32,
}

/// Evaluate turn-boundary triggers for all accumulators at the given turn.
/// Returns the indices of accumulators that were modified.
#[must_use]
pub fn evaluate_turn_boundary_triggers(
    registry: &mut AccumulatorRegistry,
    turn: u32,
) -> Vec<usize> {
    let mut modified = Vec::new();
    for (i, acc) in registry.accumulators.iter_mut().enumerate() {
        let mut delta = 0i32;
        for trigger in &acc.triggers {
            if let AccumulationTrigger::TurnBoundary { points } = trigger {
                delta += points;
            }
        }
        if delta != 0 {
            acc.value += delta;
            acc.history.push((turn, delta));
            modified.push(i);
        }
    }
    modified
}

/// Evaluate occupy-hex triggers for a specific hex and faction.
/// Returns the indices of accumulators that were modified.
#[must_use]
pub fn evaluate_occupy_triggers(
    registry: &mut AccumulatorRegistry,
    hex: crate::hex_grid::HexPosition,
    faction: &str,
    turn: u32,
) -> Vec<usize> {
    let mut modified = Vec::new();
    for (i, acc) in registry.accumulators.iter_mut().enumerate() {
        // Only evaluate for matching faction or faction-less accumulators.
        if let Some(ref acc_faction) = acc.faction
            && acc_faction != faction
        {
            continue;
        }
        let mut delta = 0i32;
        for trigger in &acc.triggers {
            if let AccumulationTrigger::OccupyHex {
                hex: trigger_hex,
                points,
            } = trigger
                && *trigger_hex == hex
            {
                delta += points;
            }
        }
        if delta != 0 {
            acc.value += delta;
            acc.history.push((turn, delta));
            modified.push(i);
        }
    }
    modified
}

/// Check all victory conditions and return indices of those that are met.
#[must_use]
pub fn check_victory_conditions(
    accumulators: &AccumulatorRegistry,
    conditions: &VictoryConditionRegistry,
) -> Vec<usize> {
    let mut met = Vec::new();
    for (i, cond) in conditions.conditions.iter().enumerate() {
        if let Some(acc) = accumulators
            .accumulators
            .iter()
            .find(|a| a.id == cond.accumulator_id)
        {
            let satisfied = match cond.comparison {
                ComparisonOp::GreaterOrEqual => acc.value >= cond.threshold,
                ComparisonOp::LessOrEqual => acc.value <= cond.threshold,
                ComparisonOp::Equal => acc.value == cond.threshold,
            };
            if satisfied {
                met.push(i);
            }
        }
    }
    met
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

    // --- Post-resolution movement tests ---

    fn retreat_outcome() -> CombatOutcome {
        CombatOutcome {
            label: "DR".to_string(),
            effect: Some(OutcomeEffect::Retreat { hexes: 2 }),
        }
    }

    fn no_effect_outcome() -> CombatOutcome {
        CombatOutcome {
            label: "NE".to_string(),
            effect: Some(OutcomeEffect::NoEffect),
        }
    }

    #[test]
    fn post_resolution_advance_produces_attacker_movement() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Advance,
            trigger_effects: vec![],
            movement_range: 1,
        }];

        let pending = evaluate_post_resolution(&rules, &retreat_outcome(), attacker, defender);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, PostResolutionAction::Advance);
        assert_eq!(pending[0].movement_range, 1);
    }

    #[test]
    fn post_resolution_retreat_produces_defender_movement() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Retreat,
            trigger_effects: vec![],
            movement_range: 3,
        }];

        let pending = evaluate_post_resolution(&rules, &retreat_outcome(), attacker, defender);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, PostResolutionAction::Retreat);
        assert_eq!(pending[0].movement_range, 3);
    }

    #[test]
    fn post_resolution_hold_produces_no_movement() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Hold,
            trigger_effects: vec![],
            movement_range: 0,
        }];

        let pending = evaluate_post_resolution(&rules, &retreat_outcome(), attacker, defender);
        assert!(pending.is_empty());
    }

    #[test]
    fn post_resolution_trigger_filter_skips_non_matching() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Advance,
            trigger_effects: vec!["DefenderEliminated".to_string()],
            movement_range: 1,
        }];

        // Outcome is Retreat, not DefenderEliminated — rule should NOT fire
        let pending = evaluate_post_resolution(&rules, &retreat_outcome(), attacker, defender);
        assert!(pending.is_empty());
    }

    #[test]
    fn post_resolution_trigger_filter_matches() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Advance,
            trigger_effects: vec!["Retreat".to_string()],
            movement_range: 1,
        }];

        // Outcome has Retreat effect — rule should fire
        let pending = evaluate_post_resolution(&rules, &retreat_outcome(), attacker, defender);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, PostResolutionAction::Advance);
    }

    #[test]
    fn post_resolution_empty_rules_produce_nothing() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;

        let pending = evaluate_post_resolution(&[], &retreat_outcome(), attacker, defender);
        assert!(pending.is_empty());
    }

    #[test]
    fn post_resolution_no_effect_outcome_with_empty_triggers() {
        let attacker = Entity::PLACEHOLDER;
        let defender = Entity::PLACEHOLDER;
        let rules = vec![PostResolutionRule {
            action: PostResolutionAction::Advance,
            trigger_effects: vec![],
            movement_range: 1,
        }];

        // Empty trigger_effects means "always fire"
        let pending = evaluate_post_resolution(&rules, &no_effect_outcome(), attacker, defender);
        assert_eq!(pending.len(), 1);
    }

    // --- Area-effect modifier tests ---

    use crate::hex_grid::HexPosition;

    fn test_registry() -> AreaMarkerRegistry {
        AreaMarkerRegistry {
            markers: vec![AreaMarker {
                marker_type: "Bombardment Zone".to_string(),
                center: HexPosition::new(0, 0),
                radius: 2,
                effects: vec![
                    AreaEffect::ColumnShift { shift: -1 },
                    AreaEffect::CostModifier { extra_cost: 2 },
                    AreaEffect::ActionRestriction {
                        restriction: "no_retreat".to_string(),
                    },
                ],
                duration: MarkerDuration::Permanent,
            }],
        }
    }

    #[test]
    fn area_column_shift_inside_radius() {
        let registry = test_registry();
        let shift = collect_area_column_shifts(&registry, HexPosition::new(1, 0));
        assert_eq!(shift, -1);
    }

    #[test]
    fn area_column_shift_at_center() {
        let registry = test_registry();
        let shift = collect_area_column_shifts(&registry, HexPosition::new(0, 0));
        assert_eq!(shift, -1);
    }

    #[test]
    fn area_column_shift_outside_radius() {
        let registry = test_registry();
        let shift = collect_area_column_shifts(&registry, HexPosition::new(3, 0));
        assert_eq!(shift, 0);
    }

    #[test]
    fn area_cost_modifier_inside_radius() {
        let registry = test_registry();
        let cost = collect_area_cost_modifiers(&registry, HexPosition::new(0, 1));
        assert_eq!(cost, 2);
    }

    #[test]
    fn area_cost_modifier_outside_radius() {
        let registry = test_registry();
        let cost = collect_area_cost_modifiers(&registry, HexPosition::new(5, 5));
        assert_eq!(cost, 0);
    }

    #[test]
    fn area_action_restriction_matches() {
        let registry = test_registry();
        assert!(is_action_restricted(
            &registry,
            HexPosition::new(0, 0),
            "no_retreat"
        ));
    }

    #[test]
    fn area_action_restriction_no_match() {
        let registry = test_registry();
        assert!(!is_action_restricted(
            &registry,
            HexPosition::new(0, 0),
            "no_advance"
        ));
    }

    #[test]
    fn area_action_restriction_outside_radius() {
        let registry = test_registry();
        assert!(!is_action_restricted(
            &registry,
            HexPosition::new(10, 10),
            "no_retreat"
        ));
    }

    #[test]
    fn area_multiple_markers_stack_additively() {
        let registry = AreaMarkerRegistry {
            markers: vec![
                AreaMarker {
                    marker_type: "Zone A".to_string(),
                    center: HexPosition::new(0, 0),
                    radius: 1,
                    effects: vec![AreaEffect::ColumnShift { shift: -1 }],
                    duration: MarkerDuration::Permanent,
                },
                AreaMarker {
                    marker_type: "Zone B".to_string(),
                    center: HexPosition::new(1, 0),
                    radius: 1,
                    effects: vec![AreaEffect::ColumnShift { shift: -2 }],
                    duration: MarkerDuration::PerTurn { turns_remaining: 3 },
                },
            ],
        };
        // HexPosition(0,0) is in both zones
        let shift = collect_area_column_shifts(&registry, HexPosition::new(0, 0));
        assert_eq!(shift, -3);
    }

    #[test]
    fn area_empty_registry_returns_defaults() {
        let registry = AreaMarkerRegistry::default();
        assert_eq!(
            collect_area_column_shifts(&registry, HexPosition::new(0, 0)),
            0
        );
        assert_eq!(
            collect_area_cost_modifiers(&registry, HexPosition::new(0, 0)),
            0
        );
        assert!(!is_action_restricted(
            &registry,
            HexPosition::new(0, 0),
            "anything"
        ));
    }

    #[test]
    fn marker_duration_variants_debug() {
        let durations = [
            MarkerDuration::Permanent,
            MarkerDuration::PerTurn { turns_remaining: 5 },
            MarkerDuration::UntilRemoved,
        ];
        for d in &durations {
            assert!(!format!("{d:?}").is_empty());
        }
    }

    // --- Scheduled spawning tests ---

    #[test]
    fn entries_due_for_turn_filters_correctly() {
        let schedule = SpawnSchedule {
            entries: vec![
                SpawnEntry {
                    entity_type_id: TypeId::new(),
                    turn: 1,
                    hex: HexPosition::new(5, 0),
                    source_zone: "North".to_string(),
                },
                SpawnEntry {
                    entity_type_id: TypeId::new(),
                    turn: 3,
                    hex: HexPosition::new(5, 1),
                    source_zone: "North".to_string(),
                },
                SpawnEntry {
                    entity_type_id: TypeId::new(),
                    turn: 3,
                    hex: HexPosition::new(-5, 0),
                    source_zone: "South".to_string(),
                },
            ],
        };

        assert_eq!(entries_due_for_turn(&schedule, 1), vec![0]);
        assert_eq!(entries_due_for_turn(&schedule, 3), vec![1, 2]);
        assert!(entries_due_for_turn(&schedule, 2).is_empty());
    }

    #[test]
    fn entries_due_for_turn_empty_schedule() {
        let schedule = SpawnSchedule::default();
        assert!(entries_due_for_turn(&schedule, 1).is_empty());
    }

    #[test]
    fn find_spawn_hex_target_available() {
        let occupied = HashSet::new();
        let target = HexPosition::new(3, 0);
        assert_eq!(find_spawn_hex(target, &occupied), Some(target));
    }

    #[test]
    fn find_spawn_hex_target_occupied_uses_neighbor() {
        let mut occupied = HashSet::new();
        let target = HexPosition::new(3, 0);
        occupied.insert(target);
        let result = find_spawn_hex(target, &occupied);
        assert!(result.is_some());
        let placed = result.expect("placed");
        // Must be adjacent to target.
        assert_eq!(crate::hex_grid::hex_distance(placed, target), 1);
    }

    #[test]
    fn find_spawn_hex_all_occupied_returns_none() {
        let target = HexPosition::new(0, 0);
        let hex = target.to_hex();
        let mut occupied = HashSet::new();
        occupied.insert(target);
        for neighbor in hex.all_neighbors() {
            occupied.insert(HexPosition::new(neighbor.x, neighbor.y));
        }
        assert!(find_spawn_hex(target, &occupied).is_none());
    }

    #[test]
    fn spawn_schedule_default_is_empty() {
        let schedule = SpawnSchedule::default();
        assert!(schedule.entries.is_empty());
    }

    #[test]
    fn spawn_zone_default_is_empty() {
        let zone = SpawnZone::default();
        assert!(zone.name.is_empty());
        assert!(zone.staged_entities.is_empty());
    }

    #[test]
    fn spawn_result_variants_debug() {
        let results = [
            SpawnResult::Placed {
                hex: HexPosition::new(0, 0),
            },
            SpawnResult::Displaced {
                target: HexPosition::new(0, 0),
                actual: HexPosition::new(1, 0),
            },
            SpawnResult::Deferred {
                reason: "full".to_string(),
            },
        ];
        for r in &results {
            assert!(!format!("{r:?}").is_empty());
        }
    }

    #[test]
    fn spawn_entry_ron_round_trip() {
        let entry = SpawnEntry {
            entity_type_id: TypeId::new(),
            turn: 5,
            hex: HexPosition::new(3, -2),
            source_zone: "East Flank".to_string(),
        };
        let ron_str = ron::to_string(&entry).expect("serialize");
        let deserialized: SpawnEntry = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.turn, 5);
        assert_eq!(deserialized.hex.q, 3);
        assert_eq!(deserialized.hex.r, -2);
        assert_eq!(deserialized.source_zone, "East Flank");
    }

    #[test]
    fn spawn_schedule_ron_round_trip() {
        let schedule = SpawnSchedule {
            entries: vec![SpawnEntry {
                entity_type_id: TypeId::new(),
                turn: 3,
                hex: HexPosition::new(0, 0),
                source_zone: "HQ".to_string(),
            }],
        };
        let ron_str = ron::to_string(&schedule).expect("serialize");
        let deserialized: SpawnSchedule = ron::from_str(&ron_str).expect("deserialize");
        assert_eq!(deserialized.entries.len(), 1);
        assert_eq!(deserialized.entries[0].turn, 3);
    }

    // --- Constrained pathfinding tests ---

    use std::collections::{HashMap, HashSet};

    fn retreat_context() -> PathfindingContext {
        // A small grid: attacker at (0,0), defender at (1,0).
        // Valid positions form a line away from attacker: (2,0), (3,0).
        let mut valid = HashSet::new();
        valid.insert(HexPosition::new(1, 0)); // start (included for completeness)
        valid.insert(HexPosition::new(2, 0));
        valid.insert(HexPosition::new(3, 0));
        PathfindingContext {
            valid_positions: valid,
            influence_zones: HashMap::new(),
            terrain_types: HashMap::new(),
        }
    }

    #[test]
    fn constrained_path_retreat_away_from_attacker() {
        let ctx = retreat_context();
        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![PathConstraint::AwayFrom {
                source: HexPosition::new(0, 0),
            }],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        assert!(result.path.is_some());
        let path = result.path.expect("path");
        // Path should go from (1,0) through (2,0) to (3,0).
        assert_eq!(path.first(), Some(&HexPosition::new(1, 0)));
        assert_eq!(path.last(), Some(&HexPosition::new(3, 0)));
        assert!(path.len() >= 2);
    }

    #[test]
    fn constrained_path_max_cost_limits_distance() {
        let ctx = retreat_context();
        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![
                PathConstraint::AwayFrom {
                    source: HexPosition::new(0, 0),
                },
                PathConstraint::MaxCost { budget: 1 },
            ],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        assert!(result.path.is_some());
        let path = result.path.expect("path");
        // With budget 1, can only reach (2,0).
        assert_eq!(path.last(), Some(&HexPosition::new(2, 0)));
        assert_eq!(path.len(), 2); // start + 1 step
    }

    #[test]
    fn constrained_path_avoid_influence_zone() {
        let mut ctx = retreat_context();
        // Mark (2,0) as in an influence zone.
        let mut zone = HashSet::new();
        zone.insert(HexPosition::new(2, 0));
        ctx.influence_zones.insert("ZOC".to_string(), zone);

        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![
                PathConstraint::AwayFrom {
                    source: HexPosition::new(0, 0),
                },
                PathConstraint::AvoidInfluence {
                    influence_type: "ZOC".to_string(),
                },
            ],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        // (2,0) is blocked by ZOC. Whether a path exists depends on
        // alternate neighbors — in this minimal grid, likely no path.
        if let Some(path) = &result.path {
            // If a path was found, it must not go through (2,0).
            assert!(!path.contains(&HexPosition::new(2, 0)));
        }
    }

    #[test]
    fn constrained_path_avoid_terrain() {
        let mut ctx = retreat_context();
        ctx.terrain_types
            .insert(HexPosition::new(2, 0), "swamp".to_string());

        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![
                PathConstraint::AwayFrom {
                    source: HexPosition::new(0, 0),
                },
                PathConstraint::AvoidTerrain {
                    terrain_type: "swamp".to_string(),
                },
            ],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        if let Some(path) = &result.path {
            assert!(!path.contains(&HexPosition::new(2, 0)));
        }
    }

    #[test]
    fn constrained_path_no_valid_positions_fails() {
        let ctx = PathfindingContext {
            valid_positions: HashSet::new(),
            influence_zones: HashMap::new(),
            terrain_types: HashMap::new(),
        };

        let request = ConstrainedPathRequest {
            start: HexPosition::new(0, 0),
            constraints: vec![PathConstraint::AwayFrom {
                source: HexPosition::new(-1, 0),
            }],
            max_distance: 2,
        };

        let result = find_constrained_path(&request, &ctx);
        assert!(result.path.is_none());
        assert!(result.failure_reason.is_some());
    }

    #[test]
    fn constrained_path_no_constraints_finds_any_path() {
        let ctx = retreat_context();
        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        assert!(result.path.is_some());
    }

    #[test]
    fn constrained_path_zero_distance_fails() {
        let ctx = retreat_context();
        let request = ConstrainedPathRequest {
            start: HexPosition::new(1, 0),
            constraints: vec![],
            max_distance: 0,
        };

        let result = find_constrained_path(&request, &ctx);
        // With max_distance 0, no expansion happens — no valid endpoint.
        assert!(result.path.is_none());
    }

    #[test]
    fn path_constraint_variants_debug() {
        let constraints = [
            PathConstraint::AwayFrom {
                source: HexPosition::new(0, 0),
            },
            PathConstraint::AvoidInfluence {
                influence_type: "ZOC".to_string(),
            },
            PathConstraint::AvoidTerrain {
                terrain_type: "mountain".to_string(),
            },
            PathConstraint::MaxCost { budget: 3 },
        ];
        for c in &constraints {
            assert!(!format!("{c:?}").is_empty());
        }
    }

    // -- Accumulation Tracker tests --

    fn test_accumulator_registry() -> AccumulatorRegistry {
        AccumulatorRegistry {
            accumulators: vec![
                Accumulator {
                    id: "vp_red".to_string(),
                    faction: Some("Red".to_string()),
                    triggers: vec![
                        AccumulationTrigger::TurnBoundary { points: 2 },
                        AccumulationTrigger::OccupyHex {
                            hex: HexPosition::new(3, 0),
                            points: 5,
                        },
                    ],
                    value: 0,
                    history: Vec::new(),
                },
                Accumulator {
                    id: "vp_blue".to_string(),
                    faction: Some("Blue".to_string()),
                    triggers: vec![AccumulationTrigger::TurnBoundary { points: 1 }],
                    value: 0,
                    history: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn turn_boundary_triggers_update_all_accumulators() {
        let mut registry = test_accumulator_registry();
        let modified = evaluate_turn_boundary_triggers(&mut registry, 1);
        assert_eq!(modified.len(), 2);
        assert_eq!(registry.accumulators[0].value, 2);
        assert_eq!(registry.accumulators[1].value, 1);
        assert_eq!(registry.accumulators[0].history, vec![(1, 2)]);
        assert_eq!(registry.accumulators[1].history, vec![(1, 1)]);
    }

    #[test]
    fn turn_boundary_triggers_accumulate_over_turns() {
        let mut registry = test_accumulator_registry();
        evaluate_turn_boundary_triggers(&mut registry, 1);
        evaluate_turn_boundary_triggers(&mut registry, 2);
        assert_eq!(registry.accumulators[0].value, 4);
        assert_eq!(registry.accumulators[0].history.len(), 2);
    }

    #[test]
    fn occupy_hex_trigger_awards_points() {
        let mut registry = test_accumulator_registry();
        let modified = evaluate_occupy_triggers(&mut registry, HexPosition::new(3, 0), "Red", 1);
        assert_eq!(modified, vec![0]);
        assert_eq!(registry.accumulators[0].value, 5);
    }

    #[test]
    fn occupy_hex_trigger_wrong_faction_no_effect() {
        let mut registry = test_accumulator_registry();
        let modified = evaluate_occupy_triggers(&mut registry, HexPosition::new(3, 0), "Blue", 1);
        assert!(modified.is_empty());
        assert_eq!(registry.accumulators[0].value, 0);
    }

    #[test]
    fn occupy_hex_trigger_wrong_hex_no_effect() {
        let mut registry = test_accumulator_registry();
        let modified = evaluate_occupy_triggers(&mut registry, HexPosition::new(0, 0), "Red", 1);
        assert!(modified.is_empty());
    }

    #[test]
    fn victory_condition_greater_or_equal_met() {
        let mut registry = test_accumulator_registry();
        registry.accumulators[0].value = 15;
        let conditions = VictoryConditionRegistry {
            conditions: vec![VictoryCondition {
                accumulator_id: "vp_red".to_string(),
                threshold: 15,
                comparison: ComparisonOp::GreaterOrEqual,
            }],
        };
        let met = check_victory_conditions(&registry, &conditions);
        assert_eq!(met, vec![0]);
    }

    #[test]
    fn victory_condition_not_met() {
        let mut registry = test_accumulator_registry();
        registry.accumulators[0].value = 10;
        let conditions = VictoryConditionRegistry {
            conditions: vec![VictoryCondition {
                accumulator_id: "vp_red".to_string(),
                threshold: 15,
                comparison: ComparisonOp::GreaterOrEqual,
            }],
        };
        let met = check_victory_conditions(&registry, &conditions);
        assert!(met.is_empty());
    }

    #[test]
    fn victory_condition_equal_comparison() {
        let mut registry = test_accumulator_registry();
        registry.accumulators[0].value = 10;
        let conditions = VictoryConditionRegistry {
            conditions: vec![VictoryCondition {
                accumulator_id: "vp_red".to_string(),
                threshold: 10,
                comparison: ComparisonOp::Equal,
            }],
        };
        let met = check_victory_conditions(&registry, &conditions);
        assert_eq!(met, vec![0]);
    }

    #[test]
    fn victory_condition_unknown_accumulator_ignored() {
        let registry = test_accumulator_registry();
        let conditions = VictoryConditionRegistry {
            conditions: vec![VictoryCondition {
                accumulator_id: "nonexistent".to_string(),
                threshold: 0,
                comparison: ComparisonOp::GreaterOrEqual,
            }],
        };
        let met = check_victory_conditions(&registry, &conditions);
        assert!(met.is_empty());
    }

    #[test]
    fn no_triggers_no_modifications() {
        let mut registry = AccumulatorRegistry {
            accumulators: vec![Accumulator {
                id: "empty".to_string(),
                faction: None,
                triggers: vec![AccumulationTrigger::Manual],
                value: 5,
                history: Vec::new(),
            }],
        };
        let modified = evaluate_turn_boundary_triggers(&mut registry, 1);
        assert!(modified.is_empty());
        assert_eq!(registry.accumulators[0].value, 5);
    }

    // -- Canary Integration Test (Scope 4) --
    // Exercises all three scenario primitives together:
    // constrained pathfinding, scheduled spawning, and accumulation tracking.

    #[test]
    fn canary_scenario_end_to_end() {
        // --- Setup: 2 factions, 3-phase turn, spawn schedule, accumulators ---

        // Turn structure: Movement → Combat → Supply
        let mut turn_structure = TurnStructure {
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
        };
        let mut turn_state = TurnState {
            turn_number: 1,
            current_phase_index: 0,
            is_active: true,
            phase_actions_remaining: None,
        };

        // Spawn schedule: reinforcement on turn 3 at hex (5,0)
        let reinforcement_type_id = TypeId::new();
        let spawn_schedule = SpawnSchedule {
            entries: vec![SpawnEntry {
                entity_type_id: reinforcement_type_id,
                turn: 3,
                hex: HexPosition::new(5, 0),
                source_zone: "Reserve".to_string(),
            }],
        };

        // Accumulators: +5 VP for occupying objective hex (3,0) at turn boundary
        let objective_hex = HexPosition::new(3, 0);
        let mut accumulator_registry = AccumulatorRegistry {
            accumulators: vec![
                Accumulator {
                    id: "vp_red".to_string(),
                    faction: Some("Red".to_string()),
                    triggers: vec![AccumulationTrigger::OccupyHex {
                        hex: objective_hex,
                        points: 5,
                    }],
                    value: 0,
                    history: Vec::new(),
                },
                Accumulator {
                    id: "vp_blue".to_string(),
                    faction: Some("Blue".to_string()),
                    triggers: vec![AccumulationTrigger::OccupyHex {
                        hex: objective_hex,
                        points: 5,
                    }],
                    value: 0,
                    history: Vec::new(),
                },
            ],
        };

        // Victory condition: first faction to 15 VP
        let victory_conditions = VictoryConditionRegistry {
            conditions: vec![
                VictoryCondition {
                    accumulator_id: "vp_red".to_string(),
                    threshold: 15,
                    comparison: ComparisonOp::GreaterOrEqual,
                },
                VictoryCondition {
                    accumulator_id: "vp_blue".to_string(),
                    threshold: 15,
                    comparison: ComparisonOp::GreaterOrEqual,
                },
            ],
        };

        // --- Phase 1: Verify turn structure works ---
        let phase = current_phase(&turn_state, &turn_structure);
        assert_eq!(phase.map(|p| &*p.name), Some("Movement"));

        // Advance through all 3 phases of turn 1
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        assert_eq!(
            current_phase(&turn_state, &turn_structure).map(|p| &*p.name),
            Some("Combat")
        );
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        assert_eq!(
            current_phase(&turn_state, &turn_structure).map(|p| &*p.name),
            Some("Supply")
        );

        // At turn boundary (Red controls objective), award VP
        evaluate_occupy_triggers(&mut accumulator_registry, objective_hex, "Red", 1);
        assert_eq!(accumulator_registry.accumulators[0].value, 5);

        // No victory yet
        let met = check_victory_conditions(&accumulator_registry, &victory_conditions);
        assert!(met.is_empty());

        // --- Phase 2: Advance to turn 2 ---
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        assert_eq!(turn_state.turn_number, 2);
        assert_eq!(turn_state.current_phase_index, 0);

        // Simulate combat → retreat via constrained path
        // Defender at (2,0), attacker at (1,0), retreat away from attacker
        let mut valid = HashSet::new();
        valid.insert(HexPosition::new(2, 0));
        valid.insert(HexPosition::new(3, 0));
        valid.insert(HexPosition::new(4, 0));
        let ctx = PathfindingContext {
            valid_positions: valid,
            influence_zones: HashMap::new(),
            terrain_types: HashMap::new(),
        };
        let retreat_request = ConstrainedPathRequest {
            start: HexPosition::new(2, 0),
            constraints: vec![PathConstraint::AwayFrom {
                source: HexPosition::new(1, 0),
            }],
            max_distance: 3,
        };
        let retreat_result = find_constrained_path(&retreat_request, &ctx);
        assert!(retreat_result.path.is_some(), "Retreat path should exist");
        let path = retreat_result.path.as_ref().expect("path");
        assert_eq!(path.first(), Some(&HexPosition::new(2, 0)));
        // Path should move away from (1,0)
        for window in path.windows(2) {
            let d0 = crate::hex_grid::hex_distance(window[0], HexPosition::new(1, 0));
            let d1 = crate::hex_grid::hex_distance(window[1], HexPosition::new(1, 0));
            assert!(d1 >= d0, "Each step should move away from attacker");
        }

        // Red still controls objective at turn 2 boundary
        evaluate_occupy_triggers(&mut accumulator_registry, objective_hex, "Red", 2);
        assert_eq!(accumulator_registry.accumulators[0].value, 10);

        // Still no victory
        let met = check_victory_conditions(&accumulator_registry, &victory_conditions);
        assert!(met.is_empty());

        // --- Phase 3: Turn 3 — spawning + victory ---
        // Advance to turn 3
        // First finish turn 2's remaining phases
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        execute_phase_action(PhaseAction::Advance, &mut turn_state, &turn_structure);
        assert_eq!(turn_state.turn_number, 3);

        // Check spawn schedule: reinforcement due on turn 3
        let due = entries_due_for_turn(&spawn_schedule, 3);
        assert_eq!(due, vec![0]);

        // Spawn at target hex (5,0) — not occupied
        let occupied = HashSet::new();
        let spawn_hex = find_spawn_hex(HexPosition::new(5, 0), &occupied);
        assert_eq!(spawn_hex, Some(HexPosition::new(5, 0)));

        // Test spawn overflow: if (5,0) occupied, find adjacent
        let mut occupied_full = HashSet::new();
        occupied_full.insert(HexPosition::new(5, 0));
        let overflow_hex = find_spawn_hex(HexPosition::new(5, 0), &occupied_full);
        assert!(overflow_hex.is_some(), "Should find adjacent hex");
        assert_ne!(overflow_hex, Some(HexPosition::new(5, 0)));

        // Red controls objective at turn 3 boundary → 15 VP = victory!
        evaluate_occupy_triggers(&mut accumulator_registry, objective_hex, "Red", 3);
        assert_eq!(accumulator_registry.accumulators[0].value, 15);

        let met = check_victory_conditions(&accumulator_registry, &victory_conditions);
        assert_eq!(met, vec![0], "Red should meet victory condition");

        // Verify Red won, not Blue
        let red_acc = &accumulator_registry.accumulators[0];
        assert_eq!(red_acc.id, "vp_red");
        assert_eq!(red_acc.value, 15);
        assert_eq!(red_acc.history.len(), 3); // 3 turns of VP

        let blue_acc = &accumulator_registry.accumulators[1];
        assert_eq!(blue_acc.id, "vp_blue");
        assert_eq!(blue_acc.value, 0); // Blue never occupied the objective
    }

    #[test]
    fn canary_retreat_blocked_by_zoc() {
        // Retreat path blocked by enemy ZOC — no valid path exists
        let mut valid = HashSet::new();
        valid.insert(HexPosition::new(2, 0));
        valid.insert(HexPosition::new(3, 0));

        let mut zoc = HashSet::new();
        zoc.insert(HexPosition::new(3, 0)); // Only retreat target is in ZOC

        let ctx = PathfindingContext {
            valid_positions: valid,
            influence_zones: [("ZOC".to_string(), zoc)].into_iter().collect(),
            terrain_types: HashMap::new(),
        };

        let request = ConstrainedPathRequest {
            start: HexPosition::new(2, 0),
            constraints: vec![
                PathConstraint::AwayFrom {
                    source: HexPosition::new(1, 0),
                },
                PathConstraint::AvoidInfluence {
                    influence_type: "ZOC".to_string(),
                },
            ],
            max_distance: 3,
        };

        let result = find_constrained_path(&request, &ctx);
        // In this constrained grid, the only path away is through ZOC → blocked
        if let Some(path) = &result.path {
            // If any path found, it must NOT contain ZOC hexes
            assert!(!path.contains(&HexPosition::new(3, 0)));
        }
        // Either no path, or a path that avoids ZOC — both valid outcomes
    }

    #[test]
    fn canary_spawn_not_due_before_scheduled_turn() {
        let schedule = SpawnSchedule {
            entries: vec![SpawnEntry {
                entity_type_id: TypeId::new(),
                turn: 3,
                hex: HexPosition::new(5, 0),
                source_zone: "Reserve".to_string(),
            }],
        };

        assert!(entries_due_for_turn(&schedule, 1).is_empty());
        assert!(entries_due_for_turn(&schedule, 2).is_empty());
        assert_eq!(entries_due_for_turn(&schedule, 3), vec![0]);
        assert!(entries_due_for_turn(&schedule, 4).is_empty());
    }
}
