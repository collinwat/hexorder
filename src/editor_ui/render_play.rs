//! Play mode panel rendering systems.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use hexorder_contracts::game_system::{
    EntityData, EntityTypeRegistry, GameSystem, SelectedUnit, UnitInstance,
};
use hexorder_contracts::mechanics::{
    ActiveCombat, CombatModifierRegistry, CombatResultsTable, PhaseAction, PhaseType,
    PostResolutionAction, PostResolutionRule, TurnState, TurnStructure, evaluate_post_resolution,
    execute_phase_action, is_phase_action_legal,
};
use hexorder_contracts::persistence::{
    AppScreen, CloseProjectEvent, LoadRequestEvent, SaveRequestEvent, Workspace,
};
use hexorder_contracts::simulation::{
    ChainRollSource, ChainStep, DicePool, ResolutionChain, SimulationRng, reset_rng, resolve_chain,
    roll_pool,
};

use std::collections::HashMap;

use super::components::{BrandTheme, EditorState};
use super::render_panels::{render_about_panel, render_workspace_header};

/// Actions that can be triggered from the play mode file menu.
/// Returned by [`render_play_file_menu`] so the caller can dispatch ECS
/// commands outside the egui closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayMenuAction {
    NewProject,
    OpenProject,
    Save,
    SaveAs,
    #[allow(dead_code)]
    ShowAbout,
}

/// Play mode panel system. Shows the turn tracker, combat panel, and mode toggle.
/// Runs only in `AppScreen::Play`.
#[allow(clippy::too_many_arguments)]
pub fn play_panel_system(
    mut contexts: EguiContexts,
    mut turn_state: ResMut<TurnState>,
    turn_structure: Res<TurnStructure>,
    game_system: Res<GameSystem>,
    workspace: Res<Workspace>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut commands: Commands,
    mut active_combat: ResMut<ActiveCombat>,
    combat_results_table: Res<CombatResultsTable>,
    combat_modifiers: Res<CombatModifierRegistry>,
    selected_unit: Res<SelectedUnit>,
    entity_types: Res<EntityTypeRegistry>,
    mut editor_state: ResMut<EditorState>,
    mut sim_rng: ResMut<SimulationRng>,
    unit_query: Query<&EntityData, With<UnitInstance>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // -- File Menu Bar --
    let mut menu_actions = Vec::new();
    egui::TopBottomPanel::top("file_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                menu_actions = render_play_file_menu(ui);
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About Hexorder").clicked() {
                    editor_state.about_panel_visible = true;
                    ui.close();
                }
            });
        });
    });

    // Dispatch menu actions outside the egui closure.
    for action in menu_actions {
        match action {
            PlayMenuAction::NewProject => commands.trigger(CloseProjectEvent),
            PlayMenuAction::OpenProject => commands.trigger(LoadRequestEvent),
            PlayMenuAction::Save => commands.trigger(SaveRequestEvent { save_as: false }),
            PlayMenuAction::SaveAs => commands.trigger(SaveRequestEvent { save_as: true }),
            PlayMenuAction::ShowAbout => editor_state.about_panel_visible = true,
        }
    }

    // -- Sidebar --
    let mut switch_to_editor = false;
    egui::SidePanel::left("play_panel")
        .default_width(280.0)
        .show(ctx, |ui| {
            switch_to_editor = render_play_sidebar(
                ui,
                &workspace,
                &game_system,
                &mut turn_state,
                &turn_structure,
                &mut active_combat,
                &combat_results_table,
                &combat_modifiers,
                &selected_unit,
                &entity_types,
                &mut editor_state,
                &mut sim_rng,
                &|e| unit_query.get(e).ok(),
            );
        });

    if switch_to_editor {
        turn_state.is_active = false;
        next_state.set(AppScreen::Editor);
    }

    // -- About Panel --
    render_about_panel(ctx, &mut editor_state);
}

