//! Design tab rendering — entity types, enums, and structs.

use bevy::prelude::*;
use bevy_egui::egui;

use hexorder_contracts::game_system::{
    EntityRole, EntityTypeRegistry, EnumRegistry, StructRegistry, TypeId,
};

use super::actions::{
    bevy_color_to_egui, color32_to_rgb, egui_color_to_bevy, format_property_type,
    index_to_property_type, rgb_to_color32,
};
use super::components::{BrandTheme, EditorAction, EditorState};

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
