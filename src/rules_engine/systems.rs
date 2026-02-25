//! Rules engine systems: valid move computation via BFS with constraint evaluation.

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

use hexorder_contracts::game_system::{
    EntityData, EntityTypeRegistry, PropertyValue, SelectedUnit, TypeId, UnitInstance,
};
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use hexorder_contracts::ontology::{
    ConceptBinding, ConceptRegistry, ConstraintExpr, ConstraintRegistry, ModifyOperation,
    RelationEffect, RelationRegistry, RelationTrigger,
};
use hexorder_contracts::validation::{ValidMoveSet, ValidationResult};

/// Computes the set of valid moves for the currently selected unit.
///
/// Runs a BFS from the unit's position, evaluating ontology relations
/// (with `OnEnter` trigger) at each step. Produces a `ValidMoveSet`
/// containing reachable positions and explanations for blocked ones.
///
/// When no unit is selected the move set is cleared. When no ontology
/// constraints exist all in-bounds positions are reachable (free movement).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn compute_valid_moves(
    selected: Res<SelectedUnit>,
    concepts: Res<ConceptRegistry>,
    relations: Res<RelationRegistry>,
    constraints: Res<ConstraintRegistry>,
    entity_types: Res<EntityTypeRegistry>,
    grid_config: Res<HexGridConfig>,
    mut valid_moves: ResMut<ValidMoveSet>,
    units: Query<(&HexPosition, &EntityData), With<UnitInstance>>,
    tiles: Query<(&HexPosition, &EntityData), (With<HexTile>, Without<UnitInstance>)>,
) {
    // Only recompute when something relevant changed.
    if !selected.is_changed()
        && !concepts.is_changed()
        && !relations.is_changed()
        && !constraints.is_changed()
    {
        return;
    }

    // If no unit is selected, clear the move set.
    let Some(unit_entity) = selected.entity else {
        valid_moves.valid_positions.clear();
        valid_moves.blocked_explanations.clear();
        valid_moves.for_entity = None;
        return;
    };

    // Look up the unit's position and data.
    let Ok((unit_pos, unit_data)) = units.get(unit_entity) else {
        valid_moves.valid_positions.clear();
        valid_moves.blocked_explanations.clear();
        valid_moves.for_entity = None;
        return;
    };

    // Build a spatial lookup for tiles.
    let tile_lookup: HashMap<HexPosition, &EntityData> =
        tiles.iter().map(|(pos, data)| (*pos, data)).collect();

    let map_radius = grid_config.map_radius;

    // Collect OnEnter relations.
    let on_enter_relations: Vec<_> = relations
        .relations
        .iter()
        .filter(|r| r.trigger == RelationTrigger::OnEnter)
        .collect();

    // Clear previous results.
    valid_moves.valid_positions.clear();
    valid_moves.blocked_explanations.clear();
    valid_moves.for_entity = Some(unit_entity);

    // If no relations and no constraints exist, free movement within bounds.
    if on_enter_relations.is_empty() && constraints.constraints.is_empty() {
        bfs_free_movement(*unit_pos, map_radius, &mut valid_moves);
        return;
    }

    // Find the unit's concept bindings.
    let unit_bindings: Vec<&ConceptBinding> = concepts
        .bindings
        .iter()
        .filter(|b| b.entity_type_id == unit_data.entity_type_id)
        .collect();

    // Determine the initial budget from ModifyProperty Subtract relations.
    let initial_budget =
        determine_budget(&unit_bindings, &on_enter_relations, unit_data, &concepts);

    // Gather context needed for step evaluation.
    let ctx = StepContext {
        unit_data,
        unit_bindings: &unit_bindings,
        on_enter_relations: &on_enter_relations,
        concepts: &concepts,
        entity_types: &entity_types,
    };

    // BFS with budget tracking.
    let mut queue: VecDeque<(HexPosition, i64)> = VecDeque::new();
    let mut best_budget: HashMap<HexPosition, i64> = HashMap::new();

    queue.push_back((*unit_pos, initial_budget));
    best_budget.insert(*unit_pos, initial_budget);

    while let Some((current_pos, remaining_budget)) = queue.pop_front() {
        let hex = current_pos.to_hex();

        for neighbor_hex in hex.all_neighbors() {
            let neighbor_pos = HexPosition::from_hex(neighbor_hex);

            if !is_within_bounds(neighbor_pos, map_radius) {
                continue;
            }

            if neighbor_pos == *unit_pos {
                continue;
            }

            let tile_data = tile_lookup.get(&neighbor_pos).copied();

            let step_result = evaluate_step(&ctx, tile_data, remaining_budget, neighbor_pos);

            match step_result {
                StepResult::Valid { new_budget } => {
                    let dominated = best_budget
                        .get(&neighbor_pos)
                        .is_some_and(|&prev| prev >= new_budget);
                    if dominated {
                        continue;
                    }
                    best_budget.insert(neighbor_pos, new_budget);
                    valid_moves.valid_positions.insert(neighbor_pos);
                    valid_moves.blocked_explanations.remove(&neighbor_pos);

                    if new_budget > 0 {
                        queue.push_back((neighbor_pos, new_budget));
                    }
                }
                StepResult::Blocked { reasons } => {
                    if !valid_moves.valid_positions.contains(&neighbor_pos) {
                        valid_moves
                            .blocked_explanations
                            .entry(neighbor_pos)
                            .or_default()
                            .extend(reasons);
                    }
                }
            }
        }
    }
}