/// Renders the play mode file menu contents.
/// Returns a list of [`PlayMenuAction`]s triggered by the user.
pub(crate) fn render_play_file_menu(ui: &mut egui::Ui) -> Vec<PlayMenuAction> {
    let mut actions = Vec::new();
    if ui.button("New          Cmd+N").clicked() {
        actions.push(PlayMenuAction::NewProject);
        ui.close();
    }
    if ui.button("Open...      Cmd+O").clicked() {
        actions.push(PlayMenuAction::OpenProject);
        ui.close();
    }
    ui.separator();
    if ui.button("Save         Cmd+S").clicked() {
        actions.push(PlayMenuAction::Save);
        ui.close();
    }
    if ui.button("Save As...   Cmd+Shift+S").clicked() {
        actions.push(PlayMenuAction::SaveAs);
        ui.close();
    }
    actions
}

/// Renders the play mode sidebar body: workspace header, editor toggle,
/// turn tracker, and combat panel.
///
/// Returns `true` if the user clicked the "Editor" button to switch back
/// to editor mode. The caller is responsible for performing the state
/// transition so that ECS mutations happen outside the egui closure.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_play_sidebar<'a>(
    ui: &mut egui::Ui,
    workspace: &Workspace,
    game_system: &GameSystem,
    turn_state: &mut TurnState,
    turn_structure: &TurnStructure,
    active_combat: &mut ActiveCombat,
    crt: &CombatResultsTable,
    modifiers: &CombatModifierRegistry,
    selected_unit: &SelectedUnit,
    entity_types: &EntityTypeRegistry,
    editor_state: &mut EditorState,
    sim_rng: &mut SimulationRng,
    unit_lookup: &dyn Fn(Entity) -> Option<&'a EntityData>,
) -> bool {
    // -- Workspace Header --
    render_workspace_header(ui, workspace, game_system);

    // -- Back to Editor --
    let mut switch_to_editor = false;
    if ui
        .button(
            egui::RichText::new("\u{25A0} Editor")
                .strong()
                .color(BrandTheme::ACCENT_AMBER),
        )
        .clicked()
    {
        switch_to_editor = true;
    }
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // -- Turn Tracker --
        render_turn_tracker(ui, turn_state, turn_structure);

        ui.separator();

        // -- Dice Panel --
        render_dice_panel(ui, editor_state, sim_rng);

        ui.separator();

        // -- Chain Panel --
        render_chain_panel(ui, editor_state, sim_rng, crt);

        ui.separator();

        // -- Combat Panel --
        let in_combat_phase = turn_structure
            .phases
            .get(turn_state.current_phase_index)
            .is_some_and(|p| p.phase_type == PhaseType::Combat);
        render_combat_panel(
            ui,
            active_combat,
            crt,
            modifiers,
            selected_unit,
            entity_types,
            editor_state,
            unit_lookup,
            in_combat_phase,
        );
    });

    switch_to_editor
}

