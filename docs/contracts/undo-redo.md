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
- `has_new_records: bool` — set by `record()`, cleared by `acknowledge_records()`

Methods:

- `record(cmd)` — push an already-executed command onto undo stack, clear redo stack
- `request_undo()` — set pending_undo flag
- `request_redo()` — set pending_redo flag
- `can_undo() -> bool`
- `can_redo() -> bool`
- `undo_description() -> Option<String>`
- `redo_description() -> Option<String>`
- `clear()` — reset both stacks (e.g., on project load)
- `has_new_records() -> bool` — whether commands have been recorded since last acknowledge
- `acknowledge_records()` — clear the flag after syncing dirty state

### SetPropertyCommand (struct)

Built-in command for property value changes.

- `entity: Entity`
- `property_id: TypeId`
- `old_value: PropertyValue`
- `new_value: PropertyValue`
- `label: String`

### SetTerrainCommand (struct)

Built-in command for reversible terrain (entity type) changes on a hex tile.

- `entity: Entity`
- `old_type_id: TypeId`
- `old_properties: HashMap<TypeId, PropertyValue>`
- `new_type_id: TypeId`
- `new_properties: HashMap<TypeId, PropertyValue>`
- `label: String`

### PlaceUnitCommand (struct)

Built-in command for reversible unit (token) placement. Undo despawns; redo respawns.

- `entity: Option<Entity>`
- `position: HexPosition`
- `entity_data: EntityData`
- `mesh: Handle<Mesh>`
- `material: Handle<StandardMaterial>`
- `transform: Transform`
- `label: String`

### DeleteUnitCommand (struct)

Built-in command for reversible unit (token) deletion. Undo respawns; redo despawns.

- `entity: Option<Entity>`
- `position: HexPosition`
- `entity_data: EntityData`
- `mesh: Handle<Mesh>`
- `material: Handle<StandardMaterial>`
- `transform: Transform`
- `label: String`

### CompoundCommand (struct)

Groups multiple commands into a single undoable step.

- `commands: Vec<Box<dyn UndoableCommand>>`
- `label: String`

Execute runs all sub-commands in order; undo reverses them in reverse order.

## Consumers

- `editor_ui` — reads `can_undo/can_redo` and descriptions for menu labels; pushes
  `SetPropertyCommand` for inspector edits; pushes `DeleteUnitCommand` for unit deletion
- `cell` — pushes `SetTerrainCommand` for terrain painting
- `unit` — pushes `PlaceUnitCommand` for unit placement
- `persistence` — calls `clear()` on project load

## Events

None. Undo/redo is triggered via `CommandExecutedEvent` from the shortcuts contract (`edit.undo`,
`edit.redo`). The UndoStack uses internal flags, not events.
