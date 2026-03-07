//! Rules tab rendering — validation, mechanics, and inspector.

use bevy_egui::egui;

use hexorder_contracts::game_system::TypeId;
use hexorder_contracts::game_system::{
    EntityData, EntityTypeRegistry, EnumRegistry, PropertyType, PropertyValue, StructRegistry,
};
use hexorder_contracts::hex_grid::{
    HexPosition, InfluenceRule, InfluenceRuleRegistry, MovementCostMatrix, StackingRule,
};
use hexorder_contracts::mechanics::{
    CombatModifierRegistry, CombatResultsTable, ModifierSource, PhaseType, PlayerOrder,
    TurnStructure,
};
use hexorder_contracts::simulation::{ColumnType, find_table_column, find_table_row};
use hexorder_contracts::validation::SchemaValidation;

use super::actions::{bevy_color_to_egui, egui_color_to_bevy};
use super::components::{BrandTheme, EditorAction, EditorState};

pub(crate) fn render_validation_tab(ui: &mut egui::Ui, validation: &SchemaValidation) {
    ui.label(
        egui::RichText::new("Validation")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    if validation.is_valid {
        ui.label(egui::RichText::new("Schema Valid").color(BrandTheme::SUCCESS));
    } else {
        ui.label(
            egui::RichText::new(format!("{} Error(s)", validation.errors.len()))
                .color(BrandTheme::DANGER),
        );
    }

    if !validation.errors.is_empty() {
        ui.add_space(4.0);
        for error in &validation.errors {
            ui.group(|ui| {
                let category_str = match error.category {
                    hexorder_contracts::validation::SchemaErrorCategory::DanglingReference => {
                        "Dangling Ref"
                    }
                    hexorder_contracts::validation::SchemaErrorCategory::RoleMismatch => {
                        "Role Mismatch"
                    }
                    hexorder_contracts::validation::SchemaErrorCategory::PropertyMismatch => {
                        "Prop Mismatch"
                    }
                    hexorder_contracts::validation::SchemaErrorCategory::MissingBinding => {
                        "Missing Binding"
                    }
                    hexorder_contracts::validation::SchemaErrorCategory::InvalidExpression => {
                        "Invalid Expr"
                    }
                };
                ui.label(
                    egui::RichText::new(category_str)
                        .small()
                        .color(BrandTheme::ACCENT_AMBER),
                );
                ui.label(egui::RichText::new(&error.message).small());
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_mechanics_tab(
    ui: &mut egui::Ui,
    turn_structure: &TurnStructure,
    crt: &CombatResultsTable,
    modifiers: &CombatModifierRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    // ── Turn Structure ──────────────────────────────────────────────
    ui.label(
        egui::RichText::new("Turn Structure")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    // Player order selector.
    ui.horizontal(|ui| {
        ui.label("Player Order:");
        let order_labels = ["Alternating", "Simultaneous", "Activation"];
        let current = match turn_structure.player_order {
            PlayerOrder::Alternating => 0,
            PlayerOrder::Simultaneous => 1,
            PlayerOrder::ActivationBased => 2,
        };
        for (i, label) in order_labels.iter().enumerate() {
            if ui.selectable_label(current == i, *label).clicked() && current != i {
                let order = match i {
                    1 => PlayerOrder::Simultaneous,
                    2 => PlayerOrder::ActivationBased,
                    _ => PlayerOrder::Alternating,
                };
                actions.push(EditorAction::SetPlayerOrder { order });
            }
        }
    });
    ui.add_space(4.0);

    // Phase list.
    ui.label(
        egui::RichText::new(format!("Phases ({})", turn_structure.phases.len()))
            .color(BrandTheme::TEXT_SECONDARY),
    );
    let mut phase_action: Option<EditorAction> = None;
    for (i, phase) in turn_structure.phases.iter().enumerate() {
        ui.horizontal(|ui| {
            let type_label = match phase.phase_type {
                PhaseType::Movement => "Mov",
                PhaseType::Combat => "Cbt",
                PhaseType::Admin => "Adm",
            };
            ui.label(
                egui::RichText::new(format!("{}.", i + 1))
                    .small()
                    .color(BrandTheme::TEXT_TERTIARY),
            );
            ui.label(&phase.name);
            ui.label(
                egui::RichText::new(format!("[{type_label}]"))
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                    .clicked()
                {
                    phase_action = Some(EditorAction::RemovePhase { id: phase.id });
                }
                if i > 0
                    && ui
                        .small_button(
                            egui::RichText::new("\u{2191}").color(BrandTheme::TEXT_PRIMARY),
                        )
                        .clicked()
                {
                    phase_action = Some(EditorAction::MovePhaseUp { id: phase.id });
                }
                if i + 1 < turn_structure.phases.len()
                    && ui
                        .small_button(
                            egui::RichText::new("\u{2193}").color(BrandTheme::TEXT_PRIMARY),
                        )
                        .clicked()
                {
                    phase_action = Some(EditorAction::MovePhaseDown { id: phase.id });
                }
            });
        });
    }
    if let Some(action) = phase_action {
        actions.push(action);
    }

    // Add phase form.
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut editor_state.new_phase_name);
    });
    ui.horizontal(|ui| {
        ui.label("Type:");
        let type_labels = ["Movement", "Combat", "Admin"];
        for (i, label) in type_labels.iter().enumerate() {
            ui.selectable_value(&mut editor_state.new_phase_type_index, i, *label);
        }
    });
    if ui.button("Add Phase").clicked() && !editor_state.new_phase_name.trim().is_empty() {
        let phase_type = match editor_state.new_phase_type_index {
            1 => PhaseType::Combat,
            2 => PhaseType::Admin,
            _ => PhaseType::Movement,
        };
        actions.push(EditorAction::AddPhase {
            name: editor_state.new_phase_name.trim().to_string(),
            phase_type,
        });
        editor_state.new_phase_name.clear();
    }

    ui.separator();

    // ── Combat Results Table ────────────────────────────────────────
    ui.label(
        egui::RichText::new("Combat Results Table")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(&crt.name)
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
    );

    // Column headers.
    ui.label(
        egui::RichText::new(format!("Columns ({})", crt.table.columns.len()))
            .color(BrandTheme::TEXT_SECONDARY),
    );
    for (i, col) in crt.table.columns.iter().enumerate() {
        ui.horizontal(|ui| {
            let type_label = match col.column_type {
                ColumnType::Ratio | ColumnType::Direct => "ratio",
                ColumnType::Differential => "diff",
            };
            ui.label(
                egui::RichText::new(&col.label)
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("({type_label} \u{2265}{:.1})", col.threshold))
                    .small()
                    .color(BrandTheme::TEXT_TERTIARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                    .clicked()
                {
                    actions.push(EditorAction::RemoveCrtColumn { index: i });
                }
            });
        });
    }

    // Add column form.
    ui.horizontal(|ui| {
        ui.label("Label:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.new_crt_col_label).desired_width(60.0));
        ui.label("Thr:");
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.new_crt_col_threshold).desired_width(40.0),
        );
    });
    ui.horizontal(|ui| {
        let col_type_labels = ["Ratio", "Diff"];
        for (i, label) in col_type_labels.iter().enumerate() {
            ui.selectable_value(&mut editor_state.new_crt_col_type_index, i, *label);
        }
        if ui.button("Add Col").clicked() && !editor_state.new_crt_col_label.trim().is_empty() {
            let threshold = editor_state
                .new_crt_col_threshold
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0);
            let column_type = match editor_state.new_crt_col_type_index {
                1 => ColumnType::Differential,
                _ => ColumnType::Ratio,
            };
            actions.push(EditorAction::AddCrtColumn {
                label: editor_state.new_crt_col_label.trim().to_string(),
                column_type,
                threshold,
            });
            editor_state.new_crt_col_label.clear();
            editor_state.new_crt_col_threshold.clear();
        }
    });

    ui.add_space(4.0);

    // Row headers.
    ui.label(
        egui::RichText::new(format!("Rows ({})", crt.table.rows.len()))
            .color(BrandTheme::TEXT_SECONDARY),
    );
    for (i, row) in crt.table.rows.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&row.label)
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("(die {}-{})", row.value_min, row.value_max))
                    .small()
                    .color(BrandTheme::TEXT_TERTIARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                    .clicked()
                {
                    actions.push(EditorAction::RemoveCrtRow { index: i });
                }
            });
        });
    }

    // Add row form.
    ui.horizontal(|ui| {
        ui.label("Label:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.new_crt_row_label).desired_width(40.0));
        ui.label("Die:");
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.new_crt_row_die_min).desired_width(30.0),
        );
        ui.label("-");
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.new_crt_row_die_max).desired_width(30.0),
        );
        if ui.button("Add Row").clicked() && !editor_state.new_crt_row_label.trim().is_empty() {
            let die_min = editor_state
                .new_crt_row_die_min
                .trim()
                .parse::<u32>()
                .unwrap_or(1);
            let die_max = editor_state
                .new_crt_row_die_max
                .trim()
                .parse::<u32>()
                .unwrap_or(die_min);
            actions.push(EditorAction::AddCrtRow {
                label: editor_state.new_crt_row_label.trim().to_string(),
                die_min,
                die_max,
            });
            editor_state.new_crt_row_label.clear();
            editor_state.new_crt_row_die_min.clear();
            editor_state.new_crt_row_die_max.clear();
        }
    });

    ui.add_space(4.0);

    // Outcome grid (editable) with live preview.
    if !crt.table.columns.is_empty() && !crt.table.rows.is_empty() {
        // Sync edit buffer when CRT dimensions change.
        let num_rows = crt.table.rows.len();
        let num_cols = crt.table.columns.len();
        if editor_state.crt_outcome_labels.len() != num_rows
            || editor_state
                .crt_outcome_labels
                .first()
                .is_some_and(|r| r.len() != num_cols)
        {
            editor_state.crt_outcome_labels = (0..num_rows)
                .map(|ri| {
                    (0..num_cols)
                        .map(|ci| {
                            crt.outcomes
                                .get(ri)
                                .and_then(|r| r.get(ci))
                                .map_or_else(|| "--".to_string(), |o| o.label.clone())
                        })
                        .collect()
                })
                .collect();
        }

        ui.label(
            egui::RichText::new("Outcome Grid")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );

        // Live preview: resolve test inputs to column/row.
        let preview_col = find_table_column(
            editor_state.table_test_input_a,
            editor_state.table_test_input_b,
            &crt.table.columns,
        );
        let preview_row = find_table_row(editor_state.table_test_die_roll, &crt.table.rows);

        egui::Grid::new("crt_outcome_grid")
            .striped(true)
            .show(ui, |ui| {
                // Header row.
                ui.label("");
                for (ci, col) in crt.table.columns.iter().enumerate() {
                    let is_highlight_col = preview_col == Some(ci);
                    let color = if is_highlight_col {
                        BrandTheme::ACCENT_AMBER
                    } else {
                        BrandTheme::ACCENT_TEAL
                    };
                    ui.label(
                        egui::RichText::new(&col.label)
                            .small()
                            .strong()
                            .color(color),
                    );
                }
                ui.end_row();

                // Data rows with editable cells.
                for (ri, row) in crt.table.rows.iter().enumerate() {
                    let is_highlight_row = preview_row == Some(ri);
                    let row_color = if is_highlight_row {
                        BrandTheme::ACCENT_AMBER
                    } else {
                        BrandTheme::ACCENT_TEAL
                    };
                    ui.label(
                        egui::RichText::new(&row.label)
                            .small()
                            .strong()
                            .color(row_color),
                    );
                    for ci in 0..num_cols {
                        let is_resolved_cell = preview_col == Some(ci) && preview_row == Some(ri);
                        let cell = &mut editor_state.crt_outcome_labels[ri][ci];
                        let mut text_edit = egui::TextEdit::singleline(cell)
                            .desired_width(28.0)
                            .font(egui::TextStyle::Small);
                        if is_resolved_cell {
                            text_edit = text_edit.text_color(BrandTheme::ACCENT_AMBER);
                        }
                        let response = ui.add(text_edit);
                        if response.lost_focus() || response.changed() {
                            actions.push(EditorAction::SetCrtOutcome {
                                row: ri,
                                col: ci,
                                label: cell.clone(),
                            });
                        }
                    }
                    ui.end_row();
                }
            });

        // Test inputs for live preview.
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Live Preview")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("A:").small());
            ui.add(
                egui::DragValue::new(&mut editor_state.table_test_input_a)
                    .range(0.0..=9999.0)
                    .speed(0.5),
            );
            ui.label(egui::RichText::new("B:").small());
            ui.add(
                egui::DragValue::new(&mut editor_state.table_test_input_b)
                    .range(0.0..=9999.0)
                    .speed(0.5),
            );
            ui.label(egui::RichText::new("Die:").small());
            ui.add(egui::DragValue::new(&mut editor_state.table_test_die_roll).range(1..=100));
        });
        // Show resolved result.
        if let (Some(ci), Some(ri)) = (preview_col, preview_row) {
            let result_label = crt
                .outcomes
                .get(ri)
                .and_then(|r| r.get(ci))
                .map_or("--", |o| o.label.as_str());
            ui.label(
                egui::RichText::new(format!(
                    "→ {} [{}] × [{}] = {}",
                    "Result:", crt.table.columns[ci].label, crt.table.rows[ri].label, result_label
                ))
                .small()
                .strong()
                .color(BrandTheme::ACCENT_AMBER),
            );
        }
    }

    ui.separator();

    // ── Combat Modifiers ────────────────────────────────────────────
    ui.label(
        egui::RichText::new("Combat Modifiers")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    for modifier in &modifiers.modifiers {
        ui.horizontal(|ui| {
            let source_label = match &modifier.source {
                ModifierSource::DefenderTerrain => "DefTerr".to_string(),
                ModifierSource::AttackerTerrain => "AtkTerr".to_string(),
                ModifierSource::AttackerProperty(p) => format!("AtkProp({p})"),
                ModifierSource::DefenderProperty(p) => format!("DefProp({p})"),
                ModifierSource::Custom(s) => format!("Custom({s})"),
            };
            let shift_str = if modifier.column_shift >= 0 {
                format!("+{}", modifier.column_shift)
            } else {
                format!("{}", modifier.column_shift)
            };
            ui.label(&modifier.name);
            ui.label(
                egui::RichText::new(format!(
                    "{shift_str} [{source_label}] p{}",
                    modifier.priority
                ))
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                    .clicked()
                {
                    actions.push(EditorAction::RemoveCombatModifier { id: modifier.id });
                }
            });
        });
    }

    // Add modifier form.
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut editor_state.new_modifier_name);
    });
    ui.horizontal(|ui| {
        ui.label("Source:");
        let source_labels = ["Def.Terrain", "Atk.Terrain", "Custom"];
        for (i, label) in source_labels.iter().enumerate() {
            ui.selectable_value(&mut editor_state.new_modifier_source_index, i, *label);
        }
    });
    if editor_state.new_modifier_source_index == 2 {
        ui.horizontal(|ui| {
            ui.label("Desc:");
            ui.text_edit_singleline(&mut editor_state.new_modifier_custom_source);
        });
    }
    ui.horizontal(|ui| {
        ui.label("Shift:");
        ui.add(egui::DragValue::new(&mut editor_state.new_modifier_shift).range(-10..=10));
        ui.label("Priority:");
        ui.add(egui::DragValue::new(&mut editor_state.new_modifier_priority).range(0..=100));
    });
    if ui.button("Add Modifier").clicked() && !editor_state.new_modifier_name.trim().is_empty() {
        let source = match editor_state.new_modifier_source_index {
            1 => ModifierSource::AttackerTerrain,
            2 => ModifierSource::Custom(editor_state.new_modifier_custom_source.trim().to_string()),
            _ => ModifierSource::DefenderTerrain,
        };
        actions.push(EditorAction::AddCombatModifier {
            name: editor_state.new_modifier_name.trim().to_string(),
            source,
            shift: editor_state.new_modifier_shift,
            priority: editor_state.new_modifier_priority,
        });
        editor_state.new_modifier_name.clear();
        editor_state.new_modifier_custom_source.clear();
        editor_state.new_modifier_shift = 0;
        editor_state.new_modifier_priority = 0;
    }
}

