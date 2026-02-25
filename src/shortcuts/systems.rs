//! Systems for the shortcuts plugin.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use hexorder_contracts::shortcuts::{
    CommandEntry, CommandExecutedEvent, CommandId, CommandPaletteState, KeyBinding, Modifiers,
    ShortcutRegistry,
};

/// Reads the current modifier key state from `ButtonInput<KeyCode>`.
fn current_modifiers(keys: &ButtonInput<KeyCode>) -> Modifiers {
    Modifiers {
        cmd: keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
        shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        ctrl: keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
    }
}

/// Intercepts Cmd+K to toggle the command palette. Runs in `PreUpdate`
/// before egui processes input, so it works even when a text field has focus.
pub fn intercept_palette_toggle(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut palette: ResMut<CommandPaletteState>,
) {
    let Some(keys) = keys else { return };

    let cmd = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
    if cmd && keys.just_pressed(KeyCode::KeyK) {
        palette.open = !palette.open;
        palette.query.clear();
        palette.selected_index = 0;
    }

    // Escape closes the palette.
    if palette.open && keys.just_pressed(KeyCode::Escape) {
        palette.open = false;
        palette.query.clear();
        palette.selected_index = 0;
    }
}

/// Checks `ButtonInput<KeyCode>` for just-pressed keys, matches against
/// the `ShortcutRegistry`, and fires `CommandExecutedEvent` for matches.
///
/// Skips all matching while the command palette is open (to prevent
/// search typing from triggering shortcuts).
pub fn match_shortcuts(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    registry: Res<ShortcutRegistry>,
    palette: Res<CommandPaletteState>,
    mut commands: Commands,
) {
    let Some(keys) = keys else { return };

    // Don't fire shortcuts while the palette is open.
    if palette.open {
        return;
    }

    let modifiers = current_modifiers(&keys);

    // Check each just-pressed key against the registry.
    for key in keys.get_just_pressed() {
        // Skip modifier keys themselves.
        if is_modifier_key(*key) {
            continue;
        }

        let binding = KeyBinding::new(*key, modifiers);
        if let Some(entry) = registry.lookup(&binding) {
            // Skip continuous commands — they are handled by their own systems.
            if entry.continuous {
                continue;
            }

            commands.trigger(CommandExecutedEvent {
                command_id: entry.id.clone(),
            });
        }
    }
}

/// Returns true if the key is a modifier (should not be treated as a shortcut trigger).
const fn is_modifier_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::SuperLeft
            | KeyCode::SuperRight
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::ControlLeft
            | KeyCode::ControlRight
    )
}

// ---------------------------------------------------------------------------
// Command Palette UI
// ---------------------------------------------------------------------------

/// Palette width in logical pixels.
const PALETTE_WIDTH: f32 = 420.0;
/// Maximum height for the results scroll area.
const RESULTS_MAX_HEIGHT: f32 = 280.0;
/// Vertical offset from top of screen (fraction of screen height).
const PALETTE_TOP_FRACTION: f32 = 0.2;

// Brand-aligned palette colors (from docs/brand.md).
const PALETTE_BG: egui::Color32 = egui::Color32::from_gray(25);
const PALETTE_BORDER: egui::Color32 = egui::Color32::from_gray(60);
const PALETTE_ROW_SELECTED: egui::Color32 = egui::Color32::from_rgb(0, 92, 128);
const PALETTE_TEXT_HINT: egui::Color32 = egui::Color32::from_gray(120);

