//! Systems for the `editor_ui` feature plugin.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::contracts::editor_ui::EditorTool;
use crate::contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityData, EntityRole, EntityType, EntityTypeRegistry,
    EnumDefinition, GameSystem, PropertyDefinition, PropertyType, PropertyValue, SelectedUnit,
    TypeId, UnitInstance,
};
use crate::contracts::hex_grid::{HexPosition, HexTile, SelectedHex};

use super::components::EditorState;

/// Deferred actions to apply after the egui closure completes.
/// Avoids side effects inside the closure (multi-pass safe).
enum EditorAction {
    CreateEntityType {
        name: String,
        role: EntityRole,
        color: Color,
    },
    DeleteEntityType {
        id: TypeId,
    },
    AddProperty {
        type_id: TypeId,
        name: String,
        prop_type: PropertyType,
        enum_options: String,
    },
    RemoveProperty {
        type_id: TypeId,
        prop_id: TypeId,
    },
    DeleteSelectedUnit,
}

/// Configures the egui dark theme every frame. This is idempotent and cheap
/// (a few struct assignments). Running every frame guarantees the theme is
/// always applied, even after a window visibility change resets the context.
pub fn configure_theme(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_gray(25);
    visuals.window_fill = egui::Color32::from_gray(25);
    visuals.extreme_bg_color = egui::Color32::from_gray(10);
    visuals.faint_bg_color = egui::Color32::from_gray(35);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_gray(30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(40);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(55);
    visuals.widgets.active.bg_fill = egui::Color32::from_gray(70);
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 92, 128);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Monospace),
    );
    ctx.set_style(style);
}

/// Main editor panel system. Renders the left side panel with all editor sections.
#[allow(clippy::too_many_arguments)]
pub fn editor_panel_system(
    mut contexts: EguiContexts,
    mut editor_tool: ResMut<EditorTool>,
    mut active_board: ResMut<ActiveBoardType>,
    mut active_token: ResMut<ActiveTokenType>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut editor_state: ResMut<EditorState>,
    selected_hex: Res<SelectedHex>,
    game_system: Option<Res<GameSystem>>,
    mut registry: Option<ResMut<EntityTypeRegistry>>,
    mut tile_data_query: Query<&mut EntityData, Without<UnitInstance>>,
    mut unit_data_query: Query<&mut EntityData, With<UnitInstance>>,
    tile_query: Query<(&HexPosition, Entity), With<HexTile>>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut actions: Vec<EditorAction> = Vec::new();

    egui::SidePanel::left("editor_panel")
        .default_width(260.0)
        .show(ctx, |ui| {
            // -- Game System Info --
            render_game_system_info(ui, &game_system);

            // -- Tool Mode --
            render_tool_mode(ui, &mut editor_tool);

            // -- Cell Palette (Paint mode) --
            if *editor_tool == EditorTool::Paint {
                render_cell_palette(ui, &registry, &mut active_board);
            }

            // -- Unit Palette (Place mode) --
            if *editor_tool == EditorTool::Place {
                render_unit_palette(ui, &registry, &mut active_token);
            }

            // -- Entity Type Editor --
            render_entity_type_editor(ui, &mut registry, &mut editor_state, &mut actions);

            ui.separator();

            // -- Unit Inspector (takes priority when a unit is selected) --
            if selected_unit.entity.is_some() {
                render_unit_inspector(
                    ui,
                    &selected_unit,
                    &mut unit_data_query,
                    &registry,
                    &mut actions,
                );
            } else {
                // -- Tile Inspector --
                render_inspector(
                    ui,
                    &selected_hex,
                    &tile_query,
                    &mut tile_data_query,
                    &registry,
                );
            }
        });

    // -- Apply deferred actions --
    apply_actions(
        actions,
        &mut registry,
        &mut tile_data_query,
        &mut active_board,
        &mut active_token,
        &mut selected_unit,
        &editor_state,
        &mut commands,
    );
}

// ---------------------------------------------------------------------------
// UI Section Renderers
// ---------------------------------------------------------------------------

fn render_game_system_info(ui: &mut egui::Ui, game_system: &Option<Res<GameSystem>>) {
    if let Some(gs) = game_system {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Hexorder").strong().size(15.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("v{}", gs.version))
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
        });
        let id_short = if gs.id.len() > 8 {
            format!("{}...", &gs.id[..8])
        } else {
            gs.id.clone()
        };
        ui.label(
            egui::RichText::new(format!("ID: {id_short}"))
                .small()
                .color(egui::Color32::from_gray(120)),
        );
        ui.separator();
    }
}