/// Shared context for step evaluation, avoiding excessive parameter counts.
struct StepContext<'a> {
    unit_data: &'a EntityData,
    unit_bindings: &'a [&'a ConceptBinding],
    on_enter_relations: &'a [&'a hexorder_contracts::ontology::Relation],
    concepts: &'a ConceptRegistry,
    entity_types: &'a EntityTypeRegistry,
}

/// Result of evaluating a single BFS step into a neighbor hex.
enum StepResult {
    Valid { new_budget: i64 },
    Blocked { reasons: Vec<ValidationResult> },
}

/// Free-movement BFS: all positions within grid bounds reachable from the
/// unit's position (no budget limit beyond map radius).
fn bfs_free_movement(start: HexPosition, map_radius: u32, valid_moves: &mut ValidMoveSet) {
    let mut queue: VecDeque<HexPosition> = VecDeque::new();
    queue.push_back(start);

    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        let hex = current.to_hex();
        for neighbor_hex in hex.all_neighbors() {
            let neighbor = HexPosition::from_hex(neighbor_hex);
            if !is_within_bounds(neighbor, map_radius) {
                continue;
            }
            if visited.contains(&neighbor) {
                continue;
            }
            visited.insert(neighbor);
            valid_moves.valid_positions.insert(neighbor);
            queue.push_back(neighbor);
        }
    }
}

/// Checks whether a hex position is within grid bounds.
/// Grid bounds: `max(|q|, |r|, |q+r|) <= map_radius`.
fn is_within_bounds(pos: HexPosition, map_radius: u32) -> bool {
    let q = pos.q.unsigned_abs();
    let r = pos.r.unsigned_abs();
    let s = (pos.q + pos.r).unsigned_abs();
    q.max(r).max(s) <= map_radius
}

/// Determines the initial movement budget for a unit based on its concept
/// bindings and the ontology relations.
///
/// Strategy:
/// 1. Look for `ModifyProperty` with `Subtract` operation to identify the
///    budget property via `target_property`.
/// 2. Find the matching property value on the unit's `EntityData`.
/// 3. Fall back to a named "budget" property.
/// 4. Fall back to a generous default if no budget property is found.
fn determine_budget(
    unit_bindings: &[&ConceptBinding],
    on_enter_relations: &[&hexorder_contracts::ontology::Relation],
    unit_data: &EntityData,
    concepts: &ConceptRegistry,
) -> i64 {
    // Strategy 1: Find ModifyProperty Subtract relations and extract the
    // target_property (which is the budget concept-local name on the subject).
    for relation in on_enter_relations {
        if let RelationEffect::ModifyProperty {
            target_property,
            operation: ModifyOperation::Subtract,
            ..
        } = &relation.effect
        {
            for binding in unit_bindings {
                if binding.concept_id != relation.concept_id
                    || binding.concept_role_id != relation.subject_role_id
                {
                    continue;
                }
                for prop_binding in &binding.property_bindings {
                    if prop_binding.concept_local_name != *target_property {
                        continue;
                    }
                    if let Some(value) = unit_data.properties.get(&prop_binding.property_id)
                        && let Some(budget) = property_value_as_i64(value)
                    {
                        return budget;
                    }
                }
            }
        }
    }

    // Strategy 2: Look through all bindings for a property with concept-local
    // name "budget" as a fallback heuristic.
    for binding in unit_bindings {
        for prop_binding in &binding.property_bindings {
            if prop_binding.concept_local_name == "budget"
                && let Some(value) = unit_data.properties.get(&prop_binding.property_id)
                && let Some(budget) = property_value_as_i64(value)
            {
                return budget;
            }
        }
    }

    // Fallback: no budget found, allow generous default.
    i64::from(concepts.concepts.len().max(1) as u32) * 10
}

