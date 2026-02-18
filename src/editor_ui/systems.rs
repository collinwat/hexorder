//! Systems for the `editor_ui` plugin.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::contracts::editor_ui::{EditorTool, ViewportMargins};
use crate::contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityType, EntityTypeRegistry,
    EnumDefinition, EnumRegistry, GameSystem, PropertyDefinition, PropertyType, PropertyValue,
    SelectedUnit, StructDefinition, StructRegistry, TypeId, UnitInstance,
};
use crate::contracts::hex_grid::{HexPosition, HexTile, SelectedHex};
use crate::contracts::ontology::{
    CompareOp, ConceptBinding, ConceptRegistry, ConceptRole, Constraint, ConstraintExpr,
    ConstraintRegistry, ModifyOperation, Relation, RelationEffect, RelationRegistry,
    RelationTrigger,
};
use crate::contracts::persistence::{
    AppScreen, CloseProjectEvent, LoadRequestEvent, NewProjectEvent, SaveRequestEvent, Workspace,
};
use crate::contracts::shortcuts::{CommandExecutedEvent, CommandId};
use crate::contracts::validation::SchemaValidation;

use crate::contracts::mechanics::{
    ActiveCombat, CombatModifierDefinition, CombatModifierRegistry, CombatOutcome,
    CombatResultsTable, CrtColumn, CrtColumnType, CrtRow, ModifierSource, Phase, PhaseType,
    PlayerOrder, TurnState, TurnStructure,
};

use super::components::{
    BrandTheme, EditorAction, EditorState, MechanicsParams, OntologyParams, OntologyTab,
    ProjectParams, SelectionParams,
};

/// Updates `ViewportMargins` from the actual egui panel layout.
/// Runs after the editor panel system so `available_rect()` reflects all panels.
pub fn update_viewport_margins(mut contexts: EguiContexts, mut margins: ResMut<ViewportMargins>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen = ctx.input(|i| i.viewport_rect());
    let available = ctx.available_rect();
    margins.left = available.left();
    margins.top = available.top();
    margins.right = screen.right() - available.right();
}

/// Debug inspector as a right-side panel.
/// Only compiled when the `inspector` feature is enabled.
/// Toggled via the `view.toggle_debug_panel` command (backtick key).
#[cfg(feature = "inspector")]
pub fn debug_inspector_panel(
    mut contexts: EguiContexts,
    margins: Res<ViewportMargins>,
    grid_config: Option<Res<crate::contracts::hex_grid::HexGridConfig>>,
    selected_hex: Res<SelectedHex>,
    camera_q: Query<(&Transform, &Projection), With<Camera3d>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    editor_state: Res<super::components::EditorState>,
) {
    if !editor_state.debug_panel_visible {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::right("debug_inspector")
        .default_width(240.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Debug Inspector")
                    .strong()
                    .size(13.0)
                    .color(BrandTheme::ACCENT_AMBER),
            );
            ui.label(
                egui::RichText::new("toggle: `")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok((transform, projection)) = camera_q.single() {
                    ui.collapsing("Camera", |ui| {
                        let t = transform.translation;
                        ui.label(format!("x: {:.3}", t.x));
                        ui.label(format!("y: {:.3}", t.y));
                        ui.label(format!("z: {:.3}", t.z));
                        if let Projection::Orthographic(ortho) = projection {
                            ui.label(format!("scale: {:.5}", ortho.scale));
                        }
                    });
                }

                ui.collapsing("Viewport Margins", |ui| {
                    ui.label(format!("left: {:.1}px", margins.left));
                    ui.label(format!("top: {:.1}px", margins.top));
                });

                if let Ok(window) = windows.single() {
                    ui.collapsing("Window / Viewport", |ui| {
                        ui.label(format!(
                            "window: {:.0} x {:.0}",
                            window.width(),
                            window.height()
                        ));
                        let vp_w = window.width() - margins.left;
                        let vp_h = window.height() - margins.top;
                        ui.label(format!("viewport: {:.0} x {:.0}", vp_w, vp_h));

                        let vp_cx = margins.left + vp_w / 2.0;
                        let vp_cy = margins.top + vp_h / 2.0;
                        let win_cx = window.width() / 2.0;
                        let win_cy = window.height() / 2.0;
                        let px_dx = vp_cx - win_cx;
                        let px_dy = vp_cy - win_cy;
                        ui.label(format!("vp center: ({:.0}, {:.0})", vp_cx, vp_cy));
                        ui.label(format!("win center: ({:.0}, {:.0})", win_cx, win_cy));
                        ui.label(format!("px offset: ({:.1}, {:.1})", px_dx, px_dy));

                        if let Ok((_, projection)) = camera_q.single() {
                            if let Projection::Orthographic(ortho) = projection {
                                let s = ortho.scale;
                                ui.label(format!(
                                    "world offset: ({:.3}, {:.3})",
                                    px_dx * s,
                                    px_dy * s
                                ));
                            }
                        }
                    });
                }

                if let Some(config) = &grid_config {
                    ui.collapsing("Grid Config", |ui| {
                        ui.label(format!("radius: {}", config.map_radius));
                        ui.label(format!(
                            "scale: ({:.2}, {:.2})",
                            config.layout.scale.x, config.layout.scale.y
                        ));
                    });
                }

                ui.collapsing("Selection", |ui| match selected_hex.position {
                    Some(pos) => {
                        ui.label(format!("hex: ({}, {})", pos.q, pos.r));
                        if let Some(config) = &grid_config {
                            let wp = config.layout.hex_to_world_pos(pos.to_hex());
                            ui.label(format!("world: ({:.2}, {:.2})", wp.x, wp.y));
                        }
                    }
                    None => {
                        ui.label("(none)");
                    }
                });
            });
        });
}

/// Configures the egui dark theme every frame. This is idempotent and cheap
/// (a few struct assignments). Running every frame guarantees the theme is
/// always applied, even after a window visibility change resets the context.
pub fn configure_theme(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BrandTheme::BG_PANEL;
    visuals.window_fill = BrandTheme::BG_PANEL;
    visuals.extreme_bg_color = BrandTheme::BG_DEEP;
    visuals.faint_bg_color = BrandTheme::BG_SURFACE;
    visuals.widgets.noninteractive.bg_fill = BrandTheme::WIDGET_NONINTERACTIVE;
    visuals.widgets.inactive.bg_fill = BrandTheme::WIDGET_INACTIVE;
    visuals.widgets.hovered.bg_fill = BrandTheme::WIDGET_HOVERED;
    visuals.widgets.active.bg_fill = BrandTheme::WIDGET_ACTIVE;
    visuals.selection.bg_fill = BrandTheme::ACCENT_TEAL;
    visuals.window_stroke = egui::Stroke::new(1.0, BrandTheme::BORDER_SUBTLE);

    // Text colors (fg_stroke)
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, BrandTheme::TEXT_SECONDARY);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(15.0, egui::FontFamily::Monospace),
    );
    ctx.set_style(style);
}

/// Launcher screen system. Renders a centered panel with New / Open buttons.
/// When "New Game System" is clicked, reveals an inline name input with Create/Cancel.
pub fn launcher_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);

            ui.label(
                egui::RichText::new("HEXORDER")
                    .size(32.0)
                    .strong()
                    .color(BrandTheme::ACCENT_AMBER),
            );
            ui.label(
                egui::RichText::new("Game System Design Tool")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .small()
                    .monospace()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.add_space(24.0);

            if editor_state.launcher_name_input_visible {
                // Show inline name input.
                ui.label("Project Name:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut editor_state.launcher_project_name)
                        .hint_text("e.g., My WW2 Campaign")
                        .desired_width(200.0),
                );

                // Request focus on first frame after reveal.
                if editor_state.launcher_request_focus {
                    response.request_focus();
                    editor_state.launcher_request_focus = false;
                }

                let trimmed_name = editor_state.launcher_project_name.trim().to_string();
                let name_valid = !trimmed_name.is_empty();

                // Enter key triggers Create.
                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                let spacing = ui.spacing().item_spacing.x;
                let half_width = (200.0 - spacing) / 2.0;
                let btn_size = egui::vec2(half_width, 0.0);

                ui.allocate_ui_with_layout(
                    egui::vec2(200.0, 24.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let create_btn = ui.add_enabled(
                            name_valid,
                            egui::Button::new(egui::RichText::new("Create").color(if name_valid {
                                BrandTheme::ACCENT_AMBER
                            } else {
                                BrandTheme::TEXT_DISABLED
                            }))
                            .min_size(btn_size),
                        );

                        if name_valid && (create_btn.clicked() || enter_pressed) {
                            commands.trigger(NewProjectEvent { name: trimmed_name });
                            editor_state.launcher_name_input_visible = false;
                            editor_state.launcher_project_name = String::new();
                        }

                        if ui
                            .add(egui::Button::new("Cancel").min_size(btn_size))
                            .clicked()
                        {
                            editor_state.launcher_name_input_visible = false;
                            editor_state.launcher_project_name = String::new();
                        }
                    },
                );
            } else {
                // Show the "New Game System" button.
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("New Game System").color(BrandTheme::ACCENT_AMBER),
                        )
                        .min_size(egui::vec2(200.0, 36.0)),
                    )
                    .clicked()
                {
                    editor_state.launcher_name_input_visible = true;
                    editor_state.launcher_project_name = String::new();
                    editor_state.launcher_request_focus = true;
                }
            }

            ui.add_space(8.0);
            if ui
                .add(egui::Button::new("Open...").min_size(egui::vec2(200.0, 36.0)))
                .clicked()
            {
                commands.trigger(LoadRequestEvent);
            }
        });
    });
}

