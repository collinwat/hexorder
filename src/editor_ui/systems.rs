//! Systems for the `editor_ui` plugin.
//!
//! This module contains the dock orchestrator, tab bar renderers, settings
//! sync/restore systems, and dock layout persistence. Rendering logic for
//! individual tabs and panels lives in sibling modules (`render_panels`,
//! `render_play`, `render_design`, `render_ontology`, `render_rules`).
//! Deferred action application and helper functions live in `actions`.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use hexorder_contracts::editor_ui::{EditorTool, ViewportMargins, ViewportRect};
use hexorder_contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityData, EntityTypeRegistry, EnumRegistry, GameSystem,
    StructRegistry, UnitInstance,
};
use hexorder_contracts::hex_grid::{HexPosition, HexTile};
use hexorder_contracts::map_gen::MapGenParams;
use hexorder_contracts::mechanic_reference::{
    MechanicCatalog, MechanicCategory, TemplateAvailability,
};
use hexorder_contracts::mechanics::CombatResultsTable;
use hexorder_contracts::persistence::{
    AppScreen, CloseProjectEvent, LoadRequestEvent, SaveRequestEvent, Workspace,
};
use hexorder_contracts::settings::{SettingsRegistry, ThemeLibrary};
use hexorder_contracts::shortcuts::{
    CommandCategory, CommandExecutedEvent, CommandId, KeyBinding, ShortcutRegistry,
};
use hexorder_contracts::validation::SchemaValidation;

use egui_dock::DockArea;

use super::components::{
    BrandTheme, DockLayoutState, DockTab, EditorAction, EditorState, MechanicsParams,
    OntologyParams, OntologyTab, ProjectParams, SelectionParams, ShortcutDisplayEntry,
    TypeRegistryParams, WorkspacePreset,
};
use super::render_rules::{render_inspector, render_unit_inspector};

// Sibling-module functions used locally and re-exported for tests via pub(super).
pub(super) use super::actions::apply_actions;
pub(super) use super::render_design::{
    render_entity_type_editor, render_enums_tab, render_structs_tab,
};
pub(super) use super::render_ontology::{
    render_concepts_tab, render_constraints_tab, render_relations_tab,
};
pub(super) use super::render_panels::{
    render_about_panel, render_cell_palette, render_edge_palette, render_tool_mode,
    render_unit_palette, render_workspace_header,
};
pub(super) use super::render_rules::{
    render_accumulators, render_influence_rules, render_mechanics_tab, render_movement_cost_matrix,
    render_spawn_schedule, render_stacking_rule, render_validation_tab,
};

// Public systems re-exported for plugin registration in mod.rs.
#[cfg(feature = "inspector")]
pub use super::render_panels::debug_inspector_panel;
pub use super::render_panels::{
    configure_theme, launcher_system, render_grid_overlay, render_toast,
};
pub use super::render_play::play_panel_system;

// Test-only re-exports: available via systems:: path for test backward-compatibility.
#[cfg(test)]
pub(super) use super::actions::{
    apply_scaffold_recipe, bevy_color_to_egui, build_constraint_expression, color32_to_rgb,
    egui_color_to_bevy, format_compare_op, format_constraint_expr, format_property_type,
    format_relation_effect, index_to_compare_op, index_to_modify_operation, index_to_property_type,
    parse_scaffold_prop_type, rgb_to_color32,
};
#[cfg(test)]
pub(super) use super::render_design::render_entity_type_section;
#[cfg(test)]
pub(super) use super::render_panels::rgb;

// ---------------------------------------------------------------------------
// Zone rendering helpers (Scope 2 — native panel layout)
// ---------------------------------------------------------------------------

/// Updates `ViewportMargins` from the `ViewportRect` set by `editor_dock_system`.
/// Runs after the dock system so `viewport_rect` has been populated for this frame.
pub fn update_viewport_margins(
    mut contexts: EguiContexts,
    mut margins: ResMut<ViewportMargins>,
    viewport_rect: Res<ViewportRect>,
) {
    let Some(rect) = viewport_rect.0 else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen = ctx.input(bevy_egui::egui::InputState::viewport_rect);

    let new_left = rect.left();
    let new_top = rect.top();
    let new_right = screen.right() - rect.right();
    let new_bottom = screen.bottom() - rect.bottom();

    margins.left = new_left;
    margins.top = new_top;
    margins.right = new_right;
    margins.bottom = new_bottom;
}

/// Disables egui's built-in `Cmd+0` zoom-reset shortcut.
///
/// egui enables `zoom_with_keyboard` by default, which maps `Cmd+0` to
/// `set_zoom_factor(1.0)`.  On macOS, after a native file dialog
/// (`Cmd+O`), the Cmd modifier can remain "stuck" in the key state.
/// When the user then presses `0` (camera reset), egui interprets it
/// as `Cmd+0` and resets its zoom factor to 1.0 — collapsing the
/// Retina `HiDPI` scaling and causing a one-frame layout jitter.
///
/// Hexorder does not use egui's zoom feature, so we disable it.
/// Runs once at startup via a run-once condition.
pub fn disable_egui_zoom_shortcuts(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}

// ---------------------------------------------------------------------------
// Dock-based editor (Scope 4 — tab support)
// ---------------------------------------------------------------------------

/// Data for the Palette tab (tool selection, cell/unit palettes, project info).
pub(crate) struct PaletteData<'a> {
    pub(crate) editor_tool: &'a mut EditorTool,
    pub(crate) active_board: &'a mut ActiveBoardType,
    pub(crate) active_token: &'a mut ActiveTokenType,
    pub(crate) active_edge: &'a mut hexorder_contracts::editor_ui::ActiveEdgeType,
    pub(crate) project_workspace: &'a Workspace,
    pub(crate) project_game_system: &'a GameSystem,
}