/// Renders the spatial influence rules editor.
pub(crate) fn render_influence_rules(
    ui: &mut egui::Ui,
    influence_rules: &mut InfluenceRuleRegistry,
    entity_types: &EntityTypeRegistry,
    editor_state: &mut EditorState,
) {
    ui.label(
        egui::RichText::new("Spatial Influence Rules")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    let mut remove_idx = None;
    for (i, rule) in influence_rules.rules.iter().enumerate() {
        let type_name = entity_types
            .get(rule.entity_type_id)
            .map_or("Unknown", |et| et.name.as_str());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(type_name)
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("range {} +{} MP", rule.range, rule.cost_modifier))
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                    .clicked()
                {
                    remove_idx = Some(i);
                }
            });
        });
    }
    if let Some(idx) = remove_idx {
        influence_rules.rules.remove(idx);
    }

    // Add rule form.
    ui.add_space(4.0);
    let token_types: Vec<_> = entity_types
        .types
        .iter()
        .filter(|et| et.role == hexorder_contracts::game_system::EntityRole::Token)
        .collect();

    if token_types.is_empty() {
        ui.label(
            egui::RichText::new("No token entity types defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
    } else {
        ui.horizontal(|ui| {
            ui.label("Type:");
            let selected_name = editor_state
                .new_influence_type_idx
                .and_then(|idx| token_types.get(idx))
                .map_or("Select...", |et| et.name.as_str());
            egui::ComboBox::from_id_salt("influence_type_picker")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for (idx, et) in token_types.iter().enumerate() {
                        if ui
                            .selectable_label(
                                editor_state.new_influence_type_idx == Some(idx),
                                &et.name,
                            )
                            .clicked()
                        {
                            editor_state.new_influence_type_idx = Some(idx);
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Range:");
            ui.add(
                egui::DragValue::new(&mut editor_state.new_influence_range)
                    .range(1..=5)
                    .speed(0.1),
            );
            ui.label("Cost:");
            ui.add(
                egui::DragValue::new(&mut editor_state.new_influence_cost)
                    .range(1..=20)
                    .speed(0.1),
            );
        });
        if ui.button("Add Rule").clicked()
            && let Some(idx) = editor_state.new_influence_type_idx
            && let Some(et) = token_types.get(idx)
        {
            influence_rules.rules.push(InfluenceRule {
                id: TypeId::new(),
                entity_type_id: et.id,
                range: editor_state.new_influence_range,
                cost_modifier: i64::from(editor_state.new_influence_cost),
            });
            editor_state.new_influence_type_idx = None;
        }
    }
}

/// Renders the stacking constraint editor.
pub(crate) fn render_stacking_rule(
    ui: &mut egui::Ui,
    stacking_rule: &mut StackingRule,
    entity_types: &EntityTypeRegistry,
    editor_state: &mut EditorState,
) {
    ui.label(
        egui::RichText::new("Stacking Constraint")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Max units per hex:");
        let mut max = stacking_rule.max_units as i32;
        if ui
            .add(egui::DragValue::new(&mut max).range(0..=20).speed(0.1))
            .changed()
        {
            stacking_rule.max_units = max.max(0) as u32;
        }
        if stacking_rule.max_units == 0 {
            ui.label(
                egui::RichText::new("(unlimited)")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
        }
    });

    if stacking_rule.is_active() {
        // Show exempt types.
        if !stacking_rule.exempt_type_ids.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Exempt types:")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            let mut remove_idx = None;
            for (i, type_id) in stacking_rule.exempt_type_ids.iter().enumerate() {
                let name = entity_types
                    .get(*type_id)
                    .map_or("Unknown", |et| et.name.as_str());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .small()
                            .color(BrandTheme::TEXT_PRIMARY),
                    );
                    if ui
                        .small_button(egui::RichText::new("x").color(BrandTheme::DANGER))
                        .clicked()
                    {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                stacking_rule.exempt_type_ids.remove(idx);
            }
        }

        // Add exempt type picker.
        let token_types: Vec<_> = entity_types
            .types
            .iter()
            .filter(|et| {
                et.role == hexorder_contracts::game_system::EntityRole::Token
                    && !stacking_rule.exempt_type_ids.contains(&et.id)
            })
            .collect();

        if !token_types.is_empty() {
            ui.horizontal(|ui| {
                let selected_name = editor_state
                    .new_stacking_exempt_idx
                    .and_then(|idx| token_types.get(idx))
                    .map_or("Select...", |et| et.name.as_str());
                egui::ComboBox::from_id_salt("stacking_exempt_picker")
                    .selected_text(selected_name)
                    .show_ui(ui, |ui| {
                        for (idx, et) in token_types.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    editor_state.new_stacking_exempt_idx == Some(idx),
                                    &et.name,
                                )
                                .clicked()
                            {
                                editor_state.new_stacking_exempt_idx = Some(idx);
                            }
                        }
                    });
                if ui.button("Add Exempt").clicked()
                    && let Some(idx) = editor_state.new_stacking_exempt_idx
                    && let Some(et) = token_types.get(idx)
                {
                    stacking_rule.exempt_type_ids.push(et.id);
                    editor_state.new_stacking_exempt_idx = None;
                }
            });
        }
    }
}