/// Renders the turn tracker section in the play panel.
pub(crate) fn render_turn_tracker(
    ui: &mut egui::Ui,
    turn_state: &mut TurnState,
    turn_structure: &TurnStructure,
) {
    ui.label(
        egui::RichText::new("Turn Tracker")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    if turn_structure.phases.is_empty() {
        ui.label(
            egui::RichText::new("No phases defined. Add phases in the Mechanics tab.")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    // Initialize turn state on first entry.
    if turn_state.turn_number == 0 {
        turn_state.turn_number = 1;
        turn_state.current_phase_index = 0;
        turn_state.is_active = true;
    }

    // Current turn and phase display.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Turn {}", turn_state.turn_number))
                .strong()
                .size(16.0)
                .color(BrandTheme::TEXT_PRIMARY),
        );
    });

    if let Some(phase) = turn_structure.phases.get(turn_state.current_phase_index) {
        let type_label = match phase.phase_type {
            PhaseType::Movement => "Movement",
            PhaseType::Combat => "Combat",
            PhaseType::Admin => "Admin",
        };
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&phase.name)
                    .strong()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("[{type_label}]"))
                    .small()
                    .color(BrandTheme::ACCENT_TEAL),
            );
        });

        ui.label(
            egui::RichText::new(format!(
                "Phase {} of {}",
                turn_state.current_phase_index + 1,
                turn_structure.phases.len()
            ))
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
        );
    }

    ui.add_space(8.0);

    // Phase list with current highlighted.
    for (i, phase) in turn_structure.phases.iter().enumerate() {
        let is_current = i == turn_state.current_phase_index;
        let text = if is_current {
            egui::RichText::new(format!("\u{25B6} {}", phase.name))
                .strong()
                .color(BrandTheme::ACCENT_AMBER)
        } else {
            egui::RichText::new(format!("  {}", phase.name)).color(BrandTheme::TEXT_SECONDARY)
        };
        ui.label(text);
    }

    ui.add_space(8.0);

    // Phase action buttons.
    ui.horizontal(|ui| {
        let can_rewind = is_phase_action_legal(PhaseAction::Rewind, turn_state, turn_structure);
        ui.add_enabled_ui(can_rewind, |ui| {
            if ui.button("\u{23EA} Prev").clicked() {
                execute_phase_action(PhaseAction::Rewind, turn_state, turn_structure);
            }
        });

        let can_advance = is_phase_action_legal(PhaseAction::Advance, turn_state, turn_structure);
        ui.add_enabled_ui(can_advance, |ui| {
            if ui.button("Next \u{23E9}").clicked() {
                execute_phase_action(PhaseAction::Advance, turn_state, turn_structure);
            }
        });

        let can_skip = is_phase_action_legal(PhaseAction::Skip, turn_state, turn_structure);
        ui.add_enabled_ui(can_skip, |ui| {
            if ui.button("Skip \u{23ED}").clicked() {
                execute_phase_action(PhaseAction::Skip, turn_state, turn_structure);
            }
        });
    });
}