/// Data for the Design tab (type editors, ontology sub-tabs).
pub(crate) struct DesignData<'a> {
    pub(crate) registry: &'a mut EntityTypeRegistry,
    pub(crate) enum_registry: &'a mut EnumRegistry,
    pub(crate) struct_registry: &'a mut StructRegistry,
    pub(crate) concept_registry: &'a mut hexorder_contracts::ontology::ConceptRegistry,
    pub(crate) relation_registry: &'a mut hexorder_contracts::ontology::RelationRegistry,
}

/// Data for the Rules tab (constraints, validation, mechanics sub-tabs).
pub(crate) struct RulesData<'a> {
    pub(crate) constraint_registry: &'a mut hexorder_contracts::ontology::ConstraintRegistry,
    pub(crate) turn_structure: &'a mut hexorder_contracts::mechanics::TurnStructure,
    pub(crate) combat_results_table: &'a mut CombatResultsTable,
    pub(crate) combat_modifiers: &'a mut hexorder_contracts::mechanics::CombatModifierRegistry,
    pub(crate) influence_rules: &'a mut hexorder_contracts::hex_grid::InfluenceRuleRegistry,
    pub(crate) stacking_rule: &'a mut hexorder_contracts::hex_grid::StackingRule,
    pub(crate) movement_cost_matrix: &'a mut hexorder_contracts::hex_grid::MovementCostMatrix,
    pub(crate) spawn_schedule: &'a mut hexorder_contracts::mechanics::SpawnSchedule,
    pub(crate) accumulator_registry: &'a mut hexorder_contracts::mechanics::AccumulatorRegistry,
    pub(crate) victory_conditions: &'a mut hexorder_contracts::mechanics::VictoryConditionRegistry,
}

/// Actions returned by `render_editor_menu_bar` for deferred dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EditorMenuAction {
    NewProject,
    OpenFile,
    Save,
    SaveAs,
    ExportPdf,
    CloseProject,
    Undo,
    Redo,
    SwitchPreset(WorkspacePreset),
    ShowAbout,
}

/// Data for the Inspector tab (selected tile/unit property editors).
pub(crate) struct InspectorData<'a> {
    pub(crate) tile_position: Option<HexPosition>,
    pub(crate) tile_entity_data: Option<&'a mut EntityData>,
    pub(crate) unit_entity_data: Option<&'a mut EntityData>,
}

/// Viewer context that borrows system resources for the duration of `DockArea::show()`.
///
/// Fields are grouped by tab ownership. Cross-cutting fields (`editor_state`, `actions`)
/// remain at the top level because they are shared across multiple tabs.
///
/// All fields use plain Rust references (no Bevy `Mut`/`NextState` wrappers) so the
/// struct can be constructed in unit tests without an ECS `World`.
pub(crate) struct EditorDockViewer<'a> {
    // Cross-cutting (shared by multiple tabs)
    pub(crate) editor_state: &'a mut EditorState,
    pub(crate) actions: &'a mut Vec<EditorAction>,
    pub(crate) next_screen: Option<AppScreen>,
    pub(crate) schema_validation: &'a SchemaValidation,
    // Single-tab fields
    pub(crate) viewport_rect: &'a mut ViewportRect,
    pub(crate) multi: &'a hexorder_contracts::editor_ui::Selection,
    pub(crate) mechanic_catalog: &'a MechanicCatalog,
    // Tab-specific groups
    pub(crate) palette: PaletteData<'a>,
    pub(crate) design: DesignData<'a>,
    pub(crate) rules: RulesData<'a>,
    pub(crate) inspector: InspectorData<'a>,
    // Map generation
    pub(crate) map_gen_params: &'a mut MapGenParams,
    pub(crate) is_generating: bool,
}

