# Plugin: Undo/Redo

## Summary

Provides cross-cutting undo/redo infrastructure using the command pattern. Every user action that
modifies state is wrapped in a command object with execute and undo methods, enabling Cmd+Z /
Cmd+Shift+Z to reverse and replay editing actions.

## Plugin

- Module: `src/undo_redo/`
- Plugin struct: `UndoRedoPlugin`
- Schedule: `Update` (undo/redo execution systems), `Startup` (shortcut registration)

## Appetite

- **Size**: Big Batch (full cycle)
- **Pitch**: #84

## Dependencies

- **Contracts consumed**: `shortcuts` (CommandId, ShortcutRegistry, CommandExecutedEvent),
  `game_system` (EntityTypeRegistry, EntityData, PropertyValue, TypeId), `hex_grid` (HexPosition,
  HexTile, HexSelectedEvent), `editor_ui` (EditorTool)
- **Contracts produced**: `undo_redo` (UndoableCommand trait, UndoStack resource,
  UndoEvent/RedoEvent)
- **Crate dependencies**: none expected

## Scope

1. Command contract and undo stack infrastructure — UndoableCommand trait in contracts, UndoStack
   resource with configurable depth (default 100), redo clearing on new action
2. Property change commands — SetProperty command capturing old/new PropertyValue, first concrete
   command exercising the full undo/redo loop end-to-end
3. Map edit commands — SetTerrain command for hex terrain painting with old/new terrain capture
4. Entity lifecycle commands — CreateEntity/DeleteEntity with entity snapshot and restore
5. Compound commands — command group wrapper for atomic multi-action undo/redo
6. Keyboard shortcuts and UI labels — Cmd+Z/Cmd+Shift+Z via shortcuts plugin, "Undo/Redo [action]"
   display in editor

## Success Criteria

- [x] [SC-1] UndoableCommand trait defined in contracts with execute/undo/description methods
- [x] [SC-2] UndoStack resource with push, undo, redo operations and configurable depth
- [x] [SC-3] Property changes are undoable — change a property, Cmd+Z reverts it
- [x] [SC-4] Terrain painting is undoable — paint a hex, Cmd+Z reverts it
- [ ] [SC-5] Entity creation/deletion is undoable — place a unit, Cmd+Z removes it
- [ ] [SC-6] Compound commands undo atomically — multi-action gesture undoes as one step
- [x] [SC-7] Redo stack is cleared when a new action is performed
- [x] [SC-8] Cmd+Z and Cmd+Shift+Z registered in ShortcutRegistry and functional
- [x] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [x] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [x] [SC-TEST] `cargo test` passes (all tests, not just this plugin's)
- [x] [SC-BOUNDARY] No imports from other plugins' internals — all cross-plugin types come from
      `crate::contracts::`

## UAT Checklist

- [ ] [UAT-1] Launch app, paint a hex, press Cmd+Z — hex reverts to previous terrain
- [ ] [UAT-2] Launch app, place a unit, press Cmd+Z — unit is removed
- [ ] [UAT-3] Launch app, perform 3 actions, press Cmd+Z three times — all three revert in order
- [ ] [UAT-4] Launch app, undo an action, press Cmd+Shift+Z — action is redone
- [ ] [UAT-5] Launch app, undo, then perform a new action — redo stack is cleared (Cmd+Shift+Z does
      nothing)

## Constraints

- No `unwrap()` in production code
- Commands must capture enough state to fully reverse — no reliance on external undo logs
- Stack is per-session only — not persisted across saves
- No undo during play mode (if play mode exists)

## Open Questions

- Should undo/redo fire toast notifications (via ToastEvent) on each action? (Nice-to-have if #121
  ships toasts first)
- What is the interaction with persistence? Does loading a file clear the undo stack?

## Deferred Items

- Persistent undo history across saves (pitch no-go)
- Branching undo tree (pitch no-go)
- Selective undo (pitch no-go)
- Rule change commands (incremental adoption — future cycle)