/// Renders the movement cost matrix section in the Mechanics tab.
///
/// Shows a classification property picker (enum properties on Token types),
/// then a 2D matrix grid: rows = `BoardPosition` entity types (terrain),
/// columns = enum option values. Each cell is a `DragValue` for the cost.
pub(crate) fn render_movement_cost_matrix(
    ui: &mut egui::Ui,
    matrix: &mut MovementCostMatrix,
    registry: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
) {
    ui.heading("Movement Cost Matrix");
    ui.separator();

    // Collect all enum properties from Token types for the classification picker.
    let enum_props: Vec<(TypeId, String, TypeId)> = registry
        .types
        .iter()
        .filter(|t| t.role == hexorder_contracts::game_system::EntityRole::Token)
        .flat_map(|t| {
            t.properties.iter().filter_map(|p| {
                if let PropertyType::Enum(enum_id) = &p.property_type {
                    Some((p.id, p.name.clone(), *enum_id))
                } else {
                    None
                }
            })
        })
        .collect();

    if enum_props.is_empty() {
        ui.label("No enum properties on Token types. Add an enum property (e.g., \"Movement Mode\") to enable the matrix.");
        return;
    }

    // Classification property picker.
    let current_label = matrix
        .classification_property_id
        .and_then(|id| enum_props.iter().find(|(pid, _, _)| *pid == id))
        .map_or("(none — matrix inactive)", |(_, name, _)| name.as_str());

    ui.horizontal(|ui| {
        ui.label("Classification:");
        egui::ComboBox::from_id_salt("mcm_classification")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        matrix.classification_property_id.is_none(),
                        "(none — matrix inactive)",
                    )
                    .clicked()
                {
                    matrix.classification_property_id = None;
                }
                for (prop_id, name, _) in &enum_props {
                    if ui
                        .selectable_label(matrix.classification_property_id == Some(*prop_id), name)
                        .clicked()
                    {
                        matrix.classification_property_id = Some(*prop_id);
                    }
                }
            });
    });

    // If no classification selected, stop here.
    let Some(prop_id) = matrix.classification_property_id else {
        return;
    };

    // Find the enum definition for the selected property.
    let Some((_, _, enum_id)) = enum_props.iter().find(|(pid, _, _)| *pid == prop_id) else {
        return;
    };
    let Some(enum_def) = enum_registry.definitions.get(enum_id) else {
        ui.label("Enum definition not found.");
        return;
    };

    if enum_def.options.is_empty() {
        ui.label("Enum has no options defined.");
        return;
    }

    // Collect terrain types (BoardPosition role).
    let terrain_types: Vec<(TypeId, &str)> = registry
        .types
        .iter()
        .filter(|t| t.role == hexorder_contracts::game_system::EntityRole::BoardPosition)
        .map(|t| (t.id, t.name.as_str()))
        .collect();

    if terrain_types.is_empty() {
        ui.label("No terrain types defined.");
        return;
    }

    // Render the matrix as a grid.
    ui.add_space(4.0);
    egui::Grid::new("movement_cost_matrix_grid")
        .striped(true)
        .show(ui, |ui| {
            // Header row: empty corner + classification options.
            ui.label("");
            for option in &enum_def.options {
                ui.label(option);
            }
            ui.end_row();

            // Data rows: terrain type name + cost cells.
            for (terrain_id, terrain_name) in &terrain_types {
                ui.label(*terrain_name);
                for option in &enum_def.options {
                    let current = matrix.get_cost(*terrain_id, option).unwrap_or(0);
                    let mut val = current;
                    let response = ui.add(egui::DragValue::new(&mut val).range(0..=99).speed(0.1));
                    if response.changed() {
                        matrix.set_cost(*terrain_id, option.clone(), val);
                    }
                }
                ui.end_row();
            }
        });
}

