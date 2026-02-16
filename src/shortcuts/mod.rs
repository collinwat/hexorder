//! Shortcuts plugin.
//!
//! Provides a centralized keyboard shortcut registry, shortcut matching
//! system, and Cmd+K command palette toggle. All keyboard shortcuts in
//! Hexorder are registered here and dispatched via `CommandExecutedEvent`.

use bevy::prelude::*;

use crate::contracts::shortcuts::{CommandPaletteState, ShortcutRegistry};

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages the shortcut registry and dispatches
/// `CommandExecutedEvent` for matched key presses.
#[derive(Debug)]
pub struct ShortcutsPlugin;

impl Plugin for ShortcutsPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources immediately so consumer plugins can register
        // shortcuts in their own build() methods.
        app.insert_resource(ShortcutRegistry::default());
        app.insert_resource(CommandPaletteState::default());

        app.add_systems(
            PreUpdate,
            systems::intercept_palette_toggle.before(bevy_egui::EguiPreUpdateSet::ProcessInput),
        );
        app.add_systems(Update, systems::match_shortcuts);
    }
}