/// Renders the content for a single dock tab.
///
/// This is a free function (not a method on `EditorDockViewer`) so it can be called
/// from unit tests without constructing a full `egui_dock::TabViewer` impl.  The
/// viewer struct groups all the data needed; the function dispatches on the tab
/// variant and delegates to the appropriate rendering helper.
#[allow(clippy::too_many_lines)]
pub(crate) fn render_dock_tab(ui: &mut egui::Ui, tab: DockTab, viewer: &mut EditorDockViewer<'_>) {
    match tab {
        DockTab::Viewport => {
            viewer.viewport_rect.0 = Some(ui.max_rect());
        }
        DockTab::Palette => {
            render_workspace_header(
                ui,
                viewer.palette.project_workspace,
                viewer.palette.project_game_system,
            );
            if viewer.editor_state.toolbar_visible {
                render_tool_mode(ui, viewer.palette.editor_tool);
            }
            if ui
                .button(
                    egui::RichText::new("\u{25B6} Play")
                        .strong()
                        .color(BrandTheme::SUCCESS),
                )
                .on_hover_text("Enter play mode to test turns and combat")
                .clicked()
            {
                viewer.next_screen = Some(AppScreen::Play);
            }
            ui.separator();
            if *viewer.palette.editor_tool == EditorTool::Paint {
                render_cell_palette(ui, viewer.design.registry, viewer.palette.active_board);
            }
            if *viewer.palette.editor_tool == EditorTool::Place {
                render_unit_palette(ui, viewer.design.registry, viewer.palette.active_token);
            }
            if *viewer.palette.editor_tool == EditorTool::EdgePaint {
                render_edge_palette(ui, viewer.design.registry, viewer.palette.active_edge);
            }
        }
        DockTab::Design => {
            render_design_tab_bar(ui, viewer.editor_state);
            egui::ScrollArea::vertical().show(ui, |ui| {
                match viewer.editor_state.active_tab {
                    OntologyTab::Types => {
                        render_entity_type_editor(
                            ui,
                            viewer.design.registry,
                            viewer.editor_state,
                            viewer.actions,
                            viewer.design.enum_registry,
                            viewer.design.struct_registry,
                        );
                    }
                    OntologyTab::Enums => {
                        render_enums_tab(
                            ui,
                            viewer.design.enum_registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    OntologyTab::Structs => {
                        render_structs_tab(
                            ui,
                            viewer.design.struct_registry,
                            viewer.design.enum_registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    OntologyTab::Concepts => {
                        render_concepts_tab(
                            ui,
                            viewer.design.concept_registry,
                            viewer.design.registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    OntologyTab::Relations => {
                        render_relations_tab(
                            ui,
                            viewer.design.relation_registry,
                            viewer.design.concept_registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    // If user had a Rules sub-tab selected, show Types as fallback.
                    _ => {
                        viewer.editor_state.active_tab = OntologyTab::Types;
                        render_entity_type_editor(
                            ui,
                            viewer.design.registry,
                            viewer.editor_state,
                            viewer.actions,
                            viewer.design.enum_registry,
                            viewer.design.struct_registry,
                        );
                    }
                }
            });
        }
        DockTab::Rules => {
            render_rules_tab_bar(ui, viewer.editor_state);
            egui::ScrollArea::vertical().show(ui, |ui| {
                match viewer.editor_state.active_tab {
                    OntologyTab::Constraints => {
                        render_constraints_tab(
                            ui,
                            viewer.rules.constraint_registry,
                            viewer.design.concept_registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    OntologyTab::Validation => {
                        render_validation_tab(ui, viewer.schema_validation);
                    }
                    OntologyTab::Mechanics => {
                        render_mechanics_tab(
                            ui,
                            viewer.rules.turn_structure,
                            viewer.rules.combat_results_table,
                            viewer.rules.combat_modifiers,
                            viewer.editor_state,
                            viewer.actions,
                        );
                        ui.add_space(12.0);
                        render_influence_rules(
                            ui,
                            viewer.rules.influence_rules,
                            viewer.design.registry,
                            viewer.editor_state,
                        );
                        ui.add_space(12.0);
                        render_stacking_rule(
                            ui,
                            viewer.rules.stacking_rule,
                            viewer.design.registry,
                            viewer.editor_state,
                        );
                        ui.add_space(12.0);
                        render_movement_cost_matrix(
                            ui,
                            viewer.rules.movement_cost_matrix,
                            viewer.design.registry,
                            viewer.design.enum_registry,
                        );
                        ui.add_space(12.0);
                        render_spawn_schedule(
                            ui,
                            viewer.rules.spawn_schedule,
                            viewer.design.registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                        ui.add_space(12.0);
                        render_accumulators(
                            ui,
                            viewer.rules.accumulator_registry,
                            viewer.rules.victory_conditions,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                    // If user had a Design sub-tab selected, show Constraints as fallback.
                    _ => {
                        viewer.editor_state.active_tab = OntologyTab::Constraints;
                        render_constraints_tab(
                            ui,
                            viewer.rules.constraint_registry,
                            viewer.design.concept_registry,
                            viewer.editor_state,
                            viewer.actions,
                        );
                    }
                }
            });
        }
        DockTab::Inspector => {
            render_inspector(
                ui,
                viewer.inspector.tile_position,
                viewer.inspector.tile_entity_data.as_deref_mut(),
                viewer.design.registry,
                viewer.design.enum_registry,
                viewer.design.struct_registry,
            );
            render_unit_inspector(
                ui,
                viewer.inspector.unit_entity_data.as_deref_mut(),
                viewer.design.registry,
                viewer.design.enum_registry,
                viewer.design.struct_registry,
                viewer.actions,
            );
        }
        DockTab::Settings => {
            render_settings_tab(ui, viewer.editor_state);
        }
        DockTab::Selection => {
            render_selection_tab(ui, viewer.multi.entities.len());
        }
        DockTab::Validation => {
            render_validation_tab(ui, viewer.schema_validation);
        }
        DockTab::MechanicReference => {
            render_mechanic_reference(ui, viewer.mechanic_catalog, viewer.actions);
        }
        DockTab::MapGenerator => {
            render_map_generator(
                ui,
                viewer.map_gen_params,
                viewer.is_generating,
                viewer.actions,
            );
        }
        DockTab::Shortcuts => {
            render_shortcuts_tab(ui, &viewer.editor_state.shortcut_entries);
        }
    }
}

impl egui_dock::TabViewer for EditorDockViewer<'_> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut DockTab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut DockTab) {
        render_dock_tab(ui, *tab, self);
    }

    fn closeable(&mut self, tab: &mut DockTab) -> bool {
        tab.is_closeable()
    }

    fn clear_background(&self, tab: &DockTab) -> bool {
        !matches!(tab, DockTab::Viewport)
    }

    fn allowed_in_windows(&self, _tab: &mut DockTab) -> bool {
        false // No floating windows (pitch no-go).
    }
}

/// Sub-tab bar for the Design dock tab (Types, Enums, Structs, Concepts, Relations).
pub(crate) fn render_design_tab_bar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal_wrapped(|ui| {
        for tab in [
            OntologyTab::Types,
            OntologyTab::Enums,
            OntologyTab::Structs,
            OntologyTab::Concepts,
            OntologyTab::Relations,
        ] {
            let label = match tab {
                OntologyTab::Types => "Types",
                OntologyTab::Enums => "Enums",
                OntologyTab::Structs => "Structs",
                OntologyTab::Concepts => "Concepts",
                OntologyTab::Relations => "Relations",
                _ => continue,
            };
            if ui
                .selectable_label(editor_state.active_tab == tab, label)
                .clicked()
            {
                editor_state.active_tab = tab;
            }
        }
    });
    ui.separator();
}

/// Sub-tab bar for the Rules dock tab (Constraints, Validation, Mechanics).
pub(crate) fn render_rules_tab_bar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal_wrapped(|ui| {
        for tab in [
            OntologyTab::Constraints,
            OntologyTab::Validation,
            OntologyTab::Mechanics,
        ] {
            let label = match tab {
                OntologyTab::Constraints => "Constraints",
                OntologyTab::Validation => "Validation",
                OntologyTab::Mechanics => "Mechanics",
                _ => continue,
            };
            if ui
                .selectable_label(editor_state.active_tab == tab, label)
                .clicked()
            {
                editor_state.active_tab = tab;
            }
        }
    });
    ui.separator();
}

/// Renders the Settings tab content (font size, theme selector).
pub(crate) fn render_settings_tab(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.label(
        egui::RichText::new("Settings")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Font size:");
        if ui.button(" \u{2212} ").clicked() && editor_state.font_size_base > 10.0 {
            editor_state.font_size_base -= 1.0;
        }
        ui.monospace(format!("{}", editor_state.font_size_base as i32));
        if ui.button(" + ").clicked() && editor_state.font_size_base < 24.0 {
            editor_state.font_size_base += 1.0;
        }
    });
    ui.add_space(4.0);
    // Theme selector
    ui.horizontal(|ui| {
        ui.label("Theme:");
        egui::ComboBox::from_id_salt("theme_selector")
            .selected_text(&editor_state.active_theme_name)
            .show_ui(ui, |ui| {
                for name in &editor_state.theme_names {
                    let selected = *name == editor_state.active_theme_name;
                    if ui.selectable_label(selected, name).clicked() {
                        editor_state.active_theme_name.clone_from(name);
                    }
                }
            });
    });
}

/// Renders the Selection tab content (multi-selection summary).
pub(crate) fn render_selection_tab(ui: &mut egui::Ui, selection_count: usize) {
    ui.label(
        egui::RichText::new("Selection")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.separator();
    if selection_count > 0 {
        ui.label(
            egui::RichText::new(format!("{selection_count} tiles selected"))
                .color(BrandTheme::ACCENT_TEAL),
        );
    } else {
        ui.label(egui::RichText::new("No selection").color(BrandTheme::TEXT_SECONDARY));
    }
}

/// Renders the Shortcuts tab content (keyboard shortcuts reference).
pub(crate) fn render_shortcuts_tab(ui: &mut egui::Ui, entries: &[ShortcutDisplayEntry]) {
    ui.label(
        egui::RichText::new("Keyboard Shortcuts")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.separator();
    if entries.is_empty() {
        ui.label(egui::RichText::new("No shortcuts loaded").color(BrandTheme::TEXT_SECONDARY));
    } else {
        let mut current_category = "";
        for entry in entries {
            if entry.category != current_category {
                if !current_category.is_empty() {
                    ui.add_space(6.0);
                }
                ui.label(
                    egui::RichText::new(&entry.category)
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                ui.separator();
                current_category = &entry.category;
            }
            ui.horizontal(|ui| {
                ui.label(&entry.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if entry.binding.is_empty() {
                        ui.label(egui::RichText::new("\u{2014}").color(BrandTheme::TEXT_SECONDARY));
                    } else {
                        ui.monospace(
                            egui::RichText::new(&entry.binding).color(BrandTheme::ACCENT_TEAL),
                        );
                    }
                });
            });
        }
    }
}

/// Renders the status bar content (tool label, workspace preset, selected hex).
pub(crate) fn render_status_bar_content(
    ui: &mut egui::Ui,
    tool: EditorTool,
    preset_label: &str,
    selected_pos: Option<HexPosition>,
) {
    ui.horizontal_centered(|ui| {
        // Left: current tool mode.
        let tool_label = match tool {
            EditorTool::Select => "Select",
            EditorTool::Paint => "Paint",
            EditorTool::Place => "Place",
            EditorTool::EdgePaint => "Edge Paint",
            EditorTool::CombatSelect => "Combat Select",
        };
        ui.label(
            egui::RichText::new(tool_label)
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );

        ui.separator();

        // Left: workspace preset.
        ui.label(
            egui::RichText::new(preset_label)
                .small()
                .color(BrandTheme::TEXT_SECONDARY),
        );

        // Right-aligned items.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Selected hex coordinates.
            if let Some(pos) = selected_pos {
                ui.label(
                    egui::RichText::new(format!("({}, {})", pos.q, pos.r))
                        .small()
                        .color(BrandTheme::TEXT_SECONDARY),
                );
            }
        });
    });
}

/// Configures the `egui_dock::Style` to match the brand theme.
pub(crate) fn configure_dock_style(base_style: &egui::Style) -> egui_dock::Style {
    let mut style = egui_dock::Style::from_egui(base_style);
    // Tab bar background.
    style.tab_bar.bg_fill = BrandTheme::BG_DEEP;
    style.tab_bar.hline_color = BrandTheme::BORDER_SUBTLE;
    // Tab body background.
    style.tab.tab_body.bg_fill = BrandTheme::BG_PANEL;
    // Active/focused tab.
    style.tab.active.text_color = BrandTheme::TEXT_PRIMARY;
    style.tab.active.bg_fill = BrandTheme::BG_PANEL;
    style.tab.focused.text_color = BrandTheme::TEXT_PRIMARY;
    style.tab.focused.bg_fill = BrandTheme::BG_PANEL;
    // Inactive tab.
    style.tab.inactive.text_color = BrandTheme::TEXT_SECONDARY;
    style.tab.inactive.bg_fill = BrandTheme::BG_DEEP;
    // Hovered tab.
    style.tab.hovered.text_color = BrandTheme::TEXT_PRIMARY;
    style.tab.hovered.bg_fill = BrandTheme::BG_SURFACE;
    // Separator (between dock zones).
    style.separator.color_idle = BrandTheme::BORDER_SUBTLE;
    style.separator.color_hovered = BrandTheme::ACCENT_TEAL;
    style.separator.color_dragged = BrandTheme::ACCENT_TEAL;
    // Overlay (drag-to-dock indicators).
    style.overlay.selection_color = egui::Color32::from_rgba_premultiplied(0, 92, 128, 80);
    style
}

/// Renders the editor menu bar (File, Edit, View, Help) and returns actions for
/// deferred dispatch. Pure function — no ECS types.
pub(crate) fn render_editor_menu_bar(
    ui: &mut egui::Ui,
    can_undo: bool,
    undo_desc: Option<&str>,
    can_redo: bool,
    redo_desc: Option<&str>,
    active_preset: WorkspacePreset,
) -> Vec<EditorMenuAction> {
    let mut actions = Vec::new();
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New          Cmd+N").clicked() {
                actions.push(EditorMenuAction::NewProject);
                ui.close();
            }
            if ui.button("Open...      Cmd+O").clicked() {
                actions.push(EditorMenuAction::OpenFile);
                ui.close();
            }
            ui.separator();
            if ui.button("Save         Cmd+S").clicked() {
                actions.push(EditorMenuAction::Save);
                ui.close();
            }
            if ui.button("Save As...   Cmd+Shift+S").clicked() {
                actions.push(EditorMenuAction::SaveAs);
                ui.close();
            }
            ui.separator();
            if ui.button("Export PDF   Cmd+Shift+E").clicked() {
                actions.push(EditorMenuAction::ExportPdf);
                ui.close();
            }
            ui.separator();
            if ui.button("Close        Cmd+W").clicked() {
                actions.push(EditorMenuAction::CloseProject);
                ui.close();
            }
        });
        ui.menu_button("Edit", |ui| {
            let undo_label = undo_desc.map_or_else(
                || "Undo         Cmd+Z".to_string(),
                |desc| format!("Undo {desc:<5}Cmd+Z"),
            );
            let undo_btn = ui.add_enabled(can_undo, egui::Button::new(undo_label));
            if undo_btn.clicked() {
                actions.push(EditorMenuAction::Undo);
                ui.close();
            }
            let redo_label = redo_desc.map_or_else(
                || "Redo         Cmd+Shift+Z".to_string(),
                |desc| format!("Redo {desc:<5}Cmd+Shift+Z"),
            );
            let redo_btn = ui.add_enabled(can_redo, egui::Button::new(redo_label));
            if redo_btn.clicked() {
                actions.push(EditorMenuAction::Redo);
                ui.close();
            }
        });
        ui.menu_button("View", |ui| {
            ui.label(
                egui::RichText::new("Workspace")
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
            );
            for (preset, label) in [
                (WorkspacePreset::MapEditing, "Map Editing      Cmd+1"),
                (WorkspacePreset::UnitDesign, "Unit Design      Cmd+2"),
                (WorkspacePreset::RuleAuthoring, "Rule Authoring   Cmd+3"),
                (WorkspacePreset::Playtesting, "Playtesting      Cmd+4"),
            ] {
                let response = ui.selectable_label(active_preset == preset, label);
                if response.clicked() {
                    actions.push(EditorMenuAction::SwitchPreset(preset));
                    ui.close();
                }
            }
        });
        ui.menu_button("Help", |ui| {
            if ui.button("About Hexorder").clicked() {
                actions.push(EditorMenuAction::ShowAbout);
                ui.close();
            }
        });
    });
    actions
}

/// Renders the Mechanic Reference panel — a browsable catalog organized by category.
pub(crate) fn render_mechanic_reference(
    ui: &mut egui::Ui,
    catalog: &MechanicCatalog,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Mechanic Reference")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.label(
        egui::RichText::new("Engelstein taxonomy — browse wargame mechanics by area")
            .small()
            .color(BrandTheme::TEXT_SECONDARY),
    );
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for category in MechanicCategory::all() {
            let entries = catalog.entries_by_category(*category);
            let header = format!("{} ({})", category.display_name(), entries.len());
            let id = ui.make_persistent_id(category.display_name());
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    ui.label(
                        egui::RichText::new(header)
                            .strong()
                            .color(BrandTheme::TEXT_PRIMARY),
                    );
                })
                .body(|ui| {
                    ui.label(
                        egui::RichText::new(category.description())
                            .small()
                            .color(BrandTheme::TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);

                    for entry in &entries {
                        let entry_id = ui.make_persistent_id(&entry.name);
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            entry_id,
                            false,
                        )
                        .show_header(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&entry.name).color(BrandTheme::ACCENT_TEAL),
                            );
                        })
                        .body(|ui| {
                            ui.label(&entry.description);
                            ui.add_space(4.0);

                            if !entry.example_games.is_empty() {
                                ui.label(
                                    egui::RichText::new("Example games:")
                                        .small()
                                        .strong()
                                        .color(BrandTheme::TEXT_SECONDARY),
                                );
                                ui.label(
                                    egui::RichText::new(entry.example_games.join(", "))
                                        .small()
                                        .color(BrandTheme::TEXT_TERTIARY),
                                );
                                ui.add_space(4.0);
                            }

                            if !entry.design_considerations.is_empty() {
                                ui.label(
                                    egui::RichText::new("Design considerations:")
                                        .small()
                                        .strong()
                                        .color(BrandTheme::TEXT_SECONDARY),
                                );
                                ui.label(
                                    egui::RichText::new(&entry.design_considerations)
                                        .small()
                                        .color(BrandTheme::TEXT_TERTIARY),
                                );
                            }

                            if let TemplateAvailability::Available {
                                preview,
                                template_id,
                            } = &entry.template
                            {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(format!("Template: {preview}"))
                                        .small()
                                        .color(BrandTheme::ACCENT_AMBER),
                                );
                                if ui.button("Use Template").clicked() {
                                    actions.push(EditorAction::ApplyTemplate {
                                        template_id: template_id.clone(),
                                    });
                                }
                            }
                        });
                    }
                });
        }
    });
}