fn render_tool_mode(ui: &mut egui::Ui, editor_tool: &mut ResMut<EditorTool>) {
    ui.label(egui::RichText::new("Tool Mode").strong());
    ui.horizontal(|ui| {
        if ui
            .selectable_label(**editor_tool == EditorTool::Select, "Select")
            .clicked()
        {
            **editor_tool = EditorTool::Select;
        }
        if ui
            .selectable_label(**editor_tool == EditorTool::Paint, "Paint")
            .clicked()
        {
            **editor_tool = EditorTool::Paint;
        }
        if ui
            .selectable_label(**editor_tool == EditorTool::Place, "Place")
            .clicked()
        {
            **editor_tool = EditorTool::Place;
        }
    });
    ui.separator();
}

fn render_cell_palette(
    ui: &mut egui::Ui,
    registry: &Option<ResMut<EntityTypeRegistry>>,
    active_board: &mut ResMut<ActiveBoardType>,
) {
    ui.label(egui::RichText::new("Cell Palette").strong());

    if let Some(registry) = registry {
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
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
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
    } else {
        ui.label("(no cell types loaded)");
    }

    ui.separator();
}

fn render_unit_palette(
    ui: &mut egui::Ui,
    registry: &Option<ResMut<EntityTypeRegistry>>,
    active_token: &mut ResMut<ActiveTokenType>,
) {
    ui.label(egui::RichText::new("Unit Palette").strong());

    if let Some(registry) = registry {
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
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
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
    } else {
        ui.label("(no unit types loaded)");
    }

    ui.separator();
}

fn render_entity_type_editor(
    ui: &mut egui::Ui,
    registry: &mut Option<ResMut<EntityTypeRegistry>>,
    editor_state: &mut ResMut<EditorState>,
    actions: &mut Vec<EditorAction>,
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
    );
}

#[allow(clippy::too_many_arguments)]
fn render_entity_type_section(
    ui: &mut egui::Ui,
    registry: &mut Option<ResMut<EntityTypeRegistry>>,
    editor_state: &mut ResMut<EditorState>,
    actions: &mut Vec<EditorAction>,
    role: EntityRole,
    section_label: &str,
    id_prefix: &str,
) {
    egui::CollapsingHeader::new(egui::RichText::new(section_label).strong())
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
                    if ui.button("+ Create").clicked() && name_valid {
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
            if let Some(registry) = registry {
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
                                        .color(egui::Color32::GRAY),
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
                                    let types = ["Bool", "Int", "Float", "String", "Color", "Enum"];
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
                                            .color(egui::Color32::GRAY),
                                    );
                                }
                                let prop_valid = !editor_state.new_prop_name.trim().is_empty();
                                ui.add_enabled_ui(prop_valid, |ui| {
                                    if ui.button("+ Add").clicked() && prop_valid {
                                        let prop_type = index_to_property_type(
                                            editor_state.new_prop_type_index,
                                        );
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
                                        egui::RichText::new("Delete Type")
                                            .color(egui::Color32::from_rgb(200, 80, 80)),
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

#[allow(clippy::type_complexity)]
fn render_inspector(
    ui: &mut egui::Ui,
    selected_hex: &Res<SelectedHex>,
    tile_query: &Query<(&HexPosition, Entity), With<HexTile>>,
    tile_data_query: &mut Query<&mut EntityData, Without<UnitInstance>>,
    registry: &Option<ResMut<EntityTypeRegistry>>,
) {
    egui::CollapsingHeader::new(egui::RichText::new("Inspector").strong())
        .default_open(true)
        .show(ui, |ui| {
            let Some(pos) = selected_hex.position else {
                ui.label(egui::RichText::new("No tile selected").color(egui::Color32::GRAY));
                return;
            };

            ui.label(format!("Position: ({}, {})", pos.q, pos.r));

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
                .as_ref()
                .and_then(|r| r.get(entity_data.entity_type_id))
                .map_or_else(|| "Unknown".to_string(), |et| et.name.clone());
            ui.label(format!("Type: {type_name}"));

            // Property value editors
            let prop_defs: Vec<_> = registry
                .as_ref()
                .and_then(|r| r.get(entity_data.entity_type_id))
                .map(|et| et.properties.clone())
                .unwrap_or_default();

            if prop_defs.is_empty() {
                ui.label(
                    egui::RichText::new("No properties")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                return;
            }

            let enum_defs: Vec<EnumDefinition> = registry
                .as_ref()
                .map(|r| r.enum_definitions.clone())
                .unwrap_or_default();

            ui.separator();
            ui.label(egui::RichText::new("Properties").small());

            for prop_def in &prop_defs {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", prop_def.name));

                    let value = entity_data
                        .properties
                        .entry(prop_def.id)
                        .or_insert_with(|| PropertyValue::default_for(&prop_def.property_type));

                    render_property_value_editor(ui, value, &prop_def.property_type, &enum_defs);
                });
            }
        });
}

fn render_unit_inspector(
    ui: &mut egui::Ui,
    selected_unit: &ResMut<SelectedUnit>,
    unit_data_query: &mut Query<&mut EntityData, With<UnitInstance>>,
    registry: &Option<ResMut<EntityTypeRegistry>>,
    actions: &mut Vec<EditorAction>,
) {
    egui::CollapsingHeader::new(egui::RichText::new("Unit Inspector").strong())
        .default_open(true)
        .show(ui, |ui| {
            let Some(entity) = selected_unit.entity else {
                ui.label(egui::RichText::new("No unit selected").color(egui::Color32::GRAY));
                return;
            };

            let Ok(mut entity_data) = unit_data_query.get_mut(entity) else {
                ui.label("Unit entity not found");
                return;
            };

            // Unit type name
            let type_name = registry
                .as_ref()
                .and_then(|r| r.get(entity_data.entity_type_id))
                .map_or_else(|| "Unknown".to_string(), |et| et.name.clone());
            ui.label(format!("Unit Type: {type_name}"));

            // Property value editors
            let prop_defs: Vec<_> = registry
                .as_ref()
                .and_then(|r| r.get(entity_data.entity_type_id))
                .map(|et| et.properties.clone())
                .unwrap_or_default();

            let enum_defs: Vec<EnumDefinition> = registry
                .as_ref()
                .map(|r| r.enum_definitions.clone())
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
                            &enum_defs,
                        );
                    });
                }
            }

            ui.separator();

            // Delete unit button
            if ui
                .button(
                    egui::RichText::new("Delete Unit").color(egui::Color32::from_rgb(200, 80, 80)),
                )
                .clicked()
            {
                actions.push(EditorAction::DeleteSelectedUnit);
            }
        });
}