/// Main editor panel system. Renders the left side panel with all editor sections.
#[allow(clippy::too_many_arguments)]
pub fn editor_panel_system(
    mut contexts: EguiContexts,
    mut editor_tool: ResMut<EditorTool>,
    mut selection: SelectionParams,
    mut editor_state: ResMut<EditorState>,
    selected_hex: Res<SelectedHex>,
    project: ProjectParams,
    mut registry: ResMut<EntityTypeRegistry>,
    mut enum_registry: ResMut<EnumRegistry>,
    mut struct_registry: ResMut<StructRegistry>,
    mut tile_data_query: Query<&mut EntityData, Without<UnitInstance>>,
    mut unit_data_query: Query<&mut EntityData, With<UnitInstance>>,
    tile_query: Query<(&HexPosition, Entity), With<HexTile>>,
    mut commands: Commands,
    mut ontology: OntologyParams,
    mut mechanics: MechanicsParams,
    mut next_state: ResMut<NextState<AppScreen>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut actions: Vec<EditorAction> = Vec::new();

    // -- File Menu Bar --
    egui::TopBottomPanel::top("file_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New          Cmd+N").clicked() {
                    commands.trigger(CloseProjectEvent);
                    ui.close();
                }
                if ui.button("Open...      Cmd+O").clicked() {
                    commands.trigger(LoadRequestEvent);
                    ui.close();
                }
                ui.separator();
                if ui.button("Save         Cmd+S").clicked() {
                    commands.trigger(SaveRequestEvent { save_as: false });
                    ui.close();
                }
                if ui.button("Save As...   Cmd+Shift+S").clicked() {
                    commands.trigger(SaveRequestEvent { save_as: true });
                    ui.close();
                }
                ui.separator();
                if ui.button("Close        Cmd+W").clicked() {
                    commands.trigger(CommandExecutedEvent {
                        command_id: CommandId("mode.close"),
                    });
                    ui.close();
                }
            });
        });
    });

    egui::SidePanel::left("editor_panel")
        .default_width(280.0)
        .show(ctx, |ui| {
            // -- Workspace Header --
            render_workspace_header(ui, &project.workspace, &project.game_system);

            // -- Tool Mode (toggleable via Cmd+T) --
            if editor_state.toolbar_visible {
                render_tool_mode(ui, &mut editor_tool);
            }

            // -- Play Mode Toggle --
            if ui
                .button(
                    egui::RichText::new("\u{25B6} Play")
                        .strong()
                        .color(BrandTheme::SUCCESS),
                )
                .clicked()
            {
                next_state.set(AppScreen::Play);
            }
            ui.separator();

            // -- Tab Bar --
            render_tab_bar(ui, &mut editor_state);

            // -- Tab Content --
            egui::ScrollArea::vertical().show(ui, |ui| {
                match editor_state.active_tab {
                    OntologyTab::Types => {
                        render_entity_type_editor(
                            ui,
                            &mut registry,
                            &mut editor_state,
                            &mut actions,
                            &enum_registry,
                            &struct_registry,
                        );
                    }
                    OntologyTab::Enums => {
                        render_enums_tab(ui, &enum_registry, &mut editor_state, &mut actions);
                    }
                    OntologyTab::Structs => {
                        render_structs_tab(
                            ui,
                            &struct_registry,
                            &enum_registry,
                            &mut editor_state,
                            &mut actions,
                        );
                    }
                    OntologyTab::Concepts => {
                        render_concepts_tab(
                            ui,
                            &mut ontology.concept_registry,
                            &registry,
                            &mut editor_state,
                            &mut actions,
                        );
                    }
                    OntologyTab::Relations => {
                        render_relations_tab(
                            ui,
                            &mut ontology.relation_registry,
                            &ontology.concept_registry,
                            &mut editor_state,
                            &mut actions,
                        );
                    }
                    OntologyTab::Constraints => {
                        render_constraints_tab(
                            ui,
                            &mut ontology.constraint_registry,
                            &ontology.concept_registry,
                            &mut editor_state,
                            &mut actions,
                        );
                    }
                    OntologyTab::Validation => {
                        render_validation_tab(ui, &ontology.schema_validation);
                    }
                    OntologyTab::Mechanics => {
                        render_mechanics_tab(
                            ui,
                            &mechanics.turn_structure,
                            &mechanics.combat_results_table,
                            &mechanics.combat_modifiers,
                            &mut editor_state,
                            &mut actions,
                        );
                    }
                }

                ui.separator();

                // -- Cell Palette (Paint mode) --
                if *editor_tool == EditorTool::Paint {
                    render_cell_palette(ui, &registry, &mut selection.active_board);
                }

                // -- Unit Palette (Place mode) --
                if *editor_tool == EditorTool::Place {
                    render_unit_palette(ui, &registry, &mut selection.active_token);
                }

                // -- Inspector (toggleable via Cmd+I) --
                if editor_state.inspector_visible {
                    // Unit Inspector takes priority when a unit is selected.
                    if selection.selected_unit.entity.is_some() {
                        render_unit_inspector(
                            ui,
                            &selection.selected_unit,
                            &mut unit_data_query,
                            &registry,
                            &enum_registry,
                            &struct_registry,
                            &mut actions,
                        );
                    } else {
                        // Tile Inspector.
                        render_inspector(
                            ui,
                            &selected_hex,
                            &tile_query,
                            &mut tile_data_query,
                            &registry,
                            &enum_registry,
                            &struct_registry,
                        );
                    }
                }
            });
        });

    // -- Apply deferred actions --
    apply_actions(
        actions,
        &mut registry,
        &mut enum_registry,
        &mut struct_registry,
        &mut tile_data_query,
        &mut selection.active_board,
        &mut selection.active_token,
        &mut selection.selected_unit,
        &editor_state,
        &mut commands,
        &mut ontology.concept_registry,
        &mut ontology.relation_registry,
        &mut ontology.constraint_registry,
        &mut mechanics.turn_structure,
        &mut mechanics.combat_results_table,
        &mut mechanics.combat_modifiers,
    );
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
    unit_query: Query<&EntityData, With<UnitInstance>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // -- File Menu Bar --
    egui::TopBottomPanel::top("file_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New          Cmd+N").clicked() {
                    commands.trigger(CloseProjectEvent);
                    ui.close();
                }
                if ui.button("Open...      Cmd+O").clicked() {
                    commands.trigger(LoadRequestEvent);
                    ui.close();
                }
                ui.separator();
                if ui.button("Save         Cmd+S").clicked() {
                    commands.trigger(SaveRequestEvent { save_as: false });
                    ui.close();
                }
                if ui.button("Save As...   Cmd+Shift+S").clicked() {
                    commands.trigger(SaveRequestEvent { save_as: true });
                    ui.close();
                }
            });
        });
    });

    egui::SidePanel::left("play_panel")
        .default_width(280.0)
        .show(ctx, |ui| {
            // -- Workspace Header --
            render_workspace_header(ui, &workspace, &game_system);

            // -- Back to Editor --
            if ui
                .button(
                    egui::RichText::new("\u{25A0} Editor")
                        .strong()
                        .color(BrandTheme::ACCENT_AMBER),
                )
                .clicked()
            {
                turn_state.is_active = false;
                next_state.set(AppScreen::Editor);
            }
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // -- Turn Tracker --
                render_turn_tracker(ui, &mut turn_state, &turn_structure);

                ui.separator();

                // -- Combat Panel --
                render_combat_panel(
                    ui,
                    &mut active_combat,
                    &combat_results_table,
                    &combat_modifiers,
                    &selected_unit,
                    &entity_types,
                    &mut editor_state,
                    &unit_query,
                );
            });
        });
}

/// Renders the turn tracker section in the play panel.
fn render_turn_tracker(
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

    // Advance phase button.
    if ui.button("Next Phase \u{23E9}").clicked() {
        let next_index = turn_state.current_phase_index + 1;
        if next_index >= turn_structure.phases.len() {
            turn_state.turn_number += 1;
            turn_state.current_phase_index = 0;
        } else {
            turn_state.current_phase_index = next_index;
        }
    }
}

/// Renders the combat resolution panel in the play panel.
#[allow(clippy::too_many_arguments)]
fn render_combat_panel(
    ui: &mut egui::Ui,
    active_combat: &mut ActiveCombat,
    crt: &CombatResultsTable,
    modifiers: &CombatModifierRegistry,
    selected_unit: &SelectedUnit,
    entity_types: &EntityTypeRegistry,
    editor_state: &mut EditorState,
    unit_query: &Query<&EntityData, With<UnitInstance>>,
) {
    use crate::contracts::mechanics::{
        apply_column_shift, evaluate_modifiers_prioritized, find_crt_column, resolve_crt,
    };

    ui.label(
        egui::RichText::new("Combat Resolution")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(4.0);

    if crt.columns.is_empty() || crt.rows.is_empty() {
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
            let name = unit_query
                .get(atk)
                .ok()
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
            let name = unit_query
                .get(def)
                .ok()
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
    let base_column = find_crt_column(atk_str, def_str, &crt.columns);
    if let Some(col_idx) = base_column {
        ui.label(
            egui::RichText::new(format!(
                "Base column: {} ({})",
                crt.columns[col_idx].label, col_idx
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
        let (total_shift, modifier_display) =
            evaluate_modifiers_prioritized(&modifiers.modifiers, crt.columns.len());
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
            let final_col = apply_column_shift(base_col, total_shift, crt.columns.len());
            active_combat.resolved_column = Some(final_col);
            active_combat.total_shift = total_shift;
            ui.label(
                egui::RichText::new(format!(
                    "Final column: {} ({})",
                    crt.columns[final_col].label, final_col
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
    let can_resolve = base_column.is_some();
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
                    apply_column_shift(resolution.column_index, shift, crt.columns.len());
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
                crate::contracts::mechanics::OutcomeEffect::NoEffect => "No effect".to_string(),
                crate::contracts::mechanics::OutcomeEffect::Retreat { hexes } => {
                    format!("Defender retreats {hexes} hex(es)")
                }
                crate::contracts::mechanics::OutcomeEffect::StepLoss { steps } => {
                    format!("Defender loses {steps} step(s)")
                }
                crate::contracts::mechanics::OutcomeEffect::AttackerStepLoss { steps } => {
                    format!("Attacker loses {steps} step(s)")
                }
                crate::contracts::mechanics::OutcomeEffect::Exchange {
                    attacker_steps,
                    defender_steps,
                } => format!("Exchange: ATK -{attacker_steps}, DEF -{defender_steps}"),
                crate::contracts::mechanics::OutcomeEffect::AttackerEliminated => {
                    "Attacker eliminated".to_string()
                }
                crate::contracts::mechanics::OutcomeEffect::DefenderEliminated => {
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

    ui.add_space(8.0);

    // -- Clear button --
    if ui.button("Clear Combat").clicked() {
        *active_combat = ActiveCombat::default();
        editor_state.combat_attacker_strength = 0.0;
        editor_state.combat_defender_strength = 0.0;
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

// ---------------------------------------------------------------------------
// UI Section Renderers
// ---------------------------------------------------------------------------

pub(crate) fn render_workspace_header(ui: &mut egui::Ui, workspace: &Workspace, gs: &GameSystem) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(&workspace.name)
                .strong()
                .size(15.0)
                .color(BrandTheme::ACCENT_AMBER),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("v{}", gs.version))
                    .small()
                    .monospace()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
        });
    });
    let id_short = if gs.id.len() > 8 {
        format!("{}...", &gs.id[..8])
    } else {
        gs.id.clone()
    };
    ui.label(
        egui::RichText::new(format!("hexorder | {id_short}"))
            .small()
            .monospace()
            .color(BrandTheme::TEXT_TERTIARY),
    );
    ui.separator();
}

pub(crate) fn render_tool_mode(ui: &mut egui::Ui, editor_tool: &mut EditorTool) {
    ui.label(
        egui::RichText::new("Tool Mode")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .selectable_label(*editor_tool == EditorTool::Select, "Select")
            .clicked()
        {
            *editor_tool = EditorTool::Select;
        }
        if ui
            .selectable_label(*editor_tool == EditorTool::Paint, "Paint")
            .clicked()
        {
            *editor_tool = EditorTool::Paint;
        }
        if ui
            .selectable_label(*editor_tool == EditorTool::Place, "Place")
            .clicked()
        {
            *editor_tool = EditorTool::Place;
        }
    });
    ui.separator();
}

pub(crate) fn render_tab_bar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal_wrapped(|ui| {
        for tab in [
            OntologyTab::Types,
            OntologyTab::Enums,
            OntologyTab::Structs,
            OntologyTab::Concepts,
            OntologyTab::Relations,
            OntologyTab::Constraints,
            OntologyTab::Validation,
            OntologyTab::Mechanics,
        ] {
            let label = match tab {
                OntologyTab::Types => "Types",
                OntologyTab::Enums => "Enums",
                OntologyTab::Structs => "Structs",
                OntologyTab::Concepts => "Concepts",
                OntologyTab::Relations => "Relations",
                OntologyTab::Constraints => "Constr.",
                OntologyTab::Validation => "Valid.",
                OntologyTab::Mechanics => "Mech.",
            };
            let text = if editor_state.active_tab == tab {
                egui::RichText::new(label).color(BrandTheme::ACCENT_AMBER)
            } else {
                egui::RichText::new(label)
            };
            if ui
                .selectable_label(editor_state.active_tab == tab, text)
                .clicked()
            {
                editor_state.active_tab = tab;
            }
        }
    });
    ui.separator();
}

pub(crate) fn render_cell_palette(
    ui: &mut egui::Ui,
    registry: &EntityTypeRegistry,
    active_board: &mut ActiveBoardType,
) {
    ui.label(
        egui::RichText::new("Cell Palette")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    for et in registry.types_by_role(EntityRole::BoardPosition) {
        let is_active = active_board.entity_type_id == Some(et.id);
        let color = bevy_color_to_egui(et.color);
        let et_id = et.id;
        let et_name = et.name.clone();

        ui.horizontal(|ui| {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 2.0, color);
                if is_active {
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(2.0, BrandTheme::ACCENT_AMBER),
                        egui::StrokeKind::Outside,
                    );
                }
            }
            if response.clicked() {
                active_board.entity_type_id = Some(et_id);
            }
            if ui.selectable_label(is_active, &et_name).clicked() {
                active_board.entity_type_id = Some(et_id);
            }
        });
    }

    ui.separator();
}

pub(crate) fn render_unit_palette(
    ui: &mut egui::Ui,
    registry: &EntityTypeRegistry,
    active_token: &mut ActiveTokenType,
) {
    ui.label(
        egui::RichText::new("Unit Palette")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    for et in registry.types_by_role(EntityRole::Token) {
        let is_active = active_token.entity_type_id == Some(et.id);
        let color = bevy_color_to_egui(et.color);
        let et_id = et.id;
        let et_name = et.name.clone();

        ui.horizontal(|ui| {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 2.0, color);
                if is_active {
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(2.0, BrandTheme::ACCENT_AMBER),
                        egui::StrokeKind::Outside,
                    );
                }
            }
            if response.clicked() {
                active_token.entity_type_id = Some(et_id);
            }
            if ui.selectable_label(is_active, &et_name).clicked() {
                active_token.entity_type_id = Some(et_id);
            }
        });
    }

    ui.separator();
}