/// Evaluates a single step of the BFS: checks whether the unit can enter
/// `target_pos` given the tile at that position and the applicable relations.
fn evaluate_step(
    ctx: &StepContext<'_>,
    tile_data: Option<&EntityData>,
    remaining_budget: i64,
    target_pos: HexPosition,
) -> StepResult {
    let mut blocked_reasons: Vec<ValidationResult> = Vec::new();
    let mut cost: i64 = 0;
    let mut has_block = false;

    for relation in ctx.on_enter_relations {
        // Find unit bindings matching the subject role of this relation.
        let unit_matches_subject = ctx.unit_bindings.iter().any(|b| {
            b.concept_id == relation.concept_id && b.concept_role_id == relation.subject_role_id
        });
        if !unit_matches_subject {
            continue;
        }

        // Find tile bindings matching the object role of this relation.
        let tile_matches_object = tile_data.is_some_and(|td| {
            ctx.concepts.bindings.iter().any(|b| {
                b.entity_type_id == td.entity_type_id
                    && b.concept_id == relation.concept_id
                    && b.concept_role_id == relation.object_role_id
            })
        });
        if !tile_matches_object {
            continue;
        }

        match &relation.effect {
            RelationEffect::ModifyProperty {
                target_property,
                source_property,
                operation,
            } => {
                let source_value = tile_data.and_then(|td| {
                    resolve_concept_property(
                        td,
                        source_property,
                        relation.concept_id,
                        relation.object_role_id,
                        &ctx.concepts.bindings,
                    )
                });

                let source_val = source_value
                    .and_then(|v| property_value_as_i64(&v))
                    .unwrap_or(0);

                match operation {
                    ModifyOperation::Subtract => {
                        cost += source_val;
                    }
                    ModifyOperation::Add => {
                        cost -= source_val;
                    }
                    _ => {}
                }

                if remaining_budget - cost < 0 {
                    let unit_type_name = ctx
                        .entity_types
                        .get(ctx.unit_data.entity_type_id)
                        .map_or("Unit", |et| et.name.as_str());
                    blocked_reasons.push(ValidationResult {
                        constraint_id: relation.id,
                        constraint_name: relation.name.clone(),
                        satisfied: false,
                        explanation: format!(
                            "{unit_type_name} cannot reach ({}, {}): path cost {cost} exceeds {target_property} of {remaining_budget}",
                            target_pos.q,
                            target_pos.r,
                        ),
                    });
                }
            }
            RelationEffect::Block { condition } => {
                let is_blocked = match condition {
                    None => true,
                    Some(expr) => {
                        evaluate_block_condition(expr, ctx.unit_data, tile_data, relation)
                    }
                };
                if is_blocked {
                    has_block = true;
                    let unit_type_name = ctx
                        .entity_types
                        .get(ctx.unit_data.entity_type_id)
                        .map_or("Unit", |et| et.name.as_str());
                    let target_type_name = tile_data
                        .and_then(|td| ctx.entity_types.get(td.entity_type_id))
                        .map_or("Unknown", |et| et.name.as_str());
                    blocked_reasons.push(ValidationResult {
                        constraint_id: relation.id,
                        constraint_name: relation.name.clone(),
                        satisfied: false,
                        explanation: format!(
                            "{unit_type_name} cannot enter {target_type_name}: {} blocks entry",
                            relation.name
                        ),
                    });
                }
            }
            RelationEffect::Allow { .. } => {
                // Allow effects are not blocking; they permit entry.
                // In a default-deny model these would allowlist, but for
                // 0.4.0 we treat absence of Block as implicit allow.
            }
        }
    }

    if has_block {
        return StepResult::Blocked {
            reasons: blocked_reasons,
        };
    }

    let new_budget = remaining_budget - cost;
    if new_budget < 0 {
        StepResult::Blocked {
            reasons: blocked_reasons,
        }
    } else {
        StepResult::Valid { new_budget }
    }
}

/// Resolves a concept-local property name to the actual `PropertyValue` on
/// an entity, using the concept bindings to map from concept-local names
/// to property IDs.
fn resolve_concept_property(
    entity_data: &EntityData,
    concept_local_name: &str,
    concept_id: TypeId,
    concept_role_id: TypeId,
    bindings: &[ConceptBinding],
) -> Option<PropertyValue> {
    for binding in bindings {
        if binding.entity_type_id != entity_data.entity_type_id
            || binding.concept_id != concept_id
            || binding.concept_role_id != concept_role_id
        {
            continue;
        }
        for prop_binding in &binding.property_bindings {
            if prop_binding.concept_local_name == concept_local_name {
                return entity_data
                    .properties
                    .get(&prop_binding.property_id)
                    .cloned();
            }
        }
    }
    None
}