/// Renders the command palette overlay when open. Runs in
/// `EguiPrimaryContextPass` so it appears above other UI.
pub fn command_palette_system(
    mut contexts: EguiContexts,
    mut palette: ResMut<CommandPaletteState>,
    registry: Res<ShortcutRegistry>,
    mut commands: Commands,
    mut focus_requested: Local<bool>,
) {
    if !palette.open {
        *focus_requested = false;
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen = ctx.content_rect();
    let top_offset = screen.height() * PALETTE_TOP_FRACTION;

    // Build filtered results before the closure (avoids borrowing palette inside).
    let query = palette.query.clone();
    let results = filtered_commands(&registry, &query);

    // Clamp selected index to valid range.
    if results.is_empty() {
        palette.selected_index = 0;
    } else {
        palette.selected_index = palette.selected_index.min(results.len() - 1);
    }

    let mut executed_command: Option<CommandId> = None;

    egui::Window::new("Command Palette")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, top_offset])
        .default_width(PALETTE_WIDTH)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(PALETTE_BG)
                .stroke(egui::Stroke::new(1.0, PALETTE_BORDER))
                .corner_radius(8.0)
                .inner_margin(8.0),
        )
        .show(ctx, |ui| {
            ui.set_min_width(PALETTE_WIDTH);

            // Search input.
            let response = ui.add(
                egui::TextEdit::singleline(&mut palette.query)
                    .hint_text("Type a command...")
                    .desired_width(f32::INFINITY),
            );

            if !*focus_requested {
                response.request_focus();
                *focus_requested = true;
            }

            // Re-filter after query may have changed.
            let results = filtered_commands(&registry, &palette.query);
            if results.is_empty() {
                palette.selected_index = 0;
            } else {
                palette.selected_index = palette.selected_index.min(results.len() - 1);
            }

            // Keyboard navigation via egui input (works while text field has focus).
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !results.is_empty() {
                palette.selected_index = (palette.selected_index + 1).min(results.len() - 1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                palette.selected_index = palette.selected_index.saturating_sub(1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter))
                && let Some(entry) = results.get(palette.selected_index)
            {
                executed_command = Some(entry.id.clone());
            }

            ui.add_space(4.0);
            ui.separator();

            // Results list.
            egui::ScrollArea::vertical()
                .max_height(RESULTS_MAX_HEIGHT)
                .show(ui, |ui| {
                    if results.is_empty() && !palette.query.is_empty() {
                        ui.label(
                            egui::RichText::new("No matching commands").color(PALETTE_TEXT_HINT),
                        );
                        return;
                    }
                    for (i, entry) in results.iter().enumerate() {
                        let selected = i == palette.selected_index;
                        if render_palette_row(ui, entry, selected) {
                            executed_command = Some(entry.id.clone());
                        }
                    }
                });
        });

    // Fire command event OUTSIDE the egui closure (multi-pass safe).
    if let Some(command_id) = executed_command {
        palette.open = false;
        palette.query.clear();
        palette.selected_index = 0;
        commands.trigger(CommandExecutedEvent { command_id });
    }
}

/// Renders one palette result row. Returns true if the row was clicked.
fn render_palette_row(ui: &mut egui::Ui, entry: &CommandEntry, selected: bool) -> bool {
    let fill = if selected {
        PALETTE_ROW_SELECTED
    } else {
        egui::Color32::TRANSPARENT
    };

    let frame_response = egui::Frame::NONE
        .fill(fill)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(&entry.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(binding) = entry.bindings.first() {
                        ui.label(
                            egui::RichText::new(binding.display_string())
                                .color(PALETTE_TEXT_HINT)
                                .small(),
                        );
                    }
                });
            });
        });

    frame_response
        .response
        .interact(egui::Sense::click())
        .clicked()
}

/// Returns filtered discrete commands matching the query, sorted by fuzzy score.
pub fn filtered_commands<'a>(registry: &'a ShortcutRegistry, query: &str) -> Vec<&'a CommandEntry> {
    let discrete = registry.discrete_commands();

    if query.is_empty() {
        return discrete;
    }

    let mut scored: Vec<(&CommandEntry, isize)> = discrete
        .into_iter()
        .filter_map(|entry| {
            sublime_fuzzy::best_match(query, &entry.name).map(|m| (entry, m.score()))
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(entry, _)| entry).collect()
}