/// Renders the tile inspector panel inside the Inspector dock tab.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_inspector(
    ui: &mut egui::Ui,
    position: Option<HexPosition>,
    entity_data: Option<&mut EntityData>,
    registry: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
) {
    egui::CollapsingHeader::new(
        egui::RichText::new("Tile Inspector")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    )
    .default_open(true)
    .show(ui, |ui| {
        let Some(pos) = position else {
            ui.label(egui::RichText::new("No tile selected").color(BrandTheme::TEXT_SECONDARY));
            return;
        };

        ui.label(egui::RichText::new(format!("Position: ({}, {})", pos.q, pos.r)).monospace());

        let Some(entity_data) = entity_data else {
            ui.label("No cell data");
            return;
        };

        // Cell type name
        let type_name = registry
            .get(entity_data.entity_type_id)
            .map_or_else(|| "Unknown".to_string(), |et| et.name.clone());
        ui.label(format!("Type: {type_name}"));

        // Property value editors
        let prop_defs: Vec<_> = registry
            .get(entity_data.entity_type_id)
            .map(|et| et.properties.clone())
            .unwrap_or_default();

        if prop_defs.is_empty() {
            ui.label(
                egui::RichText::new("No properties")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            return;
        }

        ui.separator();
        ui.label(egui::RichText::new("Properties").small());

        for prop_def in &prop_defs {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", prop_def.name));

                let value = entity_data
                    .properties
                    .entry(prop_def.id)
                    .or_insert_with(|| PropertyValue::default_for(&prop_def.property_type));

                render_property_value_editor(
                    ui,
                    value,
                    &prop_def.property_type,
                    enum_registry,
                    struct_registry,
                    registry,
                    0,
                );
            });
        }
    });
}

