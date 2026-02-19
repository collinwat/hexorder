# Contract: Undo/Redo

## Owner

`undo_redo` plugin

## Summary

Cross-cutting undo/redo infrastructure. Defines the command trait that plugins implement for
reversible actions, and the stack resource that tracks history.

## Types

### UndoableCommand (trait)

Trait for reversible commands. Implementors must be `Send + Sync + Debug`.

- `fn execute(&mut self, world: &mut World)` — apply this command's action
- `fn undo(&mut self, world: &mut World)` — reverse this command's action
- `fn description(&self) -> String` — human-readable label (e.g., "Set Attack to 5")

### UndoStack (Resource)

Manages undo/redo history.

- `undo_stack: Vec<Box<dyn UndoableCommand>>` — commands that can be undone
- `redo_stack: Vec<Box<dyn UndoableCommand>>` — commands that can be redone
- `max_depth: usize` — maximum stack size (default: 100)
- `pending_undo: bool` — flag set by observer, consumed by exclusive system
- `pending_redo: bool` — flag set by observer, consumed by exclusive system

Methods:

- `record(cmd)` — push an already-executed command onto undo stack, clear redo stack
- `request_undo()` — set pending_undo flag
- `request_redo()` — set pending_redo flag
- `can_undo() -> bool`
- `can_redo() -> bool`
- `undo_description() -> Option<String>`
- `redo_description() -> Option<String>`
- `clear()` — reset both stacks (e.g., on project load)

### SetPropertyCommand (struct)

Built-in command for property value changes.

- `entity: Entity`
- `property_id: TypeId`
- `old_value: PropertyValue`
- `new_value: PropertyValue`
- `label: String`

## Consumers

- `editor_ui` — reads `can_undo/can_redo` and descriptions for menu labels; pushes
  `SetPropertyCommand` for inspector edits
- `cell` — pushes commands for terrain painting (future Scope 3)
- `unit` — pushes commands for unit placement/deletion (future Scope 4)
- `persistence` — calls `clear()` on project load

## Events

None. Undo/redo is triggered via `CommandExecutedEvent` from the shortcuts contract (`edit.undo`,
`edit.redo`). The UndoStack uses internal flags, not events.