/// Evaluates a block condition expression. For 0.4.0, handles `IsType`,
/// `IsNotType`, `All`, `Any`, and `Not` checks. Other expression types
/// default to true (blocked, conservative).
fn evaluate_block_condition(
    expr: &ConstraintExpr,
    unit_data: &EntityData,
    tile_data: Option<&EntityData>,
    relation: &hexorder_contracts::ontology::Relation,
) -> bool {
    match expr {
        ConstraintExpr::IsType {
            role_id,
            entity_type_id,
        } => {
            let data = if *role_id == relation.subject_role_id {
                Some(unit_data)
            } else if *role_id == relation.object_role_id {
                tile_data
            } else {
                None
            };
            data.is_some_and(|d| d.entity_type_id == *entity_type_id)
        }
        ConstraintExpr::IsNotType {
            role_id,
            entity_type_id,
        } => {
            let data = if *role_id == relation.subject_role_id {
                Some(unit_data)
            } else if *role_id == relation.object_role_id {
                tile_data
            } else {
                None
            };
            data.is_some_and(|d| d.entity_type_id != *entity_type_id)
        }
        ConstraintExpr::All(exprs) => exprs
            .iter()
            .all(|e| evaluate_block_condition(e, unit_data, tile_data, relation)),
        ConstraintExpr::Any(exprs) => exprs
            .iter()
            .any(|e| evaluate_block_condition(e, unit_data, tile_data, relation)),
        ConstraintExpr::Not(inner) => {
            !evaluate_block_condition(inner, unit_data, tile_data, relation)
        }
        // For other expression types, default to blocked (conservative).
        _ => true,
    }
}

/// Extracts an `i64` value from a `PropertyValue`, coercing numeric types.
fn property_value_as_i64(value: &PropertyValue) -> Option<i64> {
    match value {
        PropertyValue::Int(v) => Some(*v),
        PropertyValue::Float(v) => Some(*v as i64),
        _ => None,
    }
}

// Combat resolution functions (calculate_odds_ratio, find_crt_column, resolve_crt,
// evaluate_modifiers_prioritized, apply_column_shift, etc.) live in
// `hexorder_contracts::mechanics` — they are pure functions on contract types, usable
// by any plugin without a boundary violation.

// ---------------------------------------------------------------------------
// Phase Advancement (0.9.0)
// ---------------------------------------------------------------------------

use hexorder_contracts::mechanics::{PhaseAdvancedEvent, TurnState, TurnStructure};

/// Advances the turn to the next phase, wrapping to the next turn if needed.
///
/// Returns `Some(PhaseAdvancedEvent)` with the new phase info, or `None` if
/// the turn structure has no phases.
#[allow(dead_code)]
pub(crate) fn advance_phase(
    state: &mut TurnState,
    structure: &TurnStructure,
) -> Option<PhaseAdvancedEvent> {
    if structure.phases.is_empty() {
        return None;
    }

    // Initialize turn number on first advance.
    if state.turn_number == 0 {
        state.turn_number = 1;
    }

    let next_index = state.current_phase_index + 1;
    if next_index >= structure.phases.len() {
        // Wrap to next turn.
        state.turn_number += 1;
        state.current_phase_index = 0;
    } else {
        state.current_phase_index = next_index;
    }

    let phase = &structure.phases[state.current_phase_index];
    Some(PhaseAdvancedEvent {
        turn_number: state.turn_number,
        phase_index: state.current_phase_index,
        phase_name: phase.name.clone(),
        phase_type: phase.phase_type,
    })
}

/// Starts the turn sequence: sets the turn to 1, phase to 0, and marks active.
///
/// Returns `Some(PhaseAdvancedEvent)` for the first phase, or `None` if no phases.
#[allow(dead_code)]
pub(crate) fn start_turn_sequence(
    state: &mut TurnState,
    structure: &TurnStructure,
) -> Option<PhaseAdvancedEvent> {
    if structure.phases.is_empty() {
        return None;
    }

    state.turn_number = 1;
    state.current_phase_index = 0;
    state.is_active = true;

    let phase = &structure.phases[0];
    Some(PhaseAdvancedEvent {
        turn_number: 1,
        phase_index: 0,
        phase_name: phase.name.clone(),
        phase_type: phase.phase_type,
    })
}