pub(crate) fn render_entity_type_editor(
    ui: &mut egui::Ui,
    registry: &mut EntityTypeRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
) {
    // -- Cell Types (BoardPosition) --
    render_entity_type_section(
        ui,
        registry,
        editor_state,
        actions,
        EntityRole::BoardPosition,
        "Cell Types",
        "ct",
        enum_registry,
        struct_registry,
    );

    // -- Unit Types (Token) --
    render_entity_type_section(
        ui,
        registry,
        editor_state,
        actions,
        EntityRole::Token,
        "Unit Types",
        "ut",
        enum_registry,
        struct_registry,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_entity_type_section(
    ui: &mut egui::Ui,
    registry: &mut EntityTypeRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
    role: EntityRole,
    section_label: &str,
    id_prefix: &str,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
) {
    egui::CollapsingHeader::new(
        egui::RichText::new(section_label)
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    )
    .default_open(false)
    .show(ui, |ui| {
        // -- Create new type --
        ui.group(|ui| {
            ui.label(egui::RichText::new("New Type").small());
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_type_name);
            });
            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut c32 = rgb_to_color32(editor_state.new_type_color);
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut c32,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    editor_state.new_type_color = color32_to_rgb(c32);
                }
            });
            let name_valid = !editor_state.new_type_name.trim().is_empty();
            ui.add_enabled_ui(name_valid, |ui| {
                if ui
                    .button(egui::RichText::new("+ Create").color(BrandTheme::ACCENT_AMBER))
                    .clicked()
                    && name_valid
                {
                    let [r, g, b] = editor_state.new_type_color;
                    actions.push(EditorAction::CreateEntityType {
                        name: editor_state.new_type_name.trim().to_string(),
                        role,
                        color: Color::srgb(r, g, b),
                    });
                    editor_state.new_type_name.clear();
                    editor_state.new_type_color = [0.5, 0.5, 0.5];
                }
            });
        });

        ui.add_space(4.0);

        // -- Edit existing types --
        {
            // Collect indices and ids for types with the matching role.
            let role_indices: Vec<usize> = registry
                .types
                .iter()
                .enumerate()
                .filter(|(_, t)| t.role == role)
                .map(|(i, _)| i)
                .collect();

            let role_type_count = role_indices.len();
            let mut delete_id = None;

            for (display_idx, &type_idx) in role_indices.iter().enumerate() {
                let type_id = registry.types[type_idx].id;
                let header_name = registry.types[type_idx].name.clone();

                egui::CollapsingHeader::new(&header_name)
                    .id_salt(format!("{id_prefix}_{display_idx}"))
                    .show(ui, |ui| {
                        // Name
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut registry.types[type_idx].name);
                        });

                        // Color
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut c32 = bevy_color_to_egui(registry.types[type_idx].color);
                            if egui::color_picker::color_edit_button_srgba(
                                ui,
                                &mut c32,
                                egui::color_picker::Alpha::Opaque,
                            )
                            .changed()
                            {
                                registry.types[type_idx].color = egui_color_to_bevy(c32);
                            }
                        });

                        // Properties list
                        ui.label(egui::RichText::new("Properties:").small());
                        if registry.types[type_idx].properties.is_empty() {
                            ui.label(
                                egui::RichText::new("  (none)")
                                    .small()
                                    .color(BrandTheme::TEXT_SECONDARY),
                            );
                        } else {
                            let mut remove_prop_id = None;
                            for prop in &registry.types[type_idx].properties {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "  {} ({})",
                                        prop.name,
                                        format_property_type(&prop.property_type)
                                    ));
                                    if ui.small_button("x").clicked() {
                                        remove_prop_id = Some(prop.id);
                                    }
                                });
                            }
                            if let Some(prop_id) = remove_prop_id {
                                actions.push(EditorAction::RemoveProperty { type_id, prop_id });
                            }
                        }

                        // Add property
                        ui.add_space(2.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Add Property").small());
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut editor_state.new_prop_name);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Type:");
                                let types = [
                                    "Bool",
                                    "Int",
                                    "Float",
                                    "String",
                                    "Color",
                                    "Enum",
                                    "EntityRef",
                                    "List",
                                    "Map",
                                    "Struct",
                                    "IntRange",
                                    "FloatRange",
                                ];
                                egui::ComboBox::from_id_salt(format!(
                                    "{id_prefix}_pt_{display_idx}"
                                ))
                                .selected_text(types[editor_state.new_prop_type_index])
                                .show_ui(ui, |ui| {
                                    for (idx, name) in types.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut editor_state.new_prop_type_index,
                                            idx,
                                            *name,
                                        );
                                    }
                                });
                            });
                            if editor_state.new_prop_type_index == 5 {
                                ui.horizontal(|ui| {
                                    ui.label("Opts:");
                                    ui.text_edit_singleline(&mut editor_state.new_enum_options);
                                });
                                ui.label(
                                    egui::RichText::new("(comma-separated)")
                                        .small()
                                        .color(BrandTheme::TEXT_SECONDARY),
                                );
                            }
                            // EntityRef (index 6) — role filter
                            if editor_state.new_prop_type_index == 6 {
                                ui.horizontal(|ui| {
                                    ui.label("Role:");
                                    let roles = ["Any", "BoardPosition", "Token"];
                                    egui::ComboBox::from_id_salt(format!(
                                        "{id_prefix}_eref_{display_idx}"
                                    ))
                                    .selected_text(roles[editor_state.new_prop_entity_ref_role])
                                    .show_ui(ui, |ui| {
                                        for (idx, name) in roles.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut editor_state.new_prop_entity_ref_role,
                                                idx,
                                                *name,
                                            );
                                        }
                                    });
                                });
                            }
                            // List (index 7) — inner type
                            if editor_state.new_prop_type_index == 7 {
                                ui.horizontal(|ui| {
                                    ui.label("Item type:");
                                    let inner_types = ["Bool", "Int", "Float", "String", "Color"];
                                    egui::ComboBox::from_id_salt(format!(
                                        "{id_prefix}_list_{display_idx}"
                                    ))
                                    .selected_text(
                                        inner_types[editor_state.new_prop_list_inner_type],
                                    )
                                    .show_ui(ui, |ui| {
                                        for (idx, name) in inner_types.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut editor_state.new_prop_list_inner_type,
                                                idx,
                                                *name,
                                            );
                                        }
                                    });
                                });
                            }
                            // Map (index 8) — enum key + value type
                            if editor_state.new_prop_type_index == 8 {
                                ui.horizontal(|ui| {
                                    ui.label("Key enum:");
                                    let enum_names: Vec<(TypeId, String)> = enum_registry
                                        .definitions
                                        .values()
                                        .map(|e| (e.id, e.name.clone()))
                                        .collect();
                                    let selected_name = editor_state
                                        .new_prop_map_enum_id
                                        .and_then(|id| {
                                            enum_names.iter().find(|(eid, _)| *eid == id)
                                        })
                                        .map_or("(select)", |(_, n)| n.as_str())
                                        .to_string();
                                    egui::ComboBox::from_id_salt(format!(
                                        "{id_prefix}_mapk_{display_idx}"
                                    ))
                                    .selected_text(&selected_name)
                                    .show_ui(ui, |ui| {
                                        for (eid, ename) in &enum_names {
                                            if ui
                                                .selectable_label(
                                                    editor_state.new_prop_map_enum_id == Some(*eid),
                                                    ename,
                                                )
                                                .clicked()
                                            {
                                                editor_state.new_prop_map_enum_id = Some(*eid);
                                            }
                                        }
                                    });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Value type:");
                                    let val_types = ["Bool", "Int", "Float", "String", "Color"];
                                    egui::ComboBox::from_id_salt(format!(
                                        "{id_prefix}_mapv_{display_idx}"
                                    ))
                                    .selected_text(val_types[editor_state.new_prop_map_value_type])
                                    .show_ui(ui, |ui| {
                                        for (idx, name) in val_types.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut editor_state.new_prop_map_value_type,
                                                idx,
                                                *name,
                                            );
                                        }
                                    });
                                });
                            }
                            // Struct (index 9)
                            if editor_state.new_prop_type_index == 9 {
                                ui.horizontal(|ui| {
                                    ui.label("Struct:");
                                    let struct_names: Vec<(TypeId, String)> = struct_registry
                                        .definitions
                                        .values()
                                        .map(|s| (s.id, s.name.clone()))
                                        .collect();
                                    let selected_name = editor_state
                                        .new_prop_struct_id
                                        .and_then(|id| {
                                            struct_names.iter().find(|(sid, _)| *sid == id)
                                        })
                                        .map_or("(select)", |(_, n)| n.as_str())
                                        .to_string();
                                    egui::ComboBox::from_id_salt(format!(
                                        "{id_prefix}_struct_{display_idx}"
                                    ))
                                    .selected_text(&selected_name)
                                    .show_ui(ui, |ui| {
                                        for (sid, sname) in &struct_names {
                                            if ui
                                                .selectable_label(
                                                    editor_state.new_prop_struct_id == Some(*sid),
                                                    sname,
                                                )
                                                .clicked()
                                            {
                                                editor_state.new_prop_struct_id = Some(*sid);
                                            }
                                        }
                                    });
                                });
                            }
                            // IntRange (index 10)
                            if editor_state.new_prop_type_index == 10 {
                                ui.horizontal(|ui| {
                                    ui.label("Min:");
                                    ui.add(egui::DragValue::new(
                                        &mut editor_state.new_prop_int_range_min,
                                    ));
                                    ui.label("Max:");
                                    ui.add(egui::DragValue::new(
                                        &mut editor_state.new_prop_int_range_max,
                                    ));
                                });
                            }
                            // FloatRange (index 11)
                            if editor_state.new_prop_type_index == 11 {
                                ui.horizontal(|ui| {
                                    ui.label("Min:");
                                    ui.add(
                                        egui::DragValue::new(
                                            &mut editor_state.new_prop_float_range_min,
                                        )
                                        .speed(0.1),
                                    );
                                    ui.label("Max:");
                                    ui.add(
                                        egui::DragValue::new(
                                            &mut editor_state.new_prop_float_range_max,
                                        )
                                        .speed(0.1),
                                    );
                                });
                            }
                            let prop_valid = !editor_state.new_prop_name.trim().is_empty();
                            ui.add_enabled_ui(prop_valid, |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("+ Add")
                                            .color(BrandTheme::ACCENT_AMBER),
                                    )
                                    .clicked()
                                    && prop_valid
                                {
                                    let prop_type =
                                        index_to_property_type(editor_state.new_prop_type_index);
                                    actions.push(EditorAction::AddProperty {
                                        type_id,
                                        name: editor_state.new_prop_name.trim().to_string(),
                                        prop_type,
                                        enum_options: editor_state.new_enum_options.clone(),
                                    });
                                    editor_state.new_prop_name.clear();
                                    editor_state.new_prop_type_index = 0;
                                    editor_state.new_enum_options.clear();
                                }
                            });
                        });

                        // Delete type
                        if role_type_count > 1 {
                            ui.add_space(4.0);
                            if ui
                                .button(
                                    egui::RichText::new("Delete Type").color(BrandTheme::DANGER),
                                )
                                .clicked()
                            {
                                delete_id = Some(type_id);
                            }
                        }
                    });
            }

            if let Some(id) = delete_id {
                actions.push(EditorAction::DeleteEntityType { id });
            }
        }
    });
}