/// Renders the dice pool panel: pool configuration, roll button, results, and seed control.
pub(crate) fn render_dice_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    sim_rng: &mut SimulationRng,
) {
    ui.label(
        egui::RichText::new("Dice")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    // Pool configuration.
    ui.horizontal(|ui| {
        let mut count = i32::from(editor_state.dice_count);
        ui.add(egui::DragValue::new(&mut count).range(1..=255).prefix(""));
        editor_state.dice_count = count.clamp(1, 255) as u8;

        ui.label("d");

        let mut sides = i32::from(editor_state.dice_sides);
        ui.add(egui::DragValue::new(&mut sides).range(1..=255).prefix(""));
        editor_state.dice_sides = sides.clamp(1, 255) as u8;

        let mut modifier = i32::from(editor_state.dice_modifier);
        ui.add(
            egui::DragValue::new(&mut modifier)
                .range(-128..=127)
                .prefix("+"),
        );
        editor_state.dice_modifier = modifier.clamp(-128, 127) as i8;
    });

    let pool = DicePool::new(
        editor_state.dice_count,
        editor_state.dice_sides,
        editor_state.dice_modifier,
    );
    ui.label(
        egui::RichText::new(format!("Pool: {pool}"))
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
    );

    ui.add_space(4.0);

    // Roll button.
    if ui.button("Roll \u{1F3B2}").clicked() {
        let result = roll_pool(sim_rng, pool, "dice panel");
        editor_state.last_dice_roll = Some(result);
    }

    // Result display.
    if let Some(roll) = &editor_state.last_dice_roll {
        ui.add_space(4.0);
        let values_str: Vec<String> = roll.values.iter().map(ToString::to_string).collect();
        ui.horizontal(|ui| {
            ui.label("Dice:");
            ui.label(
                egui::RichText::new(format!("[{}]", values_str.join(", ")))
                    .color(BrandTheme::TEXT_PRIMARY),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Total:");
            ui.label(
                egui::RichText::new(format!("{}", roll.total))
                    .strong()
                    .size(18.0)
                    .color(BrandTheme::ACCENT_AMBER),
            );
            if roll.pool.modifier != 0 {
                let sum: i16 = roll.values.iter().map(|&v| i16::from(v)).sum();
                ui.label(
                    egui::RichText::new(format!("({sum}{:+})", roll.pool.modifier))
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
            }
        });
    }

    ui.add_space(4.0);

    // Seed controls.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Seed: {}", sim_rng.seed()))
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
    });
    ui.horizontal(|ui| {
        let response = ui
            .add(egui::TextEdit::singleline(&mut editor_state.dice_seed_input).desired_width(80.0));
        let apply_seed = (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            || ui.button("Set Seed").clicked();
        if apply_seed && let Ok(seed) = editor_state.dice_seed_input.trim().parse::<u64>() {
            reset_rng(sim_rng, seed);
            editor_state.last_dice_roll = None;
        }
    });
    ui.label(
        egui::RichText::new(format!("Rolls: {}", sim_rng.roll_count()))
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
    );
}

/// Renders the resolution chain panel: build a chain from available tables, resolve, show results.
pub(crate) fn render_chain_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    sim_rng: &mut SimulationRng,
    crt: &CombatResultsTable,
) {
    let header = egui::RichText::new("Resolution Chains")
        .strong()
        .color(BrandTheme::ACCENT_AMBER);
    let expanded = egui::CollapsingHeader::new(header)
        .default_open(editor_state.chain_panel_expanded)
        .show(ui, |ui| {
            if crt.table.columns.is_empty() || crt.table.rows.is_empty() {
                ui.label(
                    egui::RichText::new("No tables available. Define a CRT in the Mechanics tab.")
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                return;
            }

            ui.label(
                egui::RichText::new(format!(
                    "Available: {} ({}×{})",
                    crt.table.name,
                    crt.table.columns.len(),
                    crt.table.rows.len()
                ))
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
            );

            ui.add_space(4.0);

            // Use combat panel strength values as initial context.
            ui.label(
                egui::RichText::new("Context values from combat strengths")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "atk={:.1}, def={:.1}",
                        editor_state.combat_attacker_strength,
                        editor_state.combat_defender_strength
                    ))
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
                );
            });

            ui.add_space(4.0);

            // Build a demo 1-step chain from the CRT.
            if ui.button("Resolve CRT Chain").clicked() {
                let chain = ResolutionChain {
                    id: hexorder_contracts::game_system::TypeId::new(),
                    name: "CRT chain".to_string(),
                    steps: vec![ChainStep {
                        table_id: crt.table.id,
                        input_a_key: "atk".to_string(),
                        input_b_key: "def".to_string(),
                        roll_source: ChainRollSource::Pool(DicePool::new(
                            editor_state.dice_count,
                            editor_state.dice_sides,
                            editor_state.dice_modifier,
                        )),
                        output_key: "result".to_string(),
                    }],
                    max_depth: 10,
                };

                let mut initial = HashMap::new();
                initial.insert("atk".to_string(), editor_state.combat_attacker_strength);
                initial.insert("def".to_string(), editor_state.combat_defender_strength);

                let mut tables = HashMap::new();
                tables.insert(crt.table.id, crt.table.clone());

                let ctx = resolve_chain(&chain, &initial, &tables, sim_rng);
                editor_state.last_chain_result = Some(ctx);
            }

            // Display results.
            if let Some(ref ctx) = editor_state.last_chain_result {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Chain Results")
                        .strong()
                        .color(BrandTheme::TEXT_PRIMARY),
                );

                for step in &ctx.step_log {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Step {}:", step.step_index + 1))
                                .small()
                                .strong()
                                .color(BrandTheme::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(&step.table_name)
                                .small()
                                .color(BrandTheme::TEXT_SECONDARY),
                        );
                    });

                    if let Some(ref res) = step.resolution {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "  Col: {} Row: {}",
                                    res.column_label, res.row_label
                                ))
                                .small()
                                .color(BrandTheme::TEXT_PRIMARY),
                            );
                        });
                        let result_text = match &res.result {
                            hexorder_contracts::simulation::TableResult::Text(s) => s.clone(),
                            hexorder_contracts::simulation::TableResult::NumericValue(v) => {
                                format!("{v:.1}")
                            }
                            hexorder_contracts::simulation::TableResult::PropertyModifier {
                                property,
                                delta,
                            } => format!("{property} {delta:+.1}"),
                        };
                        ui.label(
                            egui::RichText::new(format!("  Result: {result_text}"))
                                .strong()
                                .color(BrandTheme::ACCENT_TEAL),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("  (no match)")
                                .small()
                                .color(BrandTheme::TEXT_SECONDARY),
                        );
                    }
                }

                // Show final context values.
                if !ctx.values.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Context")
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                    let mut keys: Vec<&String> = ctx.values.keys().collect();
                    keys.sort();
                    for key in keys {
                        if let Some(&val) = ctx.values.get(key) {
                            ui.label(
                                egui::RichText::new(format!("  {key} = {val:.2}"))
                                    .small()
                                    .color(BrandTheme::TEXT_PRIMARY),
                            );
                        }
                    }
                }
            }
        });

    editor_state.chain_panel_expanded = expanded.fully_open();
}

