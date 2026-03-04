//! Undo/Redo plugin.
//!
//! Provides cross-cutting undo/redo infrastructure using the command pattern.
//! Registers Cmd+Z / Cmd+Shift+Z shortcuts and processes undo/redo operations
//! via an exclusive system with `&mut World` access.

use bevy::prelude::*;

use hexorder_contracts::shortcuts::{
    CommandCategory, CommandEntry, CommandId, KeyBinding, Modifiers, ShortcutRegistry,
};
use hexorder_contracts::undo_redo::UndoStack;
use hexorder_sdk::{HexorderPlugin, PluginId};

mod systems;

#[cfg(test)]
mod tests;

/// Plugin that manages undo/redo infrastructure.
#[derive(Debug)]
pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UndoStack>()
            .add_systems(Update, systems::process_undo_redo)
            .add_observer(systems::handle_undo_redo_command);

        register_shortcuts(
            app.world_mut()
                .resource_mut::<ShortcutRegistry>()
                .into_inner(),
        );
    }
}

impl HexorderPlugin for UndoRedoPlugin {
    fn id(&self) -> PluginId {
        PluginId("hexorder-undo-redo")
    }

    fn plugin_name(&self) -> &'static str {
        "Undo/Redo"
    }

    fn build(&self, app: &mut App) {
        Plugin::build(self, app);
    }
}

fn register_shortcuts(registry: &mut ShortcutRegistry) {
    use bevy::input::keyboard::KeyCode;

    registry.register(CommandEntry {
        id: CommandId("edit.undo"),
        name: "Undo".to_string(),
        description: "Undo the last action".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD)],
        category: CommandCategory::Edit,
        continuous: false,
    });

    registry.register(CommandEntry {
        id: CommandId("edit.redo"),
        name: "Redo".to_string(),
        description: "Redo the last undone action".to_string(),
        bindings: vec![KeyBinding::new(KeyCode::KeyZ, Modifiers::CMD_SHIFT)],
        category: CommandCategory::Edit,
        continuous: false,
    });
}
