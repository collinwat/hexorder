//! Standalone panel and overlay rendering systems for the editor UI.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use hexorder_contracts::editor_ui::{EditorTool, ToastKind};
use hexorder_contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityRole, EntityTypeRegistry, GameSystem,
};
#[cfg(feature = "inspector")]
use hexorder_contracts::hex_grid::SelectedHex;
use hexorder_contracts::hex_grid::{HexGridConfig, HexPosition, HexTile};
use hexorder_contracts::persistence::{LoadRequestEvent, NewProjectEvent, Workspace};
use hexorder_contracts::settings::{SettingsRegistry, ThemeLibrary};

use super::actions::bevy_color_to_egui;
use super::components::{BrandTheme, EditorState, GridOverlayVisible, ToastState};

/// Debug inspector as a right-side panel.
/// Only compiled when the `inspector` feature is enabled.
/// Toggled via the `view.toggle_debug_panel` command (backtick key).
#[cfg(feature = "inspector")]
pub fn debug_inspector_panel(
    mut contexts: EguiContexts,
    margins: Res<ViewportMargins>,
    grid_config: Option<Res<hexorder_contracts::hex_grid::HexGridConfig>>,
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

/// Convert an RGB `[u8; 3]` array to an `egui::Color32`.
pub(super) fn rgb(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// Configures the egui dark theme every frame. This is idempotent and cheap
/// (a few struct assignments). Running every frame guarantees the theme is
/// always applied, even after a window visibility change resets the context.
///
/// Reads the active theme from `SettingsRegistry` and looks it up in
/// `ThemeLibrary`. Falls back to the brand theme (always first in library).
pub fn configure_theme(
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    settings: Res<SettingsRegistry>,
    theme_library: Res<ThemeLibrary>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Look up active theme; fall back to first (brand) theme.
    let theme = theme_library
        .find(&settings.active_theme)
        .or_else(|| theme_library.themes.first());

    let Some(theme) = theme else {
        // No themes at all — should never happen (brand is always loaded).
        return;
    };

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = rgb(theme.bg_panel);
    visuals.window_fill = rgb(theme.bg_panel);
    visuals.extreme_bg_color = rgb(theme.bg_deep);
    visuals.faint_bg_color = rgb(theme.bg_surface);
    // Noninteractive fill derived from inactive (slightly darker).
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(
        theme.widget_inactive[0].saturating_sub(10),
        theme.widget_inactive[1].saturating_sub(10),
        theme.widget_inactive[2].saturating_sub(10),
    );
    visuals.widgets.inactive.bg_fill = rgb(theme.widget_inactive);
    visuals.widgets.hovered.bg_fill = rgb(theme.widget_hovered);
    visuals.widgets.active.bg_fill = rgb(theme.widget_active);
    visuals.selection.bg_fill = rgb(theme.accent_primary);
    visuals.window_stroke = egui::Stroke::new(1.0, rgb(theme.border));

    // Text colors (fg_stroke)
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, rgb(theme.text_primary));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, rgb(theme.text_secondary));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, rgb(theme.text_primary));
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, rgb(theme.text_primary));
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, rgb(theme.text_primary));
    ctx.set_visuals(visuals);

    let scale = editor_state.font_size_base / 15.0;
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0 * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.0 * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0 * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0 * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(15.0 * scale, egui::FontFamily::Monospace),
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

    // Paint full-screen background (avoids CentralPanel, which conflicts
    // with the editor's CentralPanel on state transition).
    let screen = ctx.available_rect();
    let panel_fill = ctx.style().visuals.panel_fill;
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("launcher_bg"),
    ))
    .rect_filled(screen, egui::CornerRadius::ZERO, panel_fill);

    egui::Area::new(egui::Id::new("launcher_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
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
                                egui::Button::new(egui::RichText::new("Create").color(
                                    if name_valid {
                                        BrandTheme::ACCENT_AMBER
                                    } else {
                                        BrandTheme::TEXT_DISABLED
                                    },
                                ))
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
                                egui::RichText::new("New Game System")
                                    .color(BrandTheme::ACCENT_AMBER),
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
            .on_hover_text("Click tiles or units to select them (1)")
            .clicked()
        {
            *editor_tool = EditorTool::Select;
        }
        if ui
            .selectable_label(*editor_tool == EditorTool::Paint, "Paint")
            .on_hover_text("Click tiles to paint cell types (2)")
            .clicked()
        {
            *editor_tool = EditorTool::Paint;
        }
        if ui
            .selectable_label(*editor_tool == EditorTool::Place, "Place")
            .on_hover_text("Click tiles to place unit tokens (3)")
            .clicked()
        {
            *editor_tool = EditorTool::Place;
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

/// Renders the About panel as a centered `egui::Window`.
pub(crate) fn render_about_panel(ctx: &egui::Context, editor_state: &mut EditorState) {
    if !editor_state.about_panel_visible {
        return;
    }

    let mut open = editor_state.about_panel_visible;
    egui::Window::new("About Hexorder")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("HEXORDER")
                        .size(28.0)
                        .strong()
                        .color(BrandTheme::ACCENT_AMBER),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Game System Design Tool")
                        .color(BrandTheme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .color(BrandTheme::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Define rules, develop aesthetics, and experiment\nwith tabletop war game systems.",
                    )
                    .small()
                    .color(BrandTheme::TEXT_SECONDARY),
                );
                ui.add_space(12.0);
                if ui.button("Close").clicked() {
                    editor_state.about_panel_visible = false;
                }
            });
        });
    if !open {
        editor_state.about_panel_visible = false;
    }
}

/// Renders the currently active toast notification at the bottom-center of the screen.
pub fn render_toast(
    mut contexts: EguiContexts,
    mut toast_state: ResMut<ToastState>,
    time: Res<Time>,
) {
    let Some(toast) = toast_state.active.as_mut() else {
        return;
    };

    toast.remaining -= time.delta_secs();
    if toast.remaining <= 0.0 {
        toast_state.active = None;
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let message = toast.message.clone();
    let text_color = match toast.kind {
        ToastKind::Success => BrandTheme::SUCCESS,
        ToastKind::Error => BrandTheme::DANGER,
        ToastKind::Info => BrandTheme::TEXT_PRIMARY,
    };

    egui::Area::new(egui::Id::new("toast_notification"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0])
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(BrandTheme::BG_SURFACE)
                .stroke(egui::Stroke::new(1.0, BrandTheme::BORDER_SUBTLE))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(16, 8))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(message).color(text_color));
                });
        });
}

/// Renders (q,r) coordinate labels on each hex tile when the grid overlay is enabled.
pub fn render_grid_overlay(
    mut contexts: EguiContexts,
    grid_overlay: Res<GridOverlayVisible>,
    tile_query: Query<&HexPosition, With<HexTile>>,
    config: Option<Res<HexGridConfig>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !grid_overlay.0 {
        return;
    }

    let Some(config) = config else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let available = ctx.available_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("grid_overlay"),
    ));
    let font = egui::FontId::new(10.0, egui::FontFamily::Monospace);

    for pos in &tile_query {
        let wp = config.layout.hex_to_world_pos(pos.to_hex());
        let world_pos = Vec3::new(wp.x, 0.0, wp.y);
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            continue;
        };
        let screen_pos = egui::pos2(viewport_pos.x, viewport_pos.y);
        if !available.contains(screen_pos) {
            continue;
        }
        let label = format!("{},{}", pos.q, pos.r);
        painter.text(
            screen_pos,
            egui::Align2::CENTER_CENTER,
            label,
            font.clone(),
            BrandTheme::TEXT_SECONDARY,
        );
    }
}