/// Renders the unit inspector panel inside the Inspector dock tab.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_unit_inspector(
    ui: &mut egui::Ui,
    entity_data: Option<&mut EntityData>,
    registry: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
    actions: &mut Vec<EditorAction>,
) {
    egui::CollapsingHeader::new(
        egui::RichText::new("Unit Inspector")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    )
    .default_open(true)
    .show(ui, |ui| {
        let Some(entity_data) = entity_data else {
            ui.label(egui::RichText::new("No unit selected").color(BrandTheme::TEXT_SECONDARY));
            return;
        };

        // Unit type name
        let type_name = registry
            .get(entity_data.entity_type_id)
            .map_or_else(|| "Unknown".to_string(), |et| et.name.clone());
        ui.label(format!("Unit Type: {type_name}"));

        // Property value editors
        let prop_defs: Vec<_> = registry
            .get(entity_data.entity_type_id)
            .map(|et| et.properties.clone())
            .unwrap_or_default();

        if !prop_defs.is_empty() {
            ui.separator();
            ui.label(egui::RichText::new("Properties").small());

            for prop_def in &prop_defs {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", prop_def.name));

                    let value = entity_data
                        .properties
                        .entry(prop_def.id)
                        .or_insert_with(|| PropertyValue::default_for(&prop_def.property_type));

                    render_property_value_editor(
                        ui,
                        value,
                        &prop_def.property_type,
                        enum_registry,
                        struct_registry,
                        registry,
                        0,
                    );
                });
            }
        }

        ui.separator();

        // Delete unit button
        if ui
            .button(egui::RichText::new("Delete Unit").color(BrandTheme::DANGER))
            .clicked()
        {
            actions.push(EditorAction::DeleteSelectedUnit);
        }
    });
}