/// Renders the Map Generator dock tab (noise parameters + generate button).
pub(crate) fn render_map_generator(
    ui: &mut egui::Ui,
    params: &mut MapGenParams,
    is_generating: bool,
    actions: &mut Vec<EditorAction>,
) {
    ui.label(
        egui::RichText::new("Map Generator")
            .strong()
            .color(BrandTheme::ACCENT_AMBER),
    );
    ui.separator();

    // Seed
    ui.horizontal(|ui| {
        ui.label("Seed:");
        ui.add(egui::DragValue::new(&mut params.seed).speed(1.0));
    });

    ui.add_space(4.0);

    // Noise parameters
    egui::CollapsingHeader::new("Noise Parameters")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Octaves:");
                let mut octaves_u32 = params.octaves as u32;
                if ui
                    .add(
                        egui::DragValue::new(&mut octaves_u32)
                            .speed(0.1)
                            .range(1..=12),
                    )
                    .changed()
                {
                    params.octaves = octaves_u32 as usize;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Frequency:");
                ui.add(
                    egui::DragValue::new(&mut params.frequency)
                        .speed(0.001)
                        .range(0.001..=1.0)
                        .max_decimals(3),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Amplitude:");
                ui.add(
                    egui::DragValue::new(&mut params.amplitude)
                        .speed(0.01)
                        .range(0.01..=5.0)
                        .max_decimals(2),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Lacunarity:");
                ui.add(
                    egui::DragValue::new(&mut params.lacunarity)
                        .speed(0.01)
                        .range(1.0..=4.0)
                        .max_decimals(2),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Persistence:");
                ui.add(
                    egui::DragValue::new(&mut params.persistence)
                        .speed(0.01)
                        .range(0.01..=1.0)
                        .max_decimals(2),
                );
            });
        });

    ui.add_space(8.0);

    // Reset to defaults
    if ui.button("Reset Defaults").clicked() {
        *params = MapGenParams::default();
    }

    ui.add_space(4.0);

    // Generate button (disabled while generation is in progress)
    ui.add_enabled_ui(!is_generating, |ui| {
        if ui
            .button("Generate Map")
            .on_hover_text("Generate terrain using current parameters")
            .clicked()
        {
            actions.push(EditorAction::GenerateMap);
        }
    });
}

