//! Systems for the `undo_redo` plugin.

use bevy::prelude::*;

use crate::contracts::shortcuts::CommandExecutedEvent;
use crate::contracts::undo_redo::UndoStack;

/// Observer: handles edit.undo and edit.redo commands from the shortcut registry.
pub fn handle_undo_redo_command(trigger: On<CommandExecutedEvent>, mut stack: ResMut<UndoStack>) {
    match trigger.event().command_id.0 {
        "edit.undo" => stack.request_undo(),
        "edit.redo" => stack.request_redo(),
        _ => {}
    }
}

/// Exclusive system: processes pending undo/redo operations.
/// Runs every frame. If no operations are pending, returns immediately.
pub fn process_undo_redo(world: &mut World) {
    let (do_undo, do_redo) = {
        let stack = world.resource::<UndoStack>();
        (stack.pending_undo, stack.pending_redo)
    };

    if do_undo {
        world.resource_mut::<UndoStack>().pending_undo = false;
        let cmd = world.resource_mut::<UndoStack>().pop_undo();
        if let Some(mut cmd) = cmd {
            cmd.undo(world);
            world.resource_mut::<UndoStack>().push_redo(cmd);
        }
    }

    if do_redo {
        world.resource_mut::<UndoStack>().pending_redo = false;
        let cmd = world.resource_mut::<UndoStack>().pop_redo();
        if let Some(mut cmd) = cmd {
            cmd.execute(world);
            world.resource_mut::<UndoStack>().push_undo(cmd);
        }
    }
}