/// Renders an inline property value editor for the Inspector dock tab.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_property_value_editor(
    ui: &mut egui::Ui,
    value: &mut PropertyValue,
    prop_type: &PropertyType,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
    entity_registry: &EntityTypeRegistry,
    depth: usize,
) {
    match value {
        PropertyValue::Bool(b) => {
            ui.checkbox(b, "");
        }
        PropertyValue::Int(i) => {
            ui.add(egui::DragValue::new(i));
        }
        PropertyValue::Float(f) => {
            ui.add(egui::DragValue::new(f).speed(0.1));
        }
        PropertyValue::String(s) => {
            ui.text_edit_singleline(s);
        }
        PropertyValue::Color(c) => {
            let mut c32 = bevy_color_to_egui(*c);
            if egui::color_picker::color_edit_button_srgba(
                ui,
                &mut c32,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                *c = egui_color_to_bevy(c32);
            }
        }
        PropertyValue::Enum(selected) => {
            if let PropertyType::Enum(enum_id) = prop_type {
                let options: Vec<String> = enum_registry
                    .get(*enum_id)
                    .map(|ed| ed.options.clone())
                    .unwrap_or_default();

                egui::ComboBox::from_id_salt(format!("ev_{enum_id:?}"))
                    .selected_text(selected.as_str())
                    .show_ui(ui, |ui| {
                        for option in &options {
                            ui.selectable_value(selected, option.clone(), option);
                        }
                    });
            }
        }
        PropertyValue::EntityRef(selected) => {
            let role_filter = if let PropertyType::EntityRef(filter) = prop_type {
                *filter
            } else {
                None
            };
            let candidates: Vec<_> = entity_registry
                .types
                .iter()
                .filter(|et| role_filter.is_none() || Some(et.role) == role_filter)
                .map(|et| (et.id, et.name.clone()))
                .collect();
            let selected_name = selected
                .and_then(|id| candidates.iter().find(|(eid, _)| *eid == id))
                .map_or("(none)".to_string(), |(_, n)| n.clone());
            egui::ComboBox::from_id_salt(format!("eref_{depth}"))
                .selected_text(&selected_name)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(selected.is_none(), "(none)").clicked() {
                        *selected = None;
                    }
                    for (eid, ename) in &candidates {
                        if ui
                            .selectable_label(*selected == Some(*eid), ename)
                            .clicked()
                        {
                            *selected = Some(*eid);
                        }
                    }
                });
        }
        PropertyValue::List(items) => {
            if depth >= 3 {
                ui.label(
                    egui::RichText::new("(nested limit)")
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                return;
            }
            let inner_type = if let PropertyType::List(inner) = prop_type {
                inner.as_ref()
            } else {
                return;
            };
            egui::CollapsingHeader::new(format!("List ({})", items.len()))
                .id_salt(format!("list_{depth}"))
                .show(ui, |ui| {
                    let mut remove_idx = None;
                    for (idx, item) in items.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("[{idx}]"));
                            render_property_value_editor(
                                ui,
                                item,
                                inner_type,
                                enum_registry,
                                struct_registry,
                                entity_registry,
                                depth + 1,
                            );
                            if ui.small_button("x").clicked() {
                                remove_idx = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = remove_idx {
                        items.remove(idx);
                    }
                    if ui
                        .button(egui::RichText::new("+ Add").color(BrandTheme::ACCENT_AMBER))
                        .clicked()
                    {
                        items.push(PropertyValue::default_for(inner_type));
                    }
                });
        }
        PropertyValue::Map(entries) => {
            if depth >= 3 {
                ui.label(
                    egui::RichText::new("(nested limit)")
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                return;
            }
            let (enum_id, value_type) = if let PropertyType::Map(eid, vt) = prop_type {
                (*eid, vt.as_ref())
            } else {
                return;
            };
            let enum_options = enum_registry
                .get(enum_id)
                .map(|ed| ed.options.clone())
                .unwrap_or_default();
            egui::CollapsingHeader::new(format!("Map ({})", entries.len()))
                .id_salt(format!("map_{depth}"))
                .show(ui, |ui| {
                    for opt in &enum_options {
                        let entry = entries.iter_mut().find(|(k, _)| k == opt);
                        if let Some((_, val)) = entry {
                            ui.horizontal(|ui| {
                                ui.label(format!("{opt}:"));
                                render_property_value_editor(
                                    ui,
                                    val,
                                    value_type,
                                    enum_registry,
                                    struct_registry,
                                    entity_registry,
                                    depth + 1,
                                );
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(format!("{opt}:"));
                                ui.label(
                                    egui::RichText::new("(default)")
                                        .small()
                                        .color(BrandTheme::TEXT_SECONDARY),
                                );
                                if ui.small_button("+").clicked() {
                                    entries.push((
                                        opt.clone(),
                                        PropertyValue::default_for(value_type),
                                    ));
                                }
                            });
                        }
                    }
                });
        }
        PropertyValue::Struct(fields) => {
            if depth >= 3 {
                ui.label(
                    egui::RichText::new("(nested limit)")
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                return;
            }
            let struct_id = if let PropertyType::Struct(sid) = prop_type {
                *sid
            } else {
                return;
            };
            let struct_def = struct_registry.get(struct_id);
            egui::CollapsingHeader::new(
                struct_def
                    .map_or("Struct", |sd| sd.name.as_str())
                    .to_string(),
            )
            .id_salt(format!("struct_{depth}"))
            .show(ui, |ui| {
                if let Some(sd) = struct_def {
                    for field in &sd.fields {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", field.name));
                            let val = fields.entry(field.id).or_insert_with(|| {
                                PropertyValue::default_for(&field.property_type)
                            });
                            render_property_value_editor(
                                ui,
                                val,
                                &field.property_type,
                                enum_registry,
                                struct_registry,
                                entity_registry,
                                depth + 1,
                            );
                        });
                    }
                } else {
                    ui.label(
                        egui::RichText::new("(unknown struct)")
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                }
            });
        }
        PropertyValue::IntRange(v) => {
            if let PropertyType::IntRange { min, max } = prop_type {
                ui.add(egui::DragValue::new(v).range(*min..=*max));
            } else {
                ui.add(egui::DragValue::new(v));
            }
        }
        PropertyValue::FloatRange(v) => {
            if let PropertyType::FloatRange { min, max } = prop_type {
                ui.add(egui::DragValue::new(v).range(*min..=*max).speed(0.1));
            } else {
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
        }
    }
}