/// Unified dock system. Renders the menu bar as a native `TopBottomPanel`, then
/// delegates all tabbed content to `DockArea`. Replaces the four separate zone systems.
#[allow(clippy::too_many_arguments)]
pub fn editor_dock_system(
    mut contexts: EguiContexts,
    mut selection: SelectionParams,
    mut editor_state: ResMut<EditorState>,
    project: ProjectParams,
    mut type_regs: TypeRegistryParams,
    mut tile_data_query: Query<&mut EntityData, Without<UnitInstance>>,
    tile_query: Query<(&HexPosition, Entity), With<HexTile>>,
    mut unit_data_query: Query<&mut EntityData, With<UnitInstance>>,
    mut commands: Commands,
    mut ontology: OntologyParams,
    mut mechanics: MechanicsParams,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut dock_layout: ResMut<DockLayoutState>,
    mut viewport_rect: ResMut<ViewportRect>,
    validation: Res<SchemaValidation>,
    mut map_gen: super::components::MapGenDockedParams,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let is_generating = map_gen.generate.is_some();

    // Menu bar as native TopBottomPanel (above dock area).
    let menu_actions = egui::TopBottomPanel::top("editor_menu_bar")
        .show(ctx, |ui| {
            render_editor_menu_bar(
                ui,
                project.undo_stack.can_undo(),
                project.undo_stack.undo_description().as_deref(),
                project.undo_stack.can_redo(),
                project.undo_stack.redo_description().as_deref(),
                dock_layout.active_preset,
            )
        })
        .inner;

    // Dispatch menu actions to ECS commands.
    for action in menu_actions {
        match action {
            EditorMenuAction::NewProject => commands.trigger(CloseProjectEvent),
            EditorMenuAction::OpenFile => commands.trigger(LoadRequestEvent),
            EditorMenuAction::Save => {
                commands.trigger(SaveRequestEvent { save_as: false });
            }
            EditorMenuAction::SaveAs => {
                commands.trigger(SaveRequestEvent { save_as: true });
            }
            EditorMenuAction::ExportPdf => {
                commands.trigger(CommandExecutedEvent {
                    command_id: CommandId("file.export_pnp"),
                });
            }
            EditorMenuAction::CloseProject => {
                commands.trigger(CommandExecutedEvent {
                    command_id: CommandId("mode.close"),
                });
            }
            EditorMenuAction::Undo => {
                commands.trigger(CommandExecutedEvent {
                    command_id: CommandId("edit.undo"),
                });
            }
            EditorMenuAction::Redo => {
                commands.trigger(CommandExecutedEvent {
                    command_id: CommandId("edit.redo"),
                });
            }
            EditorMenuAction::SwitchPreset(preset) => {
                dock_layout.apply_preset(preset);
            }
            EditorMenuAction::ShowAbout => {
                editor_state.about_panel_visible = true;
            }
        }
    }

    // Status bar at the bottom of the editor window.
    let status_tool = *selection.editor_tool;
    let status_preset = dock_layout.active_preset.to_string();
    let status_pos = selection.selected_hex.position;
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .show(ctx, |ui| {
            render_status_bar_content(ui, status_tool, &status_preset, status_pos);
        });

    // About panel (modal, renders over everything).
    render_about_panel(ctx, &mut editor_state);

    let mut actions: Vec<EditorAction> = Vec::new();

    // Pre-extract inspector data from queries.
    let tile_position = selection.selected_hex.position;
    let tile_entity = tile_position.and_then(|pos| {
        tile_query
            .iter()
            .find(|(tp, _)| **tp == pos)
            .map(|(_, e)| e)
    });
    let mut tile_entity_data = tile_entity.and_then(|e| tile_data_query.get_mut(e).ok());
    let mut unit_entity_data = selection
        .selected_unit
        .entity
        .and_then(|e| unit_data_query.get_mut(e).ok());

    // DockArea for all tabbed content.
    let mut viewer = EditorDockViewer {
        editor_state: &mut editor_state,
        actions: &mut actions,
        next_screen: None,
        schema_validation: &validation,
        viewport_rect: &mut viewport_rect,
        multi: &selection.multi,
        mechanic_catalog: &mechanics.mechanic_catalog,
        palette: PaletteData {
            editor_tool: &mut selection.editor_tool,
            active_board: &mut selection.active_board,
            active_token: &mut selection.active_token,
            active_edge: &mut selection.active_edge,
            project_workspace: &project.workspace,
            project_game_system: &project.game_system,
        },
        design: DesignData {
            registry: &mut type_regs.registry,
            enum_registry: &mut type_regs.enum_registry,
            struct_registry: &mut type_regs.struct_registry,
            concept_registry: &mut ontology.concept_registry,
            relation_registry: &mut ontology.relation_registry,
        },
        rules: RulesData {
            constraint_registry: &mut ontology.constraint_registry,
            turn_structure: &mut mechanics.turn_structure,
            combat_results_table: &mut mechanics.combat_results_table,
            combat_modifiers: &mut mechanics.combat_modifiers,
            influence_rules: &mut mechanics.influence_rules,
            stacking_rule: &mut mechanics.stacking_rule,
            movement_cost_matrix: &mut mechanics.movement_cost_matrix,
            spawn_schedule: &mut mechanics.spawn_schedule,
            accumulator_registry: &mut mechanics.accumulator_registry,
            victory_conditions: &mut mechanics.victory_conditions,
        },
        inspector: InspectorData {
            tile_position,
            tile_entity_data: tile_entity_data.as_deref_mut(),
            unit_entity_data: unit_entity_data.as_deref_mut(),
        },
        map_gen_params: &mut map_gen.params,
        is_generating,
    };

    // Configure dock area style to match brand theme.
    let style = configure_dock_style(ctx.style().as_ref());

    DockArea::new(&mut dock_layout.dock_state)
        .style(style)
        .draggable_tabs(true)
        .show_close_buttons(true)
        .show_leaf_close_all_buttons(false)
        .show_leaf_collapse_buttons(false)
        .show(ctx, &mut viewer);

    // Apply deferred screen transition (if the Play button was clicked).
    if let Some(screen) = viewer.next_screen {
        next_state.set(screen);
    }

    // Apply deferred actions.
    apply_actions(
        actions,
        &mut type_regs.registry,
        &mut type_regs.enum_registry,
        &mut type_regs.struct_registry,
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
        &mechanics.mechanic_catalog,
        &mut mechanics.spawn_schedule,
        &mut mechanics.accumulator_registry,
        &mut mechanics.victory_conditions,
    );
}

