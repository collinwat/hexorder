//! Map generation UI panel rendered via egui.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::components::{GenerateMap, MapGenPanelVisible, MapGenParams};

/// Renders the map generation parameter panel as a floating egui window.
pub fn map_gen_panel(
    mut contexts: EguiContexts,
    mut params: ResMut<MapGenParams>,
    panel_visible: Res<MapGenPanelVisible>,
    generate: Option<Res<GenerateMap>>,
    mut commands: Commands,
) {
    if !panel_visible.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let is_generating = generate.is_some();
    let mut should_generate = false;
    let mut should_reset = false;

    egui::Window::new("Map Generator")
        .default_width(260.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
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
                should_reset = true;
            }

            ui.add_space(4.0);

            // Generate button (disabled while generation is in progress)
            ui.add_enabled_ui(!is_generating, |ui| {
                if ui
                    .button("Generate Map")
                    .on_hover_text("Generate terrain using current parameters")
                    .clicked()
                {
                    should_generate = true;
                }
            });
        });

    // Perform side effects outside the egui closure to avoid multi-pass issues.
    if should_reset {
        *params = MapGenParams::default();
    }
    if should_generate {
        commands.insert_resource(GenerateMap);
    }
}