/// Renders the combat resolution panel in the play panel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_combat_panel<'a>(
    ui: &mut egui::Ui,
    active_combat: &mut ActiveCombat,
    crt: &CombatResultsTable,
    modifiers: &CombatModifierRegistry,
    selected_unit: &SelectedUnit,
    entity_types: &EntityTypeRegistry,
    editor_state: &mut EditorState,
    unit_lookup: &dyn Fn(Entity) -> Option<&'a EntityData>,
    in_combat_phase: bool,
) {
    use hexorder_contracts::mechanics::resolve_crt;
    use hexorder_contracts::simulation::{
        ColumnModifier, apply_column_shift, evaluate_column_modifiers, find_table_column,
    };

    ui.label(
        egui::RichText::new("Combat Resolution")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    if crt.table.columns.is_empty() || crt.table.rows.is_empty() {
        ui.label(
            egui::RichText::new("No CRT defined. Set up columns and rows in the Mechanics tab.")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    // -- Attacker assignment --
    ui.horizontal(|ui| {
        ui.label("Attacker:");
        if let Some(atk) = active_combat.attacker {
            let name = unit_lookup(atk)
                .and_then(|ed| entity_types.get(ed.entity_type_id))
                .map_or("(unknown)".to_string(), |et| et.name.clone());
            ui.label(
                egui::RichText::new(&name)
                    .strong()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
        } else {
            ui.label(egui::RichText::new("None").color(BrandTheme::TEXT_SECONDARY));
        }
    });
    if selected_unit.entity.is_some() && ui.button("Set Attacker from Selection").clicked() {
        active_combat.attacker = selected_unit.entity;
        // Reset resolution state when combatants change.
        active_combat.die_roll = None;
        active_combat.outcome = None;
    }

    ui.add_space(4.0);

    // -- Defender assignment --
    ui.horizontal(|ui| {
        ui.label("Defender:");
        if let Some(def) = active_combat.defender {
            let name = unit_lookup(def)
                .and_then(|ed| entity_types.get(ed.entity_type_id))
                .map_or("(unknown)".to_string(), |et| et.name.clone());
            ui.label(
                egui::RichText::new(&name)
                    .strong()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
        } else {
            ui.label(egui::RichText::new("None").color(BrandTheme::TEXT_SECONDARY));
        }
    });
    if selected_unit.entity.is_some() && ui.button("Set Defender from Selection").clicked() {
        active_combat.defender = selected_unit.entity;
        active_combat.die_roll = None;
        active_combat.outcome = None;
    }

    ui.add_space(8.0);

    // -- Strength inputs --
    ui.label(
        egui::RichText::new("Strengths")
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
    );
    ui.horizontal(|ui| {
        ui.label("ATK:");
        ui.add(egui::DragValue::new(&mut editor_state.combat_attacker_strength).speed(0.5));
        ui.label("DEF:");
        ui.add(egui::DragValue::new(&mut editor_state.combat_defender_strength).speed(0.5));
    });

    let atk_str = editor_state.combat_attacker_strength;
    let def_str = editor_state.combat_defender_strength;

    // -- Odds display --
    if def_str > 0.0 {
        let ratio = atk_str / def_str;
        ui.label(
            egui::RichText::new(format!("Odds: {ratio:.2}:1")).color(BrandTheme::TEXT_PRIMARY),
        );
    }

    // -- Column lookup --
    let base_column = find_table_column(atk_str, def_str, &crt.table.columns);
    if let Some(col_idx) = base_column {
        ui.label(
            egui::RichText::new(format!(
                "Base column: {} ({})",
                crt.table.columns[col_idx].label, col_idx
            ))
            .small()
            .color(BrandTheme::TEXT_PRIMARY),
        );
    } else {
        ui.label(
            egui::RichText::new("Below minimum column threshold")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
    }

    // -- Modifier breakdown --
    if !modifiers.modifiers.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Modifiers")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        let column_modifiers: Vec<ColumnModifier> = modifiers
            .modifiers
            .iter()
            .map(|m| ColumnModifier {
                name: m.name.clone(),
                column_shift: m.column_shift,
                cap: m.cap,
                priority: m.priority.max(0) as u32,
            })
            .collect();
        let (total_shift, modifier_display) =
            evaluate_column_modifiers(&column_modifiers, crt.table.columns.len());
        for (name, shift) in &modifier_display {
            let sign = if *shift >= 0 { "+" } else { "" };
            ui.label(
                egui::RichText::new(format!("  {name}: {sign}{shift}"))
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
        }
        ui.label(
            egui::RichText::new(format!("  Total shift: {total_shift:+}"))
                .small()
                .strong()
                .color(BrandTheme::TEXT_PRIMARY),
        );

        // Show final column after shift.
        if let Some(base_col) = base_column {
            let final_col = apply_column_shift(base_col, total_shift, crt.table.columns.len());
            active_combat.resolved_column = Some(final_col);
            active_combat.total_shift = total_shift;
            ui.label(
                egui::RichText::new(format!(
                    "Final column: {} ({})",
                    crt.table.columns[final_col].label, final_col
                ))
                .small()
                .strong()
                .color(BrandTheme::ACCENT_TEAL),
            );
        }
    } else if let Some(base_col) = base_column {
        active_combat.resolved_column = Some(base_col);
        active_combat.total_shift = 0;
    }

    ui.add_space(8.0);

    // -- Die Roll --
    if !in_combat_phase {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Advance to a Combat phase to resolve attacks.")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
    }
    let can_resolve = base_column.is_some() && in_combat_phase;
    ui.add_enabled_ui(can_resolve, |ui| {
        if ui.button("Roll Die \u{1F3B2}").clicked() {
            let roll = rand_die_roll();
            active_combat.die_roll = Some(roll);

            // Full resolution.
            let final_col = active_combat.resolved_column.unwrap_or(0);
            let shift = active_combat.total_shift;

            // Resolve using the shifted column: build a temporary single-column CRT
            // or use resolve_crt with original strengths and apply shift after.
            if let Some(resolution) = resolve_crt(crt, atk_str, def_str, roll) {
                // Apply column shift to get the actual outcome.
                let shifted_col =
                    apply_column_shift(resolution.column_index, shift, crt.table.columns.len());
                if let Some(row_outcomes) = crt.outcomes.get(resolution.row_index)
                    && let Some(outcome) = row_outcomes.get(shifted_col)
                {
                    active_combat.resolved_row = Some(resolution.row_index);
                    active_combat.outcome = Some(outcome.clone());
                }
            } else {
                // Column matched but row might not — try with just the die roll.
                let _ = final_col; // already stored in resolved_column
                active_combat.outcome = None;
            }
        }
    });

    // -- Result display --
    if let Some(roll) = active_combat.die_roll {
        ui.horizontal(|ui| {
            ui.label("Die roll:");
            ui.label(
                egui::RichText::new(format!("{roll}"))
                    .strong()
                    .size(18.0)
                    .color(BrandTheme::ACCENT_AMBER),
            );
        });
    }

    if let Some(outcome) = &active_combat.outcome {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("Result: {}", outcome.label))
                .strong()
                .size(18.0)
                .color(BrandTheme::SUCCESS),
        );

        if let Some(effect) = &outcome.effect {
            let effect_text = match effect {
                hexorder_contracts::mechanics::OutcomeEffect::NoEffect => "No effect".to_string(),
                hexorder_contracts::mechanics::OutcomeEffect::Retreat { hexes } => {
                    format!("Defender retreats {hexes} hex(es)")
                }
                hexorder_contracts::mechanics::OutcomeEffect::StepLoss { steps } => {
                    format!("Defender loses {steps} step(s)")
                }
                hexorder_contracts::mechanics::OutcomeEffect::AttackerStepLoss { steps } => {
                    format!("Attacker loses {steps} step(s)")
                }
                hexorder_contracts::mechanics::OutcomeEffect::Exchange {
                    attacker_steps,
                    defender_steps,
                } => format!("Exchange: ATK -{attacker_steps}, DEF -{defender_steps}"),
                hexorder_contracts::mechanics::OutcomeEffect::AttackerEliminated => {
                    "Attacker eliminated".to_string()
                }
                hexorder_contracts::mechanics::OutcomeEffect::DefenderEliminated => {
                    "Defender eliminated".to_string()
                }
            };
            ui.label(
                egui::RichText::new(effect_text)
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
        }
    }

    // -- Post-resolution movement preview --
    if let Some(outcome) = &active_combat.outcome
        && let (Some(atk), Some(def)) = (active_combat.attacker, active_combat.defender)
    {
        let default_rules = build_default_post_resolution_rules(outcome);
        if !default_rules.is_empty() {
            let pending = evaluate_post_resolution(&default_rules, outcome, atk, def);
            if !pending.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Post-Resolution Movement")
                        .small()
                        .strong()
                        .color(BrandTheme::ACCENT_TEAL),
                );
                for pm in &pending {
                    let action_text = match pm.action {
                        PostResolutionAction::Advance => "Attacker advances (1 hex)".to_string(),
                        PostResolutionAction::Retreat => {
                            format!("Defender retreats ({} hex)", pm.movement_range)
                        }
                        PostResolutionAction::Hold => "No movement".to_string(),
                    };
                    ui.label(
                        egui::RichText::new(format!("  \u{2192} {action_text}"))
                            .small()
                            .color(BrandTheme::TEXT_PRIMARY),
                    );
                }
            }
        }
    }

    ui.add_space(8.0);

    // -- Clear button --
    if ui.button("Clear Combat").clicked() {
        *active_combat = ActiveCombat::default();
        editor_state.combat_attacker_strength = 0.0;
        editor_state.combat_defender_strength = 0.0;
    }
}

/// Builds default post-resolution rules from the combat outcome's effect.
///
/// This derives movement rules from the structured effect rather than requiring
/// designers to define post-resolution rules separately (which is a future scope).
fn build_default_post_resolution_rules(
    outcome: &hexorder_contracts::mechanics::CombatOutcome,
) -> Vec<PostResolutionRule> {
    let Some(effect) = &outcome.effect else {
        return Vec::new();
    };
    match effect {
        hexorder_contracts::mechanics::OutcomeEffect::Retreat { hexes } => {
            vec![PostResolutionRule {
                action: PostResolutionAction::Retreat,
                trigger_effects: vec![],
                movement_range: *hexes,
            }]
        }
        hexorder_contracts::mechanics::OutcomeEffect::DefenderEliminated => {
            vec![PostResolutionRule {
                action: PostResolutionAction::Advance,
                trigger_effects: vec![],
                movement_range: 1,
            }]
        }
        _ => Vec::new(),
    }
}

/// Generates a random die roll in range [1, 6].
fn rand_die_roll() -> u32 {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0u64, |d| d.as_nanos() as u64);
    // Simple LCG for a quick 1-6 value; no external crate needed.
    ((seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1) >> 33) % 6 + 1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_die_roll_returns_value_in_range() {
        for _ in 0..100 {
            let roll = rand_die_roll();
            assert!((1..=6).contains(&roll), "roll {roll} out of range [1, 6]");
        }
    }
}