/// Syncs `Workspace.workspace_preset` from `DockLayoutState.active_preset`.
/// Runs after `editor_dock_system` so the View menu and keyboard shortcuts
/// are captured before the next save.
pub fn sync_workspace_preset(dock_layout: Res<DockLayoutState>, mut workspace: ResMut<Workspace>) {
    let preset_id = dock_layout.active_preset.as_id();
    if workspace.workspace_preset != preset_id {
        preset_id.clone_into(&mut workspace.workspace_preset);
    }
}

/// Restores the workspace preset from `SettingsRegistry` on editor entry.
/// Runs once via `OnEnter(AppScreen::Editor)`, after `SettingsReady`.
pub fn restore_workspace_preset(
    settings: Res<SettingsRegistry>,
    mut dock_layout: ResMut<DockLayoutState>,
) {
    if settings.editor.workspace_preset.is_empty() {
        return;
    }
    let preset = WorkspacePreset::from_id(&settings.editor.workspace_preset);
    if dock_layout.active_preset != preset {
        dock_layout.apply_preset(preset);
    }
}

/// Syncs `EditorState.font_size_base` → `Workspace.font_size_base` each frame.
/// Runs in the editor system chain so the save system always has the current value.
pub fn sync_font_size(editor_state: Res<EditorState>, mut workspace: ResMut<Workspace>) {
    if (workspace.font_size_base - editor_state.font_size_base).abs() > f32::EPSILON {
        workspace.font_size_base = editor_state.font_size_base;
    }
}

