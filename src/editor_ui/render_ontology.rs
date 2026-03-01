//! Ontology tab rendering — concepts, relations, and constraints.

use bevy_egui::egui;

use hexorder_contracts::game_system::{EntityRole, EntityTypeRegistry, TypeId};
use hexorder_contracts::ontology::{
    ConceptRegistry, ConstraintRegistry, RelationEffect, RelationRegistry, RelationTrigger,
};

use super::actions::{
    build_constraint_expression, format_constraint_expr, format_relation_effect,
    index_to_modify_operation,
};
use super::components::{BrandTheme, EditorAction, EditorState};

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
                    // TODO(#17): CrossCompare and IsType editors
                    ui.label(
                        egui::RichText::new("(full editor — #17)")
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