pub(crate) fn render_enums_tab(
    ui: &mut egui::Ui,
    enum_registry: &EnumRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Enums")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    // Create new enum form
    ui.group(|ui| {
        ui.label(egui::RichText::new("New Enum").small());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_state.new_enum_name);
        });
        ui.horizontal(|ui| {
            ui.label("Options:");
            ui.text_edit_singleline(&mut editor_state.new_enum_option_text);
        });
        ui.label(
            egui::RichText::new("(comma-separated)")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        let name_valid = !editor_state.new_enum_name.trim().is_empty();
        ui.add_enabled_ui(name_valid, |ui| {
            if ui
                .button(egui::RichText::new("+ Create Enum").color(BrandTheme::ACCENT_AMBER))
                .clicked()
                && name_valid
            {
                let options: Vec<String> = editor_state
                    .new_enum_option_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                actions.push(EditorAction::CreateEnum {
                    name: editor_state.new_enum_name.trim().to_string(),
                    options,
                });
                editor_state.new_enum_name.clear();
                editor_state.new_enum_option_text.clear();
            }
        });
    });

    ui.add_space(4.0);

    // List existing enums
    if enum_registry.definitions.is_empty() {
        ui.label(
            egui::RichText::new("No enums defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    let enum_snapshots: Vec<_> = enum_registry
        .definitions
        .values()
        .map(|e| (e.id, e.name.clone(), e.options.clone()))
        .collect();

    for (enum_id, name, options) in &enum_snapshots {
        let mut delete = false;

        egui::CollapsingHeader::new(name)
            .id_salt(format!("enum_{enum_id:?}"))
            .show(ui, |ui| {
                for opt in options {
                    ui.horizontal(|ui| {
                        ui.label(format!("  {opt}"));
                        if ui.small_button("x").clicked() {
                            actions.push(EditorAction::RemoveEnumOption {
                                enum_id: *enum_id,
                                option: opt.clone(),
                            });
                        }
                    });
                }

                // Add option inline
                ui.horizontal(|ui| {
                    ui.label("Add:");
                    ui.text_edit_singleline(&mut editor_state.new_enum_option_text);
                    let opt_valid = !editor_state.new_enum_option_text.trim().is_empty();
                    if ui.add_enabled(opt_valid, egui::Button::new("+")).clicked() && opt_valid {
                        actions.push(EditorAction::AddEnumOption {
                            enum_id: *enum_id,
                            option: editor_state.new_enum_option_text.trim().to_string(),
                        });
                        editor_state.new_enum_option_text.clear();
                    }
                });

                ui.add_space(4.0);
                if ui
                    .button(egui::RichText::new("Delete Enum").color(BrandTheme::DANGER))
                    .clicked()
                {
                    delete = true;
                }
            });

        if delete {
            actions.push(EditorAction::DeleteEnum { id: *enum_id });
        }
    }
}

pub(crate) fn render_structs_tab(
    ui: &mut egui::Ui,
    struct_registry: &StructRegistry,
    enum_registry: &EnumRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Structs")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    // Suppress unused warning for enum_registry (will be used for Map key picker later).
    let _ = enum_registry;

    // Create new struct form
    ui.group(|ui| {
        ui.label(egui::RichText::new("New Struct").small());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_state.new_struct_name);
        });
        let name_valid = !editor_state.new_struct_name.trim().is_empty();
        ui.add_enabled_ui(name_valid, |ui| {
            if ui
                .button(egui::RichText::new("+ Create Struct").color(BrandTheme::ACCENT_AMBER))
                .clicked()
                && name_valid
            {
                actions.push(EditorAction::CreateStruct {
                    name: editor_state.new_struct_name.trim().to_string(),
                });
                editor_state.new_struct_name.clear();
            }
        });
    });

    ui.add_space(4.0);

    if struct_registry.definitions.is_empty() {
        ui.label(
            egui::RichText::new("No structs defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    let struct_snapshots: Vec<_> = struct_registry
        .definitions
        .values()
        .map(|s| (s.id, s.name.clone(), s.fields.clone()))
        .collect();

    for (struct_id, name, fields) in &struct_snapshots {
        let mut delete = false;

        egui::CollapsingHeader::new(name)
            .id_salt(format!("struct_{struct_id:?}"))
            .show(ui, |ui| {
                for field in fields {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "  {}: {}",
                            field.name,
                            format_property_type(&field.property_type)
                        ));
                        if ui.small_button("x").clicked() {
                            actions.push(EditorAction::RemoveStructField {
                                struct_id: *struct_id,
                                field_id: field.id,
                            });
                        }
                    });
                }

                // Add field form
                ui.horizontal(|ui| {
                    ui.label("Field:");
                    ui.text_edit_singleline(&mut editor_state.new_struct_field_name);
                });
                let base_types = ["Bool", "Int", "Float", "String", "Color"];
                egui::ComboBox::from_id_salt(format!("sf_type_{struct_id:?}"))
                    .selected_text(
                        base_types
                            .get(editor_state.new_struct_field_type_index)
                            .copied()
                            .unwrap_or("Bool"),
                    )
                    .show_ui(ui, |ui| {
                        for (i, t) in base_types.iter().enumerate() {
                            ui.selectable_value(
                                &mut editor_state.new_struct_field_type_index,
                                i,
                                *t,
                            );
                        }
                    });
                let field_name_valid = !editor_state.new_struct_field_name.trim().is_empty();
                ui.add_enabled_ui(field_name_valid, |ui| {
                    if ui
                        .button(egui::RichText::new("+ Add Field").color(BrandTheme::ACCENT_AMBER))
                        .clicked()
                        && field_name_valid
                    {
                        let prop_type =
                            index_to_property_type(editor_state.new_struct_field_type_index);
                        actions.push(EditorAction::AddStructField {
                            struct_id: *struct_id,
                            name: editor_state.new_struct_field_name.trim().to_string(),
                            prop_type,
                        });
                        editor_state.new_struct_field_name.clear();
                    }
                });

                ui.add_space(4.0);
                if ui
                    .button(egui::RichText::new("Delete Struct").color(BrandTheme::DANGER))
                    .clicked()
                {
                    delete = true;
                }
            });

        if delete {
            actions.push(EditorAction::DeleteStruct { id: *struct_id });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_concepts_tab(
    ui: &mut egui::Ui,
    concept_registry: &mut ConceptRegistry,
    entity_registry: &EntityTypeRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Concepts")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    // -- Create new concept --
    ui.group(|ui| {
        ui.label(egui::RichText::new("New Concept").small());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_state.new_concept_name);
        });
        ui.horizontal(|ui| {
            ui.label("Desc:");
            ui.text_edit_singleline(&mut editor_state.new_concept_description);
        });
        let name_valid = !editor_state.new_concept_name.trim().is_empty();
        ui.add_enabled_ui(name_valid, |ui| {
            if ui
                .button(egui::RichText::new("+ Create Concept").color(BrandTheme::ACCENT_AMBER))
                .clicked()
                && name_valid
            {
                actions.push(EditorAction::CreateConcept {
                    name: editor_state.new_concept_name.trim().to_string(),
                    description: editor_state.new_concept_description.trim().to_string(),
                });
                editor_state.new_concept_name.clear();
                editor_state.new_concept_description.clear();
            }
        });
    });

    ui.add_space(4.0);

    // -- Concept list --
    if concept_registry.concepts.is_empty() {
        ui.label(
            egui::RichText::new("No concepts defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    // Snapshot concept data to avoid borrow conflicts.
    let concept_snapshots: Vec<_> = concept_registry
        .concepts
        .iter()
        .map(|c| {
            (
                c.id,
                c.name.clone(),
                c.description.clone(),
                c.role_labels.clone(),
            )
        })
        .collect();

    let binding_snapshots: Vec<_> = concept_registry
        .bindings
        .iter()
        .map(|b| {
            (
                b.id,
                b.entity_type_id,
                b.concept_id,
                b.concept_role_id,
                b.property_bindings.clone(),
            )
        })
        .collect();

    for (concept_id, concept_name, concept_desc, role_labels) in &concept_snapshots {
        let mut delete_concept = false;

        egui::CollapsingHeader::new(concept_name)
            .id_salt(format!("concept_{concept_id:?}"))
            .show(ui, |ui| {
                // Description
                ui.label(
                    egui::RichText::new(concept_desc)
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );

                // -- Role Slots --
                ui.label(egui::RichText::new("Roles:").small());
                if role_labels.is_empty() {
                    ui.label(
                        egui::RichText::new("  (none)")
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                } else {
                    let mut remove_role_id = None;
                    for role in role_labels {
                        ui.horizontal(|ui| {
                            let allowed_str: Vec<&str> = role
                                .allowed_entity_roles
                                .iter()
                                .map(|r| match r {
                                    EntityRole::BoardPosition => "Board",
                                    EntityRole::Token => "Token",
                                })
                                .collect();
                            ui.label(format!("  {} [{}]", role.name, allowed_str.join(", ")));
                            if ui.small_button("x").clicked() {
                                remove_role_id = Some(role.id);
                            }
                        });
                    }
                    if let Some(role_id) = remove_role_id {
                        actions.push(EditorAction::RemoveConceptRole {
                            concept_id: *concept_id,
                            role_id,
                        });
                    }
                }

                // Add role form
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Add Role").small());
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut editor_state.new_role_name);
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut editor_state.new_role_allowed_roles[0], "Board");
                        ui.checkbox(&mut editor_state.new_role_allowed_roles[1], "Token");
                    });
                    let role_valid = !editor_state.new_role_name.trim().is_empty()
                        && editor_state.new_role_allowed_roles.iter().any(|&v| v);
                    ui.add_enabled_ui(role_valid, |ui| {
                        if ui
                            .button(
                                egui::RichText::new("+ Add Role").color(BrandTheme::ACCENT_AMBER),
                            )
                            .clicked()
                            && role_valid
                        {
                            let mut allowed = Vec::new();
                            if editor_state.new_role_allowed_roles[0] {
                                allowed.push(EntityRole::BoardPosition);
                            }
                            if editor_state.new_role_allowed_roles[1] {
                                allowed.push(EntityRole::Token);
                            }
                            actions.push(EditorAction::AddConceptRole {
                                concept_id: *concept_id,
                                name: editor_state.new_role_name.trim().to_string(),
                                allowed_roles: allowed,
                            });
                            editor_state.new_role_name.clear();
                            editor_state.new_role_allowed_roles = vec![false, false];
                        }
                    });
                });

                // -- Bindings --
                ui.add_space(2.0);
                ui.label(egui::RichText::new("Bindings:").small());

                let concept_bindings: Vec<_> = binding_snapshots
                    .iter()
                    .filter(|(_, _, cid, _, _)| *cid == *concept_id)
                    .collect();

                if concept_bindings.is_empty() {
                    ui.label(
                        egui::RichText::new("  (none)")
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                } else {
                    let mut unbind_id = None;
                    for (binding_id, et_id, _, cr_id, prop_bindings) in &concept_bindings {
                        let et_name = entity_registry
                            .get(*et_id)
                            .map_or_else(|| format!("{et_id:?}"), |et| et.name.clone());
                        let role_name = role_labels
                            .iter()
                            .find(|r| r.id == *cr_id)
                            .map_or("?", |r| r.name.as_str());
                        ui.horizontal(|ui| {
                            ui.label(format!("  {et_name} -> {role_name}"));
                            if ui.small_button("x").clicked() {
                                unbind_id = Some(*binding_id);
                            }
                        });
                        // Show property mappings read-only
                        for pb in prop_bindings {
                            ui.label(
                                egui::RichText::new(format!(
                                    "    {:?} as \"{}\"",
                                    pb.property_id, pb.concept_local_name
                                ))
                                .small()
                                .color(BrandTheme::TEXT_SECONDARY),
                            );
                        }
                    }
                    if let Some(bid) = unbind_id {
                        actions.push(EditorAction::UnbindEntityFromConcept {
                            concept_id: *concept_id,
                            binding_id: bid,
                        });
                    }
                }

                // Add binding form
                if !role_labels.is_empty() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Bind Entity").small());

                        // Entity type ComboBox
                        let et_names: Vec<(TypeId, String)> = entity_registry
                            .types
                            .iter()
                            .map(|et| (et.id, et.name.clone()))
                            .collect();

                        let selected_et_name = editor_state
                            .binding_entity_type_id
                            .and_then(|id| et_names.iter().find(|(eid, _)| *eid == id))
                            .map_or("(select)", |(_, n)| n.as_str())
                            .to_string();

                        egui::ComboBox::from_id_salt(format!("bind_et_{concept_id:?}"))
                            .selected_text(&selected_et_name)
                            .show_ui(ui, |ui| {
                                for (et_id, et_name) in &et_names {
                                    let selected =
                                        editor_state.binding_entity_type_id == Some(*et_id);
                                    if ui.selectable_label(selected, et_name).clicked() {
                                        editor_state.binding_entity_type_id = Some(*et_id);
                                    }
                                }
                            });

                        // Concept role ComboBox
                        let selected_cr_name = editor_state
                            .binding_concept_role_id
                            .and_then(|id| role_labels.iter().find(|r| r.id == id))
                            .map_or("(select)", |r| r.name.as_str())
                            .to_string();

                        egui::ComboBox::from_id_salt(format!("bind_cr_{concept_id:?}"))
                            .selected_text(&selected_cr_name)
                            .show_ui(ui, |ui| {
                                for role in role_labels {
                                    let selected =
                                        editor_state.binding_concept_role_id == Some(role.id);
                                    if ui.selectable_label(selected, &role.name).clicked() {
                                        editor_state.binding_concept_role_id = Some(role.id);
                                    }
                                }
                            });

                        let bind_valid = editor_state.binding_entity_type_id.is_some()
                            && editor_state.binding_concept_role_id.is_some();
                        ui.add_enabled_ui(bind_valid, |ui| {
                            if ui
                                .button(
                                    egui::RichText::new("+ Bind").color(BrandTheme::ACCENT_AMBER),
                                )
                                .clicked()
                                && bind_valid
                                && let (Some(et_id), Some(cr_id)) = (
                                    editor_state.binding_entity_type_id,
                                    editor_state.binding_concept_role_id,
                                )
                            {
                                actions.push(EditorAction::BindEntityToConcept {
                                    entity_type_id: et_id,
                                    concept_id: *concept_id,
                                    concept_role_id: cr_id,
                                });
                                editor_state.binding_entity_type_id = None;
                                editor_state.binding_concept_role_id = None;
                            }
                        });
                    });
                }

                // Delete concept
                ui.add_space(4.0);
                if ui
                    .button(egui::RichText::new("Delete Concept").color(BrandTheme::DANGER))
                    .clicked()
                {
                    delete_concept = true;
                }
            });

        if delete_concept {
            actions.push(EditorAction::DeleteConcept { id: *concept_id });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_relations_tab(
    ui: &mut egui::Ui,
    relation_registry: &mut RelationRegistry,
    concept_registry: &ConceptRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Relations")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    let concepts: Vec<_> = concept_registry
        .concepts
        .iter()
        .map(|c| (c.id, c.name.clone(), c.role_labels.clone()))
        .collect();

    // -- Create new relation --
    ui.group(|ui| {
        ui.label(egui::RichText::new("New Relation").small());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_state.new_relation_name);
        });

        // Concept selector
        let concept_names: Vec<&str> = concepts.iter().map(|(_, n, _)| n.as_str()).collect();
        if !concept_names.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Concept:");
                let idx = &mut editor_state.new_relation_concept_index;
                *idx = (*idx).min(concept_names.len().saturating_sub(1));
                egui::ComboBox::from_id_salt("rel_concept")
                    .selected_text(concept_names.get(*idx).copied().unwrap_or("--"))
                    .show_ui(ui, |ui| {
                        for (i, name) in concept_names.iter().enumerate() {
                            ui.selectable_value(idx, i, *name);
                        }
                    });
            });

            // Subject/object role selectors
            let selected_concept_roles = concepts
                .get(editor_state.new_relation_concept_index)
                .map(|(_, _, roles)| roles.clone())
                .unwrap_or_default();
            let role_names: Vec<&str> = selected_concept_roles
                .iter()
                .map(|r| r.name.as_str())
                .collect();

            if !role_names.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Subject:");
                    let idx = &mut editor_state.new_relation_subject_index;
                    *idx = (*idx).min(role_names.len().saturating_sub(1));
                    egui::ComboBox::from_id_salt("rel_subject")
                        .selected_text(role_names.get(*idx).copied().unwrap_or("--"))
                        .show_ui(ui, |ui| {
                            for (i, name) in role_names.iter().enumerate() {
                                ui.selectable_value(idx, i, *name);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Object:");
                    let idx = &mut editor_state.new_relation_object_index;
                    *idx = (*idx).min(role_names.len().saturating_sub(1));
                    egui::ComboBox::from_id_salt("rel_object")
                        .selected_text(role_names.get(*idx).copied().unwrap_or("--"))
                        .show_ui(ui, |ui| {
                            for (i, name) in role_names.iter().enumerate() {
                                ui.selectable_value(idx, i, *name);
                            }
                        });
                });
            }
        }

        // Trigger selector
        let triggers = ["OnEnter", "OnExit", "WhilePresent"];
        ui.horizontal(|ui| {
            ui.label("Trigger:");
            let idx = &mut editor_state.new_relation_trigger_index;
            *idx = (*idx).min(2);
            egui::ComboBox::from_id_salt("rel_trigger")
                .selected_text(triggers[*idx])
                .show_ui(ui, |ui| {
                    for (i, name) in triggers.iter().enumerate() {
                        ui.selectable_value(idx, i, *name);
                    }
                });
        });

        // Effect selector
        let effects = ["ModifyProperty", "Block", "Allow"];
        ui.horizontal(|ui| {
            ui.label("Effect:");
            let idx = &mut editor_state.new_relation_effect_index;
            *idx = (*idx).min(2);
            egui::ComboBox::from_id_salt("rel_effect")
                .selected_text(effects[*idx])
                .show_ui(ui, |ui| {
                    for (i, name) in effects.iter().enumerate() {
                        ui.selectable_value(idx, i, *name);
                    }
                });
        });

        // ModifyProperty fields
        if editor_state.new_relation_effect_index == 0 {
            ui.horizontal(|ui| {
                ui.label("Target:");
                ui.text_edit_singleline(&mut editor_state.new_relation_target_prop);
            });
            ui.horizontal(|ui| {
                ui.label("Source:");
                ui.text_edit_singleline(&mut editor_state.new_relation_source_prop);
            });
            let operations = ["Add", "Subtract", "Multiply", "Min", "Max"];
            ui.horizontal(|ui| {
                ui.label("Op:");
                let idx = &mut editor_state.new_relation_operation_index;
                *idx = (*idx).min(4);
                egui::ComboBox::from_id_salt("rel_op")
                    .selected_text(operations[*idx])
                    .show_ui(ui, |ui| {
                        for (i, name) in operations.iter().enumerate() {
                            ui.selectable_value(idx, i, *name);
                        }
                    });
            });
        }

        // Create button
        let name_valid = !editor_state.new_relation_name.trim().is_empty() && !concepts.is_empty();
        ui.add_enabled_ui(name_valid, |ui| {
            if ui
                .button(egui::RichText::new("+ Create Relation").color(BrandTheme::ACCENT_AMBER))
                .clicked()
                && name_valid
            {
                let concept_idx = editor_state.new_relation_concept_index;
                if let Some((concept_id, _, roles)) = concepts.get(concept_idx) {
                    let subject_id = roles
                        .get(editor_state.new_relation_subject_index)
                        .map_or_else(TypeId::new, |r| r.id);
                    let object_id = roles
                        .get(editor_state.new_relation_object_index)
                        .map_or_else(TypeId::new, |r| r.id);
                    let trigger = match editor_state.new_relation_trigger_index {
                        1 => RelationTrigger::OnExit,
                        2 => RelationTrigger::WhilePresent,
                        _ => RelationTrigger::OnEnter,
                    };
                    let effect = match editor_state.new_relation_effect_index {
                        1 => RelationEffect::Block { condition: None },
                        2 => RelationEffect::Allow { condition: None },
                        _ => RelationEffect::ModifyProperty {
                            target_property: editor_state
                                .new_relation_target_prop
                                .trim()
                                .to_string(),
                            source_property: editor_state
                                .new_relation_source_prop
                                .trim()
                                .to_string(),
                            operation: index_to_modify_operation(
                                editor_state.new_relation_operation_index,
                            ),
                        },
                    };
                    actions.push(EditorAction::CreateRelation {
                        name: editor_state.new_relation_name.trim().to_string(),
                        concept_id: *concept_id,
                        subject_role_id: subject_id,
                        object_role_id: object_id,
                        trigger,
                        effect,
                    });
                    editor_state.new_relation_name.clear();
                    editor_state.new_relation_target_prop.clear();
                    editor_state.new_relation_source_prop.clear();
                }
            }
        });
    });

    ui.add_space(4.0);

    // -- Relation list --
    if relation_registry.relations.is_empty() {
        ui.label(
            egui::RichText::new("No relations defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
        return;
    }

    let relation_snapshots: Vec<_> = relation_registry
        .relations
        .iter()
        .map(|r| {
            (
                r.id,
                r.name.clone(),
                r.concept_id,
                r.subject_role_id,
                r.object_role_id,
                r.trigger,
                r.effect.clone(),
            )
        })
        .collect();

    for (rel_id, rel_name, concept_id, subj_id, obj_id, trigger, effect) in &relation_snapshots {
        let mut delete_rel = false;

        egui::CollapsingHeader::new(rel_name)
            .id_salt(format!("rel_{rel_id:?}"))
            .show(ui, |ui| {
                let concept_name = concepts
                    .iter()
                    .find(|(id, _, _)| *id == *concept_id)
                    .map_or("?", |(_, n, _)| n.as_str());
                ui.label(format!("Concept: {concept_name}"));

                // Find role names
                let role_labels = concepts
                    .iter()
                    .find(|(id, _, _)| *id == *concept_id)
                    .map(|(_, _, roles)| roles.clone())
                    .unwrap_or_default();
                let subj_name = role_labels
                    .iter()
                    .find(|r| r.id == *subj_id)
                    .map_or("?", |r| r.name.as_str());
                let obj_name = role_labels
                    .iter()
                    .find(|r| r.id == *obj_id)
                    .map_or("?", |r| r.name.as_str());
                ui.label(format!("{subj_name} -> {obj_name}"));

                let trigger_str = match trigger {
                    RelationTrigger::OnEnter => "OnEnter",
                    RelationTrigger::OnExit => "OnExit",
                    RelationTrigger::WhilePresent => "WhilePresent",
                };
                ui.label(format!("Trigger: {trigger_str}"));

                let effect_str = format_relation_effect(effect);
                ui.label(format!("Effect: {effect_str}"));

                if ui
                    .button(egui::RichText::new("Delete").color(BrandTheme::DANGER))
                    .clicked()
                {
                    delete_rel = true;
                }
            });

        if delete_rel {
            actions.push(EditorAction::DeleteRelation { id: *rel_id });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_constraints_tab(
    ui: &mut egui::Ui,
    constraint_registry: &mut ConstraintRegistry,
    concept_registry: &ConceptRegistry,
    editor_state: &mut EditorState,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Constraints")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.add_space(8.0);

    let concepts: Vec<_> = concept_registry
        .concepts
        .iter()
        .map(|c| (c.id, c.name.clone(), c.role_labels.clone()))
        .collect();

    // -- Constraint list --
    let constraint_snapshots: Vec<_> = constraint_registry
        .constraints
        .iter()
        .map(|c| {
            (
                c.id,
                c.name.clone(),
                c.description.clone(),
                c.auto_generated,
                c.expression.clone(),
            )
        })
        .collect();

    if constraint_snapshots.is_empty() {
        ui.label(
            egui::RichText::new("No constraints defined")
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );
    } else {
        for (cst_id, cst_name, _cst_desc, auto_gen, expr) in &constraint_snapshots {
            ui.horizontal(|ui| {
                if *auto_gen {
                    ui.label(
                        egui::RichText::new("[auto]")
                            .small()
                            .color(BrandTheme::ACCENT_AMBER),
                    );
                }
                ui.label(&**cst_name);
                if ui.small_button("x").clicked() {
                    actions.push(EditorAction::DeleteConstraint { id: *cst_id });
                }
            });
            ui.label(
                egui::RichText::new(format_constraint_expr(expr))
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            ui.add_space(2.0);
        }
    }

    ui.add_space(4.0);

    // -- Create constraint --
    egui::CollapsingHeader::new(egui::RichText::new("New Constraint").small())
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_constraint_name);
            });
            ui.horizontal(|ui| {
                ui.label("Desc:");
                ui.text_edit_singleline(&mut editor_state.new_constraint_description);
            });

            // Concept selector
            let concept_names: Vec<&str> = concepts.iter().map(|(_, n, _)| n.as_str()).collect();
            if !concept_names.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Concept:");
                    let idx = &mut editor_state.new_constraint_concept_index;
                    *idx = (*idx).min(concept_names.len().saturating_sub(1));
                    egui::ComboBox::from_id_salt("cst_concept")
                        .selected_text(concept_names.get(*idx).copied().unwrap_or("--"))
                        .show_ui(ui, |ui| {
                            for (i, name) in concept_names.iter().enumerate() {
                                ui.selectable_value(idx, i, *name);
                            }
                        });
                });
            }

            // Expression type
            let expr_types = ["PropertyCompare", "CrossCompare", "IsType", "PathBudget"];
            ui.horizontal(|ui| {
                ui.label("Expr:");
                let idx = &mut editor_state.new_constraint_expr_type_index;
                *idx = (*idx).min(3);
                egui::ComboBox::from_id_salt("cst_expr")
                    .selected_text(expr_types[*idx])
                    .show_ui(ui, |ui| {
                        for (i, name) in expr_types.iter().enumerate() {
                            ui.selectable_value(idx, i, *name);
                        }
                    });
            });

            // Fields based on expression type
            let selected_concept_roles = concepts
                .get(editor_state.new_constraint_concept_index)
                .map(|(_, _, roles)| roles.clone())
                .unwrap_or_default();
            let role_names: Vec<&str> = selected_concept_roles
                .iter()
                .map(|r| r.name.as_str())
                .collect();

            match editor_state.new_constraint_expr_type_index {
                0 => {
                    // PropertyCompare
                    if !role_names.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Role:");
                            let idx = &mut editor_state.new_constraint_role_index;
                            *idx = (*idx).min(role_names.len().saturating_sub(1));
                            egui::ComboBox::from_id_salt("cst_role")
                                .selected_text(role_names.get(*idx).copied().unwrap_or("--"))
                                .show_ui(ui, |ui| {
                                    for (i, name) in role_names.iter().enumerate() {
                                        ui.selectable_value(idx, i, *name);
                                    }
                                });
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Prop:");
                        ui.text_edit_singleline(&mut editor_state.new_constraint_property);
                    });
                    let ops = ["==", "!=", "<", "<=", ">", ">="];
                    ui.horizontal(|ui| {
                        ui.label("Op:");
                        let idx = &mut editor_state.new_constraint_op_index;
                        *idx = (*idx).min(5);
                        egui::ComboBox::from_id_salt("cst_op")
                            .selected_text(ops[*idx])
                            .show_ui(ui, |ui| {
                                for (i, name) in ops.iter().enumerate() {
                                    ui.selectable_value(idx, i, *name);
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        ui.text_edit_singleline(&mut editor_state.new_constraint_value_str);
                    });
                }
                3 => {
                    // PathBudget
                    if !role_names.is_empty() {
                        ui.label(egui::RichText::new("Cost:").small());
                        ui.horizontal(|ui| {
                            ui.label("Role:");
                            let idx = &mut editor_state.new_constraint_role_index;
                            *idx = (*idx).min(role_names.len().saturating_sub(1));
                            egui::ComboBox::from_id_salt("cst_cost_role")
                                .selected_text(role_names.get(*idx).copied().unwrap_or("--"))
                                .show_ui(ui, |ui| {
                                    for (i, name) in role_names.iter().enumerate() {
                                        ui.selectable_value(idx, i, *name);
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Prop:");
                            ui.text_edit_singleline(&mut editor_state.new_constraint_property);
                        });
                        ui.label(egui::RichText::new("Budget:").small());
                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut editor_state.new_constraint_value_str);
                        });
                    }
                }
                _ => {
                    // CrossCompare and IsType — simplified for now
                    ui.label(
                        egui::RichText::new("(full editor coming soon)")
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                }
            }

            let name_valid =
                !editor_state.new_constraint_name.trim().is_empty() && !concepts.is_empty();
            ui.add_enabled_ui(name_valid, |ui| {
                if ui
                    .button(
                        egui::RichText::new("+ Create Constraint").color(BrandTheme::ACCENT_AMBER),
                    )
                    .clicked()
                    && name_valid
                {
                    let concept_idx = editor_state.new_constraint_concept_index;
                    if let Some((concept_id, _, roles)) = concepts.get(concept_idx) {
                        let expression = build_constraint_expression(editor_state, roles);
                        actions.push(EditorAction::CreateConstraint {
                            name: editor_state.new_constraint_name.trim().to_string(),
                            description: editor_state.new_constraint_description.trim().to_string(),
                            concept_id: *concept_id,
                            expression,
                        });
                        editor_state.new_constraint_name.clear();
                        editor_state.new_constraint_description.clear();
                        editor_state.new_constraint_value_str.clear();
                        editor_state.new_constraint_property.clear();
                    }
                }
            });
        });
}

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
                    crate::contracts::validation::SchemaErrorCategory::DanglingReference => {
                        "Dangling Ref"
                    }
                    crate::contracts::validation::SchemaErrorCategory::RoleMismatch => {
                        "Role Mismatch"
                    }
                    crate::contracts::validation::SchemaErrorCategory::PropertyMismatch => {
                        "Prop Mismatch"
                    }
                    crate::contracts::validation::SchemaErrorCategory::MissingBinding => {
                        "Missing Binding"
                    }
                    crate::contracts::validation::SchemaErrorCategory::InvalidExpression => {
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
        egui::RichText::new(format!("Columns ({})", crt.columns.len()))
            .color(BrandTheme::TEXT_SECONDARY),
    );
    for (i, col) in crt.columns.iter().enumerate() {
        ui.horizontal(|ui| {
            let type_label = match col.column_type {
                CrtColumnType::OddsRatio => "ratio",
                CrtColumnType::Differential => "diff",
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
                1 => CrtColumnType::Differential,
                _ => CrtColumnType::OddsRatio,
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
        egui::RichText::new(format!("Rows ({})", crt.rows.len())).color(BrandTheme::TEXT_SECONDARY),
    );
    for (i, row) in crt.rows.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&row.label)
                    .small()
                    .color(BrandTheme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("(die {}-{})", row.die_value_min, row.die_value_max))
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

    // Outcome grid (editable).
    if !crt.columns.is_empty() && !crt.rows.is_empty() {
        // Sync edit buffer when CRT dimensions change.
        let num_rows = crt.rows.len();
        let num_cols = crt.columns.len();
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
        egui::Grid::new("crt_outcome_grid")
            .striped(true)
            .show(ui, |ui| {
                // Header row.
                ui.label("");
                for col in &crt.columns {
                    ui.label(
                        egui::RichText::new(&col.label)
                            .small()
                            .strong()
                            .color(BrandTheme::ACCENT_TEAL),
                    );
                }
                ui.end_row();

                // Data rows with editable cells.
                for (ri, row) in crt.rows.iter().enumerate() {
                    ui.label(
                        egui::RichText::new(&row.label)
                            .small()
                            .strong()
                            .color(BrandTheme::ACCENT_TEAL),
                    );
                    for ci in 0..num_cols {
                        let cell = &mut editor_state.crt_outcome_labels[ri][ci];
                        let response = ui.add(
                            egui::TextEdit::singleline(cell)
                                .desired_width(28.0)
                                .font(egui::TextStyle::Small),
                        );
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

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn render_inspector(
    ui: &mut egui::Ui,
    selected_hex: &SelectedHex,
    tile_query: &Query<(&HexPosition, Entity), With<HexTile>>,
    tile_data_query: &mut Query<&mut EntityData, Without<UnitInstance>>,
    registry: &EntityTypeRegistry,
    enum_registry: &EnumRegistry,
    struct_registry: &StructRegistry,
) {
    egui::CollapsingHeader::new(
        egui::RichText::new("Inspector")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    )
    .default_open(true)
    .show(ui, |ui| {
        let Some(pos) = selected_hex.position else {
            ui.label(egui::RichText::new("No tile selected").color(BrandTheme::TEXT_SECONDARY));
            return;
        };

        ui.label(egui::RichText::new(format!("Position: ({}, {})", pos.q, pos.r)).monospace());

        let Some(entity) = tile_query
            .iter()
            .find(|(tp, _)| **tp == pos)
            .map(|(_, e)| e)
        else {
            ui.label("Tile not found");
            return;
        };

        let Ok(mut entity_data) = tile_data_query.get_mut(entity) else {
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_unit_inspector(
    ui: &mut egui::Ui,
    selected_unit: &SelectedUnit,
    unit_data_query: &mut Query<&mut EntityData, With<UnitInstance>>,
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
        let Some(entity) = selected_unit.entity else {
            ui.label(egui::RichText::new("No unit selected").color(BrandTheme::TEXT_SECONDARY));
            return;
        };

        let Ok(mut entity_data) = unit_data_query.get_mut(entity) else {
            ui.label("Unit entity not found");
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

#[allow(clippy::too_many_arguments)]
fn render_property_value_editor(
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

// ---------------------------------------------------------------------------
// Deferred Action Application
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn apply_actions(
    actions: Vec<EditorAction>,
    registry: &mut EntityTypeRegistry,
    enum_registry: &mut EnumRegistry,
    struct_registry: &mut StructRegistry,
    tile_data_query: &mut Query<&mut EntityData, Without<UnitInstance>>,
    active_board: &mut ActiveBoardType,
    active_token: &mut ActiveTokenType,
    selected_unit: &mut SelectedUnit,
    editor_state: &EditorState,
    commands: &mut Commands,
    concept_registry: &mut ConceptRegistry,
    relation_registry: &mut RelationRegistry,
    constraint_registry: &mut ConstraintRegistry,
    turn_structure: &mut TurnStructure,
    combat_results_table: &mut CombatResultsTable,
    combat_modifiers: &mut CombatModifierRegistry,
) {
    for action in actions {
        match action {
            EditorAction::CreateEntityType { name, role, color } => {
                registry.types.push(EntityType {
                    id: TypeId::new(),
                    name,
                    role,
                    color,
                    properties: Vec::new(),
                });
            }
            EditorAction::DeleteEntityType { id } => {
                // Determine the role of the type being deleted.
                let role = registry.get(id).map(|et| et.role);

                match role {
                    Some(EntityRole::BoardPosition) => {
                        // Find a fallback BoardPosition type.
                        let fallback_id = registry
                            .types_by_role(EntityRole::BoardPosition)
                            .iter()
                            .find(|et| et.id != id)
                            .map(|et| et.id);
                        if let Some(fallback) = fallback_id {
                            for mut ed in tile_data_query.iter_mut() {
                                if ed.entity_type_id == id {
                                    ed.entity_type_id = fallback;
                                    ed.properties.clear();
                                }
                            }
                            if active_board.entity_type_id == Some(id) {
                                active_board.entity_type_id = Some(fallback);
                            }
                        }
                    }
                    Some(EntityRole::Token) => {
                        let fallback_id = registry
                            .types_by_role(EntityRole::Token)
                            .iter()
                            .find(|et| et.id != id)
                            .map(|et| et.id);
                        if let Some(fallback) = fallback_id
                            && active_token.entity_type_id == Some(id)
                        {
                            active_token.entity_type_id = Some(fallback);
                        }
                    }
                    None => {}
                }

                registry.types.retain(|et| et.id != id);
            }
            EditorAction::AddProperty {
                type_id,
                name,
                prop_type,
                enum_options,
            } => {
                let final_type = match &prop_type {
                    PropertyType::Enum(_) => {
                        let enum_id = TypeId::new();
                        let options: Vec<String> = enum_options
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        enum_registry.insert(EnumDefinition {
                            id: enum_id,
                            name: name.clone(),
                            options,
                        });
                        PropertyType::Enum(enum_id)
                    }
                    PropertyType::EntityRef(_) => {
                        let role = match editor_state.new_prop_entity_ref_role {
                            1 => Some(EntityRole::BoardPosition),
                            2 => Some(EntityRole::Token),
                            _ => None,
                        };
                        PropertyType::EntityRef(role)
                    }
                    PropertyType::List(_) => {
                        let inner = match editor_state.new_prop_list_inner_type {
                            1 => PropertyType::Int,
                            2 => PropertyType::Float,
                            3 => PropertyType::String,
                            4 => PropertyType::Color,
                            _ => PropertyType::Bool,
                        };
                        PropertyType::List(Box::new(inner))
                    }
                    PropertyType::Map(_, _) => {
                        let enum_id = editor_state.new_prop_map_enum_id.unwrap_or_default();
                        let val_type = match editor_state.new_prop_map_value_type {
                            1 => PropertyType::Int,
                            2 => PropertyType::Float,
                            3 => PropertyType::String,
                            4 => PropertyType::Color,
                            _ => PropertyType::Bool,
                        };
                        PropertyType::Map(enum_id, Box::new(val_type))
                    }
                    PropertyType::Struct(_) => {
                        let sid = editor_state.new_prop_struct_id.unwrap_or_default();
                        PropertyType::Struct(sid)
                    }
                    PropertyType::IntRange { .. } => PropertyType::IntRange {
                        min: editor_state.new_prop_int_range_min,
                        max: editor_state.new_prop_int_range_max,
                    },
                    PropertyType::FloatRange { .. } => PropertyType::FloatRange {
                        min: editor_state.new_prop_float_range_min,
                        max: editor_state.new_prop_float_range_max,
                    },
                    other => other.clone(),
                };

                let default_value = PropertyValue::default_for(&final_type);
                if let Some(et) = registry.types.iter_mut().find(|et| et.id == type_id) {
                    et.properties.push(PropertyDefinition {
                        id: TypeId::new(),
                        name,
                        property_type: final_type,
                        default_value,
                    });
                }
            }
            EditorAction::RemoveProperty { type_id, prop_id } => {
                // Determine role to know which query to clean up.
                let role = registry.get(type_id).map(|et| et.role);

                if let Some(et) = registry.types.iter_mut().find(|et| et.id == type_id) {
                    et.properties.retain(|p| p.id != prop_id);
                }

                if role == Some(EntityRole::BoardPosition) {
                    for mut ed in tile_data_query.iter_mut() {
                        if ed.entity_type_id == type_id {
                            ed.properties.remove(&prop_id);
                        }
                    }
                }
                // Token and unknown roles: unit_data_query is not passed to
                // apply_actions; units with removed properties get defaults on
                // next inspector render (consistent with 0.3.0 behavior).
            }
            EditorAction::DeleteSelectedUnit => {
                if let Some(entity) = selected_unit.entity {
                    commands.entity(entity).despawn();
                    selected_unit.entity = None;
                }
            }
            EditorAction::CreateConcept { name, description } => {
                concept_registry
                    .concepts
                    .push(crate::contracts::ontology::Concept {
                        id: TypeId::new(),
                        name,
                        description,
                        role_labels: Vec::new(),
                    });
            }
            EditorAction::DeleteConcept { id } => {
                concept_registry.concepts.retain(|c| c.id != id);
                concept_registry.bindings.retain(|b| b.concept_id != id);
                relation_registry.relations.retain(|r| r.concept_id != id);
                constraint_registry
                    .constraints
                    .retain(|c| c.concept_id != id);
            }
            EditorAction::AddConceptRole {
                concept_id,
                name,
                allowed_roles,
            } => {
                if let Some(concept) = concept_registry
                    .concepts
                    .iter_mut()
                    .find(|c| c.id == concept_id)
                {
                    concept.role_labels.push(ConceptRole {
                        id: TypeId::new(),
                        name,
                        allowed_entity_roles: allowed_roles,
                    });
                }
            }
            EditorAction::RemoveConceptRole {
                concept_id,
                role_id,
            } => {
                if let Some(concept) = concept_registry
                    .concepts
                    .iter_mut()
                    .find(|c| c.id == concept_id)
                {
                    concept.role_labels.retain(|r| r.id != role_id);
                }
                concept_registry
                    .bindings
                    .retain(|b| !(b.concept_id == concept_id && b.concept_role_id == role_id));
            }
            EditorAction::BindEntityToConcept {
                entity_type_id,
                concept_id,
                concept_role_id,
            } => {
                concept_registry.bindings.push(ConceptBinding {
                    id: TypeId::new(),
                    entity_type_id,
                    concept_id,
                    concept_role_id,
                    property_bindings: Vec::new(),
                });
            }
            EditorAction::UnbindEntityFromConcept {
                concept_id: _,
                binding_id,
            } => {
                concept_registry.bindings.retain(|b| b.id != binding_id);
            }
            EditorAction::CreateRelation {
                name,
                concept_id,
                subject_role_id,
                object_role_id,
                trigger,
                effect,
            } => {
                relation_registry.relations.push(Relation {
                    id: TypeId::new(),
                    name,
                    concept_id,
                    subject_role_id,
                    object_role_id,
                    trigger,
                    effect,
                });
            }
            EditorAction::DeleteRelation { id } => {
                relation_registry.relations.retain(|r| r.id != id);
                constraint_registry
                    .constraints
                    .retain(|c| c.relation_id != Some(id));
            }
            EditorAction::CreateConstraint {
                name,
                description,
                concept_id,
                expression,
            } => {
                constraint_registry.constraints.push(Constraint {
                    id: TypeId::new(),
                    name,
                    description,
                    concept_id,
                    relation_id: None,
                    expression,
                    auto_generated: false,
                });
            }
            EditorAction::DeleteConstraint { id } => {
                constraint_registry.constraints.retain(|c| c.id != id);
            }
            EditorAction::CreateEnum { name, options } => {
                enum_registry.insert(EnumDefinition {
                    id: TypeId::new(),
                    name,
                    options,
                });
            }
            EditorAction::DeleteEnum { id } => {
                enum_registry.remove(id);
            }
            EditorAction::AddEnumOption { enum_id, option } => {
                if let Some(def) = enum_registry.get_mut(enum_id) {
                    def.options.push(option);
                }
            }
            EditorAction::RemoveEnumOption { enum_id, option } => {
                if let Some(def) = enum_registry.get_mut(enum_id) {
                    def.options.retain(|o| o != &option);
                }
            }
            EditorAction::CreateStruct { name } => {
                struct_registry.insert(StructDefinition {
                    id: TypeId::new(),
                    name,
                    fields: Vec::new(),
                });
            }
            EditorAction::DeleteStruct { id } => {
                struct_registry.remove(id);
            }
            EditorAction::AddStructField {
                struct_id,
                name,
                prop_type,
            } => {
                if let Some(def) = struct_registry.get_mut(struct_id) {
                    let default_value = PropertyValue::default_for(&prop_type);
                    def.fields.push(PropertyDefinition {
                        id: TypeId::new(),
                        name,
                        property_type: prop_type,
                        default_value,
                    });
                }
            }
            EditorAction::RemoveStructField {
                struct_id,
                field_id,
            } => {
                if let Some(def) = struct_registry.get_mut(struct_id) {
                    def.fields.retain(|f| f.id != field_id);
                }
            }
            // -- Mechanics actions --
            EditorAction::SetPlayerOrder { order } => {
                turn_structure.player_order = order;
            }
            EditorAction::AddPhase { name, phase_type } => {
                turn_structure.phases.push(Phase {
                    id: TypeId::new(),
                    name,
                    phase_type,
                    description: String::new(),
                });
            }
            EditorAction::RemovePhase { id } => {
                turn_structure.phases.retain(|p| p.id != id);
            }
            EditorAction::MovePhaseUp { id } => {
                if let Some(idx) = turn_structure.phases.iter().position(|p| p.id == id)
                    && idx > 0
                {
                    turn_structure.phases.swap(idx, idx - 1);
                }
            }
            EditorAction::MovePhaseDown { id } => {
                if let Some(idx) = turn_structure.phases.iter().position(|p| p.id == id)
                    && idx + 1 < turn_structure.phases.len()
                {
                    turn_structure.phases.swap(idx, idx + 1);
                }
            }
            EditorAction::AddCrtColumn {
                label,
                column_type,
                threshold,
            } => {
                combat_results_table.columns.push(CrtColumn {
                    label,
                    column_type,
                    threshold,
                });
                // Extend each existing row with a default outcome.
                for row_outcomes in &mut combat_results_table.outcomes {
                    row_outcomes.push(CombatOutcome {
                        label: "--".to_string(),
                        effect: None,
                    });
                }
            }
            EditorAction::RemoveCrtColumn { index } => {
                if index < combat_results_table.columns.len() {
                    combat_results_table.columns.remove(index);
                    for row_outcomes in &mut combat_results_table.outcomes {
                        if index < row_outcomes.len() {
                            row_outcomes.remove(index);
                        }
                    }
                }
            }
            EditorAction::AddCrtRow {
                label,
                die_min,
                die_max,
            } => {
                combat_results_table.rows.push(CrtRow {
                    label,
                    die_value_min: die_min,
                    die_value_max: die_max,
                });
                // Add a row of default outcomes.
                let num_cols = combat_results_table.columns.len();
                combat_results_table.outcomes.push(
                    (0..num_cols)
                        .map(|_| CombatOutcome {
                            label: "--".to_string(),
                            effect: None,
                        })
                        .collect(),
                );
            }
            EditorAction::RemoveCrtRow { index } => {
                if index < combat_results_table.rows.len() {
                    combat_results_table.rows.remove(index);
                    if index < combat_results_table.outcomes.len() {
                        combat_results_table.outcomes.remove(index);
                    }
                }
            }
            EditorAction::SetCrtOutcome { row, col, label } => {
                if let Some(row_outcomes) = combat_results_table.outcomes.get_mut(row)
                    && let Some(outcome) = row_outcomes.get_mut(col)
                {
                    outcome.label = label;
                }
            }
            EditorAction::AddCombatModifier {
                name,
                source,
                shift,
                priority,
            } => {
                combat_modifiers.modifiers.push(CombatModifierDefinition {
                    id: TypeId::new(),
                    name,
                    source,
                    column_shift: shift,
                    priority,
                    cap: None,
                    terrain_type_filter: None,
                });
            }
            EditorAction::RemoveCombatModifier { id } => {
                combat_modifiers.modifiers.retain(|m| m.id != id);
            }
        }
    }

    // Suppress unused warning.
    let _ = editor_state;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_property_type(pt: &PropertyType) -> &'static str {
    match pt {
        PropertyType::Bool => "Bool",
        PropertyType::Int => "Int",
        PropertyType::Float => "Float",
        PropertyType::String => "String",
        PropertyType::Color => "Color",
        PropertyType::Enum(_) => "Enum",
        PropertyType::EntityRef(_) => "EntityRef",
        PropertyType::List(_) => "List",
        PropertyType::Map(_, _) => "Map",
        PropertyType::Struct(_) => "Struct",
        PropertyType::IntRange { .. } => "IntRange",
        PropertyType::FloatRange { .. } => "FloatRange",
    }
}

fn index_to_property_type(index: usize) -> PropertyType {
    match index {
        1 => PropertyType::Int,
        2 => PropertyType::Float,
        3 => PropertyType::String,
        4 => PropertyType::Color,
        5 => PropertyType::Enum(TypeId::new()),
        6 => PropertyType::EntityRef(None),
        7 => PropertyType::List(Box::new(PropertyType::Int)),
        8 => PropertyType::Map(TypeId::new(), Box::new(PropertyType::Int)),
        9 => PropertyType::Struct(TypeId::new()),
        10 => PropertyType::IntRange { min: 0, max: 100 },
        11 => PropertyType::FloatRange { min: 0.0, max: 1.0 },
        _ => PropertyType::Bool,
    }
}

fn bevy_color_to_egui(color: Color) -> egui::Color32 {
    match color {
        Color::Srgba(c) => egui::Color32::from_rgba_unmultiplied(
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
            (c.alpha * 255.0) as u8,
        ),
        Color::LinearRgba(c) => {
            let srgba: bevy::color::Srgba = c.into();
            egui::Color32::from_rgba_unmultiplied(
                (srgba.red * 255.0) as u8,
                (srgba.green * 255.0) as u8,
                (srgba.blue * 255.0) as u8,
                (srgba.alpha * 255.0) as u8,
            )
        }
        _ => BrandTheme::TEXT_SECONDARY,
    }
}

fn egui_color_to_bevy(color: egui::Color32) -> Color {
    let [r, g, b, _] = color.to_array();
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn rgb_to_color32(rgb: [f32; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
    )
}

fn color32_to_rgb(c: egui::Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

fn format_relation_effect(effect: &RelationEffect) -> String {
    match effect {
        RelationEffect::ModifyProperty {
            target_property,
            source_property,
            operation,
        } => {
            let op = match operation {
                ModifyOperation::Add => "+",
                ModifyOperation::Subtract => "-",
                ModifyOperation::Multiply => "*",
                ModifyOperation::Min => "min",
                ModifyOperation::Max => "max",
            };
            format!("{target_property} {op} {source_property}")
        }
        RelationEffect::Block { .. } => "Block".to_string(),
        RelationEffect::Allow { .. } => "Allow".to_string(),
    }
}

fn format_constraint_expr(expr: &ConstraintExpr) -> String {
    match expr {
        ConstraintExpr::PropertyCompare {
            property_name,
            operator,
            value,
            ..
        } => {
            let op = format_compare_op(*operator);
            format!("{property_name} {op} {value:?}")
        }
        ConstraintExpr::CrossCompare {
            left_property,
            right_property,
            operator,
            ..
        } => {
            let op = format_compare_op(*operator);
            format!("{left_property} {op} {right_property}")
        }
        ConstraintExpr::IsType { .. } => "is type".to_string(),
        ConstraintExpr::IsNotType { .. } => "is not type".to_string(),
        ConstraintExpr::PathBudget {
            cost_property,
            budget_property,
            ..
        } => {
            format!("sum(path.{cost_property}) <= {budget_property}")
        }
        ConstraintExpr::All(exprs) => {
            let parts: Vec<String> = exprs.iter().map(format_constraint_expr).collect();
            format!("({})", parts.join(" AND "))
        }
        ConstraintExpr::Any(exprs) => {
            let parts: Vec<String> = exprs.iter().map(format_constraint_expr).collect();
            format!("({})", parts.join(" OR "))
        }
        ConstraintExpr::Not(expr) => {
            format!("NOT ({})", format_constraint_expr(expr))
        }
    }
}

fn format_compare_op(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "==",
        CompareOp::Ne => "!=",
        CompareOp::Lt => "<",
        CompareOp::Le => "<=",
        CompareOp::Gt => ">",
        CompareOp::Ge => ">=",
    }
}

fn index_to_modify_operation(index: usize) -> ModifyOperation {
    match index {
        1 => ModifyOperation::Subtract,
        2 => ModifyOperation::Multiply,
        3 => ModifyOperation::Min,
        4 => ModifyOperation::Max,
        _ => ModifyOperation::Add,
    }
}

fn index_to_compare_op(index: usize) -> CompareOp {
    match index {
        1 => CompareOp::Ne,
        2 => CompareOp::Lt,
        3 => CompareOp::Le,
        4 => CompareOp::Gt,
        5 => CompareOp::Ge,
        _ => CompareOp::Eq,
    }
}

fn build_constraint_expression(
    editor_state: &EditorState,
    roles: &[ConceptRole],
) -> ConstraintExpr {
    match editor_state.new_constraint_expr_type_index {
        0 => {
            // PropertyCompare
            let role_id = roles
                .get(editor_state.new_constraint_role_index)
                .map_or_else(TypeId::new, |r| r.id);
            let value = editor_state
                .new_constraint_value_str
                .trim()
                .parse::<i64>()
                .map_or(PropertyValue::Int(0), PropertyValue::Int);
            ConstraintExpr::PropertyCompare {
                role_id,
                property_name: editor_state.new_constraint_property.trim().to_string(),
                operator: index_to_compare_op(editor_state.new_constraint_op_index),
                value,
            }
        }
        3 => {
            // PathBudget
            let cost_role_id = roles
                .get(editor_state.new_constraint_role_index)
                .map_or_else(TypeId::new, |r| r.id);
            // For PathBudget, use the first role as budget role if different, or same role
            let budget_role_idx =
                usize::from(roles.len() > 1 && editor_state.new_constraint_role_index == 0);
            let budget_role_id = roles
                .get(budget_role_idx)
                .map_or_else(TypeId::new, |r| r.id);
            ConstraintExpr::PathBudget {
                concept_id: TypeId::new(), // Will be set from the concept
                cost_property: editor_state.new_constraint_property.trim().to_string(),
                cost_role_id,
                budget_property: editor_state.new_constraint_value_str.trim().to_string(),
                budget_role_id,
            }
        }
        _ => {
            // Placeholder for CrossCompare and IsType
            ConstraintExpr::All(Vec::new())
        }
    }
}