/// Restores `SettingsRegistry.editor.font_size` → `EditorState.font_size_base` on editor entry.
/// Runs once via `OnEnter(AppScreen::Editor)`, after `SettingsReady`.
pub fn restore_font_size(settings: Res<SettingsRegistry>, mut editor_state: ResMut<EditorState>) {
    editor_state.font_size_base = settings.editor.font_size;
}

/// Restores theme state from `SettingsRegistry` + `ThemeLibrary` → `EditorState` on editor entry.
/// Populates available theme names and active theme for the Settings dock tab.
pub fn restore_theme(
    settings: Res<SettingsRegistry>,
    theme_library: Res<ThemeLibrary>,
    mut editor_state: ResMut<EditorState>,
) {
    editor_state.theme_names = theme_library
        .themes
        .iter()
        .map(|t| t.name.clone())
        .collect();
    editor_state
        .active_theme_name
        .clone_from(&settings.active_theme);
}

/// Syncs `EditorState.active_theme_name` → `SettingsRegistry.active_theme` each frame.
/// Runs after `editor_dock_system` so Settings tab changes propagate immediately.
pub fn sync_theme(editor_state: Res<EditorState>, mut settings: ResMut<SettingsRegistry>) {
    if settings.active_theme != editor_state.active_theme_name {
        settings
            .active_theme
            .clone_from(&editor_state.active_theme_name);
    }
}