fn render_property_value_editor(
    ui: &mut egui::Ui,
    value: &mut PropertyValue,
    prop_type: &PropertyType,
    enum_defs: &[EnumDefinition],
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
                let options: Vec<String> = enum_defs
                    .iter()
                    .find(|e| e.id == *enum_id)
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
    }
}

// ---------------------------------------------------------------------------
// Deferred Action Application
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn apply_actions(
    actions: Vec<EditorAction>,
    registry: &mut Option<ResMut<EntityTypeRegistry>>,
    tile_data_query: &mut Query<&mut EntityData, Without<UnitInstance>>,
    active_board: &mut ResMut<ActiveBoardType>,
    active_token: &mut ResMut<ActiveTokenType>,
    selected_unit: &mut ResMut<SelectedUnit>,
    editor_state: &ResMut<EditorState>,
    commands: &mut Commands,
) {
    for action in actions {
        match action {
            EditorAction::CreateEntityType { name, role, color } => {
                if let Some(registry) = registry.as_mut() {
                    registry.types.push(EntityType {
                        id: TypeId::new(),
                        name,
                        role,
                        color,
                        properties: Vec::new(),
                    });
                }
            }
            EditorAction::DeleteEntityType { id } => {
                if let Some(registry) = registry.as_mut() {
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
            }
            EditorAction::AddProperty {
                type_id,
                name,
                prop_type,
                enum_options,
            } => {
                if let Some(registry) = registry.as_mut() {
                    let final_type = if matches!(prop_type, PropertyType::Enum(_)) {
                        let enum_id = TypeId::new();
                        let options: Vec<String> = enum_options
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        registry.enum_definitions.push(EnumDefinition {
                            id: enum_id,
                            name: name.clone(),
                            options,
                        });
                        PropertyType::Enum(enum_id)
                    } else {
                        prop_type
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
            }
            EditorAction::RemoveProperty { type_id, prop_id } => {
                // Determine role to know which query to clean up.
                let role = registry
                    .as_ref()
                    .and_then(|r| r.get(type_id))
                    .map(|et| et.role);

                if let Some(registry) = registry.as_mut()
                    && let Some(et) = registry.types.iter_mut().find(|et| et.id == type_id)
                {
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
                // next inspector render (consistent with M3 behavior).
            }
            EditorAction::DeleteSelectedUnit => {
                if let Some(entity) = selected_unit.entity {
                    commands.entity(entity).despawn();
                    selected_unit.entity = None;
                }
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
    }
}

fn index_to_property_type(index: usize) -> PropertyType {
    match index {
        1 => PropertyType::Int,
        2 => PropertyType::Float,
        3 => PropertyType::String,
        4 => PropertyType::Color,
        5 => PropertyType::Enum(TypeId::new()),
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
        _ => egui::Color32::GRAY,
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
