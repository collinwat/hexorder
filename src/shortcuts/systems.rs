//! Systems for the shortcuts plugin.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::contracts::shortcuts::{
    CommandExecutedEvent, CommandPaletteState, KeyBinding, Modifiers, ShortcutRegistry,
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
        if palette.open {
            palette.query.clear();
        }
    }

    // Escape closes the palette.
    if palette.open && keys.just_pressed(KeyCode::Escape) {
        palette.open = false;
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