/// Populates the keyboard shortcuts reference data on `EditorState`.
/// Runs on `OnEnter(AppScreen::Editor)`, after `SettingsReady`.
pub fn restore_shortcuts(registry: Res<ShortcutRegistry>, mut editor_state: ResMut<EditorState>) {
    let commands = registry.commands();
    let mut entries: Vec<ShortcutDisplayEntry> = Vec::with_capacity(commands.len());

    // Sort by category for grouped display.
    let category_order = |cat: &CommandCategory| match cat {
        CommandCategory::File => 0,
        CommandCategory::Edit => 1,
        CommandCategory::View => 2,
        CommandCategory::Tool => 3,
        CommandCategory::Mode => 4,
        CommandCategory::Camera => 5,
    };

    let mut sorted: Vec<_> = commands.iter().collect();
    sorted.sort_by_key(|c| category_order(&c.category));

    for cmd in sorted {
        let binding_str = cmd
            .bindings
            .iter()
            .map(KeyBinding::display_string)
            .collect::<Vec<_>>()
            .join(", ");

        entries.push(ShortcutDisplayEntry {
            category: format!("{:?}", cmd.category),
            name: cmd.name.clone(),
            binding: binding_str,
        });
    }

    editor_state.shortcut_entries = entries;
}

// ---------------------------------------------------------------------------
// Dock layout persistence
// ---------------------------------------------------------------------------

/// Returns the path to the dock layout config file.
#[allow(dead_code)]
pub(super) fn dock_layout_config_path() -> std::path::PathBuf {
    #[cfg(feature = "macos-app")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home).join("Library/Application Support/hexorder/dock_layout.ron")
    }

    #[cfg(not(feature = "macos-app"))]
    {
        std::path::PathBuf::from("config").join("dock_layout.ron")
    }
}

/// Saves the dock layout to a RON config file when it changes.
/// Uses Bevy change detection on `DockLayoutState`.
#[allow(dead_code)]
pub fn save_dock_layout(dock_layout: Res<DockLayoutState>) {
    if !dock_layout.is_changed() {
        return;
    }

    let file = super::components::DockLayoutFile {
        preset: dock_layout.active_preset,
        dock_state: dock_layout.dock_state.clone(),
    };

    let path = dock_layout_config_path();
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        warn!("Failed to create dock layout config dir: {e}");
        return;
    }

    let config = ron::ser::PrettyConfig::default();
    match ron::ser::to_string_pretty(&file, config) {
        Ok(ron_str) => {
            if let Err(e) = std::fs::write(&path, ron_str) {
                warn!("Failed to write dock layout to {}: {e}", path.display());
            }
        }
        Err(e) => {
            warn!("Failed to serialize dock layout: {e}");
        }
    }
}

/// Restores the dock layout from a RON config file on editor entry.
/// Runs once via `OnEnter(AppScreen::Editor)`, after `restore_workspace_preset`.
/// If a saved layout exists, it overrides the preset-based layout.
#[allow(dead_code)]
pub fn restore_dock_layout(mut dock_layout: ResMut<DockLayoutState>) {
    let path = dock_layout_config_path();

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!(
                "No dock layout config at {}, using preset default",
                path.display()
            );
            return;
        }
        Err(e) => {
            warn!("Failed to read dock layout config {}: {e}", path.display());
            return;
        }
    };

    match ron::from_str::<super::components::DockLayoutFile>(&contents) {
        Ok(file) => {
            // Validate: a usable dock state must contain at least one tab.
            let tab_count: usize = file
                .dock_state
                .main_surface()
                .iter()
                .filter_map(|node| node.tabs())
                .map(<[super::components::DockTab]>::len)
                .sum();
            if tab_count == 0 {
                warn!(
                    "Dock layout at {} has no tabs — discarding stale layout",
                    path.display()
                );
                return;
            }
            dock_layout.dock_state = file.dock_state;
            dock_layout.active_preset = file.preset;
            info!(
                "Restored dock layout ({tab_count} tabs) from {}",
                path.display()
            );
        }
        Err(e) => {
            warn!("Failed to parse dock layout config {}: {e}", path.display());
        }
    }
}
